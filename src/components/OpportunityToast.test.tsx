/**
 * OpportunityToast Component Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import OpportunityToast from './OpportunityToast';
import { TOKENS } from '../lib/tokens';
import { mockInvoke } from '../test/setup';

describe('OpportunityToast', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it('should not render initially', () => {
    render(<OpportunityToast />);
    
    const toast = screen.queryByTestId('opportunity-toast');
    expect(toast).not.toBeInTheDocument();
  });

  it('should appear on shadow:opportunity event', async () => {
    render(<OpportunityToast />);
    
    // Dispatch opportunity event
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        title: 'Test Opportunity',
        confidence: 0.85,
        preview: 'This is a test suggestion',
      },
    }));
    
    await waitFor(() => {
      const toast = screen.getByTestId('opportunity-toast');
      expect(toast).toBeInTheDocument();
    });
  });

  it('should display opportunity content', async () => {
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        title: 'Test Opportunity',
        confidence: 0.85,
        preview: 'This is a test suggestion',
      },
    }));
    
    await waitFor(() => {
      expect(screen.getByText(/j'ai une idée/i)).toBeInTheDocument();
      expect(screen.getByText('This is a test suggestion')).toBeInTheDocument();
    });
  });

  it('should display confidence indicator', async () => {
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      expect(screen.getByText('85%')).toBeInTheDocument();
    });
  });

  it('should have "Voir" and "Ignorer" buttons', async () => {
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      expect(screen.getByText('Voir →')).toBeInTheDocument();
      expect(screen.getByText('Ignorer')).toBeInTheDocument();
    });
  });

  it('should call onOpenDock when "Voir" clicked', async () => {
    const onOpenDock = vi.fn();
    render(<OpportunityToast onOpenDock={onOpenDock} />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const voirButton = screen.getByText('Voir →');
      fireEvent.click(voirButton);
    });
    
    expect(onOpenDock).toHaveBeenCalledTimes(1);
  });

  it('should record acceptance when "Voir" clicked', async () => {
    mockInvoke.mockResolvedValue(undefined);
    
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const voirButton = screen.getByText('Voir →');
      fireEvent.click(voirButton);
    });
    
    expect(mockInvoke).toHaveBeenCalledWith('record_opportunity_response', {
      opportunityId: 'test-1',
      accepted: true,
    });
  });

  it('should dismiss when "Ignorer" clicked', async () => {
    mockInvoke.mockResolvedValue(undefined);
    
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const ignorerButton = screen.getByText('Ignorer');
      fireEvent.click(ignorerButton);
    });
    
    // Toast should disappear
    await waitFor(() => {
      const toast = screen.queryByTestId('opportunity-toast');
      expect(toast).not.toBeInTheDocument();
    });
  });

  it('should record dismissal when "Ignorer" clicked', async () => {
    mockInvoke.mockResolvedValue(undefined);
    
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const ignorerButton = screen.getByText('Ignorer');
      fireEvent.click(ignorerButton);
    });
    
    expect(mockInvoke).toHaveBeenCalledWith('record_opportunity_response', {
      opportunityId: 'test-1',
      accepted: false,
    });
  });

  it('should not show dismissed opportunities again', async () => {
    render(<OpportunityToast />);
    
    // Show and dismiss
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const ignorerButton = screen.getByText('Ignorer');
      fireEvent.click(ignorerButton);
    });
    
    // Try to show same opportunity again
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    // Should not appear
    await waitFor(() => {
      const toast = screen.queryByTestId('opportunity-toast');
      expect(toast).not.toBeInTheDocument();
    });
  });

  it('should auto-dismiss after 10 seconds', async () => {
    vi.useFakeTimers();
    
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      expect(screen.getByTestId('opportunity-toast')).toBeInTheDocument();
    });
    
    // Fast forward 10 seconds
    vi.advanceTimersByTime(10000);
    
    await waitFor(() => {
      const toast = screen.queryByTestId('opportunity-toast');
      expect(toast).not.toBeInTheDocument();
    });
    
    vi.useRealTimers();
  });

  it('should have correct z-index', async () => {
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const toast = screen.getByTestId('opportunity-toast');
      expect(toast).toHaveStyle({
        zIndex: TOKENS.zIndex.toast,
      });
    });
  });

  it('should apply glass morphism styles', async () => {
    render(<OpportunityToast />);
    
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const toast = screen.getByTestId('opportunity-toast');
      const styles = window.getComputedStyle(toast);
      
      expect(styles.backdropFilter).toContain('blur');
    });
  });

  it('should only show opportunities with confidence > 0.7', async () => {
    render(<OpportunityToast />);
    
    // Low confidence
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-1',
        confidence: 0.5,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const toast = screen.queryByTestId('opportunity-toast');
      expect(toast).not.toBeInTheDocument();
    });
    
    // High confidence
    window.dispatchEvent(new CustomEvent('shadow:opportunity', {
      detail: {
        id: 'test-2',
        confidence: 0.85,
        preview: 'Test',
      },
    }));
    
    await waitFor(() => {
      const toast = screen.getByTestId('opportunity-toast');
      expect(toast).toBeInTheDocument();
    });
  });
});



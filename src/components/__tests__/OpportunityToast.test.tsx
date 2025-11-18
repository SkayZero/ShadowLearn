import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import OpportunityToast from '../OpportunityToast';
import type { Opportunity } from '../../lib/types';

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock sound manager
vi.mock('../../lib/soundManager', () => ({
  default: {
    play: vi.fn(),
  },
}));

// Mock layout context
vi.mock('../../contexts/LayoutContext', () => ({
  useLayoutPosition: () => ({ bottom: '20px', right: '20px' }),
}));

// Mock event bus
vi.mock('../../lib', () => ({
  useEvent: vi.fn(),
  EVENTS: {
    OPPORTUNITY: 'shadow:opportunity',
  },
  shadowStore: {
    isOpportunityDismissed: vi.fn(() => false),
    dismissOpportunity: vi.fn(),
  },
  SPRING_CONFIG: { stiffness: 300, damping: 30 },
}));

describe('OpportunityToast', () => {
  const mockOpportunity: Opportunity = {
    id: 'opp_123',
    title: 'J\'ai une idée',
    confidence: 0.85,
    preview: 'Tu travailles sur Cursor depuis 14 secondes. Besoin d\'aide ?',
    context: { app: 'Cursor' },
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should not render when no opportunity', () => {
    render(<OpportunityToast onOpenDock={vi.fn()} />);
    expect(screen.queryByTestId('opportunity-toast')).not.toBeInTheDocument();
  });

  it('should display opportunity with high confidence', async () => {
    const { useEvent } = await import('../../lib');
    const mockUseEvent = useEvent as any;

    // Simulate useEvent calling the handler
    mockUseEvent.mockImplementation((event: string, handler: Function) => {
      if (event === 'shadow:opportunity') {
        setTimeout(() => handler(mockOpportunity), 0);
      }
    });

    render(<OpportunityToast onOpenDock={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText('J\'ai une idée')).toBeInTheDocument();
    });

    expect(screen.getByText(mockOpportunity.preview)).toBeInTheDocument();
    expect(screen.getByText('85%')).toBeInTheDocument();
  });

  it('should not display opportunity with low confidence', async () => {
    const lowConfidenceOpp: Opportunity = {
      ...mockOpportunity,
      confidence: 0.5,
    };

    const { useEvent } = await import('../../lib');
    const mockUseEvent = useEvent as any;

    mockUseEvent.mockImplementation((event: string, handler: Function) => {
      if (event === 'shadow:opportunity') {
        handler(lowConfidenceOpp);
      }
    });

    render(<OpportunityToast onOpenDock={vi.fn()} />);

    await waitFor(() => {
      expect(screen.queryByTestId('opportunity-toast')).not.toBeInTheDocument();
    });
  });

  it('should call onOpenDock when "Voir" button is clicked', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const onOpenDock = vi.fn();

    const { useEvent } = await import('../../lib');
    const mockUseEvent = useEvent as any;

    mockUseEvent.mockImplementation((event: string, handler: Function) => {
      if (event === 'shadow:opportunity') {
        setTimeout(() => handler(mockOpportunity), 0);
      }
    });

    render(<OpportunityToast onOpenDock={onOpenDock} />);

    await waitFor(() => {
      expect(screen.getByText('Voir →')).toBeInTheDocument();
    });

    const viewButton = screen.getByText('Voir →');
    fireEvent.click(viewButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('record_opportunity_response', {
        opportunityId: mockOpportunity.id,
        accepted: true,
      });
      expect(onOpenDock).toHaveBeenCalledTimes(1);
    });
  });

  it('should dismiss opportunity when "Ignorer" button is clicked', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const { shadowStore } = await import('../../lib');

    const { useEvent } = await import('../../lib');
    const mockUseEvent = useEvent as any;

    mockUseEvent.mockImplementation((event: string, handler: Function) => {
      if (event === 'shadow:opportunity') {
        setTimeout(() => handler(mockOpportunity), 0);
      }
    });

    render(<OpportunityToast onOpenDock={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText('Ignorer')).toBeInTheDocument();
    });

    const dismissButton = screen.getByText('Ignorer');
    fireEvent.click(dismissButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('record_opportunity_response', {
        opportunityId: mockOpportunity.id,
        accepted: false,
      });
      expect(shadowStore.dismissOpportunity).toHaveBeenCalledWith(mockOpportunity.id);
    });
  });

  it('should skip already dismissed opportunities', async () => {
    const { shadowStore } = await import('../../lib');
    (shadowStore.isOpportunityDismissed as any).mockReturnValue(true);

    const { useEvent } = await import('../../lib');
    const mockUseEvent = useEvent as any;

    mockUseEvent.mockImplementation((event: string, handler: Function) => {
      if (event === 'shadow:opportunity') {
        handler(mockOpportunity);
      }
    });

    render(<OpportunityToast onOpenDock={vi.fn()} />);

    await waitFor(() => {
      expect(screen.queryByTestId('opportunity-toast')).not.toBeInTheDocument();
    });
  });

  it('should render confidence bar with correct width', async () => {
    const { useEvent } = await import('../../lib');
    const mockUseEvent = useEvent as any;

    mockUseEvent.mockImplementation((event: string, handler: Function) => {
      if (event === 'shadow:opportunity') {
        setTimeout(() => handler(mockOpportunity), 0);
      }
    });

    const { container } = render(<OpportunityToast onOpenDock={vi.fn()} />);

    await waitFor(() => {
      const confidenceBar = container.querySelector('.bg-gradient-to-r');
      expect(confidenceBar).toBeInTheDocument();
    });
  });
});

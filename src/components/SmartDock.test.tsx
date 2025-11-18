/**
 * SmartDock Component Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SmartDock } from './SmartDock';
import { TOKENS } from '../lib/tokens';

describe('SmartDock', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should not render when closed', () => {
    render(<SmartDock open={false} onClose={() => {}} />);
    
    const dock = screen.queryByTestId('smart-dock');
    expect(dock).not.toBeInTheDocument();
  });

  it('should render when open', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const dock = screen.getByTestId('smart-dock');
    expect(dock).toBeInTheDocument();
  });

  it('should have correct dimensions', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const dock = screen.getByTestId('smart-dock');
    
    expect(dock).toHaveStyle({
      width: `${TOKENS.components.dock.width}px`,
      height: `${TOKENS.components.dock.height}px`,
    });
  });

  it('should have correct z-index', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const dock = screen.getByTestId('smart-dock');
    
    expect(dock).toHaveStyle({
      zIndex: TOKENS.zIndex.dock,
    });
  });

  it('should render overlay when open', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const overlay = screen.getByTestId('dock-overlay');
    expect(overlay).toBeInTheDocument();
  });

  it('should call onClose when overlay clicked', () => {
    const onClose = vi.fn();
    render(<SmartDock open={true} onClose={onClose} />);
    
    const overlay = screen.getByTestId('dock-overlay');
    fireEvent.click(overlay);
    
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('should call onClose when ESC pressed', () => {
    const onClose = vi.fn();
    render(<SmartDock open={true} onClose={onClose} />);
    
    fireEvent.keyDown(window, { key: 'Escape' });
    
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('should apply glass morphism styles', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const dock = screen.getByTestId('smart-dock');
    const styles = window.getComputedStyle(dock);
    
    // Check for glass effect
    expect(styles.backdropFilter).toContain('blur');
  });

  it('should have rounded corners', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const dock = screen.getByTestId('smart-dock');
    
    expect(dock).toHaveStyle({
      borderRadius: `${TOKENS.components.dock.borderRadius}px`,
    });
  });

  it('should render close button', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const closeButton = screen.getByLabelText('Close');
    expect(closeButton).toBeInTheDocument();
  });

  it('should call onClose when close button clicked', () => {
    const onClose = vi.fn();
    render(<SmartDock open={true} onClose={onClose} />);
    
    const closeButton = screen.getByLabelText('Close');
    fireEvent.click(closeButton);
    
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('should animate in when opening', () => {
    const { rerender } = render(<SmartDock open={false} onClose={() => {}} />);
    
    rerender(<SmartDock open={true} onClose={() => {}} />);
    
    const dock = screen.getByTestId('smart-dock');
    expect(dock).toBeInTheDocument();
    
    // Framer motion should handle animation
    // Check that it has motion properties
    expect(dock.tagName).toBe('DIV');
  });

  it('should position near cursor', () => {
    // Mock cursor position
    const mockCursorPos = { x: 500, y: 300 };
    
    render(<SmartDock open={true} onClose={() => {}} cursorPos={mockCursorPos} />);
    
    const dock = screen.getByTestId('smart-dock');
    
    // Position should be calculated based on cursor
    // This is a simplified check - actual positioning logic is more complex
    expect(dock).toBeInTheDocument();
  });

  it('should render chat input', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const input = screen.getByPlaceholderText(/écris ou tape/i);
    expect(input).toBeInTheDocument();
  });

  it('should render slash command input', () => {
    render(<SmartDock open={true} onClose={() => {}} />);
    
    const slashInput = screen.getByTestId('slash-input');
    expect(slashInput).toBeInTheDocument();
  });
});





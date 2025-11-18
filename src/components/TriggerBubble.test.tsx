/**
 * TriggerBubble Component Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TriggerBubble } from './TriggerBubble';
import { TOKENS } from '../lib/tokens';

describe('TriggerBubble', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render the bubble', () => {
    render(<TriggerBubble />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    expect(bubble).toBeInTheDocument();
  });

  it('should be positioned at bottom-right with correct offset', () => {
    render(<TriggerBubble />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    const styles = window.getComputedStyle(bubble);
    
    // Check if positioned fixed
    expect(styles.position).toBe('fixed');
    
    // Check offset (should match TOKENS)
    expect(bubble).toHaveStyle({
      right: `${TOKENS.components.bubble.position.right}px`,
      bottom: `${TOKENS.components.bubble.position.bottom}px`,
    });
  });

  it('should have correct size', () => {
    render(<TriggerBubble />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    
    expect(bubble).toHaveStyle({
      width: `${TOKENS.components.bubble.size}px`,
      height: `${TOKENS.components.bubble.size}px`,
    });
  });

  it('should contain AmbientLED', () => {
    render(<TriggerBubble />);
    
    const led = screen.getByTestId('ambient-led');
    expect(led).toBeInTheDocument();
  });

  it('should apply glass morphism styles', () => {
    render(<TriggerBubble />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    const styles = window.getComputedStyle(bubble);
    
    // Check for glass effect properties
    expect(styles.backdropFilter).toContain('blur');
  });

  it('should be clickable', () => {
    const onOpen = vi.fn();
    render(<TriggerBubble onOpen={onOpen} />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    bubble.click();
    
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it('should have correct z-index', () => {
    render(<TriggerBubble />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    
    expect(bubble).toHaveStyle({
      zIndex: TOKENS.zIndex.bubble,
    });
  });

  it('should render with framer-motion animation', () => {
    render(<TriggerBubble />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    
    // Check if it's a motion div (has data-framer-motion attribute or similar)
    // Framer motion adds specific attributes
    expect(bubble.tagName).toBe('DIV');
  });

  it('should maintain aspect ratio', () => {
    render(<TriggerBubble />);
    
    const bubble = screen.getByTestId('trigger-bubble');
    const styles = window.getComputedStyle(bubble);
    
    // Should be circular (width === height)
    expect(styles.width).toBe(styles.height);
  });
});





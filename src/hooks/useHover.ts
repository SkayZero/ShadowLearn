/**
 * Custom hook for handling hover states
 * Optimized to avoid recreating handlers on every render
 */

import { useState, useCallback } from 'react';

export interface HoverHandlers {
  onMouseEnter: () => void;
  onMouseLeave: () => void;
}

export interface HoverState {
  isHovered: boolean;
  handlers: HoverHandlers;
}

/**
 * Hook for managing hover state with memoized handlers
 *
 * @param initialState - Initial hover state (default: false)
 * @returns Object with isHovered state and handlers
 *
 * @example
 * ```tsx
 * const { isHovered, handlers } = useHover();
 * return (
 *   <button {...handlers}>
 *     {isHovered ? 'Hovered!' : 'Hover me'}
 *   </button>
 * );
 * ```
 */
export function useHover(initialState = false): HoverState {
  const [isHovered, setIsHovered] = useState(initialState);

  const handlers: HoverHandlers = {
    onMouseEnter: useCallback(() => setIsHovered(true), []),
    onMouseLeave: useCallback(() => setIsHovered(false), []),
  };

  return { isHovered, handlers };
}

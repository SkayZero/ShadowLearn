/**
 * Layout Context
 * Manages intelligent positioning of floating components
 * Prevents overlap and adapts to window size
 */

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';

type Zone = 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left' | 'center';

interface ComponentRegistration {
  id: string;
  zone: Zone;
  priority: number; // Higher = closer to edge
  height: number;
  width: number;
}

interface Position {
  top?: string;
  bottom?: string;
  left?: string;
  right?: string;
}

interface LayoutContextValue {
  registerComponent: (registration: ComponentRegistration) => void;
  unregisterComponent: (id: string) => void;
  getPosition: (id: string) => Position;
  windowSize: { width: number; height: number };
}

const LayoutContext = createContext<LayoutContextValue | null>(null);

export function LayoutProvider({ children }: { children: ReactNode }) {
  const [components, setComponents] = useState<Map<string, ComponentRegistration>>(new Map());
  const [windowSize, setWindowSize] = useState({ width: window.innerWidth, height: window.innerHeight });

  // Track window size
  useEffect(() => {
    const handleResize = () => {
      setWindowSize({ width: window.innerWidth, height: window.innerHeight });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const registerComponent = useCallback((registration: ComponentRegistration) => {
    setComponents(prev => {
      const next = new Map(prev);
      next.set(registration.id, registration);
      return next;
    });
  }, []);

  const unregisterComponent = useCallback((id: string) => {
    setComponents(prev => {
      const next = new Map(prev);
      next.delete(id);
      return next;
    });
  }, []);

  const getPosition = useCallback((id: string): Position => {
    const component = components.get(id);
    if (!component) return {};

    // Get all components in the same zone, sorted by priority
    const zoneComponents = Array.from(components.values())
      .filter(c => c.zone === component.zone)
      .sort((a, b) => b.priority - a.priority);

    const index = zoneComponents.findIndex(c => c.id === id);
    
    // Calculate cumulative offset
    let offset = 16; // Base padding
    for (let i = 0; i < index; i++) {
      offset += zoneComponents[i].height + 12; // 12px gap between components
    }

    switch (component.zone) {
      case 'bottom-right':
        return { bottom: `${offset}px`, right: '24px' };
      case 'bottom-left':
        return { bottom: `${offset}px`, left: '24px' };
      case 'top-right':
        return { top: `${offset}px`, right: '24px' };
      case 'top-left':
        return { top: `${offset}px`, left: '24px' };
      case 'center':
        return { top: '50%', left: '50%', transform: 'translate(-50%, -50%)' };
      default:
        return {};
    }
  }, [components]);

  return (
    <LayoutContext.Provider value={{ registerComponent, unregisterComponent, getPosition, windowSize }}>
      {children}
    </LayoutContext.Provider>
  );
}

export function useLayout() {
  const context = useContext(LayoutContext);
  if (!context) {
    throw new Error('useLayout must be used within LayoutProvider');
  }
  return context;
}

/**
 * Hook for components to register themselves in the layout system
 */
export function useLayoutPosition(
  id: string,
  zone: Zone,
  priority: number,
  estimatedHeight: number = 100,
  estimatedWidth: number = 300
) {
  const { registerComponent, unregisterComponent, getPosition } = useLayout();
  const [position, setPosition] = useState<Position>({});

  useEffect(() => {
    registerComponent({ id, zone, priority, height: estimatedHeight, width: estimatedWidth });
    return () => unregisterComponent(id);
  }, [id, zone, priority, estimatedHeight, estimatedWidth, registerComponent, unregisterComponent]);

  useEffect(() => {
    setPosition(getPosition(id));
  }, [id, getPosition]);

  return position;
}




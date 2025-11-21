/**
 * Central Event Bus for ShadowLearn
 * Replaces setInterval polling with reactive event-driven architecture
 */

import { listen, UnlistenFn } from "@tauri-apps/api/event";

// ============================================================================
// Event Types
// ============================================================================

export interface ShadowEvent<T = any> {
  timestamp: number;
  payload: T;
}

// Opportunity detected by trigger system
export interface OpportunityPayload {
  id: string;
  title: string;
  confidence: number;
  preview: string;
  context: any;
}

// Context update from backend
export interface ContextUpdatePayload {
  app_name: string;
  window_title: string;
  idle_seconds: number;
  session_duration_minutes: number;
  recent_screenshots: number;
  pending_suggestion?: string;
}

// Flow state for Ambient LED
export interface FlowStatePayload {
  flow_state: "deep" | "normal" | "blocked";
  confidence: number;
}

// Micro-suggestions for Pills
export interface MicroSuggestionPayload {
  id: string;
  text: string;
  type: "continue" | "help" | "reminder";
}

// Trigger state change
export interface TriggerStatePayload {
  state: string;
  reason?: string;
  cooldown_seconds?: number;
}

// Pause detection
export interface PausePayload {
  paused: boolean;
  reason?: "meeting" | "lunch" | "coffee" | "away";
  duration_seconds?: number;
}

// Streak update
export interface StreakPayload {
  current_days: number;
  longest_days: number;
  milestone_reached?: boolean;
}

// ============================================================================
// Event Names (matching Rust backend)
// ============================================================================

export const EVENTS = {
  // Core system events
  OPPORTUNITY: "shadow:opportunity",
  CONTEXT_UPDATE: "shadow:context_update",
  FLOW_STATE: "shadow:flow_state",
  TRIGGER_STATE: "shadow:trigger_state",
  
  // Feature events
  MICRO_SUGGESTION: "shadow:micro_suggestion",
  PAUSE_DETECTED: "shadow:pause_detected",
  RESUME_DETECTED: "shadow:resume_detected",
  STREAK_UPDATE: "shadow:streak_update",
  
  // User interactions
  FEEDBACK_SUBMITTED: "shadow:feedback_submitted",
  COMMAND_EXECUTED: "shadow:command_executed",
} as const;

// ============================================================================
// Event Bus Class
// ============================================================================

class EventBus {
  private listeners: Map<string, UnlistenFn[]> = new Map();

  /**
   * Subscribe to an event
   */
  async on<T>(
    event: string,
    handler: (payload: T) => void | Promise<void>
  ): Promise<UnlistenFn> {
    const unlisten = await listen<ShadowEvent<T>>(event, (e) => {
      handler(e.payload.payload);
    });

    // Track listener for cleanup
    const existing = this.listeners.get(event) || [];
    this.listeners.set(event, [...existing, unlisten]);

    return unlisten;
  }

  /**
   * Subscribe to an event once
   */
  async once<T>(
    event: string,
    handler: (payload: T) => void | Promise<void>
  ): Promise<UnlistenFn> {
    let unlisten: UnlistenFn | null = null;

    unlisten = await this.on<T>(event, async (payload) => {
      await handler(payload);
      if (unlisten) {
        unlisten();
        this.removeListener(event, unlisten);
      }
    });

    return unlisten;
  }

  /**
   * Unsubscribe all listeners for an event
   */
  async off(event: string): Promise<void> {
    const listeners = this.listeners.get(event);
    if (listeners) {
      listeners.forEach((unlisten) => unlisten());
      this.listeners.delete(event);
    }
  }

  /**
   * Remove a specific listener
   */
  private removeListener(event: string, unlisten: UnlistenFn) {
    const listeners = this.listeners.get(event);
    if (listeners) {
      const filtered = listeners.filter((l) => l !== unlisten);
      this.listeners.set(event, filtered);
    }
  }

  /**
   * Cleanup all listeners
   */
  async cleanup(): Promise<void> {
    for (const listeners of this.listeners.values()) {
      listeners.forEach((unlisten) => unlisten());
    }
    this.listeners.clear();
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const eventBus = new EventBus();

// ============================================================================
// React Hook for Event Listening
// ============================================================================

import { useEffect, useState, useRef } from "react";

/**
 * React hook for listening to Tauri events
 * Automatically handles cleanup on unmount
 * 
 * Uses Tauri's listen() API directly to avoid abstraction layers
 */
export function useEvent<T>(
  event: string,
  handler: (payload: T) => void | Promise<void>
) {
  const savedHandler = useRef(handler);
  
  // Update ref when handler changes (without re-registering listener)
  useEffect(() => {
    savedHandler.current = handler;
  }, [handler]);

  useEffect(() => {
    let unlistenTauriFn: UnlistenFn | undefined;
    let isMounted = true;
    
    
    // === PART 1: Tauri Events (from backend) ===
    const setupTauriListener = async () => {
      try {
        const tauriEvent = await import('@tauri-apps/api/event');
        
        unlistenTauriFn = await tauriEvent.listen<T>(event, (tauriEvent) => {
          if (!isMounted) return;
          
          
          try {
            // Try to extract payload, fallback to direct payload if no wrapper
            let payload: T;
            const rawPayload = tauriEvent.payload as any;
            
            if (rawPayload && typeof rawPayload === 'object' && 'payload' in rawPayload) {
              payload = rawPayload.payload;
            } else {
              payload = rawPayload;
            }
            
            const result = savedHandler.current(payload);
            
            if (result instanceof Promise) {
              result.catch((err) => {
                console.error(`[useEvent ${event}] Async handler error:`, err);
              });
            }
          } catch (err) {
            console.error(`[useEvent ${event}] Handler error:`, err);
          }
        });
        
        
        if (!isMounted && unlistenTauriFn) {
          unlistenTauriFn();
          unlistenTauriFn = undefined;
        }
      } catch (err) {
        console.error(`[useEvent ${event}] Tauri setup failed:`, err);
      }
    };
    
    // === PART 2: DOM Events (for manual testing/fallback) ===
    const handleDOMEvent = (e: Event) => {
      if (!isMounted) return;
      
      const customEvent = e as CustomEvent;
      
      try {
        const result = savedHandler.current(customEvent.detail as T);
        
        if (result instanceof Promise) {
          result.catch((err) => {
            console.error(`[useEvent ${event}] DOM async handler error:`, err);
          });
        }
      } catch (err) {
        console.error(`[useEvent ${event}] DOM handler error:`, err);
      }
    };
    
    // Register DOM listener
    window.addEventListener(event, handleDOMEvent);
    
    // Setup Tauri listener
    setupTauriListener();
    
    // Cleanup
    return () => {
      isMounted = false;
      
      // Cleanup Tauri
      if (unlistenTauriFn) {
        try {
          unlistenTauriFn();
        } catch (err) {
          console.error(`[useEvent ${event}] Tauri cleanup error:`, err);
        }
      }
      
      // Cleanup DOM
      window.removeEventListener(event, handleDOMEvent);
    };
  }, [event]);
}

/**
 * React hook for subscribing to event state
 * Returns the latest payload received
 */
export function useEventState<T>(event: string, initialValue: T): T {
  const [state, setState] = useState<T>(initialValue);

  useEvent<T>(event, (payload) => {
    setState(payload);
  });

  return state;
}





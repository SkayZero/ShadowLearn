import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface ShortcutConfig {
  screenshot_analyze: string;
  toggle_bubbles: string;
  open_dashboard: string;
  dismiss_bubble: string;
  enabled: boolean;
}

export type ShortcutAction =
  | 'screenshot-analyze'
  | 'toggle-bubbles'
  | 'open-dashboard'
  | 'dismiss-bubble';

export interface ShortcutHandlers {
  onScreenshotAnalyze?: () => void;
  onToggleBubbles?: () => void;
  onOpenDashboard?: () => void;
  onDismissBubble?: () => void;
}

/**
 * Hook pour gérer les raccourcis clavier globaux
 *
 * @example
 * ```tsx
 * const { config, shortcuts } = useShortcuts({
 *   onScreenshotAnalyze: () => console.log('Screenshot!'),
 *   onToggleBubbles: () => setShowBubbles(prev => !prev),
 *   onOpenDashboard: () => navigate('/dashboard'),
 *   onDismissBubble: () => setActiveBubble(null),
 * });
 * ```
 */
export function useShortcuts(handlers: ShortcutHandlers = {}) {
  const [config, setConfig] = useState<ShortcutConfig | null>(null);
  const [shortcuts, setShortcuts] = useState<Record<string, ShortcutAction>>({});

  useEffect(() => {
    // Load configuration
    invoke<ShortcutConfig>('get_shortcuts_config')
      .then(setConfig)
      .catch((e) => console.error('Failed to load shortcuts config:', e));

    // Load registered shortcuts
    invoke<Record<string, ShortcutAction>>('list_shortcuts')
      .then(setShortcuts)
      .catch((e) => console.error('Failed to list shortcuts:', e));

    // Listen for shortcut events
    const unlisten = listen<ShortcutAction>('shortcut-triggered', (event) => {
      const action = event.payload;
      console.log('🎹 Shortcut triggered:', action);

      switch (action) {
        case 'screenshot-analyze':
          handlers.onScreenshotAnalyze?.();
          break;
        case 'toggle-bubbles':
          handlers.onToggleBubbles?.();
          break;
        case 'open-dashboard':
          handlers.onOpenDashboard?.();
          break;
        case 'dismiss-bubble':
          handlers.onDismissBubble?.();
          break;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [
    handlers.onScreenshotAnalyze,
    handlers.onToggleBubbles,
    handlers.onOpenDashboard,
    handlers.onDismissBubble,
  ]);

  /**
   * Trigger a shortcut action manually (useful for UI buttons)
   */
  const triggerAction = async (action: ShortcutAction) => {
    try {
      await invoke('trigger_shortcut_action', { action });
    } catch (e) {
      console.error('Failed to trigger shortcut action:', e);
    }
  };

  return {
    config,
    shortcuts,
    triggerAction,
  };
}

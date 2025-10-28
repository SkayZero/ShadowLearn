import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface WindowLifecycleOptions {
  onShow?: () => void;
  onHide?: () => void;
  onFocus?: () => void;
  onBlur?: () => void;
}

/**
 * Hook to manage window lifecycle events and hotkeys
 */
export const useWindowLifecycle = (options: WindowLifecycleOptions = {}) => {
  useEffect(() => {
    const window = getCurrentWindow();
    const unlistenPromises: Promise<() => void>[] = [];

    // Listen to window events
    if (options.onShow) {
      unlistenPromises.push(
        window.onFocusChanged(({ payload: focused }) => {
          if (focused && options.onShow) {
            options.onShow();
          }
        })
      );
    }

    if (options.onBlur) {
      unlistenPromises.push(
        window.onFocusChanged(({ payload: focused }) => {
          if (!focused && options.onBlur) {
            options.onBlur();
          }
        })
      );
    }

    // Cleanup
    return () => {
      Promise.all(unlistenPromises).then((unlisteners) => {
        unlisteners.forEach((unlisten) => unlisten());
      });
    };
  }, [options]);

  // Hotkeys for window management
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;

      // Cmd+Shift+1 → Toggle Chat
      if (mod && e.shiftKey && e.key === '1') {
        e.preventDefault();
        try {
          await invoke('toggle_window', { label: 'chat' });
        } catch (error) {
          console.error('Failed to toggle chat window:', error);
        }
      }

      // Cmd+Shift+2 → Toggle Context
      if (mod && e.shiftKey && e.key === '2') {
        e.preventDefault();
        try {
          await invoke('toggle_window', { label: 'context' });
        } catch (error) {
          console.error('Failed to toggle context window:', error);
        }
      }

      // Cmd+W → Hide current window (macOS convention)
      if (mod && e.key === 'w') {
        e.preventDefault();
        try {
          const window = getCurrentWindow();
          await window.hide();
        } catch (error) {
          console.error('Failed to hide window:', error);
        }
      }

      // REMOVED: Escape key handler - prevents accidental window closing
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);
};

export default useWindowLifecycle;


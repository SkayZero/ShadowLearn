import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface UseDesktopFocusOptions {
  enabled?: boolean;
  delay?: number;
}

const useDesktopFocus = (options: UseDesktopFocusOptions = {}) => {
  const { enabled = true, delay = 100 } = options;

  useEffect(() => {
    if (!enabled) return;

    let timeoutId: number;

    const handleVisibilityChange = async () => {
      if (document.hidden) {
        // Window is hidden, clear any pending focus
        clearTimeout(timeoutId);
      } else {
        // Window becomes visible, set up focus after delay
        timeoutId = setTimeout(async () => {
          try {
            const currentWindow = await getCurrentWindow();
            const label = currentWindow.label;
            
            // Focus the current window
            await invoke('focus_window', { label });
          } catch (error) {
            console.error('Failed to focus window:', error);
          }
        }, delay);
      }
    };

    const handleFocus = async () => {
      try {
        const currentWindow = await getCurrentWindow();
        const label = currentWindow.label;
        
        // Focus the current window
        await invoke('focus_window', { label });
      } catch (error) {
        console.error('Failed to focus window:', error);
      }
    };

    // Listen for visibility changes (when switching between apps)
    document.addEventListener('visibilitychange', handleVisibilityChange);
    
    // Listen for window focus events
    window.addEventListener('focus', handleFocus);

    return () => {
      clearTimeout(timeoutId);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('focus', handleFocus);
    };
  }, [enabled, delay]);
};

export default useDesktopFocus;

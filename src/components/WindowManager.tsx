import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface WindowManagerProps {
  children: React.ReactNode;
}

const WindowManager: React.FC<WindowManagerProps> = ({ children }) => {
  useEffect(() => {
    const handleFocus = async () => {
      try {
        const currentWindow = await getCurrentWindow();
        await invoke('show_window', { label: currentWindow.label });
      } catch (error) {
        console.error('Failed to show window:', error);
      }
    };

    // Listen for focus events to show window
    window.addEventListener('focus', handleFocus);
    document.addEventListener('visibilitychange', () => {
      if (!document.hidden) {
        handleFocus();
      }
    });

    return () => {
      window.removeEventListener('focus', handleFocus);
      document.removeEventListener('visibilitychange', handleFocus);
    };
  }, []);

  return <>{children}</>;
};

export default WindowManager;

import { useEffect } from 'react';

interface KeyboardShortcutsOptions {
  onToggleDock?: () => void;
  onOpenDigest?: () => void;
  onTogglePause?: () => void;
  onCloseModal?: () => void;
}

/**
 * Custom hook for keyboard shortcuts
 * Note: Global shortcuts (Cmd+Shift+*) require macOS accessibility permissions
 * These shortcuts only work when the window is focused
 */
export function useKeyboardShortcuts(options: KeyboardShortcutsOptions) {
  const {
    onToggleDock,
    onOpenDigest,
    onTogglePause,
    onCloseModal,
  } = options;

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Check if Cmd (meta) is pressed
      const isCmd = event.metaKey || event.ctrlKey;
      
      // Close modals with Esc
      if (event.key === 'Escape' && onCloseModal) {
        onCloseModal();
        return;
      }

      // Only register shortcuts when Cmd is held
      if (!isCmd) return;

      // Cmd+Shift+D -> Toggle Dock
      if (event.shiftKey && event.key === 'd' && onToggleDock) {
        event.preventDefault();
        onToggleDock();
        return;
      }

      // Cmd+Shift+S -> Open Digest
      if (event.shiftKey && event.key === 's' && onOpenDigest) {
        event.preventDefault();
        onOpenDigest();
        return;
      }

      // Cmd+Shift+P -> Toggle Pause
      if (event.shiftKey && event.key === 'p' && onTogglePause) {
        event.preventDefault();
        onTogglePause();
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [onToggleDock, onOpenDigest, onTogglePause, onCloseModal]);
}



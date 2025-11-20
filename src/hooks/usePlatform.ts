/**
 * Platform Detection Hook
 * Detects the OS platform and adds a CSS class to body
 */

import { useEffect } from 'react';
import { platform } from '@tauri-apps/plugin-os';

export function usePlatform() {
  useEffect(() => {
    const detectPlatform = async () => {
      try {
        const os = platform();

        // Add platform-specific class to body
        document.body.classList.add(`platform-${os}`);

        console.log(`[Platform] Detected: ${os}`);
      } catch (error) {
        console.error('[Platform] Detection failed:', error);
        // Fallback: assume not macOS if detection fails
        document.body.classList.add('platform-unknown');
      }
    };

    detectPlatform();
  }, []);
}

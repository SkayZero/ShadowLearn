/**
 * Platform Detection Hook
 * Detects the OS platform and adds a CSS class to body
 */

import { useEffect } from 'react';

export function usePlatform() {
  useEffect(() => {
    // Detect platform using navigator.userAgent
    // Tauri apps run in webview, userAgent contains OS info
    const userAgent = navigator.userAgent.toLowerCase();

    let os = 'unknown';
    if (userAgent.includes('mac')) {
      os = 'macos';
    } else if (userAgent.includes('win')) {
      os = 'windows';
    } else if (userAgent.includes('linux')) {
      os = 'linux';
    }

    // Add platform-specific class to body
    document.body.classList.add(`platform-${os}`);

    console.log(`[Platform] Detected: ${os} (UserAgent: ${userAgent})`);
  }, []);
}

/**
 * Platform Detection Hook
 * Detects OS and applies platform-specific CSS class
 */

import { useEffect } from 'react';

export type Platform = 'macos' | 'windows' | 'linux' | 'unknown';

export function usePlatform(): Platform {
  useEffect(() => {
    const userAgent = navigator.userAgent.toLowerCase();
    let platform: Platform = 'unknown';

    if (userAgent.includes('mac')) {
      platform = 'macos';
    } else if (userAgent.includes('win')) {
      platform = 'windows';
    } else if (userAgent.includes('linux')) {
      platform = 'linux';
    }

    // Add platform class to document element
    document.documentElement.classList.add(`platform-${platform}`);

    console.log(`[Platform] Detected: ${platform}`);

    return () => {
      // Cleanup on unmount
      document.documentElement.classList.remove(
        'platform-macos',
        'platform-windows',
        'platform-linux',
        'platform-unknown'
      );
    };
  }, []);

  // Return platform for immediate use
  const userAgent = navigator.userAgent.toLowerCase();
  if (userAgent.includes('mac')) return 'macos';
  if (userAgent.includes('win')) return 'windows';
  if (userAgent.includes('linux')) return 'linux';
  return 'unknown';
}

import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useEvent } from '../lib/eventBus';

/**
 * Screen change event payload from Rust backend
 */
export interface ScreenChange {
  timestamp: number;
  image_path: string;
  image_base64: string;
  analysis: string | null;
}

/**
 * Hook for managing screen monitoring
 *
 * Features:
 * - Start/stop monitoring
 * - Listen to screen-change events
 * - Get Claude Vision suggestions automatically
 *
 * @example
 * ```tsx
 * const { isMonitoring, startMonitoring, stopMonitoring, latestChange } = useScreenMonitor();
 *
 * useEffect(() => {
 *   if (latestChange?.analysis) {
 *     showNotification(latestChange.analysis);
 *   }
 * }, [latestChange]);
 * ```
 */
export function useScreenMonitor() {
  const [isMonitoring, setIsMonitoring] = useState(false);
  const [latestChange, setLatestChange] = useState<ScreenChange | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Listen to screen-change events from backend
  useEvent<ScreenChange>('screen-change', (change) => {
    console.log('📸 Screen change detected:', change);
    setLatestChange(change);

    // Auto-show suggestion if Claude Vision provided analysis
    if (change.analysis) {
      console.log('✨ Claude Vision suggestion:', change.analysis);

      // Emit custom event for other components to listen
      window.dispatchEvent(
        new CustomEvent('shadow:suggestion', {
          detail: {
            id: `screen-${change.timestamp}`,
            type: 'screen-monitor',
            text: change.analysis,
            timestamp: change.timestamp,
          },
        })
      );
    }
  });

  const startMonitoring = useCallback(async () => {
    try {
      setError(null);
      await invoke('start_screen_monitor');
      setIsMonitoring(true);
      console.log('🎬 Screen monitoring started');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      console.error('Failed to start screen monitoring:', err);
    }
  }, []);

  const stopMonitoring = useCallback(async () => {
    try {
      setError(null);
      await invoke('stop_screen_monitor');
      setIsMonitoring(false);
      console.log('🛑 Screen monitoring stopped');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      console.error('Failed to stop screen monitoring:', err);
    }
  }, []);

  const resetDetector = useCallback(async () => {
    try {
      setError(null);
      await invoke('reset_monitor_detector');
      console.log('🔄 Screen monitor detector reset');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      console.error('Failed to reset detector:', err);
    }
  }, []);

  const getStatus = useCallback(async (): Promise<boolean> => {
    try {
      const status = await invoke<boolean>('get_monitor_status');
      setIsMonitoring(status);
      return status;
    } catch (err) {
      console.error('Failed to get monitor status:', err);
      return false;
    }
  }, []);

  return {
    isMonitoring,
    latestChange,
    error,
    startMonitoring,
    stopMonitoring,
    resetDetector,
    getStatus,
  };
}

/**
 * Hook for displaying screen monitoring suggestions
 *
 * @example
 * ```tsx
 * const suggestion = useScreenSuggestion();
 *
 * if (suggestion) {
 *   return <SuggestionCard text={suggestion.text} onDismiss={() => {}} />;
 * }
 * ```
 */
export function useScreenSuggestion() {
  const [suggestion, setSuggestion] = useState<{
    id: string;
    type: string;
    text: string;
    timestamp: number;
  } | null>(null);

  useEvent<any>('shadow:suggestion', (payload) => {
    if (payload.type === 'screen-monitor') {
      setSuggestion(payload);
    }
  });

  const dismiss = useCallback(() => {
    setSuggestion(null);
  }, []);

  return {
    suggestion,
    dismiss,
  };
}

import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ActiveApp {
  bundle_id: string;
  name: string;
  window_title: string;
  pid: number;
  timestamp: number;
  tcc_status?: 'Granted' | 'Denied' | 'Unknown';
}

export interface Context {
  id: string;
  app: ActiveApp;
  clipboard: string | null;
  idle_seconds: number;
  timestamp: number;
  capture_duration_ms: number;
}

interface UseContextCaptureOptions {
  onCapture?: (context: Context) => void;
  onError?: (error: string) => void;
}

export function useContextCapture(options?: UseContextCaptureOptions) {
  const [context, setContext] = useState<Context | null>(null);
  const [isCapturing, setIsCapturing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const capture = useCallback(async () => {
    setIsCapturing(true);
    setError(null);

    try {
      const result = await invoke<Context>('capture_context');
      setContext(result);
      options?.onCapture?.(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      options?.onError?.(errorMessage);
      throw err;
    } finally {
      setIsCapturing(false);
    }
  }, [options]);

  return {
    context,
    capture,
    isCapturing,
    error,
  };
}


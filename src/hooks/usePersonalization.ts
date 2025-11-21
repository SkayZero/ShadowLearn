import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface UsagePatterns {
  favorite_apps: { [key: string]: AppUsageStats };
  productive_hours: number[];
  active_weekdays: number[];
  avg_idle_before_trigger: number;
  avg_response_time_ms: number;
  frequently_ignored_apps: { [key: string]: number };
  clipboard_patterns: { [key: string]: number };
}

export interface AppUsageStats {
  total_triggers: number;
  accepted_triggers: number;
  ignored_triggers: number;
  acceptance_rate: number;
  peak_hours: number[];
  last_used: string | null;
}

export interface SmartSuggestions {
  recommended_apps: string[];
  optimal_trigger_hour: number | null;
  recommended_thresholds: RecommendedThresholds;
  apps_to_mute: string[];
}

export interface RecommendedThresholds {
  idle_threshold: number;
  base_cooldown: number;
  dismiss_cooldown: number;
  debounce_threshold: number;
}

export function usePersonalization() {
  const [patterns, setPatterns] = useState<UsagePatterns | null>(null);
  const [suggestions, setSuggestions] = useState<SmartSuggestions | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPatterns = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const fetchedPatterns: UsagePatterns = await invoke('get_usage_patterns');
      setPatterns(fetchedPatterns);
    } catch (err) {
      console.error('Failed to fetch usage patterns:', err);
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const fetchSuggestions = useCallback(async () => {
    try {
      const fetchedSuggestions: SmartSuggestions = await invoke('get_smart_suggestions');
      setSuggestions(fetchedSuggestions);
    } catch (err) {
      console.error('Failed to fetch smart suggestions:', err);
      setError(err as string);
    }
  }, []);

  const recordEvent = useCallback(async (
    eventType: string,
    appName: string,
    context?: string,
    userResponse?: string
  ) => {
    try {
      await invoke('record_ml_event', {
        eventType,
        appName,
        context,
        userResponse,
      });
    } catch (err) {
      console.error('Failed to record ML event:', err);
    }
  }, []);

  const applySuggestions = useCallback(async (suggestions: SmartSuggestions) => {
    try {
      await invoke('apply_smart_suggestions', { suggestions });
    } catch (err) {
      console.error('Failed to apply smart suggestions:', err);
      setError(err as string);
    }
  }, []);

  const savePatterns = useCallback(async () => {
    try {
      await invoke('save_ml_patterns');
    } catch (err) {
      console.error('Failed to save ML patterns:', err);
      setError(err as string);
    }
  }, []);

  const loadPatterns = useCallback(async () => {
    try {
      await invoke('load_ml_patterns');
      await fetchPatterns(); // Refresh patterns after loading
    } catch (err) {
      console.error('Failed to load ML patterns:', err);
      setError(err as string);
    }
  }, [fetchPatterns]);

  useEffect(() => {
    fetchPatterns();
    fetchSuggestions();
    
    // Refresh patterns and suggestions every 30 seconds
    const interval = setInterval(() => {
      fetchPatterns();
      fetchSuggestions();
    }, 30000);
    
    return () => clearInterval(interval);
  }, [fetchPatterns, fetchSuggestions]);

  return {
    patterns,
    suggestions,
    isLoading,
    error,
    fetchPatterns,
    fetchSuggestions,
    recordEvent,
    applySuggestions,
    savePatterns,
    loadPatterns,
  };
}

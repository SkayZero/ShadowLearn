import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ProductivityMetrics {
  today: DayMetrics;
  week: WeekMetrics;
  trends: TrendData;
  insights: Insight[];
  flow_sessions: FlowSession[];
  top_productive_hours: ProductiveHour[];
}

export interface DayMetrics {
  date: string;
  suggestions_shown: number;
  suggestions_accepted: number;
  acceptance_rate: number;
  time_saved_minutes: number;
  flow_time_minutes: number;
  interruptions: number;
  top_apps: AppMetric[];
}

export interface WeekMetrics {
  week_number: number;
  total_suggestions: number;
  total_accepted: number;
  total_time_saved: number;
  total_flow_time: number;
  daily_breakdown: DayMetrics[];
  best_day: string | null;
  improvement_vs_last_week: number;
}

export interface TrendData {
  acceptance_rate_trend: TrendPoint[];
  flow_time_trend: TrendPoint[];
  productivity_score_trend: TrendPoint[];
}

export interface TrendPoint {
  date: string;
  value: number;
}

export interface Insight {
  id: string;
  category: 'Achievement' | 'Pattern' | 'Improvement' | 'Warning';
  title: string;
  description: string;
  impact: 'High' | 'Medium' | 'Low';
  action: string | null;
}

export interface FlowSession {
  start_time: number;
  end_time: number;
  duration_minutes: number;
  app_name: string;
  quality_score: number;
}

export interface ProductiveHour {
  hour: number;
  productivity_score: number;
  flow_sessions: number;
  acceptance_rate: number;
}

export interface AppMetric {
  name: string;
  usage_count: number;
  acceptance_rate: number;
  time_saved: number;
}

export function useProductivity(autoRefresh = false, refreshInterval = 30000) {
  const [metrics, setMetrics] = useState<ProductivityMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchMetrics = async () => {
    try {
      setLoading(true);
      const data = await invoke<ProductivityMetrics>('get_productivity_metrics');
      setMetrics(data);
      setError(null);
    } catch (err) {
      console.error('Failed to fetch productivity metrics:', err);
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const recordEvent = async (
    eventType: 'suggestion' | 'interruption',
    appName: string,
    accepted: boolean
  ) => {
    try {
      await invoke('record_productivity_event', {
        eventType,
        appName,
        accepted,
      });
      // Refresh metrics after recording
      await fetchMetrics();
    } catch (err) {
      console.error('Failed to record productivity event:', err);
    }
  };

  const recordFlowSession = async (
    appName: string,
    durationMinutes: number,
    qualityScore: number
  ) => {
    try {
      await invoke('record_flow_session_event', {
        appName,
        durationMinutes,
        qualityScore,
      });
      // Refresh metrics after recording
      await fetchMetrics();
    } catch (err) {
      console.error('Failed to record flow session:', err);
    }
  };

  useEffect(() => {
    fetchMetrics();

    if (autoRefresh) {
      const interval = setInterval(fetchMetrics, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [autoRefresh, refreshInterval]);

  return {
    metrics,
    loading,
    error,
    refresh: fetchMetrics,
    recordEvent,
    recordFlowSession,
  };
}

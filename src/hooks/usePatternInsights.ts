/**
 * Pattern Insights Hook
 * Phase 2.1 - Access to ML-learned workflow patterns and predictions
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface WorkflowPattern {
  id: string;
  name: string;
  sequence: ActionSignature[];
  occurrences: number;
  confidence: number;
  last_seen: string;
  created_at: string;
  avg_duration_secs: number;
  tags: string[];
}

export interface ActionSignature {
  app_name: string;
  action_type: ActionType;
  window_pattern?: string;
}

export type ActionType =
  | 'app_switch'
  | 'window_focus'
  | 'file_open'
  | 'file_save'
  | 'typing'
  | 'click'
  | 'scroll'
  | 'copy'
  | 'paste'
  | 'command'
  | { custom: string };

export interface Prediction {
  predicted_action: ActionSignature;
  confidence: number;
  reasoning: string;
  pattern_id?: string;
  alternative_predictions: AlternativePrediction[];
}

export interface AlternativePrediction {
  action: ActionSignature;
  confidence: number;
}

export interface PatternStats {
  total_actions_recorded: number;
  total_patterns_discovered: number;
  total_sequences_tracked: number;
  avg_pattern_confidence: number;
}

export interface PredictionStats {
  patterns_loaded: number;
  recent_actions_count: number;
  cached_predictions: number;
}

export interface RepetitionStats {
  total_tasks_detected: number;
  total_repetitions: number;
  total_time_wasted_mins: number;
  avg_automation_potential: number;
  high_priority_tasks: number;
}

export interface PatternSystemStats {
  learning: PatternStats;
  prediction: PredictionStats;
  repetition: RepetitionStats;
}

export interface UserAction {
  app_name: string;
  action_type: ActionType;
  window_title?: string;
  timestamp: number;
  context: Record<string, string>;
}

/**
 * Hook to access workflow patterns and predictions
 */
export function usePatternInsights() {
  const [patterns, setPatterns] = useState<WorkflowPattern[]>([]);
  const [prediction, setPrediction] = useState<Prediction | null>(null);
  const [stats, setStats] = useState<PatternSystemStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Record a user action
  const recordAction = useCallback(async (action: UserAction) => {
    try {
      await invoke('record_user_action', { action });
    } catch (err) {
      console.error('[Pattern] Failed to record action:', err);
      setError(String(err));
    }
  }, []);

  // Get next action prediction
  const getPrediction = useCallback(async () => {
    try {
      setIsLoading(true);
      const pred = await invoke<Prediction | null>('get_next_action_prediction');
      setPrediction(pred);
      setError(null);
      return pred;
    } catch (err) {
      console.error('[Pattern] Failed to get prediction:', err);
      setError(String(err));
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load all learned patterns
  const loadPatterns = useCallback(async () => {
    try {
      setIsLoading(true);
      const pats = await invoke<WorkflowPattern[]>('get_learned_patterns');
      setPatterns(pats);
      setError(null);
      return pats;
    } catch (err) {
      console.error('[Pattern] Failed to load patterns:', err);
      setError(String(err));
      return [];
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Get patterns by tag
  const getPatternsByTag = useCallback(async (tag: string) => {
    try {
      const pats = await invoke<WorkflowPattern[]>('get_patterns_by_tag', { tag });
      return pats;
    } catch (err) {
      console.error('[Pattern] Failed to get patterns by tag:', err);
      setError(String(err));
      return [];
    }
  }, []);

  // Load statistics
  const loadStats = useCallback(async () => {
    try {
      const st = await invoke<PatternSystemStats>('get_pattern_system_stats');
      setStats(st);
      return st;
    } catch (err) {
      console.error('[Pattern] Failed to load stats:', err);
      setError(String(err));
      return null;
    }
  }, []);

  // Save patterns to disk
  const savePatterns = useCallback(async () => {
    try {
      await invoke('save_patterns_to_disk');
      console.log('[Pattern] ✅ Patterns saved to disk');
    } catch (err) {
      console.error('[Pattern] Failed to save patterns:', err);
      setError(String(err));
    }
  }, []);

  // Clear all patterns
  const clearPatterns = useCallback(async () => {
    try {
      await invoke('clear_pattern_storage');
      setPatterns([]);
      setPrediction(null);
      setStats(null);
      console.log('[Pattern] 🗑️ Patterns cleared');
    } catch (err) {
      console.error('[Pattern] Failed to clear patterns:', err);
      setError(String(err));
    }
  }, []);

  // Load patterns on mount
  useEffect(() => {
    loadPatterns();
    loadStats();
  }, [loadPatterns, loadStats]);

  // Auto-refresh prediction periodically (every 30s)
  useEffect(() => {
    const interval = setInterval(() => {
      getPrediction();
    }, 30000);

    return () => clearInterval(interval);
  }, [getPrediction]);

  return {
    patterns,
    prediction,
    stats,
    isLoading,
    error,
    recordAction,
    getPrediction,
    loadPatterns,
    getPatternsByTag,
    loadStats,
    savePatterns,
    clearPatterns,
  };
}

/**
 * Hook to access repetitive task detection
 */
export function useRepetitiveTaskDetection() {
  const [tasks, setTasks] = useState<RepetitiveTask[]>([]);
  const [highPriorityTasks, setHighPriorityTasks] = useState<RepetitiveTask[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadTasks = useCallback(async () => {
    try {
      setIsLoading(true);
      const allTasks = await invoke<RepetitiveTask[]>('get_all_repetitive_tasks');
      const highPriority = await invoke<RepetitiveTask[]>('get_high_priority_repetitive_tasks');
      setTasks(allTasks);
      setHighPriorityTasks(highPriority);
      setError(null);
      return { allTasks, highPriority };
    } catch (err) {
      console.error('[Repetition] Failed to load tasks:', err);
      setError(String(err));
      return { allTasks: [], highPriority: [] };
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load tasks on mount
  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  // Auto-refresh every 5 minutes
  useEffect(() => {
    const interval = setInterval(() => {
      loadTasks();
    }, 300000);

    return () => clearInterval(interval);
  }, [loadTasks]);

  return {
    tasks,
    highPriorityTasks,
    isLoading,
    error,
    loadTasks,
  };
}

export interface RepetitiveTask {
  id: string;
  name: string;
  actions: ActionSignature[];
  repetitions: number;
  last_occurrence: string;
  first_seen: string;
  avg_interval_mins: number;
  automation_potential: number;
  automation_suggestion: string;
  time_wasted_mins: number;
}

/**
 * Helper to format action type for display
 */
export function formatActionType(actionType: ActionType): string {
  if (typeof actionType === 'string') {
    switch (actionType) {
      case 'app_switch':
        return 'Switch App';
      case 'window_focus':
        return 'Focus Window';
      case 'file_open':
        return 'Open File';
      case 'file_save':
        return 'Save File';
      case 'typing':
        return 'Type';
      case 'click':
        return 'Click';
      case 'scroll':
        return 'Scroll';
      case 'copy':
        return 'Copy';
      case 'paste':
        return 'Paste';
      case 'command':
        return 'Command';
      default:
        return actionType;
    }
  } else if (typeof actionType === 'object' && 'custom' in actionType) {
    return actionType.custom;
  }
  return 'Unknown';
}

/**
 * Helper to format duration
 */
export function formatDuration(seconds: number): string {
  if (seconds < 60) {
    return `${Math.round(seconds)}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  return remainingSeconds > 0 ? `${minutes}m ${remainingSeconds}s` : `${minutes}m`;
}

/**
 * Helper to format confidence as percentage
 */
export function formatConfidence(confidence: number): string {
  return `${Math.round(confidence * 100)}%`;
}

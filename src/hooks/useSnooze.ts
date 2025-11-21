import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export type SnoozeDuration = '30min' | '2h' | 'today';

export interface SnoozeStatus {
  snoozedUntil: number | null; // Unix timestamp
  isSnoozed: boolean;
  remainingMinutes: number | null;
}

/**
 * Hook React pour gérer le système de snooze
 */
export function useSnooze() {
  const [status, setStatus] = useState<SnoozeStatus>({
    snoozedUntil: null,
    isSnoozed: false,
    remainingMinutes: null,
  });

  const fetchStatus = async () => {
    try {
      const snoozedUntil = await invoke<number | null>('get_snooze_status');
      
      if (snoozedUntil) {
        const now = Math.floor(Date.now() / 1000);
        const remaining = snoozedUntil - now;
        
        setStatus({
          snoozedUntil,
          isSnoozed: remaining > 0,
          remainingMinutes: remaining > 0 ? Math.ceil(remaining / 60) : null,
        });
      } else {
        setStatus({
          snoozedUntil: null,
          isSnoozed: false,
          remainingMinutes: null,
        });
      }
    } catch (e) {
      console.error('Failed to fetch snooze status:', e);
    }
  };

  useEffect(() => {
    fetchStatus();
    
    // Poll every 30s to update remaining time
    const interval = setInterval(fetchStatus, 30000);
    
    return () => clearInterval(interval);
  }, []);

  const snooze = async (duration: SnoozeDuration) => {
    try {
      await invoke('snooze_triggers', { duration });
      await fetchStatus();
    } catch (e) {
      console.error('Failed to snooze:', e);
      throw e;
    }
  };

  const unsnooze = async () => {
    try {
      await invoke('unsnooze_triggers');
      await fetchStatus();
    } catch (e) {
      console.error('Failed to unsnooze:', e);
      throw e;
    }
  };

  return {
    status,
    snooze,
    unsnooze,
    refresh: fetchStatus,
  };
}


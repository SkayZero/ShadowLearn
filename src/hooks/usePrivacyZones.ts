import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface PrivacyZonesConfig {
  enabled: boolean;
  zones: PrivacyZone[];
  blur_instead_of_skip: boolean;
}

export type PrivacyZone =
  | {
      type: 'rectangle';
      x: number;
      y: number;
      width: number;
      height: number;
      label: string;
    }
  | {
      type: 'window';
      app_name: string;
      fuzzy_match: boolean;
    }
  | {
      type: 'region';
      region: PredefinedRegion;
    };

export type PredefinedRegion = 'top-bar' | 'taskbar' | 'system-tray' | 'dock';

/**
 * Hook pour gérer les zones de confidentialité
 *
 * @example
 * ```tsx
 * const { config, addZone, removeZone, setEnabled, isAppProtected } = usePrivacyZones();
 *
 * // Add a privacy zone
 * addZone({
 *   type: 'window',
 *   app_name: 'Banking App',
 *   fuzzy_match: false,
 * });
 *
 * // Check if app is protected
 * const protected = await isAppProtected('1Password');
 * ```
 */
export function usePrivacyZones() {
  const [config, setConfig] = useState<PrivacyZonesConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const loadConfig = async () => {
    try {
      const cfg = await invoke<PrivacyZonesConfig>('get_privacy_zones_config');
      setConfig(cfg);
    } catch (e) {
      console.error('Failed to load privacy zones config:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadConfig();
  }, []);

  const addZone = async (zone: PrivacyZone) => {
    try {
      await invoke('add_privacy_zone', { zone });
      await loadConfig(); // Reload config
    } catch (e) {
      console.error('Failed to add privacy zone:', e);
      throw e;
    }
  };

  const removeZone = async (zone: PrivacyZone) => {
    try {
      const removed = await invoke<boolean>('remove_privacy_zone', { zone });
      if (removed) {
        await loadConfig(); // Reload config
      }
      return removed;
    } catch (e) {
      console.error('Failed to remove privacy zone:', e);
      throw e;
    }
  };

  const setEnabled = async (enabled: boolean) => {
    try {
      await invoke('set_privacy_zones_enabled', { enabled });
      await loadConfig(); // Reload config
    } catch (e) {
      console.error('Failed to set privacy zones enabled:', e);
      throw e;
    }
  };

  const isAppProtected = async (appName: string): Promise<boolean> => {
    try {
      return await invoke<boolean>('is_app_protected', { appName });
    } catch (e) {
      console.error('Failed to check if app is protected:', e);
      return false;
    }
  };

  return {
    config,
    loading,
    addZone,
    removeZone,
    setEnabled,
    isAppProtected,
    reload: loadConfig,
  };
}

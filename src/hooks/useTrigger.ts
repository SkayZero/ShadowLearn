import { useEffect, useRef, useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface Context {
  id: string;
  app: {
    name: string;
    bundle_id: string;
    window_title: string;
  };
  clipboard: string | null;
  idle_seconds: number;
  timestamp: number;
  capture_duration_ms: number;
}

export interface TriggerStats {
  total_triggers: number;
  triggers_per_app: Record<string, number>;
  current_cooldown_ms: number | null;
  allowlist: string[];
  cooldown_base_ms: number;
  cooldown_dismiss_ms: number;
}

/**
 * Hook React pour gérer les triggers proactifs avec SmartBubble
 * @param onTrigger Callback appelé quand un trigger est déclenché
 * @param autoStart Si true, démarre automatiquement la boucle de trigger
 * @param enableSmartPositioning Si true, utilise le SmartBubble pour le positionnement
 */
export function useTrigger(
  onTrigger: (ctx: Context) => void, 
  autoStart = true,
  enableSmartPositioning = true
) {
  const hasStarted = useRef(false);
  const [triggerContext, setTriggerContext] = useState<Context | null>(null);
  const [showBubble, setShowBubble] = useState(false);
  const lastTriggerTime = useRef<number>(0);
  
  useEffect(() => {
    const setupListener = async () => {
      // Start trigger loop if autoStart (only once)
      if (autoStart && !hasStarted.current) {
        hasStarted.current = true;
        try {
          await invoke('start_trigger_loop');
          console.log('✅ Trigger loop started');
        } catch (e) {
          console.error('❌ Failed to start trigger loop:', e);
        }
      }

      const unlisten = await listen<Context>('trigger_fired', (event) => {
        const ctx = event.payload;
        console.log('🔔 Trigger fired:', ctx.app.name);
        
        // Anti-spam: cooldown de 30s sur le frontend
        const now = Date.now();
        if (now - lastTriggerTime.current < 30000) {
          console.log('🚫 Trigger ignoré (cooldown frontend 30s)');
          return;
        }
        lastTriggerTime.current = now;
        
        // Marquer la bulle comme visible côté backend
        if (enableSmartPositioning) {
          invoke('set_bubble_visible', { visible: true }).catch(console.error);
        }
        
        // Afficher la bulle avec SmartBubble
        if (enableSmartPositioning) {
          setTriggerContext(ctx);
          setShowBubble(true);
        }
        
        // Appeler le callback
        onTrigger(ctx);
      });

      return unlisten;
    };

    let unlistenFn: (() => void) | null = null;
    
    setupListener().then((fn) => {
      unlistenFn = fn;
    });

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [onTrigger, autoStart, enableSmartPositioning]);

  // Fonctions pour gérer la bulle
  const hideBubble = useCallback(() => {
    setShowBubble(false);
    setTriggerContext(null);
    
    // Marquer la bulle comme invisible côté backend
    if (enableSmartPositioning) {
      invoke('set_bubble_visible', { visible: false }).catch(console.error);
    }
  }, [enableSmartPositioning]);

  const handleUserInteraction = useCallback(() => {
    // Enregistrer l'interaction utilisateur (verrou 45s)
    invoke('record_user_interaction').catch(console.error);
    hideBubble();
  }, [hideBubble]);

  return {
    triggerContext,
    showBubble,
    hideBubble,
    handleUserInteraction,
    enableSmartPositioning,
  };
}

/**
 * Démarre manuellement la boucle de trigger
 */
export async function startTriggerLoop(): Promise<void> {
  await invoke('start_trigger_loop');
  console.log('✅ Trigger loop started');
}

/**
 * Enregistre qu'une bulle a été dismiss sans action
 */
export async function dismissBubble(): Promise<void> {
  await invoke('record_bubble_dismissed');
  console.log('❌ Bubble dismissed');
}

/**
 * Enregistre qu'un utilisateur a agi
 */
export async function recordUserAction(): Promise<void> {
  await invoke('record_user_action');
  console.log('✅ User action recorded');
}

/**
 * Récupère les statistiques de triggers
 */
export async function getTriggerStats(): Promise<TriggerStats> {
  return await invoke<TriggerStats>('get_trigger_stats');
}

/**
 * Ajoute une app à l'allowlist
 */
export async function addToAllowlist(appName: string): Promise<void> {
  await invoke('add_to_allowlist', { appName });
  console.log(`➕ Added '${appName}' to allowlist`);
}

/**
 * Retire une app de l'allowlist
 */
export async function removeFromAllowlist(appName: string): Promise<void> {
  await invoke('remove_from_allowlist', { appName });
  console.log(`➖ Removed '${appName}' from allowlist`);
}


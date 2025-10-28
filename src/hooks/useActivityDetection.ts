import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

type ActivityType = 'keyboard' | 'mouse' | 'scroll';

/**
 * Hook pour détecter l'activité utilisateur et reset l'idle timer
 * @param enabled Si true, écoute les événements clavier/souris
 * @param debounceMs Délai de throttle pour éviter trop d'appels (défaut: 300ms pour souris, 500ms pour clavier)
 */
export function useActivityDetection(enabled = true) {
  const lastResetTime = useRef<Record<ActivityType, number>>({
    keyboard: 0,
    mouse: 0,
    scroll: 0,
  });

  useEffect(() => {
    if (!enabled) return;

    const resetActivity = (activityType: ActivityType, throttleMs: number) => {
      const now = Date.now();
      if (now - lastResetTime.current[activityType] < throttleMs) {
        return; // Throttle
      }
      
      lastResetTime.current[activityType] = now;
      
      // Appel au backend pour reset l'idle timer avec type
      invoke('reset_user_activity', { activityType }).catch((e) => {
        console.warn(`Failed to reset user activity (${activityType}):`, e);
      });
    };

    // Handlers par type d'activité
    const handleKeyboard = () => resetActivity('keyboard', 500); // 500ms throttle
    const handleMouse = () => resetActivity('mouse', 300); // 300ms throttle
    const handleScroll = () => resetActivity('scroll', 300); // 300ms throttle

    // Événements clavier
    const keyboardEvents = ['keydown', 'keyup'];
    // Événements souris
    const mouseEvents = ['mousedown', 'mousemove', 'click', 'contextmenu', 'touchstart', 'touchmove'];
    // Événements scroll
    const scrollEvents = ['scroll', 'wheel'];

    // Ajouter les listeners
    keyboardEvents.forEach(event => {
      document.addEventListener(event, handleKeyboard, { passive: true });
      window.addEventListener(event, handleKeyboard, { passive: true });
    });

    mouseEvents.forEach(event => {
      document.addEventListener(event, handleMouse, { passive: true });
      window.addEventListener(event, handleMouse, { passive: true });
    });

    scrollEvents.forEach(event => {
      document.addEventListener(event, handleScroll, { passive: true });
      window.addEventListener(event, handleScroll, { passive: true });
    });

    // Cleanup
    return () => {
      keyboardEvents.forEach(event => {
        document.removeEventListener(event, handleKeyboard);
        window.removeEventListener(event, handleKeyboard);
      });

      mouseEvents.forEach(event => {
        document.removeEventListener(event, handleMouse);
        window.removeEventListener(event, handleMouse);
      });

      scrollEvents.forEach(event => {
        document.removeEventListener(event, handleScroll);
        window.removeEventListener(event, handleScroll);
      });
    };
  }, [enabled]);
}

export default useActivityDetection;

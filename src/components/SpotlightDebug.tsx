/**
 * Debug button to test Spotlight window
 * Temporary component for testing - can be removed later
 */

import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function SpotlightDebug() {
  const [status, setStatus] = useState<string>('');

  const handleToggleSpotlight = async () => {
    console.log('🔍 [DEBUG] Clicking Toggle Spotlight button...');
    setStatus('⏳ Appel toggle_spotlight...');

    try {
      console.log('🔍 [DEBUG] Invoking toggle_spotlight command...');
      const isVisible = await invoke<boolean>('toggle_spotlight');
      console.log('🔍 [DEBUG] toggle_spotlight returned:', isVisible);

      setStatus(isVisible ? '✅ Spotlight ouvert' : '🔒 Spotlight fermé');
      setTimeout(() => setStatus(''), 2000);
    } catch (error) {
      console.error('❌ [DEBUG] Failed to toggle spotlight:', error);
      setStatus(`❌ Erreur: ${error}`);
      setTimeout(() => setStatus(''), 3000);
    }
  };

  const handleTestHUDState = async () => {
    try {
      const { emit } = await import('@tauri-apps/api/event');
      // Test different HUD states
      await emit('hud:state-change', { state: 'opportunity', count: 3 });
      setStatus('💡 HUD → État "opportunity" (3)');
      setTimeout(() => setStatus(''), 2000);
    } catch (error) {
      console.error('Failed to change HUD state:', error);
      setStatus(`❌ Erreur: ${error}`);
    }
  };

  const handleGetShortcuts = async () => {
    try {
      const shortcuts = await invoke<any>('get_shortcuts_config');
      console.log('📋 Shortcuts config:', shortcuts);
      setStatus(`⌨️ Spotlight: ${shortcuts.toggle_spotlight}`);
      setTimeout(() => setStatus(''), 3000);
    } catch (error) {
      console.error('Failed to get shortcuts:', error);
    }
  };

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '80px',
        right: '20px',
        padding: '12px',
        background: 'rgba(0, 0, 0, 0.8)',
        borderRadius: '8px',
        border: '1px solid rgba(255, 255, 255, 0.2)',
        display: 'flex',
        flexDirection: 'column',
        gap: '8px',
        zIndex: 9999,
      }}
    >
      <div
        style={{
          fontSize: '11px',
          color: '#87CEEB',
          fontWeight: 'bold',
          marginBottom: '4px',
        }}
      >
        🔧 DEBUG SPOTLIGHT
      </div>

      <button
        onClick={handleToggleSpotlight}
        style={{
          padding: '6px 12px',
          background: 'linear-gradient(135deg, #87CEEB, #4682B4)',
          border: 'none',
          borderRadius: '6px',
          color: 'white',
          fontSize: '12px',
          fontWeight: '600',
          cursor: 'pointer',
        }}
      >
        Toggle Spotlight
      </button>

      <button
        onClick={handleTestHUDState}
        style={{
          padding: '6px 12px',
          background: 'linear-gradient(135deg, #FACC15, #EAB308)',
          border: 'none',
          borderRadius: '6px',
          color: 'white',
          fontSize: '12px',
          fontWeight: '600',
          cursor: 'pointer',
        }}
      >
        Test HUD State
      </button>

      <button
        onClick={handleGetShortcuts}
        style={{
          padding: '6px 12px',
          background: 'rgba(255, 255, 255, 0.1)',
          border: '1px solid rgba(255, 255, 255, 0.3)',
          borderRadius: '6px',
          color: 'white',
          fontSize: '12px',
          fontWeight: '600',
          cursor: 'pointer',
        }}
      >
        Show Shortcuts
      </button>

      {status && (
        <div
          style={{
            fontSize: '11px',
            color: '#fff',
            padding: '4px 8px',
            background: 'rgba(255, 255, 255, 0.1)',
            borderRadius: '4px',
            marginTop: '4px',
          }}
        >
          {status}
        </div>
      )}
    </div>
  );
}

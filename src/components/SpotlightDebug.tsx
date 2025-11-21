/**
 * Spotlight Configuration & Debug Panel
 * Allows user to configure shortcut and test Spotlight
 */

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function SpotlightDebug() {
  const [status, setStatus] = useState<string>('');
  const [currentShortcut, setCurrentShortcut] = useState<string>('Cmd+J');

  // Load current shortcut on mount
  useEffect(() => {
    loadCurrentShortcut();
  }, []);

  const loadCurrentShortcut = async () => {
    try {
      const config = await invoke<any>('get_shortcuts_config');
      setCurrentShortcut(config.toggle_spotlight);
      console.log('📋 Current shortcut:', config.toggle_spotlight);
    } catch (error) {
      console.error('Failed to load shortcut config:', error);
    }
  };

  const handleToggleSpotlight = async () => {
    console.log('🔍 [DEBUG] Toggle Spotlight...');
    setStatus('⏳ Ouverture...');

    try {
      const isVisible = await invoke<boolean>('toggle_spotlight');
      setStatus(isVisible ? '✅ Spotlight ouvert' : '🔒 Spotlight fermé');
      setTimeout(() => setStatus(''), 2000);
    } catch (error) {
      console.error('❌ [DEBUG] Failed:', error);
      setStatus(`❌ Erreur: ${error}`);
      setTimeout(() => setStatus(''), 3000);
    }
  };

  const handleTestHUDClick = async () => {
    setStatus('🔍 Test du HUD...');
    console.log('[DEBUG] Simulating HUD click...');

    // Simulate what the HUD does
    try {
      const isVisible = await invoke<boolean>('toggle_spotlight');
      setStatus(isVisible ? '✅ HUD → Spotlight ouvert' : '🔒 HUD → fermé');
      setTimeout(() => setStatus(''), 2000);
    } catch (error) {
      console.error('[DEBUG] HUD test failed:', error);
      setStatus(`❌ Erreur: ${error}`);
    }
  };

  const suggestionsList = [
    { key: 'Cmd+J', desc: 'Simple (peut conflictuer)' },
    { key: 'Cmd+Shift+Space', desc: 'Triple modificateur (rarement pris)' },
    { key: 'Cmd+Option+L', desc: 'Option+Cmd combo' },
    { key: 'Cmd+Shift+;', desc: 'Avec caractère spécial' },
    { key: 'F13', desc: 'Touche fonction (si dispo)' },
  ];

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '80px',
        right: '20px',
        padding: '14px',
        background: 'rgba(0, 0, 0, 0.85)',
        borderRadius: '10px',
        border: '1px solid rgba(255, 255, 255, 0.2)',
        display: 'flex',
        flexDirection: 'column',
        gap: '10px',
        zIndex: 9999,
        minWidth: '240px',
        maxWidth: '300px',
      }}
    >
      {/* Header */}
      <div
        style={{
          fontSize: '12px',
          color: '#87CEEB',
          fontWeight: 'bold',
          borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
          paddingBottom: '8px',
        }}
      >
        🔧 SETUP SPOTLIGHT
      </div>

      {/* Current Shortcut Display */}
      <div
        style={{
          fontSize: '11px',
          color: 'rgba(255, 255, 255, 0.7)',
          padding: '8px',
          background: 'rgba(255, 255, 255, 0.05)',
          borderRadius: '6px',
          border: '1px solid rgba(255, 255, 255, 0.1)',
        }}
      >
        <div style={{ marginBottom: '4px', fontWeight: '600', color: '#fff' }}>
          Raccourci actuel:
        </div>
        <div style={{ fontSize: '13px', color: '#87CEEB', fontWeight: 'bold' }}>
          {currentShortcut}
        </div>
        <div style={{ fontSize: '10px', color: 'rgba(255, 255, 255, 0.5)', marginTop: '4px' }}>
          ⚠️ Ce raccourci peut être déjà pris par une autre app
        </div>
      </div>

      {/* Test Buttons */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
        <button
          onClick={handleToggleSpotlight}
          style={{
            padding: '8px 12px',
            background: 'linear-gradient(135deg, #87CEEB, #4682B4)',
            border: 'none',
            borderRadius: '6px',
            color: 'white',
            fontSize: '12px',
            fontWeight: '600',
            cursor: 'pointer',
          }}
        >
          ▶ Test Spotlight
        </button>

        <button
          onClick={handleTestHUDClick}
          style={{
            padding: '8px 12px',
            background: 'linear-gradient(135deg, #10b981, #059669)',
            border: 'none',
            borderRadius: '6px',
            color: 'white',
            fontSize: '12px',
            fontWeight: '600',
            cursor: 'pointer',
          }}
        >
          ▶ Test HUD Click
        </button>
      </div>

      {/* Shortcut Suggestions */}
      <div style={{ marginTop: '8px' }}>
        <div
          style={{
            fontSize: '11px',
            color: 'rgba(255, 255, 255, 0.7)',
            marginBottom: '6px',
            fontWeight: '600',
          }}
        >
          💡 Suggestions de raccourcis:
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
          {suggestionsList.map((suggestion) => (
            <div
              key={suggestion.key}
              style={{
                fontSize: '10px',
                padding: '6px 8px',
                background: 'rgba(255, 255, 255, 0.05)',
                borderRadius: '4px',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                cursor: 'pointer',
              }}
              onClick={() => {
                setStatus(`💡 ${suggestion.key} copié`);
                setTimeout(() => setStatus(''), 2000);
              }}
            >
              <code style={{ color: '#87CEEB', fontWeight: '600' }}>
                {suggestion.key}
              </code>
              <span style={{ color: 'rgba(255, 255, 255, 0.5)', fontSize: '9px' }}>
                {suggestion.desc}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Note */}
      <div
        style={{
          fontSize: '9px',
          color: 'rgba(255, 255, 255, 0.4)',
          marginTop: '4px',
          fontStyle: 'italic',
          borderTop: '1px solid rgba(255, 255, 255, 0.1)',
          paddingTop: '8px',
        }}
      >
        ℹ️ Pour configurer: modifie src-tauri/src/shortcuts/manager.rs ligne 25
      </div>

      {/* Status */}
      {status && (
        <div
          style={{
            fontSize: '11px',
            color: '#fff',
            padding: '6px 10px',
            background: 'rgba(135, 206, 235, 0.2)',
            borderRadius: '6px',
            marginTop: '4px',
            border: '1px solid rgba(135, 206, 235, 0.3)',
          }}
        >
          {status}
        </div>
      )}
    </div>
  );
}

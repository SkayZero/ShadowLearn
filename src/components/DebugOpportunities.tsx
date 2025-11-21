/**
 * Debug Panel for Phase 3A Testing
 * Allows manual triggering of mock opportunities
 * TO BE REMOVED after Phase 3A validation
 */

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

type OpportunityType = 'refacto' | 'debug' | 'learn' | 'tip';

export function DebugOpportunities() {
  const [loading, setLoading] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);
  const [lastSuccess, setLastSuccess] = useState<string | null>(null);

  const triggerOpportunity = async (type: OpportunityType) => {
    setLoading(true);
    setLastError(null);
    setLastSuccess(null);

    try {
      await invoke('trigger_mock_opportunity', { opportunityType: type });
      setLastSuccess(`✅ Triggered ${type} opportunity`);
      console.log(`[Debug] Triggered mock opportunity: ${type}`);

      // Auto-clear success message
      setTimeout(() => setLastSuccess(null), 3000);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setLastError(`❌ Failed: ${errorMsg}`);
      console.error('[Debug] Failed to trigger opportunity:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '20px',
        right: '20px',
        padding: '16px',
        background: 'rgba(0, 0, 0, 0.9)',
        border: '2px solid #10b981',
        borderRadius: '12px',
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
        zIndex: 9999,
        minWidth: '280px',
      }}
    >
      <div
        style={{
          marginBottom: '12px',
          fontSize: '14px',
          fontWeight: '700',
          color: '#10b981',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
        }}
      >
        🧪 Phase 3A Debug
      </div>

      <p
        style={{
          margin: '0 0 12px 0',
          fontSize: '12px',
          color: '#888',
          lineHeight: '1.4',
        }}
      >
        Trigger mock opportunities to test HUD → Spotlight → Actions flow
      </p>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        <button
          onClick={() => triggerOpportunity('refacto')}
          disabled={loading}
          style={{
            padding: '10px 16px',
            background: loading ? '#374151' : '#3b82f6',
            border: 'none',
            borderRadius: '8px',
            color: 'white',
            fontSize: '13px',
            fontWeight: '600',
            cursor: loading ? 'not-allowed' : 'pointer',
            transition: 'all 0.2s',
            opacity: loading ? 0.5 : 1,
          }}
        >
          🔧 Refacto Pattern
        </button>

        <button
          onClick={() => triggerOpportunity('debug')}
          disabled={loading}
          style={{
            padding: '10px 16px',
            background: loading ? '#374151' : '#ef4444',
            border: 'none',
            borderRadius: '8px',
            color: 'white',
            fontSize: '13px',
            fontWeight: '600',
            cursor: loading ? 'not-allowed' : 'pointer',
            transition: 'all 0.2s',
            opacity: loading ? 0.5 : 1,
          }}
        >
          🐛 Debug Assistance
        </button>

        <button
          onClick={() => triggerOpportunity('learn')}
          disabled={loading}
          style={{
            padding: '10px 16px',
            background: loading ? '#374151' : '#8b5cf6',
            border: 'none',
            borderRadius: '8px',
            color: 'white',
            fontSize: '13px',
            fontWeight: '600',
            cursor: loading ? 'not-allowed' : 'pointer',
            transition: 'all 0.2s',
            opacity: loading ? 0.5 : 1,
          }}
        >
          📚 Learning Tip
        </button>

        <button
          onClick={() => triggerOpportunity('tip')}
          disabled={loading}
          style={{
            padding: '10px 16px',
            background: loading ? '#374151' : '#10b981',
            border: 'none',
            borderRadius: '8px',
            color: 'white',
            fontSize: '13px',
            fontWeight: '600',
            cursor: loading ? 'not-allowed' : 'pointer',
            transition: 'all 0.2s',
            opacity: loading ? 0.5 : 1,
          }}
        >
          💡 Quick Tip
        </button>
      </div>

      {lastSuccess && (
        <div
          style={{
            marginTop: '12px',
            padding: '8px 12px',
            background: 'rgba(16, 185, 129, 0.2)',
            border: '1px solid #10b981',
            borderRadius: '6px',
            fontSize: '12px',
            color: '#10b981',
          }}
        >
          {lastSuccess}
        </div>
      )}

      {lastError && (
        <div
          style={{
            marginTop: '12px',
            padding: '8px 12px',
            background: 'rgba(239, 68, 68, 0.2)',
            border: '1px solid #ef4444',
            borderRadius: '6px',
            fontSize: '12px',
            color: '#ef4444',
          }}
        >
          {lastError}
        </div>
      )}

      <div
        style={{
          marginTop: '12px',
          paddingTop: '12px',
          borderTop: '1px solid rgba(255, 255, 255, 0.1)',
          fontSize: '11px',
          color: '#666',
          textAlign: 'center',
        }}
      >
        Cmd+Shift+Y to open Spotlight
      </div>
    </div>
  );
}

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface TriggerState {
  type: string;
  data: any;
}

export function StatusIndicator() {
  const [state, setState] = useState<TriggerState | null>(null);
  const [explanation, setExplanation] = useState<string>("");
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    const updateStatus = async () => {
      try {
        const currentState = await invoke<TriggerState>('get_trigger_state');
        const explain = await invoke<string>('get_state_explanation');
        setState(currentState);
        setExplanation(explain);
      } catch (e) {
        console.error('Failed to get state:', e);
      }
    };

    updateStatus();
    const interval = setInterval(updateStatus, 1000);
    return () => clearInterval(interval);
  }, []);

  if (!state) return null;

  return (
    <div
      style={{
        position: 'fixed',
        top: '60px', // Below header
        left: '16px',
        maxWidth: '320px',
        zIndex: 45, // Below input field
        backgroundColor: 'var(--glass-bg)',
        backdropFilter: 'var(--glass-backdrop)',
        WebkitBackdropFilter: 'var(--glass-backdrop)',
        borderRadius: '8px',
        border: '1px solid var(--glass-border)',
        boxShadow: 'var(--glass-shadow)',
        cursor: 'pointer',
        transition: 'all 0.2s ease',
      }}
      onClick={() => setExpanded(!expanded)}
    >
      {/* Compact view - Notification style */}
      <div style={{ padding: '10px 12px', display: 'flex', alignItems: 'center', gap: '8px' }}>
        <div style={{ fontSize: '16px' }}>{getStateIcon(state.type)}</div>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div
            style={{
              fontSize: '12px',
              fontWeight: '500',
              color: 'var(--text-primary)',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {explanation}
          </div>
          {state.type === 'Cooldown' && state.data?.remaining_seconds && (
            <div style={{ fontSize: '10px', color: 'var(--text-muted)' }}>
              {state.data.remaining_seconds}s restantes
            </div>
          )}
        </div>
        <div
          style={{
            color: 'var(--text-muted)',
            fontSize: '10px',
            transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
            transition: 'transform 0.2s',
          }}
        >
          ▼
        </div>
      </div>

      {/* Expanded view */}
      {expanded && (
        <div
          style={{
            borderTop: '1px solid rgba(255,255,255,0.1)',
            maxHeight: '200px',
            overflowY: 'auto',
            padding: '0 12px 12px 12px',
          }}
        >
          <StateDetails state={state} />
        </div>
      )}
    </div>
  );
}

function getStateIcon(stateType: string): string {
  const icons: Record<string, string> = {
    Observing: "👀",
    IdleDetected: "⏱️",
    ContextConfirmed: "🎯",
    PromptShown: "💬",
    UserResponded: "✅",
    Cooldown: "⏸️",
  };

  return icons[stateType] || "❓";
}

function StateDetails({ state }: { state: TriggerState }) {
  const [history, setHistory] = useState<any[]>([]);

  useEffect(() => {
    const loadHistory = async () => {
      const h = await invoke<any[]>('get_state_history', { limit: 5 });
      setHistory(h);
    };
    loadHistory();
  }, [state]);

  return (
    <div style={{ padding: '12px', display: 'flex', flexDirection: 'column', gap: '8px', fontSize: '12px', color: 'white' }}>
      <div style={{ fontWeight: '600', marginBottom: '8px' }}>Détails de l'état</div>

      {/* Current state details */}
      <div
        style={{
          backgroundColor: 'rgba(255,255,255,0.05)',
          padding: '8px',
          borderRadius: '6px',
          fontSize: '11px',
          fontFamily: 'monospace',
          overflow: 'auto',
        }}
      >
        <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
          {JSON.stringify(state, null, 2)}
        </pre>
      </div>

      {/* History */}
      <div style={{ fontWeight: '600', marginTop: '12px', marginBottom: '8px' }}>
        Historique récent
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
        {history.map((transition, i) => (
          <div
            key={i}
            style={{
              fontSize: '11px',
              padding: '6px',
              backgroundColor: 'rgba(255,255,255,0.05)',
              borderRadius: '4px',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '2px' }}>
              <span style={{ fontFamily: 'monospace', fontSize: '10px' }}>
                {new Date(transition.timestamp * 1000).toLocaleTimeString()}
              </span>
            </div>
            <div>{transition.explanation}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

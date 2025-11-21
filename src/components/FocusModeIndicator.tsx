import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';

interface FocusState {
  is_in_focus: boolean;
  focus_start_time: number | null;
  focus_duration_minutes: number;
  notifications_blocked: number;
  focus_quality_score: number;
  current_app: string;
}

export function FocusModeIndicator() {
  const [focusState, setFocusState] = useState<FocusState | null>(null);
  const [showDetails, setShowDetails] = useState(false);

  useEffect(() => {
    const checkFocus = async () => {
      try {
        const state = await invoke<FocusState>('get_focus_state');
        setFocusState(state);
      } catch (error) {
        console.error('Failed to get focus state:', error);
      }
    };

    checkFocus();
    const interval = setInterval(checkFocus, 5000);

    return () => clearInterval(interval);
  }, []);

  const endFocus = async () => {
    try {
      await invoke('end_focus_session');
      setFocusState(null);
    } catch (error) {
      console.error('Failed to end focus:', error);
    }
  };

  if (!focusState?.is_in_focus) {
    return null;
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        style={{
          position: 'fixed',
          top: '16px',
          right: '16px',
          background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.9), rgba(124, 58, 237, 0.9))',
          backdropFilter: 'blur(12px)',
          borderRadius: '16px',
          padding: showDetails ? '20px' : '12px 16px',
          boxShadow: '0 8px 32px rgba(139, 92, 246, 0.4)',
          border: '1px solid rgba(255, 255, 255, 0.2)',
          zIndex: 1000,
          cursor: 'pointer',
          minWidth: showDetails ? '280px' : 'auto',
        }}
        onClick={() => !showDetails && setShowDetails(true)}
      >
        {!showDetails ? (
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <motion.div
              animate={{ scale: [1, 1.2, 1] }}
              transition={{ repeat: Infinity, duration: 2 }}
              style={{ fontSize: '20px' }}
            >
              🧘
            </motion.div>
            <div>
              <div style={{ fontSize: '14px', fontWeight: '600', color: 'white' }}>
                Focus Mode
              </div>
              <div style={{ fontSize: '11px', color: 'rgba(255, 255, 255, 0.8)' }}>
                {focusState.focus_duration_minutes}min • {focusState.notifications_blocked} blocked
              </div>
            </div>
          </div>
        ) : (
          <div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '16px' }}>
              <div>
                <div style={{ fontSize: '18px', fontWeight: '700', color: 'white', marginBottom: '4px' }}>
                  🧘 Focus Mode
                </div>
                <div style={{ fontSize: '12px', color: 'rgba(255, 255, 255, 0.8)' }}>
                  {focusState.current_app}
                </div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setShowDetails(false);
                }}
                style={{
                  background: 'transparent',
                  border: 'none',
                  color: 'white',
                  fontSize: '20px',
                  cursor: 'pointer',
                  padding: '0',
                }}
              >
                ×
              </button>
            </div>

            <div style={{ marginBottom: '16px' }}>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
                <StatBox
                  label="Duration"
                  value={`${focusState.focus_duration_minutes}min`}
                />
                <StatBox
                  label="Quality"
                  value={`${Math.round(focusState.focus_quality_score * 100)}%`}
                />
                <StatBox
                  label="Blocked"
                  value={focusState.notifications_blocked}
                />
                <StatBox
                  label="Deep Work"
                  value={focusState.focus_duration_minutes >= 25 ? '✅' : '⏳'}
                />
              </div>
            </div>

            <button
              onClick={(e) => {
                e.stopPropagation();
                endFocus();
              }}
              style={{
                width: '100%',
                padding: '10px',
                background: 'rgba(255, 255, 255, 0.2)',
                border: 'none',
                borderRadius: '8px',
                color: 'white',
                fontWeight: '600',
                cursor: 'pointer',
                fontSize: '14px',
              }}
            >
              End Focus
            </button>
          </div>
        )}
      </motion.div>
    </AnimatePresence>
  );
}

function StatBox({ label, value }: { label: string; value: string | number }) {
  return (
    <div
      style={{
        padding: '12px',
        background: 'rgba(255, 255, 255, 0.15)',
        borderRadius: '8px',
        textAlign: 'center',
      }}
    >
      <div style={{ fontSize: '18px', fontWeight: '700', color: 'white' }}>
        {value}
      </div>
      <div style={{ fontSize: '10px', color: 'rgba(255, 255, 255, 0.7)', marginTop: '2px', textTransform: 'uppercase' }}>
        {label}
      </div>
    </div>
  );
}

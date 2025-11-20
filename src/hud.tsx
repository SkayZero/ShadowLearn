/**
 * HUD Indicator Window
 * Small persistent indicator in corner of screen
 * Shows opportunity status with color-coded states
 */

import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { motion } from 'framer-motion';
import { useTheme } from './contexts/ThemeContext';
import { ThemeProvider } from './contexts/ThemeContext';
import './styles/island-globals.css';

type HUDState = 'idle' | 'opportunity' | 'urgent';

function HUDIndicator() {
  const { theme } = useTheme();
  const [state, setState] = useState<HUDState>('idle');
  const [opportunityCount, setOpportunityCount] = useState(0);

  useEffect(() => {
    const setupListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');

      // Listen for HUD state changes
      const unlistenState = await listen<{ state: HUDState; count?: number }>(
        'hud:state-change',
        (event) => {
          console.log('[HUD] State change:', event.payload);
          setState(event.payload.state);
          if (event.payload.count !== undefined) {
            setOpportunityCount(event.payload.count);
          }
        }
      );

      return () => {
        unlistenState();
      };
    };

    setupListeners();
  }, []);

  const handleClick = async () => {
    console.log('[HUD] Clicked - opening Spotlight');
    const { emit } = await import('@tauri-apps/api/event');

    // Emit event to show Spotlight
    await emit('hud:click', {});

    // Backend will handle showing the Spotlight window
  };

  // Color scheme based on state
  const getStateColors = () => {
    switch (state) {
      case 'idle':
        return {
          bg: 'rgba(74, 222, 128, 0.3)', // Green
          border: 'rgba(74, 222, 128, 0.6)',
          glow: 'rgba(74, 222, 128, 0.4)',
          emoji: '✓',
        };
      case 'opportunity':
        return {
          bg: 'rgba(250, 204, 21, 0.3)', // Yellow
          border: 'rgba(250, 204, 21, 0.6)',
          glow: 'rgba(250, 204, 21, 0.5)',
          emoji: '💡',
        };
      case 'urgent':
        return {
          bg: 'rgba(239, 68, 68, 0.3)', // Red
          border: 'rgba(239, 68, 68, 0.6)',
          glow: 'rgba(239, 68, 68, 0.5)',
          emoji: '🔥',
        };
    }
  };

  const colors = getStateColors();
  const isPulsing = state === 'opportunity' || state === 'urgent';

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'transparent',
        cursor: 'pointer',
      }}
      onClick={handleClick}
    >
      <motion.div
        animate={
          isPulsing
            ? {
                scale: [1, 1.1, 1],
                boxShadow: [
                  `0 0 0 0 ${colors.glow}`,
                  `0 0 20px 10px ${colors.glow}`,
                  `0 0 0 0 ${colors.glow}`,
                ],
              }
            : {}
        }
        transition={
          isPulsing
            ? {
                duration: 2,
                repeat: Infinity,
                ease: 'easeInOut',
              }
            : {}
        }
        whileHover={{ scale: 1.15 }}
        whileTap={{ scale: 0.95 }}
        style={{
          width: '60px',
          height: '60px',
          borderRadius: '50%',
          background: `linear-gradient(135deg, ${colors.bg}, rgba(0, 0, 0, 0.1))`,
          backdropFilter: 'blur(10px)',
          WebkitBackdropFilter: 'blur(10px)',
          border: `2px solid ${colors.border}`,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: '2px',
          transition: 'all 0.2s',
          position: 'relative',
        }}
      >
        {/* Emoji indicator */}
        <span
          style={{
            fontSize: '24px',
            filter: 'drop-shadow(0 2px 4px rgba(0,0,0,0.3))',
          }}
        >
          {colors.emoji}
        </span>

        {/* Opportunity count badge */}
        {opportunityCount > 0 && (
          <motion.div
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            style={{
              position: 'absolute',
              top: '-4px',
              right: '-4px',
              width: '20px',
              height: '20px',
              borderRadius: '50%',
              background: 'linear-gradient(135deg, #ef4444, #dc2626)',
              border: '2px solid white',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '11px',
              fontWeight: '700',
              color: 'white',
              boxShadow: '0 2px 8px rgba(0,0,0,0.3)',
            }}
          >
            {opportunityCount > 9 ? '9+' : opportunityCount}
          </motion.div>
        )}
      </motion.div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <HUDIndicator />
    </ThemeProvider>
  </React.StrictMode>
);

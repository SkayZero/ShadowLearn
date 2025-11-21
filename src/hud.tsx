/**
 * HUD Indicator Window
 * "Luciole dans la nuit" - Ambient LED indicator
 * Shows opportunity status with theme-adapted colors
 * Draggable with position saved per-app
 */

import React, { useState, useEffect, useRef } from 'react';
import ReactDOM from 'react-dom/client';
import { motion } from 'framer-motion';
import { useTheme } from './contexts/ThemeContext';
import { ThemeProvider } from './contexts/ThemeContext';
import { hexToRgba } from './utils/helpers';
import './styles/island-globals.css';

type HUDState = 'normal' | 'opportunity' | 'blocked';

function HUDIndicator() {
  const { theme } = useTheme();
  const [state, setState] = useState<HUDState>('normal');
  const [opportunityCount, setOpportunityCount] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const [showFeedback, setShowFeedback] = useState(false);
  const lastClickRef = useRef<number>(0);

  useEffect(() => {
    console.log('[HUD] Component mounted, setting up listeners...');

    const setupListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');

      // Listen for HUD state changes
      const unlistenState = await listen<{ state: HUDState; count?: number }>(
        'hud:state-change',
        (event) => {
          console.log('[HUD] State change event received:', event.payload);
          setState(event.payload.state);
          if (event.payload.count !== undefined) {
            setOpportunityCount(event.payload.count);
          }
        }
      );

      // Listen for HUD pulse events (from opportunity triggers)
      const unlistenPulse = await listen<{ state: HUDState }>(
        'hud:pulse',
        (event) => {
          console.log('🔔 [HUD] Pulse event received:', event.payload);
          setState(event.payload.state);
          setOpportunityCount((prev) => prev + 1);

          // Auto-reset to normal after 30 seconds if no action
          setTimeout(() => {
            console.log('[HUD] Auto-reset to normal after 30s');
            setState('normal');
          }, 30000);
        }
      );

      console.log('[HUD] Event listeners registered successfully');

      return () => {
        console.log('[HUD] Cleaning up listeners');
        unlistenState();
        unlistenPulse();
      };
    };

    setupListeners();
  }, []);

  const handleMouseDown = async (e: React.MouseEvent) => {
    // Detect double-click
    const now = Date.now();
    const timeSinceLastClick = now - lastClickRef.current;

    if (timeSinceLastClick < 300) {
      // Double-click detected!
      e.preventDefault();
      e.stopPropagation();

      // Show feedback animation
      setShowFeedback(true);
      setTimeout(() => setShowFeedback(false), 200);

      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke<boolean>('toggle_spotlight');
      } catch (error) {
        console.error('[HUD] Failed to toggle spotlight:', error);
      }

      lastClickRef.current = 0; // Reset
      return;
    }

    lastClickRef.current = now;

    // Single click: start drag
    setIsDragging(true);

    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      await window.startDragging();
    } catch (error) {
      console.error('[HUD] Failed to start dragging:', error);
    } finally {
      setIsDragging(false);
    }
  };

  // Get colors from theme based on state
  const getStateColors = () => {
    switch (state) {
      case 'normal':
        return {
          color: theme.led.normal,
          opacity: 0.25,
          glowStrength: 0.3,
          pulseSpeed: 0, // No pulse
        };
      case 'opportunity':
        return {
          color: theme.led.normal,
          opacity: 0.5,
          glowStrength: 0.5,
          pulseSpeed: 2, // Slow pulse
        };
      case 'blocked':
        return {
          color: theme.led.blocked,
          opacity: 0.7,
          glowStrength: 0.7,
          pulseSpeed: 1.5, // Faster pulse
        };
    }
  };

  const colors = getStateColors();
  const isPulsing = colors.pulseSpeed > 0;

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'transparent',
        cursor: isDragging ? 'grabbing' : 'grab',
      }}
      onMouseDown={handleMouseDown}
    >
      {/* Ambient LED Ring */}
      <motion.div
        animate={
          isPulsing
            ? {
                scale: [1, 1.05, 1],
                opacity: [colors.opacity, colors.opacity * 1.2, colors.opacity],
              }
            : {}
        }
        transition={
          isPulsing
            ? {
                duration: colors.pulseSpeed,
                repeat: Infinity,
                ease: 'easeInOut',
              }
            : {}
        }
        style={{
          width: '60px',
          height: '60px',
          borderRadius: '50%',
          position: 'relative',
          transition: 'all 0.3s ease',
        }}
      >
        {/* Outer glow ring */}
        <motion.div
          animate={
            isPulsing
              ? {
                  boxShadow: [
                    `0 0 10px 2px ${hexToRgba(colors.color, colors.glowStrength * 0.5)}`,
                    `0 0 20px 4px ${hexToRgba(colors.color, colors.glowStrength)}`,
                    `0 0 10px 2px ${hexToRgba(colors.color, colors.glowStrength * 0.5)}`,
                  ],
                }
              : {}
          }
          transition={
            isPulsing
              ? {
                  duration: colors.pulseSpeed,
                  repeat: Infinity,
                  ease: 'easeInOut',
                }
              : {}
          }
          style={{
            position: 'absolute',
            inset: 0,
            borderRadius: '50%',
            background: `radial-gradient(circle, ${hexToRgba(colors.color, colors.opacity)}, transparent 70%)`,
            boxShadow: `0 0 15px 3px ${hexToRgba(colors.color, colors.glowStrength * 0.6)}`,
          }}
        />

        {/* Inner ring */}
        <div
          style={{
            position: 'absolute',
            inset: '8px',
            borderRadius: '50%',
            border: `2px solid ${hexToRgba(colors.color, colors.opacity * 1.5)}`,
            background: `radial-gradient(circle, ${hexToRgba(colors.color, colors.opacity * 0.3)}, transparent)`,
          }}
        />

        {/* Center dot */}
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            width: '12px',
            height: '12px',
            borderRadius: '50%',
            background: hexToRgba(colors.color, colors.opacity * 2),
            boxShadow: `0 0 8px 2px ${hexToRgba(colors.color, colors.glowStrength)}`,
          }}
        />

        {/* Double-click feedback */}
        {showFeedback && (
          <motion.div
            initial={{ scale: 0.8, opacity: 1 }}
            animate={{ scale: 1.3, opacity: 0 }}
            transition={{ duration: 0.2 }}
            style={{
              position: 'absolute',
              inset: '-10px',
              borderRadius: '50%',
              border: `3px solid ${hexToRgba(colors.color, 0.8)}`,
              pointerEvents: 'none',
            }}
          />
        )}

        {/* Opportunity count badge */}
        {opportunityCount > 0 && (
          <motion.div
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            style={{
              position: 'absolute',
              top: '-6px',
              right: '-6px',
              minWidth: '22px',
              height: '22px',
              borderRadius: '11px',
              background: `linear-gradient(135deg, ${theme.led.blocked}, ${hexToRgba(theme.led.blocked, 0.8)})`,
              border: `2px solid ${theme.glass.bg}`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              padding: '0 6px',
              fontSize: '11px',
              fontWeight: '700',
              color: theme.text.primary,
              boxShadow: `0 2px 8px ${hexToRgba(theme.led.blocked, 0.5)}`,
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

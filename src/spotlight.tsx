/**
 * Spotlight Window
 * Global overlay that appears with Cmd+Shift+L to show opportunities
 * Inspired by macOS Spotlight
 */

import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { motion, AnimatePresence } from 'framer-motion';
import { useTheme } from './contexts/ThemeContext';
import { ThemeProvider } from './contexts/ThemeContext';
import type { Opportunity } from './lib';
import './styles/island-globals.css';

function SpotlightWindow() {
  const { theme } = useTheme();
  const [opportunity, setOpportunity] = useState<Opportunity | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const setupListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');

      // Listen for show/hide events
      const unlistenShow = await listen('spotlight:show', (event: any) => {
        console.log('[Spotlight] Show event received:', event.payload);
        if (event.payload?.opportunity) {
          setOpportunity(event.payload.opportunity);
        }
        setIsVisible(true);
      });

      const unlistenHide = await listen('spotlight:hide', () => {
        console.log('[Spotlight] Hide event received');
        setIsVisible(false);
      });

      return () => {
        unlistenShow();
        unlistenHide();
      };
    };

    setupListeners();
  }, []);

  useEffect(() => {
    // Listen for Escape key to close
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleClose();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleClose = async () => {
    setIsVisible(false);
    const window = getCurrentWindow();
    await window.hide();
  };

  const handleDiscuss = async () => {
    console.log('[Spotlight] Discuss clicked');
    // TODO: Open chat with opportunity context
    handleClose();
  };

  const handleView = async () => {
    console.log('[Spotlight] View clicked');
    // TODO: Show more details
    handleClose();
  };

  const handleIgnore = async () => {
    console.log('[Spotlight] Ignore clicked');
    if (opportunity) {
      // TODO: Record ignored opportunity
    }
    handleClose();
  };

  // Mock opportunity for testing
  const mockOpportunity: Opportunity = {
    id: 'mock-1',
    type: 'refacto',
    suggestion: 'Tu répètes ce pattern 3 fois. Je peux te suggérer une factorisation.',
    context: {
      app_name: 'Cursor',
      file: 'main.rs',
      lines: [42, 89, 134],
    },
    confidence: 0.85,
    created_at: Date.now(),
  };

  const displayOpportunity = opportunity || mockOpportunity;

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'transparent',
      }}
      onClick={handleClose}
    >
      <AnimatePresence>
        {isVisible && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9, y: -20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: -20 }}
            transition={{ type: 'spring', damping: 25, stiffness: 300 }}
            onClick={(e) => e.stopPropagation()}
            style={{
              width: '480px',
              maxHeight: '400px',
              background: `linear-gradient(135deg, ${theme.glass.bg}, rgba(30, 41, 59, 0.95))`,
              backdropFilter: 'blur(20px)',
              WebkitBackdropFilter: 'blur(20px)',
              border: `1px solid ${theme.glass.border}`,
              borderRadius: '16px',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
              padding: '24px',
              overflow: 'auto',
            }}
          >
            {/* Header */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                marginBottom: '20px',
              }}
            >
              <span style={{ fontSize: '24px' }}>🔍</span>
              <h2
                style={{
                  fontSize: '18px',
                  fontWeight: '700',
                  color: theme.text.primary,
                  margin: 0,
                }}
              >
                ShadowLearn
              </h2>
            </div>

            {/* Opportunity Badge */}
            <div
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '6px',
                padding: '4px 12px',
                background: 'rgba(135, 206, 235, 0.2)',
                border: '1px solid rgba(135, 206, 235, 0.4)',
                borderRadius: '12px',
                marginBottom: '16px',
              }}
            >
              <span style={{ fontSize: '16px' }}>💡</span>
              <span
                style={{
                  fontSize: '12px',
                  fontWeight: '600',
                  color: theme.accent,
                  textTransform: 'uppercase',
                  letterSpacing: '0.5px',
                }}
              >
                Opportunité détectée
              </span>
            </div>

            {/* Suggestion */}
            <p
              style={{
                fontSize: '15px',
                lineHeight: '1.6',
                color: theme.text.primary,
                marginBottom: '16px',
              }}
            >
              {displayOpportunity.suggestion}
            </p>

            {/* Context (if available) */}
            {displayOpportunity.context && typeof displayOpportunity.context === 'object' && (
              <div
                style={{
                  fontSize: '13px',
                  color: theme.text.secondary,
                  padding: '12px',
                  background: 'rgba(255, 255, 255, 0.05)',
                  borderRadius: '8px',
                  marginBottom: '20px',
                }}
              >
                <strong>📍 Contexte :</strong>{' '}
                {displayOpportunity.context.app_name || 'App'}
                {displayOpportunity.context.file && ` — ${displayOpportunity.context.file}`}
                {displayOpportunity.context.lines && ` (lignes ${displayOpportunity.context.lines.join(', ')})`}
              </div>
            )}

            {/* Actions */}
            <div
              style={{
                display: 'flex',
                gap: '10px',
                marginTop: '20px',
              }}
            >
              <button
                onClick={handleView}
                style={{
                  flex: 1,
                  padding: '12px 20px',
                  background: `linear-gradient(135deg, ${theme.accent}, ${theme.accentLight})`,
                  border: 'none',
                  borderRadius: '10px',
                  color: 'white',
                  fontWeight: '600',
                  fontSize: '14px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-2px)';
                  e.currentTarget.style.boxShadow = '0 4px 12px rgba(135, 206, 235, 0.4)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.boxShadow = 'none';
                }}
              >
                ✓ Voir
              </button>

              <button
                onClick={handleDiscuss}
                style={{
                  flex: 1,
                  padding: '12px 20px',
                  background: 'rgba(255, 255, 255, 0.1)',
                  border: `1px solid ${theme.glass.border}`,
                  borderRadius: '10px',
                  color: theme.text.primary,
                  fontWeight: '600',
                  fontSize: '14px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.15)';
                  e.currentTarget.style.transform = 'translateY(-2px)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.1)';
                  e.currentTarget.style.transform = 'translateY(0)';
                }}
              >
                💬 Discuter
              </button>

              <button
                onClick={handleIgnore}
                style={{
                  padding: '12px 20px',
                  background: 'transparent',
                  border: `1px solid ${theme.glass.border}`,
                  borderRadius: '10px',
                  color: theme.text.muted,
                  fontWeight: '500',
                  fontSize: '14px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = theme.text.primary;
                  e.currentTarget.style.color = theme.text.primary;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = theme.glass.border;
                  e.currentTarget.style.color = theme.text.muted;
                }}
              >
                ✕
              </button>
            </div>

            {/* Hint */}
            <p
              style={{
                marginTop: '16px',
                fontSize: '12px',
                color: theme.text.muted,
                textAlign: 'center',
              }}
            >
              Appuyez sur <kbd style={{ padding: '2px 6px', background: 'rgba(255,255,255,0.1)', borderRadius: '4px' }}>Esc</kbd> pour fermer
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <SpotlightWindow />
    </ThemeProvider>
  </React.StrictMode>
);

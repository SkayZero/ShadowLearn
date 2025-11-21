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
  useTheme(); // Keep theme sync
  const [opportunity, setOpportunity] = useState<Opportunity | null>(null);
  const [isVisible, setIsVisible] = useState(false); // Hidden by default

  useEffect(() => {
    const setupListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');
      const window = getCurrentWindow();

      // Listen for window visibility changes
      const checkVisibility = async () => {
        const visible = await window.isVisible();
        console.log('🔍 [Spotlight] Window visibility:', visible);
        setIsVisible(visible);
      };

      // Check initial visibility
      checkVisibility();

      // Listen for show/hide events
      const unlistenShow = await listen('spotlight:show', (event: any) => {
        console.log('🔍 [Spotlight] Show event received:', event.payload);
        if (event.payload?.opportunity) {
          setOpportunity(event.payload.opportunity);
        }
        setIsVisible(true);
      });

      const unlistenHide = await listen('spotlight:hide', () => {
        console.log('🔍 [Spotlight] Hide event received');
        setIsVisible(false);
      });

      // Listen for window focus changes (more reliable for detecting visibility)
      const unlistenFocus = await window.listen('tauri://focus', () => {
        console.log('🔍 [Spotlight] Window focused - showing content');
        setIsVisible(true);
      });

      const unlistenBlur = await window.listen('tauri://blur', () => {
        console.log('🔍 [Spotlight] Window blurred');
        // Don't auto-hide on blur, let user control it
      });

      return () => {
        unlistenShow();
        unlistenHide();
        unlistenFocus();
        unlistenBlur();
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
    title: 'Suggestion de refactorisation',
    preview: 'Tu répètes ce pattern 3 fois. Je peux te suggérer une factorisation.',
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
        alignItems: 'flex-start', // Top alignment for 20% positioning
        justifyContent: 'center',
        paddingTop: '20vh', // 20% from top like macOS Spotlight
        background: 'transparent', // No backdrop dimming - user wants to see app behind
      }}
      onClick={handleClose}
    >
      <AnimatePresence>
        {isVisible && (
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: -10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: -10 }}
            transition={{ type: 'spring', damping: 30, stiffness: 400 }}
            onClick={(e) => e.stopPropagation()}
            style={{
              width: '600px',
              height: '500px',
              background: 'var(--glass-bg)',
              backdropFilter: 'var(--glass-backdrop)',
              WebkitBackdropFilter: 'var(--glass-backdrop)',
              border: '1px solid var(--glass-border)',
              borderRadius: '24px',
              boxShadow: 'var(--glass-shadow)',
              padding: '32px',
              overflow: 'hidden',
              display: 'flex',
              flexDirection: 'column',
            }}
          >
            {/* Header */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                marginBottom: '24px',
                paddingBottom: '16px',
                borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
              }}
            >
              <span style={{ fontSize: '28px' }}>🔍</span>
              <h2
                style={{
                  fontSize: '20px',
                  fontWeight: '700',
                  color: 'var(--text-primary)',
                  margin: 0,
                  letterSpacing: '-0.5px',
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
                gap: '8px',
                padding: '6px 14px',
                background: 'linear-gradient(135deg, rgba(135, 206, 235, 0.25), rgba(16, 185, 129, 0.25))',
                border: '1px solid var(--accent-light)',
                borderRadius: '12px',
                marginBottom: '20px',
              }}
            >
              <span style={{ fontSize: '18px' }}>💡</span>
              <span
                style={{
                  fontSize: '13px',
                  fontWeight: '600',
                  color: 'var(--accent-primary)',
                  textTransform: 'uppercase',
                  letterSpacing: '0.5px',
                }}
              >
                Opportunité détectée
              </span>
            </div>

            {/* Content Area - flex grow to take available space */}
            <div
              style={{
                flex: '1 1 auto',
                display: 'flex',
                flexDirection: 'column',
                minHeight: 0, // Allow shrinking
              }}
            >
              {/* Suggestion */}
              <p
                style={{
                  fontSize: '16px',
                  lineHeight: '1.6',
                  color: 'var(--text-primary)',
                  marginBottom: '20px',
                  fontWeight: '400',
                }}
              >
                {displayOpportunity.suggestion}
              </p>

              {/* Context (if available) */}
              {displayOpportunity.context && typeof displayOpportunity.context === 'object' && (
                <div
                  style={{
                    fontSize: '14px',
                    color: 'var(--text-secondary)',
                    padding: '14px 16px',
                    background: 'rgba(255, 255, 255, 0.05)',
                    borderRadius: '10px',
                    marginBottom: '20px',
                    borderLeft: '3px solid var(--accent-primary)',
                  }}
                >
                  <strong style={{ color: 'var(--accent-primary)' }}>📍 Contexte</strong>
                  <div style={{ marginTop: '6px', fontSize: '13px' }}>
                    {displayOpportunity.context.app_name || 'App'}
                    {displayOpportunity.context.file && ` • ${displayOpportunity.context.file}`}
                    {displayOpportunity.context.lines && ` • Lignes ${displayOpportunity.context.lines.join(', ')}`}
                  </div>
                </div>
              )}
            </div>

            {/* Actions - fixed at bottom */}
            <div
              style={{
                display: 'flex',
                gap: '12px',
                marginTop: 'auto',
              }}
            >
              <button
                onClick={handleView}
                style={{
                  flex: 1,
                  padding: '14px 24px',
                  background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-light))',
                  border: 'none',
                  borderRadius: '12px',
                  color: 'white',
                  fontWeight: '600',
                  fontSize: '15px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                  boxShadow: '0 2px 8px rgba(0, 0, 0, 0.15)',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = 'translateY(-1px)';
                  e.currentTarget.style.boxShadow = '0 4px 16px rgba(135, 206, 235, 0.35)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = 'translateY(0)';
                  e.currentTarget.style.boxShadow = '0 2px 8px rgba(0, 0, 0, 0.15)';
                }}
              >
                ✓ Voir
              </button>

              <button
                onClick={handleDiscuss}
                style={{
                  flex: 1,
                  padding: '14px 24px',
                  background: 'rgba(255, 255, 255, 0.08)',
                  border: '1px solid var(--glass-border)',
                  borderRadius: '12px',
                  color: 'var(--text-primary)',
                  fontWeight: '600',
                  fontSize: '15px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.15)';
                  e.currentTarget.style.transform = 'translateY(-1px)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.08)';
                  e.currentTarget.style.transform = 'translateY(0)';
                }}
              >
                💬 Discuter
              </button>

              <button
                onClick={handleIgnore}
                style={{
                  padding: '14px 20px',
                  background: 'transparent',
                  border: '1px solid var(--glass-border)',
                  borderRadius: '12px',
                  color: 'var(--text-muted)',
                  fontWeight: '500',
                  fontSize: '15px',
                  cursor: 'pointer',
                  transition: 'all 0.2s ease',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = 'var(--text-primary)';
                  e.currentTarget.style.color = 'var(--text-primary)';
                  e.currentTarget.style.background = 'rgba(255, 255, 255, 0.05)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = 'var(--glass-border)';
                  e.currentTarget.style.color = 'var(--text-muted)';
                  e.currentTarget.style.background = 'transparent';
                }}
              >
                ✕
              </button>
            </div>

            {/* Hint */}
            <p
              style={{
                marginTop: '16px',
                fontSize: '13px',
                color: 'var(--text-muted)',
                textAlign: 'center',
                flexShrink: 0, // Don't shrink hint
              }}
            >
              Appuyez sur <kbd style={{ padding: '3px 8px', background: 'rgba(255,255,255,0.1)', borderRadius: '6px', fontSize: '12px', fontWeight: '600' }}>Esc</kbd> pour fermer
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

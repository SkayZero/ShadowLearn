/**
 * Spotlight Window
 * Global overlay that appears with Cmd+Shift+L to show opportunities
 * Inspired by macOS Spotlight
 */

import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { motion, AnimatePresence } from 'framer-motion';
import { useTheme } from './contexts/ThemeContext';
import { ThemeProvider } from './contexts/ThemeContext';
import { OpportunityProvider, useOpportunities } from './contexts/OpportunityContext';
import './styles/island-globals.css';

function SpotlightWindow() {
  useTheme(); // Keep theme sync
  const { latestOpportunity, markAsViewed, markAsActioned, markAsIgnored } = useOpportunities();
  const [isVisible, setIsVisible] = useState(false); // Hidden by default
  const [viewModal, setViewModal] = useState(false); // For "Voir" modal

  useEffect(() => {
    const setupListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');
      const window = getCurrentWindow();

      // Listen for show/hide events
      const unlistenShow = await listen('spotlight:show', () => {
        setIsVisible(true);
      });

      const unlistenHide = await listen('spotlight:hide', () => {
        setIsVisible(false);
      });

      // Listen for window focus changes (more reliable for detecting visibility)
      const unlistenFocus = await window.listen('tauri://focus', () => {
        setIsVisible(true);
      });

      const unlistenBlur = await window.listen('tauri://blur', () => {
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
    setViewModal(false);
    const window = getCurrentWindow();
    await window.hide();
  };

  const handleDiscuss = async () => {
    if (!latestOpportunity) return;

    // Mark as actioned
    markAsActioned(latestOpportunity.id);

    // Show chat window
    try {
      await invoke('show_window', { window_label: 'chat' });
    } catch (err) {
      console.error('Failed to show chat window:', err);
    }

    // Emit chat:prefill event with opportunity context
    try {
      await emit('chat:prefill', {
        opportunityId: latestOpportunity.id,
        context: latestOpportunity,
      });
    } catch (err) {
      console.error('Failed to emit chat:prefill:', err);
    }

    handleClose();
  };

  const handleView = async () => {
    if (!latestOpportunity) return;

    // Mark as viewed
    markAsViewed(latestOpportunity.id);

    // Show modal (don't close Spotlight)
    setViewModal(true);
  };

  const handleIgnore = async () => {
    if (!latestOpportunity) return;

    // Mark as ignored
    markAsIgnored(latestOpportunity.id);

    handleClose();
  };

  // Get type emoji
  const getTypeEmoji = (type: string) => {
    switch (type) {
      case 'refacto': return '🔧';
      case 'debug': return '🐛';
      case 'learn': return '📚';
      case 'tip': return '💡';
      default: return '💡';
    }
  };

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
              width: '900px',
              height: '700px',
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
            {/* Header - Draggable */}
            <div
              data-tauri-drag-region
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                marginBottom: '24px',
                paddingBottom: '16px',
                borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
                cursor: 'move',
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

            {/* Opportunity Badge or Empty State */}
            {latestOpportunity ? (
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
                <span style={{ fontSize: '18px' }}>{getTypeEmoji(latestOpportunity.type)}</span>
                <span
                  style={{
                    fontSize: '13px',
                    fontWeight: '600',
                    color: 'var(--accent-primary)',
                    textTransform: 'uppercase',
                    letterSpacing: '0.5px',
                  }}
                >
                  {latestOpportunity.type} • {Math.round(latestOpportunity.confidence * 100)}% confiance
                </span>
              </div>
            ) : (
              <div
                style={{
                  padding: '12px 16px',
                  background: 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.1)',
                  borderRadius: '12px',
                  marginBottom: '20px',
                  textAlign: 'center',
                }}
              >
                <span style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>
                  Aucune opportunité en attente
                </span>
              </div>
            )}

            {/* Content Area - flex grow to take available space */}
            <div
              style={{
                flex: '1 1 auto',
                display: 'flex',
                flexDirection: 'column',
                minHeight: 0, // Allow shrinking
              }}
            >
              {latestOpportunity ? (
                <>
                  {/* Title */}
                  <h3
                    style={{
                      fontSize: '18px',
                      fontWeight: '700',
                      color: 'var(--text-primary)',
                      marginBottom: '12px',
                      letterSpacing: '-0.3px',
                    }}
                  >
                    {latestOpportunity.title}
                  </h3>

                  {/* Description */}
                  <p
                    style={{
                      fontSize: '15px',
                      lineHeight: '1.6',
                      color: 'var(--text-secondary)',
                      marginBottom: '20px',
                      fontWeight: '400',
                    }}
                  >
                    {latestOpportunity.description}
                  </p>
                </>
              ) : (
                <div style={{ textAlign: 'center', padding: '40px 20px' }}>
                  <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔍</div>
                  <p style={{ fontSize: '16px', color: 'var(--text-primary)', marginBottom: '8px', fontWeight: '600' }}>
                    Aucune opportunité disponible
                  </p>
                  <p style={{ fontSize: '14px', color: 'var(--text-secondary)', lineHeight: '1.5' }}>
                    Trigger une nouvelle opportunité depuis le panneau debug<br/>
                    pour tester le système.
                  </p>
                </div>
              )}

              {/* Context */}
              {latestOpportunity && latestOpportunity.context && (
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
                    {latestOpportunity.context.app}
                    {latestOpportunity.context.file && ` • ${latestOpportunity.context.file}`}
                    {latestOpportunity.context.line && ` • Ligne ${latestOpportunity.context.line}`}
                  </div>
                  {latestOpportunity.context.codeSnippet && (
                    <pre
                      style={{
                        marginTop: '8px',
                        padding: '8px',
                        background: 'rgba(0, 0, 0, 0.3)',
                        borderRadius: '6px',
                        fontSize: '12px',
                        fontFamily: 'monospace',
                        overflow: 'auto',
                        maxHeight: '100px',
                      }}
                    >
                      {latestOpportunity.context.codeSnippet}
                    </pre>
                  )}
                </div>
              )}

              {/* View Modal (only when viewModal is true) */}
              {viewModal && latestOpportunity && (
                <div
                  style={{
                    marginTop: '12px',
                    padding: '16px',
                    background: 'rgba(135, 206, 235, 0.15)',
                    border: '1px solid var(--accent-light)',
                    borderRadius: '10px',
                    fontSize: '14px',
                    lineHeight: '1.5',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                    <strong style={{ color: 'var(--accent-primary)' }}>👁 Détails complets</strong>
                    <button
                      onClick={() => setViewModal(false)}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-muted)',
                        cursor: 'pointer',
                        fontSize: '16px',
                      }}
                    >
                      ✕
                    </button>
                  </div>
                  <div style={{ color: 'var(--text-secondary)' }}>
                    <p><strong>ID:</strong> {latestOpportunity.id}</p>
                    <p><strong>Type:</strong> {latestOpportunity.type}</p>
                    <p><strong>Confiance:</strong> {Math.round(latestOpportunity.confidence * 100)}%</p>
                    <p><strong>Status:</strong> {latestOpportunity.status}</p>
                    <p><strong>Timestamp:</strong> {new Date(latestOpportunity.timestamp * 1000).toLocaleString()}</p>
                  </div>
                </div>
              )}
            </div>

            {/* Actions - fixed at bottom */}
            {latestOpportunity && (
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
            )}

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
      <OpportunityProvider>
        <SpotlightWindow />
      </OpportunityProvider>
    </ThemeProvider>
  </React.StrictMode>
);

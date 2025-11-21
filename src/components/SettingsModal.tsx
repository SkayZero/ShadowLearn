/**
 * SettingsModal
 * Manage app allowlist, muted apps, and other preferences
 */

import { motion, AnimatePresence } from 'framer-motion';
import { useTheme } from '../contexts/ThemeContext';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const { theme } = useTheme();
  const [mutedApps, setMutedApps] = useState<string[]>([]);
  const [allowlist, setAllowlist] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  // Load data when modal opens
  useEffect(() => {
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

  const loadSettings = async () => {
    setLoading(true);
    try {
      const stats: any = await invoke('get_extended_trigger_stats');
      if (stats && typeof stats === 'object') {
        if ('muted_apps' in stats && Array.isArray(stats.muted_apps)) {
          setMutedApps(stats.muted_apps);
        }
        if ('allowlist' in stats && Array.isArray(stats.allowlist)) {
          setAllowlist(stats.allowlist);
        }
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleUnmute = async (app: string) => {
    try {
      await invoke('unmute_app', { appName: app });
      console.log(`✅ Unmuted ${app}`);
      // Reload settings
      await loadSettings();
    } catch (error) {
      console.error('Failed to unmute app:', error);
    }
  };

  if (!isOpen) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.6)',
          backdropFilter: 'blur(4px)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 100,
        }}
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          transition={{ type: 'spring', damping: 25, stiffness: 300 }}
          onClick={(e) => e.stopPropagation()}
          style={{
            width: '90%',
            maxWidth: '600px',
            maxHeight: '80vh',
            background: theme.glass.bg,
            backdropFilter: 'blur(20px)',
            WebkitBackdropFilter: 'blur(20px)',
            border: `1px solid ${theme.glass.border}`,
            borderRadius: '20px',
            boxShadow: theme.glass.shadow,
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: '24px 28px',
              borderBottom: `1px solid ${theme.glass.border}`,
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <span style={{ fontSize: '24px' }}>⚙️</span>
              <h2
                style={{
                  fontSize: '20px',
                  fontWeight: '700',
                  color: theme.text.primary,
                  margin: 0,
                }}
              >
                Paramètres
              </h2>
            </div>
            <button
              onClick={onClose}
              style={{
                background: 'transparent',
                border: 'none',
                color: theme.text.muted,
                fontSize: '24px',
                cursor: 'pointer',
                padding: '4px 8px',
                lineHeight: 1,
              }}
            >
              ✕
            </button>
          </div>

          {/* Content */}
          <div
            style={{
              padding: '28px',
              overflowY: 'auto',
              flex: 1,
            }}
          >
            {loading ? (
              <div style={{ textAlign: 'center', color: theme.text.secondary, padding: '40px' }}>
                Chargement...
              </div>
            ) : (
              <>
                {/* Muted Apps Section */}
                <section style={{ marginBottom: '32px' }}>
                  <h3
                    style={{
                      fontSize: '16px',
                      fontWeight: '600',
                      color: theme.text.primary,
                      marginBottom: '12px',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                    }}
                  >
                    🔇 Applications mutées ({mutedApps.length})
                  </h3>
                  <p
                    style={{
                      fontSize: '13px',
                      color: theme.text.muted,
                      marginBottom: '16px',
                      lineHeight: 1.5,
                    }}
                  >
                    ShadowLearn ne génère pas d'opportunités pour ces apps.
                  </p>
                  <div
                    style={{
                      maxHeight: '200px',
                      overflowY: 'auto',
                      background: 'rgba(255, 255, 255, 0.03)',
                      borderRadius: '12px',
                      padding: mutedApps.length > 0 ? '12px' : '24px',
                    }}
                  >
                    {mutedApps.length > 0 ? (
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                        {mutedApps.map((app) => (
                          <div
                            key={app}
                            style={{
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'center',
                              padding: '8px 12px',
                              background: 'rgba(239, 68, 68, 0.1)',
                              border: '1px solid rgba(239, 68, 68, 0.3)',
                              borderRadius: '8px',
                            }}
                          >
                            <span
                              style={{
                                fontSize: '13px',
                                fontWeight: '500',
                                color: theme.text.primary,
                              }}
                            >
                              {app}
                            </span>
                            <button
                              onClick={() => handleUnmute(app)}
                              style={{
                                padding: '4px 12px',
                                background: 'rgba(16, 185, 129, 0.2)',
                                border: '1px solid rgba(16, 185, 129, 0.5)',
                                borderRadius: '6px',
                                color: theme.accent,
                                cursor: 'pointer',
                                fontSize: '12px',
                                fontWeight: '600',
                                transition: 'all 0.2s',
                              }}
                              onMouseEnter={(e) => {
                                e.currentTarget.style.background = 'rgba(16, 185, 129, 0.3)';
                              }}
                              onMouseLeave={(e) => {
                                e.currentTarget.style.background = 'rgba(16, 185, 129, 0.2)';
                              }}
                            >
                              🔊 Réactiver
                            </button>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div style={{ textAlign: 'center', color: theme.text.muted, fontSize: '13px' }}>
                        Aucune application mutée
                      </div>
                    )}
                  </div>
                </section>

                {/* Allowlist Section */}
                <section>
                  <h3
                    style={{
                      fontSize: '16px',
                      fontWeight: '600',
                      color: theme.text.primary,
                      marginBottom: '12px',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                    }}
                  >
                    ✅ Applications surveillées ({allowlist.length})
                  </h3>
                  <p
                    style={{
                      fontSize: '13px',
                      color: theme.text.muted,
                      marginBottom: '16px',
                      lineHeight: 1.5,
                    }}
                  >
                    ShadowLearn peut générer des opportunités pour ces apps.
                  </p>
                  <div
                    style={{
                      maxHeight: '200px',
                      overflowY: 'auto',
                      background: 'rgba(255, 255, 255, 0.03)',
                      borderRadius: '12px',
                      padding: '16px',
                    }}
                  >
                    {allowlist.length > 0 ? (
                      <div
                        style={{
                          display: 'grid',
                          gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
                          gap: '8px',
                        }}
                      >
                        {allowlist.map((app) => (
                          <div
                            key={app}
                            style={{
                              padding: '8px 12px',
                              background: 'rgba(16, 185, 129, 0.1)',
                              border: `1px solid ${theme.glass.border}`,
                              borderRadius: '8px',
                              fontSize: '12px',
                              fontWeight: '500',
                              color: theme.text.secondary,
                              textAlign: 'center',
                              overflow: 'hidden',
                              textOverflow: 'ellipsis',
                              whiteSpace: 'nowrap',
                            }}
                            title={app}
                          >
                            {app}
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div style={{ textAlign: 'center', color: theme.text.muted, fontSize: '13px' }}>
                        Aucune application dans l'allowlist
                      </div>
                    )}
                  </div>
                </section>
              </>
            )}
          </div>

          {/* Footer */}
          <div
            style={{
              padding: '16px 28px',
              borderTop: `1px solid ${theme.glass.border}`,
              display: 'flex',
              justifyContent: 'flex-end',
            }}
          >
            <button
              onClick={onClose}
              style={{
                padding: '10px 24px',
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
                e.currentTarget.style.transform = 'translateY(-1px)';
                e.currentTarget.style.boxShadow = `0 4px 12px ${theme.glow}`;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = 'none';
              }}
            >
              Fermer
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

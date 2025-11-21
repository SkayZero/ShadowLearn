/**
 * Settings Window
 * Separate window for app configuration
 * Manage allowlist, muted apps, and preferences
 */

import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import HeaderDraggable from './components/HeaderDraggable';
import WindowManager from './components/WindowManager';
import { ThemeProvider } from './contexts/ThemeContext';
import { useTheme } from './contexts/ThemeContext';
import useWindowLifecycle from './hooks/useWindowLifecycle';
import './styles/island-globals.css';

function SettingsWindow() {
  const { theme } = useTheme();
  const [mutedApps, setMutedApps] = useState<string[]>([]);
  const [allowlist, setAllowlist] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  useWindowLifecycle({
    onFocus: () => {},
    onBlur: () => {},
  });

  useEffect(() => {
    loadSettings();
  }, []);

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
      await loadSettings();
    } catch (error) {
      console.error('Failed to unmute app:', error);
    }
  };

  return (
    <ThemeProvider>
      <WindowManager>
        <div className="sl-island">
          <HeaderDraggable
            title="ShadowLearn — Réglages"
            showMinimize={true}
            rightContent={
              <button
                onClick={async () => {
                  try {
                    await invoke('show_window', { windowLabel: 'chat' });
                  } catch (error) {
                    console.error('❌ Failed to show chat:', error);
                  }
                }}
                style={{
                  padding: '6px 12px',
                  background: 'rgba(135, 206, 235, 0.2)',
                  border: '1px solid rgba(135, 206, 235, 0.5)',
                  borderRadius: '6px',
                  color: 'white',
                  cursor: 'pointer',
                  fontSize: '0.85em',
                  fontWeight: '600',
                }}
              >
                💬 Chat
              </button>
            }
          />

          <div className="sl-body">
            <div style={{ padding: '24px' }}>
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
                        marginBottom: '8px',
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
                        maxHeight: '220px',
                        overflowY: 'auto',
                        background: 'rgba(255, 255, 255, 0.03)',
                        borderRadius: '12px',
                        padding: mutedApps.length > 0 ? '12px' : '24px',
                        border: `1px solid ${theme.glass.border}`,
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
                                padding: '10px 14px',
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
                                  padding: '5px 14px',
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
                        marginBottom: '8px',
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
                        maxHeight: '220px',
                        overflowY: 'auto',
                        background: 'rgba(255, 255, 255, 0.03)',
                        borderRadius: '12px',
                        padding: '16px',
                        border: `1px solid ${theme.glass.border}`,
                      }}
                    >
                      {allowlist.length > 0 ? (
                        <div
                          style={{
                            display: 'grid',
                            gridTemplateColumns: 'repeat(auto-fill, minmax(120px, 1fr))',
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
          </div>
        </div>
      </WindowManager>
    </ThemeProvider>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <SettingsWindow />
  </React.StrictMode>
);

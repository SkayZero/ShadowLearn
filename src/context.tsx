import React, { useEffect, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import HeaderDraggable from './components/HeaderDraggable';
import WindowManager from './components/WindowManager';
import { AmbientLED } from './components/AmbientLED';
import { ContextPreviewCard } from './components/ContextPreviewCard';
import useWindowLifecycle from './hooks/useWindowLifecycle';
import useDesktopFocus from './hooks/useDesktopFocus';
import { useContextCapture } from './hooks/useContextCapture';
import useActivityDetection from './hooks/useActivityDetection';
import './styles/island-globals.css';

interface ContextInfo {
  app: string;
  lastActivity: string;
  clipboard: string;
}

function ContextWindow() {
  const [context, setContext] = useState<ContextInfo>({
    app: 'No active application',
    lastActivity: 'Idle',
    clipboard: 'Empty',
  });
  
  const [mutedApps, setMutedApps] = useState<string[]>([]);
  const [allowlist, setAllowlist] = useState<string[]>([]);
  const [showContextPreview, setShowContextPreview] = useState(false);

  useWindowLifecycle({
    onFocus: () => {},
    onBlur: () => {},
  });

  useDesktopFocus({ enabled: true, delay: 150 });

  const { capture } = useContextCapture();

  // Activity detection - reset idle timer on user activity
  useActivityDetection(true); // Throttled: 300ms mouse, 500ms keyboard

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const ctx = await capture();
        if (ctx) {
          setContext({
            app: ctx.app.name,
            lastActivity: `Idle: ${Math.round(ctx.idle_seconds)}s`,
            clipboard: ctx.clipboard || 'Empty',
          });
        }
      } catch (error) {
        console.error('Failed to capture context:', error);
      }
      
      // Récupérer les apps mutées et l'allowlist
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
        console.error('Failed to get trigger stats:', error);
      }
    }, 2000);

    capture();
    return () => clearInterval(interval);
  }, [capture]);

  return (
    <WindowManager>
      <div className="sl-island">
        <HeaderDraggable 
          title="ShadowLearn — Contexte"
          showMinimize={true}
          rightContent={
            <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
              {/* Clueless Phase 1: Ambient LED */}
              <AmbientLED size={12} />
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
                  background: 'rgba(99, 102, 241, 0.2)',
                  border: '1px solid rgba(99, 102, 241, 0.5)',
                  borderRadius: '6px',
                  color: 'white',
                  cursor: 'pointer',
                  fontSize: '0.85em',
                  fontWeight: '600',
                }}
              >
                💬 Chat
              </button>
            </div>
          }
        />

        <div className="sl-body">
          <div 
            style={{ padding: '20px', position: 'relative' }}
            onMouseEnter={() => setShowContextPreview(true)}
            onMouseLeave={() => setShowContextPreview(false)}
          >
            {/* Clueless Phase 1: Context Preview Card */}
            <ContextPreviewCard 
              visible={showContextPreview}
              onClose={() => setShowContextPreview(false)}
            />
            
            <div style={{ marginBottom: '24px' }}>
              <h3 style={{ fontSize: '14px', color: 'rgba(255,255,255,0.5)', marginBottom: '8px' }}>
                Application active
              </h3>
              <div style={{ fontSize: '18px', fontWeight: '600' }}>
                {context.app}
              </div>
            </div>

            <div style={{ marginBottom: '24px' }}>
              <h3 style={{ fontSize: '14px', color: 'rgba(255,255,255,0.5)', marginBottom: '8px' }}>
                Statut
              </h3>
              <div style={{ fontSize: '18px', fontWeight: '600' }}>
                {context.lastActivity}
              </div>
            </div>

            <div style={{ marginBottom: '24px' }}>
              <h3 style={{ fontSize: '14px', color: 'rgba(255,255,255,0.5)', marginBottom: '8px' }}>
                Presse-papiers
              </h3>
              <div style={{ fontSize: '14px', color: 'rgba(255,255,255,0.4)', wordBreak: 'break-word' }}>
                {context.clipboard}
              </div>
            </div>

            <div style={{ marginBottom: '24px' }}>
              <h3 style={{ fontSize: '14px', color: 'rgba(255,255,255,0.5)', marginBottom: '8px' }}>
                Apps mutées ({mutedApps.length})
              </h3>
              <div style={{ fontSize: '12px', color: mutedApps.length > 0 ? 'rgba(239, 68, 68, 0.8)' : 'rgba(16, 185, 129, 0.6)' }}>
                {mutedApps.length > 0 ? (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    {mutedApps.map((app, idx) => (
                      <div key={idx} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <span>{app}</span>
                        <button
                          onClick={async () => {
                            try {
                              await invoke('unmute_app', { appName: app });
                              console.log(`✅ Unmuted ${app}`);
                            } catch (error) {
                              console.error('Failed to unmute app:', error);
                            }
                          }}
                          style={{
                            padding: '4px 8px',
                            background: 'rgba(16, 185, 129, 0.2)',
                            border: '1px solid rgba(16, 185, 129, 0.5)',
                            borderRadius: '4px',
                            color: 'white',
                            cursor: 'pointer',
                            fontSize: '11px',
                          }}
                        >
                          🔊 Dé-muter
                        </button>
                      </div>
                    ))}
                  </div>
                ) : (
                  'Aucune app mutée'
                )}
              </div>
            </div>

            <div>
              <h3 style={{ fontSize: '14px', color: 'rgba(255,255,255,0.5)', marginBottom: '8px' }}>
                Allowlist ({allowlist.length})
              </h3>
              <div style={{ fontSize: '12px', color: 'rgba(16, 185, 129, 0.6)', maxHeight: '100px', overflowY: 'auto' }}>
                {allowlist.length > 0 
                  ? allowlist.slice(0, 10).join(', ') + (allowlist.length > 10 ? ` ... +${allowlist.length - 10} autres` : '')
                  : 'Aucune app dans l\'allowlist'
                }
              </div>
            </div>
          </div>
        </div>
      </div>
    </WindowManager>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ContextWindow />
  </React.StrictMode>
);

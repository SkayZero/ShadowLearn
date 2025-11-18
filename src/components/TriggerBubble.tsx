import React from 'react';
import { Context } from '../hooks/useTrigger';
import { BaseBubble, BubbleButton } from './BaseBubble';

interface TriggerBubbleProps {
  context: Context | null;
  isVisible: boolean;
  onHide: () => void;
  onUserInteraction: () => void;
}

export const TriggerBubble: React.FC<TriggerBubbleProps> = ({
  context,
  isVisible,
  onHide,
  onUserInteraction,
}) => {
  if (!context) return null;

  return (
    <BaseBubble
      isVisible={isVisible}
      title="Besoin d'aide ?"
      icon="💡"
      onClose={onHide}
      actions={
        <>
          <BubbleButton onClick={onHide} variant="secondary">
            ❌ Plus tard
          </BubbleButton>
          <BubbleButton onClick={onUserInteraction} variant="primary">
            ✅ Afficher une suggestion
          </BubbleButton>
        </>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
        {/* Application active */}
        <div>
          <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '8px', color: '#666' }}>
            📱 Application active
          </h3>
          <div style={{ fontSize: '14px' }}>
            <strong>{context.app.name}</strong>
            {context.app.window_title && (
              <div style={{ fontSize: '12px', color: '#999', marginTop: '4px' }}>
                {context.app.window_title}
              </div>
            )}
          </div>
        </div>

        {/* Clipboard récent */}
        {context.clipboard && (
          <div>
            <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '8px', color: '#666' }}>
              📋 Clipboard récent
            </h3>
            <div
              style={{
                fontSize: '13px',
                background: 'rgba(0, 0, 0, 0.03)',
                padding: '8px',
                borderRadius: '6px',
                fontFamily: 'monospace',
                maxHeight: '100px',
                overflow: 'auto',
              }}
            >
              {context.clipboard.length > 100
                ? `${context.clipboard.substring(0, 100)}...`
                : context.clipboard}
            </div>
          </div>
        )}

        {/* Temps d'inactivité */}
        <div>
          <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '8px', color: '#666' }}>
            ⏱️ Temps d'inactivité
          </h3>
          <div style={{ fontSize: '14px' }}>{Math.round(context.idle_seconds)} secondes</div>
        </div>

        {/* Performance */}
        <div>
          <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '8px', color: '#666' }}>
            ⚡ Performance
          </h3>
          <div style={{ fontSize: '14px' }}>Capture: {context.capture_duration_ms}ms</div>
        </div>
      </div>
    </BaseBubble>
  );
};

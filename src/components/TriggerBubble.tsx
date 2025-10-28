import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Context } from '../hooks/useTrigger';
import './TriggerBubble.css';

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
  if (!context) {
    return null;
  }

  const handleInteraction = () => {
    onUserInteraction();
  };

  const handleDismiss = () => {
    onHide();
  };

  if (!isVisible) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, scale: 0.9, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.9, y: 20 }}
        transition={{ duration: 0.2 }}
        className="trigger-bubble"
        style={{
          position: 'fixed',
          bottom: '100px',
          right: '24px',
          width: '400px',
          zIndex: 50,
        }}
      >
      <div className="trigger-bubble-header">
        <div className="trigger-bubble-title">
          💡 Besoin d'aide ?
        </div>
        <button 
          className="trigger-bubble-close"
          onClick={handleDismiss}
          title="Fermer"
        >
          ×
        </button>
      </div>

      <div className="trigger-bubble-content">
        <div className="trigger-context-section">
          <h3>📱 Application active</h3>
          <div className="context-item">
            <strong>{context.app.name}</strong>
            {context.app.window_title && (
              <div className="context-subtitle">{context.app.window_title}</div>
            )}
          </div>
        </div>

        {context.clipboard && (
          <div className="trigger-context-section">
            <h3>📋 Clipboard récent</h3>
            <div className="context-item clipboard-content">
              {context.clipboard.length > 100 
                ? `${context.clipboard.substring(0, 100)}...`
                : context.clipboard
              }
            </div>
          </div>
        )}

        <div className="trigger-context-section">
          <h3>⏱️ Temps d'inactivité</h3>
          <div className="context-item">
            {Math.round(context.idle_seconds)} secondes
          </div>
        </div>

        <div className="trigger-context-section">
          <h3>⚡ Performance</h3>
          <div className="context-item">
            Capture: {context.capture_duration_ms}ms
          </div>
        </div>
      </div>

        <div className="trigger-bubble-actions">
        <button 
          className="trigger-action-btn primary"
          onClick={handleInteraction}
        >
          ✅ Afficher une suggestion
        </button>
        <button 
          className="trigger-action-btn secondary"
          onClick={handleDismiss}
        >
          ❌ Plus tard
        </button>
      </div>
      </motion.div>
    </AnimatePresence>
  );
};

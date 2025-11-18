import React, { useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useScreenSuggestion } from '../hooks/useScreenMonitor';

interface ScreenMonitorBubbleProps {
  /**
   * Auto-dismiss after this many seconds (0 = never auto-dismiss)
   * @default 30
   */
  autoDismissSeconds?: number;

  /**
   * Custom CSS class
   */
  className?: string;
}

/**
 * Bubble component for displaying screen monitoring suggestions
 *
 * Automatically appears when a screen change with Claude Vision analysis is detected.
 * Integrates seamlessly with ShadowLearn's UI.
 *
 * @example
 * ```tsx
 * <ScreenMonitorBubble autoDismissSeconds={30} />
 * ```
 */
export const ScreenMonitorBubble: React.FC<ScreenMonitorBubbleProps> = ({
  autoDismissSeconds = 30,
  className = '',
}) => {
  const { suggestion, dismiss } = useScreenSuggestion();

  // Auto-dismiss after timeout
  useEffect(() => {
    if (suggestion && autoDismissSeconds > 0) {
      const timer = setTimeout(() => {
        dismiss();
      }, autoDismissSeconds * 1000);

      return () => clearTimeout(timer);
    }
  }, [suggestion, autoDismissSeconds, dismiss]);

  return (
    <AnimatePresence>
      {suggestion && (
        <motion.div
          initial={{ opacity: 0, scale: 0.9, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.9, y: 20 }}
          transition={{ duration: 0.3 }}
          className={`screen-monitor-bubble ${className}`}
          style={{
            position: 'fixed',
            bottom: '120px',
            right: '24px',
            width: '400px',
            maxWidth: '90vw',
            zIndex: 100,
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            borderRadius: '16px',
            padding: '20px',
            boxShadow: '0 10px 40px rgba(0,0,0,0.3)',
            color: 'white',
          }}
        >
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'flex-start',
              marginBottom: '12px',
            }}
          >
            <div
              style={{
                fontSize: '18px',
                fontWeight: 600,
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
              }}
            >
              <span>🤖</span>
              <span>Claude Vision Suggestion</span>
            </div>
            <button
              onClick={dismiss}
              style={{
                background: 'rgba(255,255,255,0.2)',
                border: 'none',
                borderRadius: '50%',
                width: '28px',
                height: '28px',
                cursor: 'pointer',
                color: 'white',
                fontSize: '18px',
                fontWeight: 'bold',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                transition: 'background 0.2s',
              }}
              onMouseOver={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.3)';
              }}
              onMouseOut={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.2)';
              }}
              title="Fermer"
            >
              ×
            </button>
          </div>

          <div
            style={{
              fontSize: '15px',
              lineHeight: '1.6',
              marginBottom: '16px',
              background: 'rgba(255,255,255,0.1)',
              padding: '12px',
              borderRadius: '8px',
            }}
          >
            {suggestion.text}
          </div>

          <div
            style={{
              display: 'flex',
              gap: '8px',
              justifyContent: 'flex-end',
            }}
          >
            <button
              onClick={dismiss}
              style={{
                background: 'rgba(255,255,255,0.2)',
                border: 'none',
                borderRadius: '8px',
                padding: '8px 16px',
                cursor: 'pointer',
                color: 'white',
                fontSize: '14px',
                fontWeight: 500,
                transition: 'background 0.2s',
              }}
              onMouseOver={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.3)';
              }}
              onMouseOut={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.2)';
              }}
            >
              Ignorer
            </button>
            <button
              onClick={() => {
                // TODO: Integrate with chat window
                console.log('Opening chat with suggestion:', suggestion.text);
                dismiss();
              }}
              style={{
                background: 'rgba(255,255,255,0.9)',
                border: 'none',
                borderRadius: '8px',
                padding: '8px 16px',
                cursor: 'pointer',
                color: '#667eea',
                fontSize: '14px',
                fontWeight: 600,
                transition: 'background 0.2s',
              }}
              onMouseOver={(e) => {
                e.currentTarget.style.background = 'white';
              }}
              onMouseOut={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.9)';
              }}
            >
              Ouvrir le Chat
            </button>
          </div>

          {autoDismissSeconds > 0 && (
            <div
              style={{
                marginTop: '12px',
                fontSize: '12px',
                opacity: 0.7,
                textAlign: 'center',
              }}
            >
              Auto-dismiss dans {autoDismissSeconds}s
            </div>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
};

export default ScreenMonitorBubble;

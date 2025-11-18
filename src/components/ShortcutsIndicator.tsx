import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useShortcuts, ShortcutAction } from '../hooks/useShortcuts';

interface ShortcutsIndicatorProps {
  onScreenshotAnalyze?: () => void;
  onToggleBubbles?: () => void;
  onOpenDashboard?: () => void;
  onDismissBubble?: () => void;
}

const actionLabels: Record<ShortcutAction, { icon: string; label: string }> = {
  'screenshot-analyze': { icon: '📸', label: 'Screenshot & Analyze' },
  'toggle-bubbles': { icon: '💬', label: 'Toggle Bubbles' },
  'open-dashboard': { icon: '📊', label: 'Open Dashboard' },
  'dismiss-bubble': { icon: '❌', label: 'Dismiss Bubble' },
};

/**
 * Indicator showing available keyboard shortcuts
 * Displays in bottom-right corner, can be toggled with a button
 */
export const ShortcutsIndicator: React.FC<ShortcutsIndicatorProps> = (props) => {
  const [isOpen, setIsOpen] = useState(false);
  const { config, shortcuts, triggerAction } = useShortcuts(props);

  if (!config || !config.enabled) {
    return null;
  }

  const shortcutEntries = Object.entries(shortcuts) as [string, ShortcutAction][];

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '20px',
        right: '20px',
        zIndex: 9999,
      }}
    >
      {/* Toggle Button */}
      <motion.button
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.95 }}
        onClick={() => setIsOpen(!isOpen)}
        style={{
          position: 'absolute',
          bottom: 0,
          right: 0,
          width: '48px',
          height: '48px',
          borderRadius: '50%',
          background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
          border: 'none',
          color: 'white',
          fontSize: '20px',
          cursor: 'pointer',
          boxShadow: '0 4px 12px rgba(102, 126, 234, 0.4)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
        title="Keyboard Shortcuts"
      >
        ⌨️
      </motion.button>

      {/* Shortcuts Panel */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, y: 20, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 20, scale: 0.9 }}
            transition={{ duration: 0.2 }}
            style={{
              position: 'absolute',
              bottom: '60px',
              right: 0,
              minWidth: '320px',
              background: 'white',
              borderRadius: '16px',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
              padding: '20px',
            }}
          >
            <h3
              style={{
                margin: '0 0 16px 0',
                fontSize: '18px',
                fontWeight: 600,
                color: '#333',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
              }}
            >
              ⌨️ Keyboard Shortcuts
            </h3>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
              {shortcutEntries.map(([key, action]) => {
                const info = actionLabels[action];
                return (
                  <motion.div
                    key={action}
                    whileHover={{ x: 4 }}
                    onClick={() => triggerAction(action)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '12px',
                      background: 'rgba(102, 126, 234, 0.05)',
                      borderRadius: '8px',
                      cursor: 'pointer',
                      transition: 'background 0.2s',
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.background = 'rgba(102, 126, 234, 0.1)';
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.background = 'rgba(102, 126, 234, 0.05)';
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span style={{ fontSize: '18px' }}>{info.icon}</span>
                      <span style={{ fontSize: '14px', color: '#666' }}>{info.label}</span>
                    </div>
                    <kbd
                      style={{
                        padding: '4px 8px',
                        background: 'white',
                        border: '1px solid #ddd',
                        borderRadius: '4px',
                        fontSize: '12px',
                        fontFamily: 'monospace',
                        color: '#667eea',
                        fontWeight: 600,
                      }}
                    >
                      {key}
                    </kbd>
                  </motion.div>
                );
              })}
            </div>

            <div
              style={{
                marginTop: '16px',
                padding: '12px',
                background: 'rgba(102, 126, 234, 0.05)',
                borderRadius: '8px',
                fontSize: '12px',
                color: '#666',
              }}
            >
              💡 <strong>Tip:</strong> Click on any shortcut to trigger it manually
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

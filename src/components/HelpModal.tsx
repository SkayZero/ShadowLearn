/**
 * HelpModal
 * Simple modal with FAQ and keyboard shortcuts
 */

import { motion, AnimatePresence } from 'framer-motion';
import { useTheme } from '../contexts/ThemeContext';

interface HelpModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function HelpModal({ isOpen, onClose }: HelpModalProps) {
  const { theme } = useTheme();

  if (!isOpen) return null;

  const shortcuts = [
    { key: 'Cmd/Ctrl + K', description: 'Ouvrir/fermer le Dock' },
    { key: 'Cmd/Ctrl + D', description: 'Ouvrir le Digest du jour' },
    { key: 'Escape', description: 'Fermer les modales' },
  ];

  const faqs = [
    {
      q: 'Comment activer ShadowLearn ?',
      a: 'Cliquez sur le bouton "✗ Inactif" dans le header pour passer en mode "✓ Actif".',
    },
    {
      q: 'Que sont les opportunités ?',
      a: 'Ce sont des suggestions intelligentes basées sur votre activité. Quand une apparaît, cliquez sur "Voir" pour l\'explorer dans le chat.',
    },
    {
      q: 'Comment discuter d\'une opportunité ?',
      a: 'Cliquez sur "💬 Discuter" dans l\'OpportunityLayer pour poser des questions à l\'assistant.',
    },
    {
      q: 'Où trouver mes statistiques ?',
      a: 'Ouvrez le Dock (🎛️) puis cliquez sur "📊 Voir le Digest" pour voir vos stats quotidiennes.',
    },
  ];

  const slashCommands = [
    { icon: '❓', command: '/help', description: 'Afficher l\'aide générale' },
    { icon: '📝', command: '/resume', description: 'Résumer le texte sélectionné' },
    { icon: '🔍', command: '/explain', description: 'Expliquer un concept' },
    { icon: '🐛', command: '/debug', description: 'Analyser une erreur' },
    { icon: '✨', command: '/improve', description: 'Suggérer des améliorations' },
    { icon: '🌐', command: '/translate', description: 'Traduire du texte' },
  ];

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
            background: theme.glass.bg,
            backdropFilter: theme.glass.backdrop,
            border: `1px solid ${theme.glass.border}`,
            borderRadius: '16px',
            padding: '24px',
            maxWidth: '600px',
            width: '90%',
            maxHeight: '80vh',
            overflow: 'auto',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4)',
          }}
        >
          {/* Header */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              marginBottom: '20px',
            }}
          >
            <h2
              style={{
                fontSize: '20px',
                fontWeight: '700',
                color: theme.text.primary,
                margin: 0,
              }}
            >
              ❓ Aide ShadowLearn
            </h2>
            <button
              onClick={onClose}
              style={{
                background: 'transparent',
                border: 'none',
                color: theme.text.muted,
                cursor: 'pointer',
                fontSize: '24px',
                padding: '4px 8px',
                transition: 'color 0.2s',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.color = theme.text.primary)}
              onMouseLeave={(e) => (e.currentTarget.style.color = theme.text.muted)}
            >
              ✕
            </button>
          </div>

          {/* Shortcuts Section */}
          <div style={{ marginBottom: '24px' }}>
            <h3
              style={{
                fontSize: '16px',
                fontWeight: '600',
                color: theme.accent,
                marginBottom: '12px',
              }}
            >
              ⌨️ Raccourcis clavier
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {shortcuts.map((shortcut, i) => (
                <div
                  key={i}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    padding: '8px 12px',
                    background: 'rgba(255, 255, 255, 0.05)',
                    borderRadius: '8px',
                  }}
                >
                  <span
                    style={{
                      fontSize: '13px',
                      color: theme.text.secondary,
                    }}
                  >
                    {shortcut.description}
                  </span>
                  <code
                    style={{
                      fontSize: '12px',
                      color: theme.accent,
                      background: 'rgba(135, 206, 235, 0.1)',
                      padding: '2px 8px',
                      borderRadius: '4px',
                      fontFamily: 'monospace',
                    }}
                  >
                    {shortcut.key}
                  </code>
                </div>
              ))}
            </div>
          </div>

          {/* FAQ Section */}
          <div style={{ marginBottom: '24px' }}>
            <h3
              style={{
                fontSize: '16px',
                fontWeight: '600',
                color: theme.accent,
                marginBottom: '12px',
              }}
            >
              💬 Questions fréquentes
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
              {faqs.map((faq, i) => (
                <div key={i}>
                  <div
                    style={{
                      fontSize: '14px',
                      fontWeight: '600',
                      color: theme.text.primary,
                      marginBottom: '6px',
                    }}
                  >
                    {faq.q}
                  </div>
                  <div
                    style={{
                      fontSize: '13px',
                      color: theme.text.secondary,
                      lineHeight: '1.5',
                    }}
                  >
                    {faq.a}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Slash Commands Section */}
          <div>
            <h3
              style={{
                fontSize: '16px',
                fontWeight: '600',
                color: theme.accent,
                marginBottom: '12px',
              }}
            >
              ⚡ Commandes disponibles
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {slashCommands.map((cmd, i) => (
                <div
                  key={i}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px',
                    padding: '8px 12px',
                    background: 'rgba(255, 255, 255, 0.05)',
                    borderRadius: '8px',
                  }}
                >
                  <span style={{ fontSize: '18px' }}>{cmd.icon}</span>
                  <div style={{ flex: 1 }}>
                    <div
                      style={{
                        fontSize: '13px',
                        color: theme.text.secondary,
                      }}
                    >
                      {cmd.description}
                    </div>
                  </div>
                  <code
                    style={{
                      fontSize: '12px',
                      color: theme.accent,
                      background: 'rgba(135, 206, 235, 0.1)',
                      padding: '2px 8px',
                      borderRadius: '4px',
                      fontFamily: 'monospace',
                    }}
                  >
                    {cmd.command}
                  </code>
                </div>
              ))}
            </div>
          </div>

          {/* Footer */}
          <div
            style={{
              marginTop: '24px',
              padding: '12px',
              background: 'rgba(135, 206, 235, 0.1)',
              borderRadius: '8px',
              textAlign: 'center',
            }}
          >
            <div
              style={{
                fontSize: '13px',
                color: theme.text.secondary,
              }}
            >
              💡 Tape simplement la commande dans le chat pour l'utiliser !
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

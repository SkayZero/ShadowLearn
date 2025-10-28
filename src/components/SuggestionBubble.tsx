import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import './SuggestionBubble.css';

interface SuggestionResponse {
  suggestion: {
    id: string;
    artefact_type: string;
  };
  artefact: {
    artefact_type: string;
    file_path: string | null;
    content: string | null;
  };
  context: {
    app: {
      name: string;
    };
  };
}

interface SuggestionBubbleProps {
  suggestion: SuggestionResponse;
  onClose: () => void;
}

export function SuggestionBubble({ suggestion, onClose }: SuggestionBubbleProps) {
  const [expanded, setExpanded] = useState(false);
  const [, setActionTaken] = useState(false);

  const handleDownload = async () => {
    if (!suggestion.artefact.file_path) return;

    try {
      // Open file if it exists
      // await open(suggestion.artefact.file_path);
      
      await invoke('record_artifact_feedback', {
        suggestionId: suggestion.suggestion.id,
        outcome: 'positive',
      });
      
      setActionTaken(true);
      // await recordUserAction();
    } catch (e) {
      console.error('Download failed:', e);
    }
  };

  const handleHelpful = async () => {
    try {
      await invoke('record_artifact_feedback', {
        suggestionId: suggestion.suggestion.id,
        outcome: 'positive',
      });
      // await recordUserAction();
      onClose();
    } catch (e) {
      console.error('Failed to record helpful:', e);
    }
  };

  const handleDismiss = async () => {
    try {
      await invoke('record_artifact_feedback', {
        suggestionId: suggestion.suggestion.id,
        outcome: 'negative',
      });
      // await dismissBubble();
      onClose();
    } catch (e) {
      console.error('Failed to dismiss:', e);
    }
  };

  const getIconForType = (type: string): string => {
    const icons: Record<string, string> = {
      midi: '🎵',
      python: '🐍',
      blend: '🎨',
      json: '📄',
      tutorial: '📚',
      text: '📝',
    };
    return icons[type.toLowerCase()] || '📦';
  };

  const formatArtefactType = (type: string): string => {
    const names: Record<string, string> = {
      midi: 'MIDI Pattern',
      python: 'Python Script',
      blend: 'Blender Script',
      json: 'JSON Config',
      tutorial: 'Tutorial',
      text: 'Text File',
    };
    return names[type.toLowerCase()] || type;
  };

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: 20, scale: 0.9 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -20, scale: 0.9 }}
        transition={{ duration: 0.3, ease: 'easeOut' }}
        className="suggestion-bubble"
      >
        {/* Header */}
        <div className="suggestion-bubble-header">
          <div className="suggestion-bubble-title">
            <span className="suggestion-icon">{getIconForType(suggestion.artefact.artefact_type)}</span>
            <div>
              <div className="suggestion-title-text">ShadowLearn</div>
              <div className="suggestion-subtitle">{formatArtefactType(suggestion.artefact.artefact_type)}</div>
            </div>
          </div>
          <button
            onClick={handleDismiss}
            className="suggestion-close-btn"
            aria-label="Close"
          >
            ✕
          </button>
        </div>

        {/* Content */}
        <div className="suggestion-bubble-content">
          {/* Context hint */}
          <div className="suggestion-context">
            Détecté dans <span className="suggestion-app-name">{suggestion.context.app.name}</span>
          </div>

          {/* Quick actions */}
          {suggestion.artefact.file_path && (
            <motion.button
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              onClick={handleDownload}
              className="suggestion-download-btn"
            >
              <span>📥</span>
              <span>Ouvrir {formatArtefactType(suggestion.artefact.artefact_type)}</span>
            </motion.button>
          )}

          {/* Copy content if available */}
          {suggestion.artefact.content && (
            <motion.button
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              onClick={async () => {
                if (suggestion.artefact.content) {
                  await navigator.clipboard.writeText(suggestion.artefact.content);
                  alert('✅ Contenu copié dans le presse-papiers');
                }
              }}
              className="suggestion-copy-btn"
            >
              <span>📋</span>
              <span>Copier le contenu</span>
            </motion.button>
          )}

          {/* Explanation */}
          {suggestion.artefact.content && (
            <div className="suggestion-explanation">
              <button
                onClick={() => setExpanded(!expanded)}
                className="suggestion-explanation-toggle"
              >
                <span>💡 Explication</span>
                <span>{expanded ? '▲' : '▼'}</span>
              </button>
              
              <AnimatePresence>
                {expanded && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: 'auto', opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="suggestion-explanation-content"
                  >
                    <pre className="suggestion-code-preview">
                      {suggestion.artefact.content}
                    </pre>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          )}

          {/* Feedback */}
          <div className="suggestion-feedback">
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={handleHelpful}
              className="suggestion-helpful-btn"
            >
              ✓ Utile
            </motion.button>
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={handleDismiss}
              className="suggestion-skip-btn"
            >
              Ignorer
            </motion.button>
          </div>
        </div>
      </motion.div>
    </AnimatePresence>
  );
}


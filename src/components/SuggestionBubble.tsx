import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { BaseBubble, BubbleButton } from './BaseBubble';

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

export function SuggestionBubble({ suggestion, onClose }: SuggestionBubbleProps) {
  const [expanded, setExpanded] = useState(false);

  const handleDownload = async () => {
    if (!suggestion.artefact.file_path) return;
    try {
      await invoke('record_artifact_feedback', {
        suggestionId: suggestion.suggestion.id,
        outcome: 'positive',
      });
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
      onClose();
    } catch (e) {
      console.error('Failed to dismiss:', e);
    }
  };

  const handleCopyContent = async () => {
    if (suggestion.artefact.content) {
      await navigator.clipboard.writeText(suggestion.artefact.content);
      alert('✅ Contenu copié dans le presse-papiers');
    }
  };

  return (
    <BaseBubble
      isVisible={true}
      title="ShadowLearn"
      subtitle={formatArtefactType(suggestion.artefact.artefact_type)}
      icon={getIconForType(suggestion.artefact.artefact_type)}
      onClose={handleDismiss}
      actions={
        <>
          <BubbleButton onClick={handleDismiss} variant="secondary">
            Ignorer
          </BubbleButton>
          <BubbleButton onClick={handleHelpful} variant="primary">
            ✓ Utile
          </BubbleButton>
        </>
      }
    >
      <div style={{ fontSize: '13px', color: '#666', marginBottom: '16px' }}>
        Détecté dans <strong style={{ color: '#667eea' }}>{suggestion.context.app.name}</strong>
      </div>

      {suggestion.artefact.file_path && (
        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={handleDownload}
          style={{
            width: '100%',
            padding: '12px',
            marginBottom: '8px',
            background: '#667eea',
            color: 'white',
            border: 'none',
            borderRadius: '8px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            fontSize: '14px',
            fontWeight: 500,
          }}
        >
          <span>📥</span>
          <span>Ouvrir {formatArtefactType(suggestion.artefact.artefact_type)}</span>
        </motion.button>
      )}

      {suggestion.artefact.content && (
        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={handleCopyContent}
          style={{
            width: '100%',
            padding: '12px',
            marginBottom: '16px',
            background: 'rgba(102, 126, 234, 0.1)',
            color: '#667eea',
            border: '1px solid rgba(102, 126, 234, 0.3)',
            borderRadius: '8px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            fontSize: '14px',
            fontWeight: 500,
          }}
        >
          <span>📋</span>
          <span>Copier le contenu</span>
        </motion.button>
      )}

      {suggestion.artefact.content && (
        <div>
          <button
            onClick={() => setExpanded(!expanded)}
            style={{
              width: '100%',
              padding: '12px',
              background: 'rgba(0, 0, 0, 0.05)',
              border: 'none',
              borderRadius: '8px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              fontSize: '14px',
              fontWeight: 500,
              color: '#333',
            }}
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
                style={{ overflow: 'hidden' }}
              >
                <pre
                  style={{
                    marginTop: '8px',
                    padding: '12px',
                    background: '#f5f5f5',
                    borderRadius: '8px',
                    fontSize: '12px',
                    fontFamily: 'monospace',
                    overflowX: 'auto',
                    maxHeight: '300px',
                  }}
                >
                  {suggestion.artefact.content}
                </pre>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      )}
    </BaseBubble>
  );
}

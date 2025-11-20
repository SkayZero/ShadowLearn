/**
 * OpportunityLayer
 * Contextual layer that appears at the top of the Chat when a complex opportunity is opened
 * Can be minimized, expanded, or discussed inline
 */

import { motion, AnimatePresence } from 'framer-motion';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTheme } from '../contexts/ThemeContext';
import { SOFT_SPRING, type Opportunity } from '../lib';

interface OpportunityLayerProps {
  opportunity: Opportunity | null;
  onClose: () => void;
  onDiscuss: (opportunityText: string) => void;
  onApply: () => void;
}

export function OpportunityLayer({
  opportunity,
  onClose,
  onDiscuss,
  onApply,
}: OpportunityLayerProps) {
  const { theme } = useTheme();
  const [isMinimized, setIsMinimized] = useState(false);
  const [isApplying, setIsApplying] = useState(false);

  if (!opportunity) return null;

  const handleApply = async () => {
    setIsApplying(true);
    try {
      await invoke('record_opportunity_response', {
        opportunityId: opportunity.id,
        accepted: true,
      });
      onApply();
    } catch (e) {
      console.error('Failed to record opportunity response:', e);
    } finally {
      setIsApplying(false);
    }
  };

  const handleIgnore = async () => {
    try {
      await invoke('record_opportunity_response', {
        opportunityId: opportunity.id,
        accepted: false,
      });
      onClose();
    } catch (e) {
      console.error('Failed to record opportunity response:', e);
    }
  };

  const handleDiscuss = () => {
    // Create a formatted message to discuss the opportunity
    const discussMessage = `💡 À propos de cette suggestion :\n\n${opportunity.suggestion}\n\nContexte : ${opportunity.context || 'Aucun contexte supplémentaire'}`;
    onDiscuss(discussMessage);
  };

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        transition={SOFT_SPRING}
        style={{
          position: 'relative',
          width: '100%',
          background: `linear-gradient(135deg, ${theme.accent}15, ${theme.accentLight}10)`,
          border: `1px solid ${theme.glass.border}`,
          borderRadius: '12px',
          marginBottom: '16px',
          overflow: 'hidden',
          boxShadow: '0 4px 16px rgba(0, 0, 0, 0.2)',
        }}
      >
        {/* Header */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '12px 16px',
            background: `${theme.accent}20`,
            borderBottom: isMinimized ? 'none' : `1px solid ${theme.glass.border}`,
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: '20px' }}>💡</span>
            <span
              style={{
                fontSize: '14px',
                fontWeight: '600',
                color: theme.text.primary,
              }}
            >
              Suggestion détectée
            </span>
          </div>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={() => setIsMinimized(!isMinimized)}
              style={{
                background: 'transparent',
                border: 'none',
                color: theme.text.muted,
                cursor: 'pointer',
                fontSize: '18px',
                padding: '4px 8px',
                transition: 'color 0.2s',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.color = theme.text.primary)}
              onMouseLeave={(e) => (e.currentTarget.style.color = theme.text.muted)}
            >
              {isMinimized ? '▼' : '▲'}
            </button>
            <button
              onClick={onClose}
              style={{
                background: 'transparent',
                border: 'none',
                color: theme.text.muted,
                cursor: 'pointer',
                fontSize: '18px',
                padding: '4px 8px',
                transition: 'color 0.2s',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.color = theme.text.primary)}
              onMouseLeave={(e) => (e.currentTarget.style.color = theme.text.muted)}
            >
              ✕
            </button>
          </div>
        </div>

        {/* Content (hidden when minimized) */}
        {!isMinimized && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            style={{ overflow: 'hidden' }}
          >
            <div style={{ padding: '16px' }}>
              {/* Context */}
              {opportunity.context && (
                <div
                  style={{
                    fontSize: '13px',
                    color: theme.text.secondary,
                    marginBottom: '12px',
                    padding: '8px 12px',
                    background: 'rgba(255, 255, 255, 0.05)',
                    borderRadius: '8px',
                  }}
                >
                  <strong>Contexte détecté :</strong> {opportunity.context}
                </div>
              )}

              {/* Suggestion */}
              <div
                style={{
                  fontSize: '14px',
                  color: theme.text.primary,
                  marginBottom: '16px',
                  lineHeight: '1.6',
                }}
              >
                {opportunity.suggestion}
              </div>

              {/* Actions */}
              <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                <button
                  onClick={handleApply}
                  disabled={isApplying}
                  style={{
                    flex: 1,
                    minWidth: '120px',
                    padding: '10px 16px',
                    background: `linear-gradient(135deg, ${theme.accent}, ${theme.accentLight})`,
                    border: 'none',
                    borderRadius: '8px',
                    color: 'white',
                    fontWeight: '600',
                    cursor: isApplying ? 'not-allowed' : 'pointer',
                    fontSize: '13px',
                    opacity: isApplying ? 0.6 : 1,
                    transition: 'all 0.2s',
                  }}
                  onMouseEnter={(e) => {
                    if (!isApplying) e.currentTarget.style.transform = 'translateY(-1px)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.transform = 'translateY(0)';
                  }}
                >
                  {isApplying ? '⏳ Application...' : '✓ Appliquer'}
                </button>

                <button
                  onClick={handleDiscuss}
                  style={{
                    flex: 1,
                    minWidth: '120px',
                    padding: '10px 16px',
                    background: 'rgba(255, 255, 255, 0.1)',
                    border: `1px solid ${theme.glass.border}`,
                    borderRadius: '8px',
                    color: theme.text.primary,
                    fontWeight: '600',
                    cursor: 'pointer',
                    fontSize: '13px',
                    transition: 'all 0.2s',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = 'rgba(255, 255, 255, 0.15)';
                    e.currentTarget.style.transform = 'translateY(-1px)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = 'rgba(255, 255, 255, 0.1)';
                    e.currentTarget.style.transform = 'translateY(0)';
                  }}
                >
                  💬 Discuter
                </button>

                <button
                  onClick={handleIgnore}
                  style={{
                    padding: '10px 16px',
                    background: 'transparent',
                    border: `1px solid ${theme.glass.border}`,
                    borderRadius: '8px',
                    color: theme.text.muted,
                    fontWeight: '500',
                    cursor: 'pointer',
                    fontSize: '13px',
                    transition: 'all 0.2s',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = theme.text.primary;
                    e.currentTarget.style.borderColor = theme.text.muted;
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = theme.text.muted;
                    e.currentTarget.style.borderColor = theme.glass.border;
                  }}
                >
                  Ignorer
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </motion.div>
    </AnimatePresence>
  );
}

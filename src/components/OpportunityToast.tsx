/**
 * One-Tap Help Toast
 * Shows opportunities detected by the trigger system
 */

import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { createPortal } from "react-dom";
import { 
  useEvent, 
  EVENTS, 
  shadowStore, 
  type Opportunity,
  SPRING_CONFIG 
} from "../lib";
import soundManager from "../lib/soundManager";
import { useLayoutPosition } from "../contexts/LayoutContext";
import { useTheme } from "../contexts/ThemeContext";

interface OpportunityToastProps {
  onOpenDock?: () => void;
  onOpenDigest?: () => void;
}

export default function OpportunityToast({ onOpenDock }: OpportunityToastProps) {
  const [opportunity, setOpportunity] = useState<Opportunity | null>(null);
  const { theme } = useTheme();
  
  // Register in layout system - Priority 2 (below QuickActions)
  const position = useLayoutPosition('opportunity-toast', 'bottom-right', 2, 200, 384);
  
  // Debug: Log state changes
  useEffect(() => {
    console.log('[OpportunityToast] 📦 State changed - opportunity:', opportunity);
  }, [opportunity]);

  // Listen for opportunities from backend
  useEvent<Opportunity>(EVENTS.OPPORTUNITY, (opp) => {
    console.log('[OpportunityToast] 🎯 Handler called with:', opp);
    console.log('[OpportunityToast] 🆔 Opportunity ID:', opp.id);
    console.log('[OpportunityToast] 📊 Confidence:', opp.confidence);
    
    // Skip if already dismissed
    if (shadowStore.isOpportunityDismissed(opp.id)) {
      console.log('[OpportunityToast] ⚠️ SKIPPED - Already dismissed:', opp.id);
      return;
    }

    // Only show high-confidence opportunities
    if (opp.confidence > 0.7) {
      console.log('[OpportunityToast] ✅ Showing toast for:', opp.id);
      setOpportunity(opp);
      
      // Play toast-in sound (Cluely)
      soundManager.play('toast-in');

      // Auto-dismiss after 10s
      setTimeout(() => {
        console.log('[OpportunityToast] ⏱️ Auto-dismissing:', opp.id);
        setOpportunity(null);
        soundManager.play('toast-out');
      }, 10000);
    } else {
      console.log('[OpportunityToast] ⚠️ SKIPPED - Low confidence:', opp.confidence);
    }
  });

  const handleView = async () => {
    if (!opportunity) return;

    try {
      // Record user accepted
      await invoke("record_opportunity_response", {
        opportunityId: opportunity.id,
        accepted: true,
      });

      // Open dock to show details
      onOpenDock?.();
    } catch (e) {
      console.error("Failed to record opportunity response:", e);
    }

    setOpportunity(null);
  };

  const handleDismiss = async () => {
    if (!opportunity) return;

    try {
      // Record dismissed
      await invoke("record_opportunity_response", {
        opportunityId: opportunity.id,
        accepted: false,
      });

      // Mark as dismissed in store
      shadowStore.dismissOpportunity(opportunity.id);
    } catch (e) {
      console.error("Failed to record opportunity response:", e);
    }

    setOpportunity(null);
  };

  // Render using Portal to escape parent overflow:hidden
  return createPortal(
    <AnimatePresence mode="wait">
      {opportunity && (
        <motion.div
          key={opportunity.id}
          initial={{ opacity: 0, y: 20, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: 20, scale: 0.95 }}
          transition={SPRING_CONFIG}
          className=""
          data-testid="opportunity-toast"
          style={{
            position: 'fixed',
            ...position,
            maxWidth: '384px',
            zIndex: 9999,
          }}
        >
          {/* Same style as QuickActions */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "12px",
              padding: "16px",
              background: "var(--glass-bg)",
              backdropFilter: "var(--glass-backdrop)",
              WebkitBackdropFilter: "var(--glass-backdrop)",
              border: `1px solid ${theme.glass.border}`,
              borderRadius: "var(--radius-xl)",
              boxShadow: theme.glass.shadow,
              transition: `all ${theme.transitionSpeed}ms ease`,
            }}
          >
            {/* Header */}
            <div style={{ display: "flex", alignItems: "start", gap: "12px" }}>
              <motion.div
                animate={{
                  rotate: [0, 10, -10, 10, 0],
                  scale: [1, 1.1, 1],
                }}
                transition={{
                  duration: 0.5,
                  ease: "easeInOut",
                }}
                style={{ fontSize: "24px" }}
              >
                💡
              </motion.div>
              <div style={{ flex: 1 }}>
                <div
                  style={{
                    fontSize: "13px",
                    fontWeight: "600",
                    color: "var(--text-primary)",
                    marginBottom: "4px",
                  }}
                >
                  J'ai une idée
                </div>
                <div
                  style={{
                    fontSize: "11px",
                    color: "var(--text-muted)",
                    lineHeight: "1.4",
                  }}
                >
                  {opportunity.preview}
                </div>
              </div>
            </div>

            {/* Confidence indicator */}
            <div>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  fontSize: "11px",
                  color: "var(--text-muted)",
                  marginBottom: "4px",
                }}
              >
                <span>Confiance</span>
                <span>{Math.round(opportunity.confidence * 100)}%</span>
              </div>
              <div
                style={{
                  height: "4px",
                  background: "rgba(255, 255, 255, 0.1)",
                  borderRadius: "2px",
                  overflow: "hidden",
                }}
              >
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${opportunity.confidence * 100}%` }}
                  transition={{ duration: 0.5, ease: "easeOut" }}
                  style={{
                    height: "100%",
                    background: `linear-gradient(90deg, ${theme.accent}, ${theme.accentLight})`,
                    borderRadius: "2px",
                  }}
                />
              </div>
            </div>

            {/* Actions */}
            <div style={{ display: "flex", gap: "8px" }}>
              <button
                onClick={handleView}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = theme.accent;
                  e.currentTarget.style.transform = "scale(1.02)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = `${theme.accent}cc`;
                  e.currentTarget.style.transform = "scale(1)";
                }}
                style={{
                  flex: 1,
                  padding: "8px 12px",
                  background: `${theme.accent}cc`,
                  color: "white",
                  border: "none",
                  borderRadius: "var(--radius-lg)",
                  fontSize: "12px",
                  fontWeight: "600",
                  cursor: "pointer",
                  transition: "all 0.2s",
                }}
              >
                Voir →
              </button>
              <button
                onClick={handleDismiss}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = "rgba(255, 255, 255, 0.15)";
                  e.currentTarget.style.transform = "scale(1.02)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
                  e.currentTarget.style.transform = "scale(1)";
                }}
                style={{
                  padding: "8px 12px",
                  background: "rgba(255, 255, 255, 0.08)",
                  color: "var(--text-secondary)",
                  border: "1px solid rgba(255, 255, 255, 0.1)",
                  borderRadius: "var(--radius-lg)",
                  fontSize: "12px",
                  cursor: "pointer",
                  transition: "all 0.2s",
                }}
              >
                Ignorer
              </button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>,
    document.body
  );
}


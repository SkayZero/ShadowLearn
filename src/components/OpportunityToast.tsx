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

interface OpportunityToastProps {
  onOpenDock?: () => void;
  onOpenDigest?: () => void;
}

export default function OpportunityToast({ onOpenDock }: OpportunityToastProps) {
  const [opportunity, setOpportunity] = useState<Opportunity | null>(null);
  
  // Register in layout system - Priority 2 (below QuickActions)
  const position = useLayoutPosition('opportunity-toast', 'bottom-right', 2, 180, 384);
  
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
          {/* CLUELY DESIGN - Ultra transparent glass toast */}
          <div
            className="rounded-2xl p-4 shadow-xl"
            style={{
              background: "rgba(15, 23, 42, 0.3)",
              backdropFilter: "blur(40px) saturate(200%)",
              border: "1px solid rgba(255, 255, 255, 0.05)",
              boxShadow: "0 8px 32px rgba(0, 0, 0, 0.2)",
            }}
          >
            {/* Header */}
            <div className="flex items-start gap-3 mb-3">
              <motion.div
                animate={{
                  rotate: [0, 10, -10, 10, 0],
                  scale: [1, 1.1, 1],
                }}
                transition={{
                  duration: 0.5,
                  ease: "easeInOut",
                }}
                className="text-2xl"
              >
                💡
              </motion.div>
              <div className="flex-1">
                <h3 className="font-semibold text-white text-sm">
                  J'ai une idée
                </h3>
                <p className="text-xs text-gray-300 mt-0.5 line-clamp-2">
                  {opportunity.preview}
                </p>
              </div>
            </div>

            {/* Confidence indicator */}
            <div className="mb-3">
              <div className="flex items-center justify-between text-xs text-gray-300 mb-1">
                <span>Confiance</span>
                <span>{Math.round(opportunity.confidence * 100)}%</span>
              </div>
              <div className="h-1 bg-gray-700 rounded-full overflow-hidden">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${opportunity.confidence * 100}%` }}
                  transition={{ duration: 0.5, ease: "easeOut" }}
                  className="h-full bg-gradient-to-r from-emerald-500 to-sky-500"
                />
              </div>
            </div>

            {/* Actions */}
            <div className="flex gap-2">
              <button
                onClick={handleView}
                className="flex-1 px-4 py-2 rounded-lg text-sm font-medium transition-all transform hover:scale-105 active:scale-95"
                style={{
                  background: "rgba(59, 130, 246, 0.8)",
                  color: "white",
                }}
              >
                Voir →
              </button>
              <button
                onClick={handleDismiss}
                className="px-4 py-2 rounded-lg text-sm transition-all"
                style={{
                  background: "rgba(255, 255, 255, 0.1)",
                  color: "rgba(255, 255, 255, 0.8)",
                  border: "1px solid rgba(255, 255, 255, 0.2)",
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


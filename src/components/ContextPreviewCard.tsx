/**
 * Context Preview Card
 * Shows preview of current context on hover
 */

import { motion, AnimatePresence } from "framer-motion";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useEvent, EVENTS, shadowStore, type ContextPreview, SOFT_SPRING } from "../lib";
import { useTheme } from "../contexts/ThemeContext";

export interface ContextPreviewCardProps {
  visible: boolean;
  onClose?: () => void;
}

export function ContextPreviewCard({ visible, onClose }: ContextPreviewCardProps) {
  const { theme } = useTheme();
  const [context, setContext] = useState<ContextPreview | null>(
    shadowStore.getContext()
  );

  // Listen for context updates
  useEvent<ContextPreview>(EVENTS.CONTEXT_UPDATE, (ctx) => {
    setContext(ctx);
    shadowStore.updateContext(ctx);
  });

  // Fetch context initially and periodically
  useEffect(() => {
    const fetchContext = async () => {
      try {
        const ctx = await invoke<ContextPreview>("get_context_preview");
        setContext(ctx);
        shadowStore.updateContext(ctx);
      } catch (e) {
        console.error("Failed to fetch context preview:", e);
      }
    };

    if (visible) {
      fetchContext();
    }
  }, [visible]);

  if (!context) return null;

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          initial={{ opacity: 0, y: 10, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: 10, scale: 0.95 }}
          transition={SOFT_SPRING}
          className="absolute bottom-full right-0 mb-3 w-80 z-50"
        >
          {/* Glass morphism card matching theme */}
          <div
            className="rounded-2xl p-4 shadow-2xl"
            style={{
              background: theme.glass.bg,
              backdropFilter: "blur(40px) saturate(180%)",
              WebkitBackdropFilter: "blur(40px) saturate(180%)",
              border: `1px solid ${theme.glass.border}`,
              boxShadow: theme.glass.shadow,
              transition: `all ${theme.transitionSpeed}ms ease`,
            }}
          >
            <div className="flex items-center justify-between mb-3">
              <h4
                className="font-semibold text-sm"
                style={{ color: theme.text.primary }}
              >
                Contexte actuel
              </h4>
              {onClose && (
                <button
                  onClick={onClose}
                  className="transition-colors text-xs"
                  style={{ color: theme.text.muted }}
                  onMouseEnter={(e) => e.currentTarget.style.color = theme.text.secondary}
                  onMouseLeave={(e) => e.currentTarget.style.color = theme.text.muted}
                >
                  ✕
                </button>
              )}
            </div>

            {/* Current app */}
            <div
              className="flex items-center gap-3 mb-3 p-3 rounded-xl"
              style={{
                background: `${theme.accent}15`,
                border: `1px solid ${theme.glass.border}`,
              }}
            >
              <span className="text-2xl">💻</span>
              <div className="flex-1 min-w-0">
                <div
                  className="font-medium text-sm"
                  style={{ color: theme.text.primary }}
                >
                  {context.app_name}
                </div>
                <div
                  className="text-xs truncate"
                  style={{ color: theme.text.secondary }}
                >
                  {context.window_title}
                </div>
              </div>
            </div>

            {/* Work duration */}
            {context.session_duration_minutes > 0 && (
              <div className="flex items-center justify-between text-sm mb-3 px-3">
                <span style={{ color: theme.text.secondary }}>Session en cours</span>
                <span
                  className="font-medium"
                  style={{ color: theme.accent }}
                >
                  {context.session_duration_minutes} min
                </span>
              </div>
            )}

            {/* Idle time */}
            {context.idle_seconds > 0 && (
              <div className="flex items-center justify-between text-sm mb-3 px-3">
                <span className="text-gray-300">Inactif depuis</span>
                <span className="font-medium text-white">
                  {Math.round(context.idle_seconds)}s
                </span>
              </div>
            )}

            {/* Recent screenshots */}
            {context.recent_screenshots > 0 && (
              <div className="flex items-center justify-between text-sm mb-3 px-3">
                <span className="text-gray-300">Captures récentes</span>
                <span className="font-medium text-white">
                  {context.recent_screenshots}
                </span>
              </div>
            )}

            {/* Domain tag */}
            {context.domain && (
              <div className="px-3 mb-3">
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                  {context.domain}
                </span>
              </div>
            )}

            {/* Pending suggestion */}
            {context.pending_suggestion && (
              <div className="mt-3 pt-3 border-t px-3" style={{ borderColor: "rgba(255, 255, 255, 0.1)" }}>
                <div className="text-xs text-gray-300 mb-1">
                  💡 Suggestion prête
                </div>
                <div className="text-sm text-white line-clamp-2">
                  {context.pending_suggestion}
                </div>
              </div>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}


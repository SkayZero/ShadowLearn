import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { useEvent, EVENTS, type MicroSuggestion } from "../lib";

interface SmartPill {
  id: string;
  text: string;
  icon: string;
  type: "suggestion" | "action" | "info";
  confidence: number;
  action: () => Promise<void>;
}

interface SmartPillsProps {
  context?: {
    app?: string;
    selectedText?: string;
    recentActivity?: string;
  };
}

export function SmartPills({ context }: SmartPillsProps) {
  const [pills, setPills] = useState<SmartPill[]>([]);
  const [dismissedIds, setDismissedIds] = useState<Set<string>>(new Set());

  // Listen for micro-suggestions from backend
  useEvent<MicroSuggestion[]>(EVENTS.MICRO_SUGGESTION, (suggestions) => {
    const newPills: SmartPill[] = suggestions
      .filter(s => !dismissedIds.has(s.id))
      .map(s => ({
        id: s.id,
        text: s.text,
        icon: s.type === "help" ? "💡" : s.type === "reminder" ? "⏰" : "▶️",
        type: "suggestion" as const,
        confidence: 0.8,
        action: async () => {
          await invoke("dismiss_pill", { pillId: s.id });
          dismissPill(s.id);
        },
      }));
    
    setPills(prev => {
      // Merge new pills with existing ones, avoiding duplicates
      const existingIds = new Set(prev.map(p => p.id));
      const uniqueNew = newPills.filter(p => !existingIds.has(p.id));
      return [...prev, ...uniqueNew];
    });
  });

  useEffect(() => {
    // Génère des suggestions intelligentes basées sur le contexte
    const generatePills = async () => {
      const newPills: SmartPill[] = [];

      // Suggestion basée sur l'app
      if (context?.app?.includes("Code") || context?.app?.includes("Cursor")) {
        newPills.push({
          id: "code-review",
          text: "Revoir ce code ?",
          icon: "👁️",
          type: "suggestion",
          confidence: 0.85,
          action: async () => {
            await invoke("chat_with_ai", {
              message: "/améliorer le code actuel",
            });
            dismissPill("code-review");
          },
        });

        newPills.push({
          id: "explain-code",
          text: "Besoin d'explications ?",
          icon: "💡",
          type: "suggestion",
          confidence: 0.75,
          action: async () => {
            await invoke("chat_with_ai", {
              message: "/expliquer ce code",
            });
            dismissPill("explain-code");
          },
        });
      }

      // Suggestion basée sur le texte sélectionné
      if (context?.selectedText && context.selectedText.length > 50) {
        newPills.push({
          id: "summarize-text",
          text: "Résumer ça ?",
          icon: "📝",
          type: "suggestion",
          confidence: 0.9,
          action: async () => {
            await invoke("chat_with_ai", {
              message: `/résumer ${context.selectedText}`,
            });
            dismissPill("summarize-text");
          },
        });
      }

      // Suggestion basée sur l'activité récente
      if (context?.recentActivity === "stuck") {
        newPills.push({
          id: "take-break",
          text: "Prendre une pause ?",
          icon: "☕",
          type: "info",
          confidence: 0.7,
          action: async () => {
            dismissPill("take-break");
          },
        });
      }

      // Suggestion générale: raccourci clavier
      if (Math.random() > 0.7) {
        newPills.push({
          id: "keyboard-shortcut",
          text: "Astuce: Cmd+Shift+Space",
          icon: "⌨️",
          type: "info",
          confidence: 0.6,
          action: async () => {
            dismissPill("keyboard-shortcut");
          },
        });
      }

      // Filtrer les pills déjà dismissées
      const activePills = newPills.filter(
        (pill) => !dismissedIds.has(pill.id)
      );

      setPills(activePills);
    };

    generatePills();
    
    // Refresh pills every 30 seconds
    const interval = setInterval(generatePills, 30000);
    return () => clearInterval(interval);
  }, [context, dismissedIds]);

  const dismissPill = (id: string) => {
    setDismissedIds((prev) => new Set(prev).add(id));
    setPills((prev) => prev.filter((p) => p.id !== id));
  };

  if (pills.length === 0) {
    return null;
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        style={{
          position: "fixed",
          top: "80px", // Below header + status indicator
          right: "var(--space-24)",
          display: "flex",
          flexDirection: "column",
          gap: "var(--space-8)",
          zIndex: 45, // z-toasts-pills
          maxWidth: "320px",
        }}
      >
        {pills.map((pill, index) => (
          <motion.div
            key={pill.id}
            initial={{ opacity: 0, x: 50, scale: 0.9 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: 50, scale: 0.9 }}
            transition={{ delay: index * 0.1 }}
            onClick={pill.action}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "10px",
              padding: "10px 14px",
              background: "var(--glass-bg)",
              backdropFilter: "var(--glass-backdrop)",
              WebkitBackdropFilter: "var(--glass-backdrop)",
              border: "1px solid var(--glass-border)",
              borderRadius: "999px", // Full pill shape
              boxShadow: "var(--glass-shadow)",
              cursor: "pointer",
              transition: "all 0.2s ease",
              position: "relative",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "rgba(135, 206, 235, 0.2)";
              e.currentTarget.style.borderColor = "var(--accent-primary)";
              e.currentTarget.style.transform = "scale(1.05)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "var(--glass-bg)";
              e.currentTarget.style.borderColor = "var(--glass-border)";
              e.currentTarget.style.transform = "scale(1)";
            }}
          >
            {/* Icon */}
            <span style={{ fontSize: "16px", flexShrink: 0 }}>
              {pill.icon}
            </span>

            {/* Text */}
            <span
              style={{
                fontSize: "13px",
                fontWeight: "500",
                color: "var(--text-primary)",
                whiteSpace: "nowrap",
                overflow: "hidden",
                textOverflow: "ellipsis",
              }}
            >
              {pill.text}
            </span>

            {/* Confidence indicator (optional, subtle) */}
            {pill.confidence > 0.8 && (
              <span
                style={{
                  fontSize: "10px",
                  color: "var(--accent-emerald)",
                  fontWeight: "600",
                }}
              >
                ✓
              </span>
            )}

            {/* Dismiss button */}
            <button
              onClick={(e) => {
                e.stopPropagation();
                dismissPill(pill.id);
              }}
              style={{
                marginLeft: "4px",
                padding: "2px 6px",
                background: "transparent",
                border: "none",
                color: "var(--text-muted)",
                cursor: "pointer",
                fontSize: "14px",
                transition: "color 0.2s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.color = "var(--text-primary)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.color = "var(--text-muted)";
              }}
            >
              ×
            </button>
          </motion.div>
        ))}
      </motion.div>
    </AnimatePresence>
  );
}




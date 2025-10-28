import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { useLayoutPosition } from "../contexts/LayoutContext";

interface QuickAction {
  id: string;
  label: string;
  icon: string;
  description: string;
  action: () => Promise<void>;
}

interface ContextualActionsProps {
  context?: {
    app?: string;
    selectedText?: string;
    url?: string;
    language?: string;
  };
  onOpenDock?: () => void;
  onOpenDigest?: () => void;
}

export function QuickActions({ context, onOpenDock, onOpenDigest }: ContextualActionsProps) {
  const [actions, setActions] = useState<QuickAction[]>([]);
  const [isVisible, setIsVisible] = useState(false);
  
  // Register in layout system - Priority 1 (closest to edge)
  const position = useLayoutPosition('quick-actions', 'bottom-right', 1, 150, 200);

  useEffect(() => {
    // Génère des actions contextuelles basées sur le contexte actuel
    const generateActions = async () => {
      const contextActions: QuickAction[] = [];

      // Actions universelles toujours visibles
      if (onOpenDock) {
        contextActions.push({
          id: "open-dock",
          label: "Ouvrir Dock",
          icon: "🎛️",
          description: "Afficher le panneau principal",
          action: async () => {
            console.log("Opening dock");
            onOpenDock();
          },
        });
      }
      
      // Action: Stats/Digest
      if (onOpenDigest) {
        contextActions.push({
          id: "view-stats",
          label: "Voir mes stats",
          icon: "📊",
          description: "Digest du jour",
          action: async () => {
            console.log("Opening digest");
            onOpenDigest();
          },
        });
      }

      // Si du texte est sélectionné
      if (context?.selectedText) {
        contextActions.push({
          id: "explain",
          label: "Expliquer",
          icon: "💡",
          description: "Expliquer la sélection",
          action: async () => {
            console.log("Explaining:", context.selectedText);
            await invoke("chat_with_ai", {
              message: `/expliquer ${context.selectedText}`,
            });
          },
        });

        contextActions.push({
          id: "simplify",
          label: "Simplifier",
          icon: "🔄",
          description: "Reformuler plus simplement",
          action: async () => {
            console.log("Simplifying:", context.selectedText);
            await invoke("chat_with_ai", {
              message: `/pasclair ${context.selectedText}`,
            });
          },
        });

        contextActions.push({
          id: "translate",
          label: "Traduire",
          icon: "🌐",
          description: "Traduire le texte",
          action: async () => {
            console.log("Translating:", context.selectedText);
            await invoke("chat_with_ai", {
              message: `/traduire ${context.selectedText}`,
            });
          },
        });
      }

      // Si dans un IDE (VSCode, Cursor, etc.)
      if (context?.app?.includes("Code") || context?.app?.includes("Cursor")) {
        contextActions.push({
          id: "debug",
          label: "Débugger",
          icon: "🐛",
          description: "Analyser l'erreur",
          action: async () => {
            console.log("Debugging code");
            await invoke("chat_with_ai", {
              message: "/debug",
            });
          },
        });

        contextActions.push({
          id: "improve",
          label: "Améliorer",
          icon: "✨",
          description: "Suggérer des améliorations",
          action: async () => {
            console.log("Improving code");
            await invoke("chat_with_ai", {
              message: "/améliorer",
            });
          },
        });
      }

      // Si sur un navigateur web
      if (context?.app?.includes("Chrome") || context?.app?.includes("Safari") || context?.app?.includes("Firefox")) {
        contextActions.push({
          id: "summarize",
          label: "Résumer",
          icon: "📝",
          description: "Résumer la page",
          action: async () => {
            console.log("Summarizing page");
            await invoke("chat_with_ai", {
              message: "/résumer cette page",
            });
          },
        });
      }

      // Action universelle: Aide
      contextActions.push({
        id: "help",
        label: "Aide",
        icon: "❓",
        description: "Afficher l'aide",
        action: async () => {
          console.log("Showing help");
          await invoke("chat_with_ai", {
            message: "/help",
          });
        },
      });

      setActions(contextActions);
      setIsVisible(contextActions.length > 0);
    };

    generateActions();
  }, [context]);

  if (!isVisible || actions.length === 0) {
    return null;
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, scale: 0.9, y: 10 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.9, y: 10 }}
        transition={{ duration: 0.2 }}
        style={{
          position: "fixed",
          ...position,
          display: "flex",
          flexDirection: "column",
          gap: "var(--space-8)",
          zIndex: 45, // z-toasts-pills
        }}
      >
        {actions.map((action, index) => (
          <motion.button
            key={action.id}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 20 }}
            transition={{ delay: index * 0.05 }}
            onClick={action.action}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "12px",
              padding: "12px 16px",
              background: "var(--glass-bg)",
              backdropFilter: "var(--glass-backdrop)",
              WebkitBackdropFilter: "var(--glass-backdrop)",
              border: "1px solid var(--glass-border)",
              borderRadius: "var(--radius-xl)",
              boxShadow: "var(--glass-shadow)",
              cursor: "pointer",
              transition: "all 0.2s ease",
              minWidth: "200px",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "rgba(135, 206, 235, 0.2)";
              e.currentTarget.style.borderColor = "var(--accent-primary)";
              e.currentTarget.style.transform = "translateX(-4px)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "var(--glass-bg)";
              e.currentTarget.style.borderColor = "var(--glass-border)";
              e.currentTarget.style.transform = "translateX(0)";
            }}
          >
            <span style={{ fontSize: "20px" }}>{action.icon}</span>
            <div style={{ flex: 1, textAlign: "left" }}>
              <div
                style={{
                  fontSize: "13px",
                  fontWeight: "600",
                  color: "var(--text-primary)",
                }}
              >
                {action.label}
              </div>
              <div
                style={{
                  fontSize: "11px",
                  color: "var(--text-muted)",
                }}
              >
                {action.description}
              </div>
            </div>
          </motion.button>
        ))}
      </motion.div>
    </AnimatePresence>
  );
}


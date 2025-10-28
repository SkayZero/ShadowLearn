/**
 * Slash Commands with Autocomplete
 * Type "/" to show command palette
 */

import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

export interface Command {
  trigger: string;
  label: string;
  description: string;
  icon: string;
  action: (args: string) => Promise<void>;
}

export const COMMANDS: Command[] = [
  {
    trigger: "/help",
    label: "Aide",
    description: "Afficher l'aide générale",
    icon: "❓",
    action: async (args) => {
      console.log("Help requested:", args);
    },
  },
  {
    trigger: "/resume",
    label: "Résumer",
    description: "Résumer le texte sélectionné",
    icon: "📝",
    action: async (args) => {
      await invoke("execute_slash_command", {
        command: "resume",
        context: args,
      });
    },
  },
  {
    trigger: "/explain",
    label: "Expliquer",
    description: "Expliquer un concept",
    icon: "🔍",
    action: async (args) => {
      await invoke("execute_slash_command", {
        command: "explain",
        context: args,
      });
    },
  },
  {
    trigger: "/debug",
    label: "Débugger",
    description: "Analyser une erreur",
    icon: "🐛",
    action: async (args) => {
      await invoke("execute_slash_command", {
        command: "debug",
        context: args,
      });
    },
  },
  {
    trigger: "/improve",
    label: "Améliorer",
    description: "Suggérer des améliorations",
    icon: "✨",
    action: async (args) => {
      await invoke("execute_slash_command", {
        command: "improve",
        context: args,
      });
    },
  },
  {
    trigger: "/translate",
    label: "Traduire",
    description: "Traduire du texte",
    icon: "🌐",
    action: async (args) => {
      await invoke("execute_slash_command", {
        command: "translate",
        context: args,
      });
    },
  },
];

export interface SlashCommandsProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  onCommandResult?: (result: string) => void; // NEW: Callback pour afficher le résultat
  placeholder?: string;
  onOpenDigest?: () => void;
  onOpenDock?: () => void;
}

export function SlashCommands({
  value,
  onChange,
  onSubmit,
  onCommandResult,
  placeholder = "Écris ou tape / pour les commandes...",
}: SlashCommandsProps) {
  const [showCommands, setShowCommands] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Filter commands based on input
  const filteredCommands = COMMANDS.filter((cmd) =>
    cmd.trigger.toLowerCase().includes(value.toLowerCase().trim())
  );

  // Show command palette when "/" is typed
  useEffect(() => {
    if (value.startsWith("/") && value.length > 0) {
      setShowCommands(true);
      setSelectedIndex(0);
    } else {
      setShowCommands(false);
    }
  }, [value]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showCommands) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => (i + 1) % filteredCommands.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex(
        (i) => (i - 1 + filteredCommands.length) % filteredCommands.length
      );
    } else if (e.key === "Enter" || e.key === "Tab") {
      if (filteredCommands.length > 0) {
        e.preventDefault();
        selectCommand(filteredCommands[selectedIndex]);
      }
    } else if (e.key === "Escape") {
      setShowCommands(false);
    }
  };

  const selectCommand = (cmd: Command) => {
    onChange(cmd.trigger + " ");
    setShowCommands(false);
    inputRef.current?.focus();
  };

  const handleSubmitWithCommand = async () => {
    const parts = value.split(" ");
    const cmdTrigger = parts[0];
    const args = parts.slice(1).join(" ");

    const command = COMMANDS.find((c) => c.trigger === cmdTrigger);

    if (command) {
      try {
        // Execute command and get result
        const result = await invoke<{ success: boolean; message: string }>("execute_slash_command", {
          command: cmdTrigger.replace("/", ""),
          context: args,
        });
        
        // Display result in chat
        if (onCommandResult && result.message) {
          onCommandResult(result.message);
        }
        
        onChange("");
      } catch (error) {
        console.error("Command execution failed:", error);
        if (onCommandResult) {
          onCommandResult(`❌ Erreur: ${error}`);
        }
      }
    } else {
      // Call onSubmit without arguments
      onSubmit();
    }
  };

  return (
    <div style={{ position: "relative", width: "100%" }}>
      {/* Command palette */}
      <AnimatePresence>
        {showCommands && filteredCommands.length > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 8, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 8, scale: 0.96 }}
            transition={{ duration: 0.15 }}
            style={{
              position: "absolute",
              bottom: "100%",
              left: 0,
              right: 0,
              marginBottom: "8px",
              background: "var(--glass-bg)",
              backdropFilter: "var(--glass-backdrop)",
              WebkitBackdropFilter: "var(--glass-backdrop)",
              border: "1px solid var(--glass-border)",
              borderRadius: "12px",
              overflow: "hidden",
              boxShadow: "var(--glass-shadow)",
              zIndex: 99999, // High z-index pour le command palette
            }}
          >
            {filteredCommands.map((cmd, idx) => (
              <button
                key={cmd.trigger}
                onClick={() => selectCommand(cmd)}
                style={{
                  width: "100%",
                  textAlign: "left",
                  padding: "12px 16px",
                  display: "flex",
                  alignItems: "center",
                  gap: "12px",
                  border: "none",
                  background:
                    idx === selectedIndex
                      ? "var(--glass-emerald-tint)"
                      : "transparent",
                  borderLeft:
                    idx === selectedIndex
                      ? "2px solid var(--accent-primary)"
                      : "2px solid transparent",
                  color: "var(--text-primary)",
                  cursor: "pointer",
                  transition: "all 0.15s ease",
                }}
                onMouseEnter={() => setSelectedIndex(idx)}
              >
                <span style={{ fontSize: "24px" }}>{cmd.icon}</span>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontWeight: 500,
                      fontSize: "14px",
                      color: "var(--text-primary)",
                    }}
                  >
                    {cmd.label}
                  </div>
                  <div
                    style={{
                      fontSize: "12px",
                      color: "var(--text-muted)",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {cmd.description}
                  </div>
                </div>
                <kbd
                  style={{
                    padding: "2px 8px",
                    background: "rgba(255, 255, 255, 0.1)",
                    borderRadius: "4px",
                    fontSize: "11px",
                    fontFamily: "monospace",
                    color: "var(--text-secondary)",
                  }}
                >
                  {cmd.trigger}
                </kbd>
              </button>
            ))}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Input */}
      <div style={{ display: "flex", gap: "8px", position: "relative", zIndex: 99999 }}>
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => {
            handleKeyDown(e);
            if (e.key === "Enter" && !showCommands) {
              handleSubmitWithCommand();
            }
          }}
          placeholder={placeholder}
          style={{
            flex: 1,
            padding: "12px 16px",
            background: "rgba(255, 255, 255, 0.05)",
            border: "1px solid rgba(255, 255, 255, 0.1)",
            borderRadius: "12px",
            color: "var(--text-primary)",
            fontSize: "14px",
            outline: "none",
            transition: "all 0.2s ease",
          }}
          onFocus={(e) => {
            e.target.style.background = "rgba(255, 255, 255, 0.08)";
            e.target.style.borderColor = "var(--accent-primary)";
          }}
          onBlur={(e) => {
            e.target.style.background = "rgba(255, 255, 255, 0.05)";
            e.target.style.borderColor = "rgba(255, 255, 255, 0.1)";
          }}
        />
        <button
          onClick={handleSubmitWithCommand}
          style={{
            padding: "12px 24px",
            background: "linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-emerald) 100%)",
            border: "none",
            borderRadius: "12px",
            color: "white",
            fontSize: "14px",
            fontWeight: 600,
            cursor: "pointer",
            transition: "all 0.2s ease",
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = "translateY(-1px)";
            e.currentTarget.style.boxShadow = "0 4px 12px rgba(135, 206, 235, 0.4)";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = "translateY(0)";
            e.currentTarget.style.boxShadow = "none";
          }}
        >
          Envoyer
        </button>
      </div>
    </div>
  );
}


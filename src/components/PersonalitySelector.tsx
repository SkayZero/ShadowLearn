import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

export type Personality = "friendly" | "professional" | "concise" | "casual" | "motivational";

interface PersonalityOption {
  id: Personality;
  name: string;
  icon: string;
  description: string;
  example: string;
}

const PERSONALITIES: PersonalityOption[] = [
  {
    id: "friendly",
    name: "Ami sympathique",
    icon: "😊",
    description: "Chaleureux et encourageant",
    example: "Super ! Je vois que tu travailles sur ce bug. Je peux t'aider à le résoudre ensemble ?",
  },
  {
    id: "professional",
    name: "Professionnel",
    icon: "👔",
    description: "Formel et précis",
    example: "J'ai identifié une erreur dans le code. Je recommande d'ajouter une validation des entrées.",
  },
  {
    id: "concise",
    name: "Minimaliste",
    icon: "⚡",
    description: "Direct et court",
    example: "Bug ligne 42. Fix: ajouter null check.",
  },
  {
    id: "casual",
    name: "Décontracté",
    icon: "🤙",
    description: "Relax et cool",
    example: "Yo ! J'ai vu un petit souci dans ton code. On check ça vite fait ?",
  },
  {
    id: "motivational",
    name: "Coach",
    icon: "💪",
    description: "Motivant et positif",
    example: "Tu es sur la bonne voie ! Corrigeons ce petit détail et tu seras au top ! 🚀",
  },
];

interface PersonalitySelectorProps {
  compact?: boolean;
  onPersonalityChange?: (personality: Personality) => void;
}

export function PersonalitySelector({
  compact = false,
  onPersonalityChange,
}: PersonalitySelectorProps) {
  const [selectedPersonality, setSelectedPersonality] = useState<Personality>("friendly");
  const [showExample, setShowExample] = useState(false);

  useEffect(() => {
    // Load saved personality from backend
    loadPersonality();
  }, []);

  const loadPersonality = async () => {
    try {
      const personality = await invoke<string>("get_personality");
      setSelectedPersonality(personality as Personality);
    } catch (error) {
      console.error("Failed to load personality:", error);
    }
  };

  const handlePersonalityChange = async (personality: Personality) => {
    setSelectedPersonality(personality);
    setShowExample(true);
    
    // Save to backend
    try {
      await invoke("set_personality", { personality });
      console.log("Personality changed to:", personality);
    } catch (error) {
      console.error("Failed to save personality:", error);
    }

    onPersonalityChange?.(personality);

    // Hide example after 3s
    setTimeout(() => setShowExample(false), 3000);
  };

  const selectedOption = PERSONALITIES.find((p) => p.id === selectedPersonality);

  if (compact) {
    return (
      <div
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: "8px",
          padding: "6px 12px",
          background: "var(--glass-bg)",
          border: "1px solid var(--glass-border)",
          borderRadius: "999px",
          cursor: "pointer",
        }}
        onClick={() => {
          const currentIndex = PERSONALITIES.findIndex((p) => p.id === selectedPersonality);
          const nextIndex = (currentIndex + 1) % PERSONALITIES.length;
          handlePersonalityChange(PERSONALITIES[nextIndex].id);
        }}
      >
        <span style={{ fontSize: "16px" }}>{selectedOption?.icon}</span>
        <span
          style={{
            fontSize: "12px",
            fontWeight: "500",
            color: "var(--text-primary)",
          }}
        >
          {selectedOption?.name}
        </span>
      </div>
    );
  }

  return (
    <div
      style={{
        padding: "20px",
        background: "var(--glass-bg)",
        backdropFilter: "var(--glass-backdrop)",
        WebkitBackdropFilter: "var(--glass-backdrop)",
        border: "1px solid var(--glass-border)",
        borderRadius: "var(--radius-xl)",
        boxShadow: "var(--glass-shadow)",
      }}
    >
      <h3
        style={{
          fontSize: "16px",
          fontWeight: "600",
          color: "var(--text-primary)",
          marginBottom: "16px",
        }}
      >
        🎭 Personnalité
      </h3>

      <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
        {PERSONALITIES.map((personality) => (
          <motion.button
            key={personality.id}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={() => handlePersonalityChange(personality.id)}
            style={{
              padding: "12px 16px",
              background:
                selectedPersonality === personality.id
                  ? "rgba(135, 206, 235, 0.2)"
                  : "rgba(255, 255, 255, 0.05)",
              border: "1px solid",
              borderColor:
                selectedPersonality === personality.id
                  ? "var(--accent-primary)"
                  : "rgba(255, 255, 255, 0.1)",
              borderRadius: "8px",
              cursor: "pointer",
              transition: "all 0.2s",
              textAlign: "left",
              display: "flex",
              alignItems: "center",
              gap: "12px",
            }}
          >
            <span style={{ fontSize: "24px" }}>{personality.icon}</span>
            <div style={{ flex: 1 }}>
              <div
                style={{
                  fontSize: "14px",
                  fontWeight: "600",
                  color: "var(--text-primary)",
                }}
              >
                {personality.name}
              </div>
              <div
                style={{
                  fontSize: "12px",
                  color: "var(--text-muted)",
                  marginTop: "2px",
                }}
              >
                {personality.description}
              </div>
            </div>
            {selectedPersonality === personality.id && (
              <div style={{ color: "var(--accent-primary)", fontSize: "18px" }}>✓</div>
            )}
          </motion.button>
        ))}
      </div>

      {/* Example preview */}
      {showExample && selectedOption && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          exit={{ opacity: 0, height: 0 }}
          style={{
            marginTop: "16px",
            padding: "12px",
            background: "rgba(16, 185, 129, 0.1)",
            borderRadius: "8px",
            fontSize: "12px",
            color: "var(--text-primary)",
            lineHeight: "1.5",
          }}
        >
          <strong style={{ display: "block", marginBottom: "4px" }}>Exemple :</strong>
          {selectedOption.example}
        </motion.div>
      )}
    </div>
  );
}




import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { useTheme } from "../contexts/ThemeContext";
import { Personality, THEMES } from "../lib/themes";

interface PersonalityOption {
  id: Personality;
  name: string;
  icon: string;
  description: string;
  example: string;
}

const PERSONALITIES: PersonalityOption[] = [
  {
    id: "aerya",
    name: "AERYA",
    icon: "🌊",
    description: "Assistant équilibré, bienveillant",
    example: "Je suis là pour t'accompagner. Ensemble, trouvons la meilleure solution.",
  },
  {
    id: "aura",
    name: "AURA",
    icon: "🔮",
    description: "Sage calme, méditatif",
    example: "Prends un moment pour respirer. Observons ensemble ce défi avec clarté et sérénité.",
  },
  {
    id: "spark",
    name: "SPARK",
    icon: "⚡",
    description: "Énergique, motivant",
    example: "Allez ! On fonce ! Ce bug n'a aucune chance face à ton talent ! 🚀",
  },
  {
    id: "nova",
    name: "NOVA",
    icon: "✨",
    description: "Visionnaire, poétique",
    example: "Chaque ligne de code est une étoile dans ta constellation. Créons quelque chose de beau.",
  },
  {
    id: "kai",
    name: "KAI",
    icon: "⚙️",
    description: "Pratique, mentor tech",
    example: "Erreur détectée ligne 42. Stack trace analysé. Solution optimale : refactoring.",
  },
  {
    id: "echo",
    name: "ECHO",
    icon: "🎨",
    description: "Artiste rêveur",
    example: "Ton code est une toile. Laisse-moi t'aider à y ajouter les touches finales.",
  },
  {
    id: "void",
    name: "VOID",
    icon: "⬛",
    description: "Minimaliste, silencieux",
    example: "Bug. Fix. Done.",
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
  const { personality: selectedPersonality, setPersonality } = useTheme();
  const [showExample, setShowExample] = useState(false);

  const handlePersonalityChange = async (personality: Personality) => {
    setShowExample(true);
    
    try {
      await setPersonality(personality);
      onPersonalityChange?.(personality);
    } catch (error) {
      console.error("Failed to change personality:", error);
    }

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
                  ? "var(--theme-glass-bg)"
                  : "rgba(255, 255, 255, 0.05)",
              border: "1px solid",
              borderColor:
                selectedPersonality === personality.id
                  ? "var(--theme-accent)"
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
              <div style={{ color: "var(--theme-accent)", fontSize: "18px" }}>✓</div>
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




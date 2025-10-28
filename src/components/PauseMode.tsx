import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface PauseModeProps {
  compact?: boolean;
  onPauseChange?: (isPaused: boolean) => void;
}

type PauseDuration = "30min" | "1h" | "2h" | "4h" | "until_tomorrow" | "custom";

interface PauseDurationOption {
  id: PauseDuration;
  label: string;
  minutes?: number;
}

const PAUSE_DURATIONS: PauseDurationOption[] = [
  { id: "30min", label: "30 minutes", minutes: 30 },
  { id: "1h", label: "1 heure", minutes: 60 },
  { id: "2h", label: "2 heures", minutes: 120 },
  { id: "4h", label: "4 heures", minutes: 240 },
  { id: "until_tomorrow", label: "Jusqu'à demain", minutes: undefined },
];

export function PauseMode({ compact = false, onPauseChange }: PauseModeProps) {
  const [isPaused, setIsPaused] = useState(false);
  const [showDurationPicker, setShowDurationPicker] = useState(false);
  const [remainingTime, setRemainingTime] = useState<number | null>(null); // minutes

  useEffect(() => {
    loadPauseState();
    const interval = setInterval(loadPauseState, 60000); // Check every minute
    return () => clearInterval(interval);
  }, []);

  const loadPauseState = async () => {
    try {
      // Mock for now - replace with actual backend call
      // const state = await invoke<{is_paused: boolean, remaining_minutes: number}>("get_pause_state");
      // setIsPaused(state.is_paused);
      // setRemainingTime(state.remaining_minutes);
    } catch (error) {
      console.error("Failed to load pause state:", error);
    }
  };

  const togglePause = () => {
    if (isPaused) {
      // Resume
      resumeActivity();
    } else {
      // Show duration picker
      setShowDurationPicker(true);
    }
  };

  const pauseActivity = async (duration: PauseDuration) => {
    try {
      // await invoke("pause_activity", { duration });
      console.log("Activity paused for:", duration);
      setIsPaused(true);
      setShowDurationPicker(false);
      
      // Set remaining time based on duration
      const option = PAUSE_DURATIONS.find((d) => d.id === duration);
      if (option?.minutes) {
        setRemainingTime(option.minutes);
      }
      
      onPauseChange?.(true);
    } catch (error) {
      console.error("Failed to pause activity:", error);
    }
  };

  const resumeActivity = async () => {
    try {
      // await invoke("resume_activity");
      console.log("Activity resumed");
      setIsPaused(false);
      setRemainingTime(null);
      onPauseChange?.(false);
    } catch (error) {
      console.error("Failed to resume activity:", error);
    }
  };

  const formatRemainingTime = (minutes: number): string => {
    if (minutes >= 60) {
      const hours = Math.floor(minutes / 60);
      const mins = minutes % 60;
      return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
    }
    return `${minutes}m`;
  };

  if (compact) {
    return (
      <div
        onClick={togglePause}
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: "6px",
          padding: "6px 12px",
          background: isPaused
            ? "rgba(239, 68, 68, 0.2)"
            : "rgba(255, 255, 255, 0.05)",
          border: "1px solid",
          borderColor: isPaused ? "rgba(239, 68, 68, 0.4)" : "var(--glass-border)",
          borderRadius: "999px",
          cursor: "pointer",
          transition: "all 0.2s",
        }}
      >
        <span style={{ fontSize: "14px" }}>{isPaused ? "⏸️" : "▶️"}</span>
        <span
          style={{
            fontSize: "12px",
            fontWeight: "500",
            color: "var(--text-primary)",
          }}
        >
          {isPaused && remainingTime
            ? formatRemainingTime(remainingTime)
            : isPaused
            ? "En pause"
            : "Actif"}
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
        position: "relative",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div>
          <h3
            style={{
              fontSize: "16px",
              fontWeight: "600",
              color: "var(--text-primary)",
              margin: 0,
            }}
          >
            {isPaused ? "⏸️ Mode Pause" : "▶️ Actif"}
          </h3>
          <p
            style={{
              fontSize: "12px",
              color: "var(--text-muted)",
              margin: "4px 0 0 0",
            }}
          >
            {isPaused
              ? remainingTime
                ? `Reprend dans ${formatRemainingTime(remainingTime)}`
                : "Les suggestions sont désactivées"
              : "ShadowLearn t'observe et t'aide"}
          </p>
        </div>

        {/* Toggle button */}
        <motion.button
          whileTap={{ scale: 0.95 }}
          onClick={togglePause}
          style={{
            padding: "10px 20px",
            background: isPaused
              ? "linear-gradient(135deg, rgba(16, 185, 129, 0.6), rgba(5, 150, 105, 0.6))"
              : "rgba(239, 68, 68, 0.6)",
            border: "none",
            borderRadius: "8px",
            color: "white",
            fontWeight: "600",
            cursor: "pointer",
            fontSize: "13px",
          }}
        >
          {isPaused ? "Reprendre" : "Pause"}
        </motion.button>
      </div>

      {/* Duration picker */}
      <AnimatePresence>
        {showDurationPicker && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            exit={{ opacity: 0, height: 0 }}
            style={{
              marginTop: "16px",
              paddingTop: "16px",
              borderTop: "1px solid rgba(255, 255, 255, 0.1)",
            }}
          >
            <div
              style={{
                fontSize: "13px",
                fontWeight: "600",
                color: "var(--text-primary)",
                marginBottom: "12px",
              }}
            >
              Durée de la pause
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
              {PAUSE_DURATIONS.map((duration) => (
                <motion.button
                  key={duration.id}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={() => pauseActivity(duration.id)}
                  style={{
                    padding: "10px 16px",
                    background: "rgba(255, 255, 255, 0.05)",
                    border: "1px solid rgba(255, 255, 255, 0.1)",
                    borderRadius: "8px",
                    cursor: "pointer",
                    transition: "all 0.2s",
                    textAlign: "left",
                    fontSize: "13px",
                    color: "var(--text-primary)",
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = "rgba(255, 255, 255, 0.1)";
                    e.currentTarget.style.borderColor = "var(--accent-primary)";
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = "rgba(255, 255, 255, 0.05)";
                    e.currentTarget.style.borderColor = "rgba(255, 255, 255, 0.1)";
                  }}
                >
                  {duration.label}
                </motion.button>
              ))}
              <button
                onClick={() => setShowDurationPicker(false)}
                style={{
                  padding: "8px",
                  background: "transparent",
                  border: "none",
                  color: "var(--text-muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                }}
              >
                Annuler
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Info */}
      {isPaused && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          style={{
            marginTop: "12px",
            padding: "10px 12px",
            background: "rgba(239, 68, 68, 0.1)",
            borderRadius: "8px",
            fontSize: "12px",
            color: "var(--text-secondary)",
          }}
        >
          ℹ️ Pendant la pause, aucune suggestion ne sera affichée.
        </motion.div>
      )}
    </div>
  );
}




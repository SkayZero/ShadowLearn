import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

interface StreakData {
  current_streak: number;
  longest_streak: number;
  total_days: number;
  last_activity: string; // ISO date
  streak_broken: boolean;
}

interface StreakTrackerProps {
  compact?: boolean;
}

export function StreakTracker({ compact = false }: StreakTrackerProps) {
  const [streak, setStreak] = useState<StreakData | null>(null);
  const [showCelebration, setShowCelebration] = useState(false);

  useEffect(() => {
    loadStreak();
    const interval = setInterval(loadStreak, 60000); // Refresh every minute
    return () => clearInterval(interval);
  }, []);

  const loadStreak = async () => {
    try {
      const streakData = await invoke<StreakData>("get_streak");
      
      // Check if new milestone reached
      if (streak && streakData.current_streak > streak.current_streak) {
        setShowCelebration(true);
        setTimeout(() => setShowCelebration(false), 3000);
      }
      
      setStreak(streakData);
    } catch (error) {
      console.error("Failed to load streak:", error);
      // Fallback to mock data
      const mockStreak: StreakData = {
        current_streak: 7,
        longest_streak: 14,
        total_days: 45,
        last_activity: new Date().toISOString(),
        streak_broken: false,
      };
      setStreak(mockStreak);
    }
  };

  if (!streak) {
    return null;
  }

  if (compact) {
    return (
      <div
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: "6px",
          padding: "4px 10px",
          background: streak.streak_broken
            ? "rgba(239, 68, 68, 0.2)"
            : "linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(135, 206, 235, 0.2))",
          borderRadius: "999px",
          border: "1px solid",
          borderColor: streak.streak_broken
            ? "rgba(239, 68, 68, 0.3)"
            : "var(--accent-emerald)",
        }}
      >
        <span style={{ fontSize: "14px" }}>
          {streak.streak_broken ? "💔" : "🔥"}
        </span>
        <span
          style={{
            fontSize: "12px",
            fontWeight: "600",
            color: "var(--text-primary)",
          }}
        >
          {streak.current_streak} jour{streak.current_streak > 1 ? "s" : ""}
        </span>
      </div>
    );
  }

  return (
    <div
      style={{
        position: "relative",
        padding: "16px",
        background: "var(--glass-bg)",
        backdropFilter: "var(--glass-backdrop)",
        WebkitBackdropFilter: "var(--glass-backdrop)",
        border: "1px solid var(--glass-border)",
        borderRadius: "var(--radius-xl)",
        boxShadow: "var(--glass-shadow)",
      }}
    >
      {/* Celebration overlay */}
      <AnimatePresence>
        {showCelebration && (
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.8 }}
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              background: "rgba(16, 185, 129, 0.9)",
              borderRadius: "var(--radius-xl)",
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              zIndex: 10,
            }}
          >
            <motion.div
              animate={{ rotate: [0, 10, -10, 0] }}
              transition={{ repeat: Infinity, duration: 0.5 }}
              style={{ fontSize: "48px" }}
            >
              🔥
            </motion.div>
            <div
              style={{
                marginTop: "12px",
                fontSize: "18px",
                fontWeight: "700",
                color: "white",
              }}
            >
              Nouveau record !
            </div>
            <div style={{ fontSize: "14px", color: "rgba(255,255,255,0.9)" }}>
              {streak.current_streak} jours consécutifs
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Main content */}
      <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
        {/* Fire icon */}
        <motion.div
          animate={
            streak.streak_broken
              ? {}
              : {
                  scale: [1, 1.1, 1],
                  rotate: [0, 5, -5, 0],
                }
          }
          transition={{ repeat: Infinity, duration: 2 }}
          style={{ fontSize: "40px" }}
        >
          {streak.streak_broken ? "💔" : "🔥"}
        </motion.div>

        {/* Stats */}
        <div style={{ flex: 1 }}>
          <div
            style={{
              fontSize: "24px",
              fontWeight: "700",
              color: "var(--text-primary)",
            }}
          >
            {streak.current_streak} jour{streak.current_streak > 1 ? "s" : ""}
          </div>
          <div
            style={{
              fontSize: "12px",
              color: "var(--text-muted)",
              marginTop: "2px",
            }}
          >
            {streak.streak_broken
              ? "Série interrompue"
              : "Série en cours"}
          </div>

          {/* Progress bar to next milestone */}
          {!streak.streak_broken && (
            <div
              style={{
                marginTop: "8px",
                width: "100%",
                height: "4px",
                background: "rgba(255,255,255,0.1)",
                borderRadius: "2px",
                overflow: "hidden",
              }}
            >
              <motion.div
                initial={{ width: 0 }}
                animate={{
                  width: `${(streak.current_streak % 7) * (100 / 7)}%`,
                }}
                transition={{ duration: 0.5 }}
                style={{
                  height: "100%",
                  background:
                    "linear-gradient(90deg, var(--accent-emerald), var(--accent-primary))",
                }}
              />
            </div>
          )}
        </div>

        {/* Best streak */}
        <div
          style={{
            textAlign: "center",
            padding: "8px 12px",
            background: "rgba(255,255,255,0.05)",
            borderRadius: "8px",
          }}
        >
          <div
            style={{
              fontSize: "18px",
              fontWeight: "700",
              color: "var(--accent-primary)",
            }}
          >
            {streak.longest_streak}
          </div>
          <div
            style={{
              fontSize: "10px",
              color: "var(--text-muted)",
              marginTop: "2px",
            }}
          >
            Record
          </div>
        </div>
      </div>

      {/* Motivational message */}
      {!streak.streak_broken && streak.current_streak > 0 && (
        <div
          style={{
            marginTop: "12px",
            padding: "8px 12px",
            background: "rgba(16, 185, 129, 0.1)",
            borderRadius: "8px",
            fontSize: "12px",
            color: "var(--text-primary)",
            textAlign: "center",
          }}
        >
          {getMotivationalMessage(streak.current_streak)}
        </div>
      )}
    </div>
  );
}

function getMotivationalMessage(streak: number): string {
  if (streak >= 30) return "🏆 Incroyable ! Tu es une légende !";
  if (streak >= 14) return "💪 Impressionnant ! Continue comme ça !";
  if (streak >= 7) return "✨ Super ! Une semaine complète !";
  if (streak >= 3) return "🚀 Bien joué ! Tu es lancé !";
  return "🌱 C'est un début prometteur !";
}




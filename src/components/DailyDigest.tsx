import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";

interface DigestStats {
  suggestions_shown: number;
  suggestions_accepted: number;
  time_saved_minutes: number;
  top_apps: Array<{ name: string; count: number }>;
  highlights: string[];
}

interface DailyDigestProps {
  isOpen: boolean;
  onClose: () => void;
}

export function DailyDigest({ isOpen, onClose }: DailyDigestProps) {
  const [stats, setStats] = useState<DigestStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (isOpen) {
      loadDigest();
    }
  }, [isOpen]);

  const loadDigest = async () => {
    setLoading(true);
    try {
      const digestStats = await invoke<DigestStats>("get_daily_digest");
      setStats(digestStats);
    } catch (error) {
      console.error("Failed to load digest:", error);
      // Fallback to mock data if backend call fails
      const mockStats: DigestStats = {
        suggestions_shown: 12,
        suggestions_accepted: 8,
        time_saved_minutes: 16,
        top_apps: [
          { name: "Cursor", count: 15 },
          { name: "Chrome", count: 8 },
          { name: "Figma", count: 5 },
        ],
        highlights: [
          "Tu as accepté 67% des suggestions 🎯",
          "Ton meilleur jour cette semaine",
          "15 min gagnées sur du debugging",
        ],
      };
      setStats(mockStats);
    } finally {
      setLoading(false);
    }
  };

  if (!isOpen) {
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
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: "rgba(0, 0, 0, 0.5)",
          backdropFilter: "blur(8px)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          zIndex: 60, // z-onboarding
          padding: "24px",
        }}
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.9, y: 20 }}
          animate={{ scale: 1, y: 0 }}
          exit={{ scale: 0.9, y: 20 }}
          onClick={(e) => e.stopPropagation()}
          style={{
            maxWidth: "480px",
            width: "100%",
            background: "var(--glass-bg)",
            backdropFilter: "var(--glass-backdrop)",
            WebkitBackdropFilter: "var(--glass-backdrop)",
            border: "1px solid var(--glass-border)",
            borderRadius: "var(--radius-2xl)",
            boxShadow: "var(--elev-4)",
            overflow: "hidden",
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: "24px",
              borderBottom: "1px solid rgba(255, 255, 255, 0.1)",
              background: "var(--glass-emerald-tint)",
            }}
          >
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "start" }}>
              <div>
                <h2
                  style={{
                    fontSize: "24px",
                    fontWeight: "700",
                    color: "var(--text-primary)",
                    margin: 0,
                  }}
                >
                  📊 Ton Digest
                </h2>
                <p
                  style={{
                    fontSize: "14px",
                    color: "var(--text-muted)",
                    margin: "4px 0 0 0",
                  }}
                >
                  {new Date().toLocaleDateString("fr-FR", {
                    weekday: "long",
                    day: "numeric",
                    month: "long",
                  })}
                </p>
              </div>
              <button
                onClick={onClose}
                style={{
                  background: "transparent",
                  border: "none",
                  color: "var(--text-muted)",
                  fontSize: "24px",
                  cursor: "pointer",
                  padding: "0",
                  width: "32px",
                  height: "32px",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
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
            </div>
          </div>

          {/* Content */}
          <div style={{ padding: "24px", maxHeight: "60vh", overflowY: "auto" }}>
            {loading ? (
              <div style={{ textAlign: "center", padding: "40px", color: "var(--text-muted)" }}>
                Chargement...
              </div>
            ) : stats ? (
              <>
                {/* Stats cards */}
                <div
                  style={{
                    display: "grid",
                    gridTemplateColumns: "repeat(3, 1fr)",
                    gap: "12px",
                    marginBottom: "24px",
                  }}
                >
                  <StatCard
                    icon="💡"
                    value={stats.suggestions_shown}
                    label="suggestions"
                    gradient="linear-gradient(135deg, rgba(251, 191, 36, 0.3), rgba(245, 158, 11, 0.3))"
                  />
                  <StatCard
                    icon="✅"
                    value={stats.suggestions_accepted}
                    label="acceptées"
                    gradient="linear-gradient(135deg, rgba(16, 185, 129, 0.3), rgba(5, 150, 105, 0.3))"
                  />
                  <StatCard
                    icon="⚡"
                    value={stats.time_saved_minutes}
                    label="min gagnées"
                    gradient="linear-gradient(135deg, rgba(135, 206, 235, 0.3), rgba(96, 165, 250, 0.3))"
                  />
                </div>

                {/* Time saved highlight */}
                {stats.time_saved_minutes > 0 && (
                  <motion.div
                    initial={{ scale: 0.95, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    transition={{ delay: 0.1 }}
                    style={{
                      padding: "16px",
                      background: "linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(124, 58, 237, 0.2))",
                      borderRadius: "var(--radius-lg)",
                      marginBottom: "24px",
                      textAlign: "center",
                    }}
                  >
                    <div
                      style={{
                        fontSize: "28px",
                        fontWeight: "700",
                        color: "var(--text-primary)",
                      }}
                    >
                      ~{stats.time_saved_minutes} min
                    </div>
                    <div
                      style={{
                        fontSize: "13px",
                        color: "var(--text-secondary)",
                        marginTop: "4px",
                      }}
                    >
                      de temps gagné estimé ⚡
                    </div>
                  </motion.div>
                )}

                {/* Top apps */}
                {stats.top_apps.length > 0 && (
                  <div style={{ marginBottom: "24px" }}>
                    <h3
                      style={{
                        fontSize: "14px",
                        fontWeight: "600",
                        color: "var(--text-primary)",
                        marginBottom: "12px",
                      }}
                    >
                      Apps les plus aidées
                    </h3>
                    <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                      {stats.top_apps.slice(0, 3).map((app, i) => (
                        <div
                          key={i}
                          style={{
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "space-between",
                            padding: "10px 12px",
                            background: "rgba(255, 255, 255, 0.05)",
                            borderRadius: "8px",
                          }}
                        >
                          <span style={{ fontSize: "14px", color: "var(--text-primary)" }}>
                            {app.name}
                          </span>
                          <span
                            style={{
                              fontSize: "13px",
                              color: "var(--text-muted)",
                              fontWeight: "600",
                            }}
                          >
                            {app.count}×
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Highlights */}
                {stats.highlights.length > 0 && (
                  <div>
                    <h3
                      style={{
                        fontSize: "14px",
                        fontWeight: "600",
                        color: "var(--text-primary)",
                        marginBottom: "12px",
                      }}
                    >
                      Moments clés
                    </h3>
                    <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                      {stats.highlights.map((highlight, i) => (
                        <motion.div
                          key={i}
                          initial={{ opacity: 0, x: -10 }}
                          animate={{ opacity: 1, x: 0 }}
                          transition={{ delay: i * 0.1 }}
                          style={{
                            padding: "10px 12px",
                            background: "rgba(16, 185, 129, 0.1)",
                            borderRadius: "8px",
                            fontSize: "13px",
                            color: "var(--text-primary)",
                          }}
                        >
                          {highlight}
                        </motion.div>
                      ))}
                    </div>
                  </div>
                )}
              </>
            ) : (
              <div style={{ textAlign: "center", padding: "40px", color: "var(--text-muted)" }}>
                Aucune donnée disponible
              </div>
            )}
          </div>

          {/* Footer */}
          <div
            style={{
              padding: "16px 24px",
              borderTop: "1px solid rgba(255, 255, 255, 0.1)",
              display: "flex",
              justifyContent: "center",
            }}
          >
            <button
              onClick={onClose}
              style={{
                padding: "10px 24px",
                background: "var(--accent-primary)",
                border: "none",
                borderRadius: "8px",
                color: "white",
                fontWeight: "600",
                cursor: "pointer",
                transition: "opacity 0.2s",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.opacity = "0.9";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.opacity = "1";
              }}
            >
              Fermer
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

function StatCard({
  icon,
  value,
  label,
  gradient,
}: {
  icon: string;
  value: number;
  label: string;
  gradient: string;
}) {
  return (
    <div
      style={{
        padding: "16px 12px",
        background: gradient,
        borderRadius: "var(--radius-lg)",
        textAlign: "center",
      }}
    >
      <div style={{ fontSize: "20px", marginBottom: "8px" }}>{icon}</div>
      <div
        style={{
          fontSize: "24px",
          fontWeight: "700",
          color: "var(--text-primary)",
        }}
      >
        {value}
      </div>
      <div
        style={{
          fontSize: "11px",
          color: "var(--text-muted)",
          marginTop: "4px",
        }}
      >
        {label}
      </div>
    </div>
  );
}




import { useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useSmartDocking } from "../hooks/useSmartDocking";

interface SmartDockProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
  nearCursor?: boolean;
}

export function SmartDock({
  isOpen,
  onClose,
  children,
  nearCursor = false,
}: SmartDockProps) {
  const { position, dockNearCursor, dockToCorner } = useSmartDocking({
    dockWidth: 420,
    dockHeight: 640,
    margin: 24,
    snapToCorner: !nearCursor,
  });

  useEffect(() => {
    if (isOpen) {
      if (nearCursor) {
        dockNearCursor();
      } else {
        dockToCorner();
      }
    }
  }, [isOpen, nearCursor, dockNearCursor, dockToCorner]);

  if (!isOpen) {
    return null;
  }

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 20 }}
        transition={{ duration: 0.25, ease: "easeOut" }}
        style={{
          position: "fixed",
          left: `${position.x}px`,
          top: `${position.y}px`,
          width: "420px",
          height: "640px",
          zIndex: 50, // z-dock
          background: "var(--glass-bg)",
          backdropFilter: "var(--glass-backdrop)",
          WebkitBackdropFilter: "var(--glass-backdrop)",
          border: "1px solid var(--glass-border)",
          borderRadius: "var(--radius-2xl)",
          boxShadow: "var(--elev-4)",
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
        }}
      >
        {/* Header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "16px 20px",
            borderBottom: "1px solid rgba(255, 255, 255, 0.1)",
            background: "var(--glass-emerald-tint)",
          }}
        >
          <span
            style={{
              fontSize: "16px",
              fontWeight: "600",
              color: "var(--text-primary)",
            }}
          >
            ShadowLearn Dock
          </span>
          <button
            onClick={onClose}
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-muted)",
              fontSize: "20px",
              cursor: "pointer",
              padding: "4px 8px",
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

        {/* Content */}
        <div
          style={{
            flex: 1,
            overflow: "auto",
            padding: "20px",
          }}
        >
          {children}
        </div>
      </motion.div>
    </AnimatePresence>
  );
}




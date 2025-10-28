/**
 * Message Feedback Component
 * Thumbs up/down instant feedback + emotional response
 */

import { useState } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { shadowStore } from "../lib";

export interface MessageFeedbackProps {
  messageId: string;
  onFeedback?: (helpful: boolean) => void;
}

const EMOTIONAL_RESPONSES = {
  positive: [
    "Parfait 😌",
    "Super ! 🙌",
    "Content de t'aider ✨",
    "Excellent ! 🎯",
  ],
  negative: [
    "Merci, je ferai mieux 🤝",
    "Noté, j'apprends ! 📝",
    "Compris, je m'améliore 💪",
    "Merci pour ton feedback 🙏",
  ],
};

function getRandomResponse(helpful: boolean): string {
  const responses = helpful
    ? EMOTIONAL_RESPONSES.positive
    : EMOTIONAL_RESPONSES.negative;
  return responses[Math.floor(Math.random() * responses.length)];
}

export function MessageFeedback({
  messageId,
  onFeedback,
}: MessageFeedbackProps) {
  const [feedback, setFeedback] = useState<boolean | null>(
    shadowStore.hasFeedback(messageId)
      ? shadowStore.getFeedback(messageId) ?? null
      : null
  );
  const [showThanks, setShowThanks] = useState(false);
  const [emotionalResponse, setEmotionalResponse] = useState("");

  const handleFeedback = async (helpful: boolean) => {
    setFeedback(helpful);
    setShowThanks(true);

    // Get emotional response
    const response = getRandomResponse(helpful);
    setEmotionalResponse(response);

    // Record in store
    shadowStore.recordFeedback(messageId, helpful);

    // Record feedback in backend
    try {
      await invoke("record_message_feedback", {
        messageId,
        helpful,
      });
    } catch (e) {
      console.error("Failed to record feedback:", e);
    }

    // Notify parent
    onFeedback?.(helpful);

    // Hide thanks message after 2s
    setTimeout(() => {
      setShowThanks(false);
    }, 2000);
  };

  if (feedback !== null && showThanks) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.9 }}
        transition={{ duration: 0.2 }}
        style={{ 
          display: "flex", 
          alignItems: "center", 
          gap: "8px", 
          fontSize: "14px", 
          color: "#10b981",
          marginTop: "8px"
        }}
      >
        <span style={{ fontSize: "18px" }}>✓</span>
        <span>{emotionalResponse}</span>
      </motion.div>
    );
  }

  if (feedback !== null) {
    return null; // Already submitted
  }

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "8px", marginTop: "8px" }}>
      <span style={{ fontSize: "12px", color: "rgba(255, 255, 255, 0.6)" }}>Cette réponse t'aide ?</span>
      <div style={{ display: "flex", gap: "4px" }}>
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={() => handleFeedback(true)}
          style={{
            padding: "6px",
            borderRadius: "8px",
            background: "transparent",
            border: "none",
            cursor: "pointer",
          }}
          aria-label="Utile"
        >
          <span style={{ fontSize: "18px", opacity: 0.6 }}>👍</span>
        </motion.button>
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={() => handleFeedback(false)}
          style={{
            padding: "6px",
            borderRadius: "8px",
            background: "transparent",
            border: "none",
            cursor: "pointer",
          }}
          aria-label="Pas utile"
        >
          <span style={{ fontSize: "18px", opacity: 0.6 }}>👎</span>
        </motion.button>
      </div>
    </div>
  );
}


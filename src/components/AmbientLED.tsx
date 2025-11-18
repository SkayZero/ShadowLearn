/**
 * Ambient LED Component
 * Breathing LED that reflects user's flow state
 */

import { motion } from "framer-motion";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useEvent, EVENTS, type FlowStateData } from "../lib";
import { useTheme } from "../contexts/ThemeContext";

export interface AmbientLEDProps {
  className?: string;
  size?: number;
}

export function AmbientLED({ className = "", size = 12 }: AmbientLEDProps) {
  const { theme } = useTheme();
  const [flowState, setFlowState] = useState<FlowStateData>({
    flow_state: "normal",
    confidence: 0.5,
  });

  // Listen for flow state updates from backend
  useEvent<FlowStateData>(EVENTS.FLOW_STATE, (state) => {
    setFlowState(state);
  });

  // Poll flow state periodically
  useEffect(() => {
    const detectFlow = async () => {
      try {
        const state = await invoke<FlowStateData>("detect_flow_state");
        setFlowState(state);
      } catch (e) {
        // Silently fail, use last known state
      }
    };

    // Initial detection
    detectFlow();

    // Poll every 10 seconds
    const interval = setInterval(detectFlow, 10000);
    return () => clearInterval(interval);
  }, []);

  // Animation configuration based on flow state
  const animationDuration = {
    deep: 3, // Slow breathing = deep work
    normal: 2,
    blocked: 1, // Fast pulse = blocked
  }[flowState.flow_state];

  // Color based on flow state + theme personality
  const color = theme.led[flowState.flow_state];

  return (
    <motion.div
      animate={{
        scale: [1, 1.3, 1],
        opacity: [0.5, 1, 0.5],
      }}
      transition={{
        duration: animationDuration,
        repeat: Infinity,
        ease: "easeInOut",
      }}
      className={className}
      data-testid="ambient-led"
      style={{
        width: size,
        height: size,
        borderRadius: '50%',
        backgroundColor: color,
        boxShadow: `0 0 ${size * 2}px ${color}`,
        border: `1px solid ${color}`,
        transition: `background-color ${theme.transitionSpeed}ms ease-in-out, box-shadow ${theme.transitionSpeed}ms ease-in-out`,
      }}
      title={`${theme.name} - Flow: ${flowState.flow_state} (${Math.round(flowState.confidence * 100)}%)`}
    />
  );
}


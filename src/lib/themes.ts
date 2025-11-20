/**
 * Theme System - Personality-based Ambiances
 * Each personality has its own visual identity
 */

export type Personality = "aerya" | "aura" | "spark" | "nova" | "kai" | "echo" | "void";

export interface Theme {
  id: Personality;
  name: string;
  description: string;
  icon: string;
  
  // Visual
  background: string;
  gradient: [string, string];
  accent: string;
  accentLight: string;
  text: {
    primary: string;
    secondary: string;
    muted: string;
  };
  
  // Glass morphism
  glass: {
    bg: string;
    border: string;
    shadow: string;
  };
  
  // LED colors (combined with flow state)
  led: {
    deep: string;      // Deep focus
    normal: string;    // Normal flow
    blocked: string;   // Blocked/stuck
  };
  
  // Effects
  glow: string;
  particleSpeed: number;
  
  // Typography
  font: string;
  
  // Animations
  transitionSpeed: number; // ms
}

export const THEMES: Record<Personality, Theme> = {
  aerya: {
    id: "aerya",
    name: "AERYA",
    description: "Assistant équilibré, bienveillant",
    icon: "🌊",
    
    background: "#0a1520",
    gradient: ["#1a2f3f", "#1a3f3a"],
    accent: "#6ee7b7",
    accentLight: "#a7f3d0",
    text: {
      primary: "#f0fdfa",
      secondary: "#ccfbf1",
      muted: "#99f6e4",
    },
    
    glass: {
      bg: "rgba(110, 231, 183, 0.45)",  // Increased opacity (no backdrop-filter)
      border: "rgba(110, 231, 183, 0.25)",
      shadow: "0 8px 32px rgba(110, 231, 183, 0.15)",
    },
    
    led: {
      deep: "#34d399",
      normal: "#6ee7b7",
      blocked: "#a7f3d0",
    },
    
    glow: "rgba(110, 231, 183, 0.3)",
    particleSpeed: 0.5,
    font: "'Inter', sans-serif",
    transitionSpeed: 500,
  },
  
  aura: {
    id: "aura",
    name: "AURA",
    description: "Sage calme, méditatif",
    icon: "🔮",
    
    background: "#0f0c1a",
    gradient: ["#1a1530", "#1a1a2e"],
    accent: "#c4b5fd",
    accentLight: "#e9d5ff",
    text: {
      primary: "#f5f3ff",
      secondary: "#e9d5ff",
      muted: "#c4b5fd",
    },
    
    glass: {
      bg: "rgba(196, 181, 253, 0.45)",  // Increased opacity
      border: "rgba(196, 181, 253, 0.25)",
      shadow: "0 8px 32px rgba(196, 181, 253, 0.15)",
    },
    
    led: {
      deep: "#a78bfa",
      normal: "#c4b5fd",
      blocked: "#ddd6fe",
    },
    
    glow: "rgba(196, 181, 253, 0.3)",
    particleSpeed: 0.3,
    font: "'Inter', sans-serif",
    transitionSpeed: 800,
  },
  
  spark: {
    id: "spark",
    name: "SPARK",
    description: "Énergique, motivant",
    icon: "⚡",
    
    background: "#1a1512",
    gradient: ["#2a2015", "#2a1810"],
    accent: "#fbbf24",
    accentLight: "#fcd34d",
    text: {
      primary: "#ffffff",
      secondary: "#fef3c7",
      muted: "#fde68a",
    },
    
    glass: {
      bg: "rgba(251, 191, 36, 0.45)",  // Increased opacity
      border: "rgba(251, 191, 36, 0.25)",
      shadow: "0 8px 32px rgba(251, 191, 36, 0.15)",
    },
    
    led: {
      deep: "#f59e0b",
      normal: "#fbbf24",
      blocked: "#fcd34d",
    },
    
    glow: "rgba(251, 191, 36, 0.3)",
    particleSpeed: 1.2,
    font: "'Satoshi', sans-serif",
    transitionSpeed: 300,
  },
  
  nova: {
    id: "nova",
    name: "NOVA",
    description: "Visionnaire, poétique",
    icon: "✨",
    
    background: "#0a1520",
    gradient: ["#1a2535", "#1a2f3f"],
    accent: "#7dd3fc",
    accentLight: "#bae6fd",
    text: {
      primary: "#f0f9ff",
      secondary: "#bae6fd",
      muted: "#7dd3fc",
    },
    
    glass: {
      bg: "rgba(125, 211, 252, 0.45)",  // Increased opacity
      border: "rgba(125, 211, 252, 0.25)",
      shadow: "0 8px 32px rgba(125, 211, 252, 0.15)",
    },
    
    led: {
      deep: "#38bdf8",
      normal: "#7dd3fc",
      blocked: "#bae6fd",
    },
    
    glow: "rgba(125, 211, 252, 0.3)",
    particleSpeed: 0.6,
    font: "'Space Grotesk', sans-serif",
    transitionSpeed: 600,
  },
  
  kai: {
    id: "kai",
    name: "KAI",
    description: "Pratique, mentor tech",
    icon: "⚙️",
    
    background: "#0a1512",
    gradient: ["#1a2520", "#1a2f28"],
    accent: "#6ee7b7",
    accentLight: "#a7f3d0",
    text: {
      primary: "#ecfdf5",
      secondary: "#a7f3d0",
      muted: "#6ee7b7",
    },
    
    glass: {
      bg: "rgba(110, 231, 183, 0.45)",  // Increased opacity (no backdrop-filter)
      border: "rgba(110, 231, 183, 0.25)",
      shadow: "0 8px 32px rgba(110, 231, 183, 0.15)",
    },
    
    led: {
      deep: "#34d399",
      normal: "#6ee7b7",
      blocked: "#a7f3d0",
    },
    
    glow: "rgba(110, 231, 183, 0.3)",
    particleSpeed: 0.4,
    font: "'IBM Plex Mono', monospace",
    transitionSpeed: 400,
  },
  
  echo: {
    id: "echo",
    name: "ECHO",
    description: "Artiste rêveur",
    icon: "🎨",
    
    background: "#1a0a1a",
    gradient: ["#2a1530", "#2a1a2e"],
    accent: "#f9a8d4",
    accentLight: "#fbcfe8",
    text: {
      primary: "#fdf4ff",
      secondary: "#f9a8d4",
      muted: "#f472b6",
    },
    
    glass: {
      bg: "rgba(249, 168, 212, 0.45)",  // Increased opacity
      border: "rgba(249, 168, 212, 0.25)",
      shadow: "0 8px 32px rgba(249, 168, 212, 0.15)",
    },
    
    led: {
      deep: "#ec4899",
      normal: "#f9a8d4",
      blocked: "#fbcfe8",
    },
    
    glow: "rgba(249, 168, 212, 0.3)",
    particleSpeed: 0.5,
    font: "'Playfair Display', serif",
    transitionSpeed: 700,
  },
  
  void: {
    id: "void",
    name: "VOID",
    description: "Minimaliste, silencieux",
    icon: "⬛",
    
    background: "#0a0a0a",
    gradient: ["#121212", "#1a1a1a"],
    accent: "#e5e5e5",
    accentLight: "#f5f5f5",
    text: {
      primary: "#ffffff",
      secondary: "#e5e5e5",
      muted: "#a3a3a3",
    },
    
    glass: {
      bg: "rgba(255, 255, 255, 0.45)",  // Increased opacity
      border: "rgba(255, 255, 255, 0.15)",
      shadow: "0 8px 32px rgba(0, 0, 0, 0.5)",
    },
    
    led: {
      deep: "#737373",
      normal: "#a3a3a3",
      blocked: "#d4d4d4",
    },
    
    glow: "rgba(255, 255, 255, 0.1)",
    particleSpeed: 0.1,
    font: "'Helvetica Neue', sans-serif",
    transitionSpeed: 500,
  },
};

// Helper to get theme
export function getTheme(personality: Personality): Theme {
  return THEMES[personality];
}

// Convert theme to CSS variables
export function getThemeCSSVariables(theme: Theme): Record<string, string> {
  return {
    '--theme-bg': theme.background,
    '--theme-gradient-from': theme.gradient[0],
    '--theme-gradient-to': theme.gradient[1],
    '--theme-accent': theme.accent,
    '--theme-accent-light': theme.accentLight,
    
    '--theme-text-primary': theme.text.primary,
    '--theme-text-secondary': theme.text.secondary,
    '--theme-text-muted': theme.text.muted,
    
    '--theme-glass-bg': theme.glass.bg,
    '--theme-glass-border': theme.glass.border,
    '--theme-glass-shadow': theme.glass.shadow,
    
    '--theme-glow': theme.glow,
    '--theme-font': theme.font,
    
    '--theme-transition-speed': `${theme.transitionSpeed}ms`,
  };
}


/**
 * Design Tokens - Cluely-like 10/10
 * Centralized constants for consistent UX
 */

export const TOKENS = {
  // Colors - Flow States
  colors: {
    observing: '#10b981',    // Emerald - watching/ready
    idle: '#f59e0b',         // Amber - idle/blocked
    analyzing: '#3b82f6',    // Blue - processing
    cooldown: '#8b5cf6',     // Purple - cooldown
    error: '#ef4444',        // Red - error state
    success: '#10b981',      // Emerald - success
    
    // Accent colors
    primary: '#3b82f6',      // Blue
    secondary: '#8b5cf6',    // Purple
    accent: '#87CEEB',       // Sky Blue (Cluely signature)
  },

  // Glass Morphism
  glass: {
    bg: 'rgba(255, 255, 255, 0.85)',
    bgDark: 'rgba(17, 24, 39, 0.85)',
    blur: '12px',
    border: 'rgba(255, 255, 255, 0.3)',
    borderDark: 'rgba(255, 255, 255, 0.1)',
    shadow: '0 8px 32px rgba(31, 38, 135, 0.15)',
    shadowLarge: '0 12px 48px rgba(31, 38, 135, 0.25)',
  },

  // Z-Index Hierarchy
  zIndex: {
    dock: 1000,        // Chat dock (highest priority)
    toast: 900,        // Notifications/toasts
    pills: 800,        // Micro-suggestions
    bubble: 700,       // Trigger bubble
    overlay: 600,      // Modal overlays
    contextCard: 500,  // Context preview cards
  },

  // Animations - Consistent Easing
  easing: {
    // Main easing (Cluely signature)
    primary: 'cubic-bezier(0.33, 1, 0.68, 1)',
    
    // Additional easings
    bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
    smooth: 'cubic-bezier(0.4, 0, 0.2, 1)',
    sharp: 'cubic-bezier(0.4, 0, 0.6, 1)',
  },

  // Durations (ms)
  duration: {
    instant: 100,
    fast: 150,
    normal: 200,
    slow: 300,
    slower: 500,
  },

  // Spacing (px)
  spacing: {
    xs: 4,
    sm: 8,
    md: 12,
    lg: 16,
    xl: 24,
    xxl: 32,
  },

  // Component Specific
  components: {
    bubble: {
      size: 56,           // 56×56px
      position: {
        right: 24,        // 24px from right
        bottom: 24,       // 24px from bottom
      },
      led: {
        size: 12,         // 12px LED
        glowSize: 24,     // 12px * 2 for glow effect
      },
    },
    
    dock: {
      width: 420,         // 420px width
      height: 640,        // 640px height
      borderRadius: 16,   // 16px rounded corners
      padding: 24,        // 24px internal padding
    },
    
    toast: {
      width: 384,         // 384px (24rem)
      maxWidth: '90vw',   // Responsive
      position: {
        right: 24,
        bottom: 96,       // 96px from bottom (above bubble)
      },
      stackGap: 12,       // 12px between stacked toasts
    },
    
    pills: {
      height: 44,         // 44px height
      borderRadius: 22,   // Pill shape (height / 2)
      padding: {
        x: 16,
        y: 12,
      },
      stackGap: 8,        // 8px between pills
    },
  },

  // Typography
  typography: {
    fontSize: {
      xs: 12,
      sm: 14,
      base: 16,
      lg: 18,
      xl: 20,
      xxl: 24,
    },
    fontWeight: {
      normal: 400,
      medium: 500,
      semibold: 600,
      bold: 700,
    },
    lineHeight: {
      tight: 1.2,
      normal: 1.5,
      relaxed: 1.75,
    },
  },

  // Performance Targets
  performance: {
    bubbleToDock: 180,    // Max 180ms p95
    toastAppear: 120,     // Max 120ms p95
    pillsExpand: 150,     // Max 150ms p95
    minFPS: 60,           // Minimum 60 FPS for animations
  },
} as const;

// Type-safe token access
export type TokenPath = 
  | `colors.${keyof typeof TOKENS.colors}`
  | `glass.${keyof typeof TOKENS.glass}`
  | `zIndex.${keyof typeof TOKENS.zIndex}`
  | `easing.${keyof typeof TOKENS.easing}`;

/**
 * Helper to get token value
 * @example getToken('colors.primary') -> '#3b82f6'
 */
export function getToken(path: string): any {
  const keys = path.split('.');
  let value: any = TOKENS;
  
  for (const key of keys) {
    value = value[key];
    if (value === undefined) {
      console.warn(`Token not found: ${path}`);
      return undefined;
    }
  }
  
  return value;
}

/**
 * CSS Variables for tokens (optional, for CSS usage)
 */
export function getCSSVariables(): Record<string, string> {
  return {
    // Colors
    '--shadow-color-primary': TOKENS.colors.primary,
    '--shadow-color-accent': TOKENS.colors.accent,
    '--shadow-color-success': TOKENS.colors.success,
    '--shadow-color-error': TOKENS.colors.error,
    
    // Glass
    '--shadow-glass-bg': TOKENS.glass.bg,
    '--shadow-glass-blur': TOKENS.glass.blur,
    '--shadow-glass-border': TOKENS.glass.border,
    
    // Easing
    '--shadow-easing': TOKENS.easing.primary,
    
    // Spacing
    '--shadow-spacing-sm': `${TOKENS.spacing.sm}px`,
    '--shadow-spacing-md': `${TOKENS.spacing.md}px`,
    '--shadow-spacing-lg': `${TOKENS.spacing.lg}px`,
  };
}





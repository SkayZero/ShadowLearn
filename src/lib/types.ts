/**
 * Shared TypeScript types for ShadowLearn features
 */

// ============================================================================
// Opportunity & Suggestions
// ============================================================================

export interface Opportunity {
  id: string;
  title: string;
  confidence: number;
  preview: string;
  context?: any;
  created_at?: number;
}

export interface MicroSuggestion {
  id: string;
  text: string;
  type: "continue" | "help" | "reminder";
  confidence?: number;
}

// ============================================================================
// Context & Flow
// ============================================================================

export interface ContextPreview {
  app_name: string;
  window_title: string;
  idle_seconds: number;
  session_duration_minutes: number;
  recent_screenshots: number;
  pending_suggestion?: string;
  domain?: string;
}

export type FlowState = "deep" | "normal" | "blocked";

export interface FlowStateData {
  flow_state: FlowState;
  confidence: number;
  typing_speed?: number;
  focus_score?: number;
}

// ============================================================================
// Personality & Preferences
// ============================================================================

export type PersonalityMode = "default" | "mentor" | "buddy" | "pro";

export interface PersonalityConfig {
  name: string;
  description: string;
  icon: string;
  tone: string;
}

export const PERSONALITIES: Record<PersonalityMode, PersonalityConfig> = {
  default: {
    name: "Équilibré",
    description: "Ton sobre et efficace",
    icon: "🎯",
    tone: "neutral",
  },
  mentor: {
    name: "Mentor",
    description: "Pédagogique et détaillé",
    icon: "👨‍🏫",
    tone: "educational",
  },
  buddy: {
    name: "Pote",
    description: "Casual et sympa",
    icon: "🤙",
    tone: "casual",
  },
  pro: {
    name: "Expert",
    description: "Technique et précis",
    icon: "💼",
    tone: "professional",
  },
};

// ============================================================================
// Streaks & Stats
// ============================================================================

export interface StreakData {
  current_days: number;
  longest_days: number;
  milestones_unlocked: string[];
  last_activity?: string;
}

export interface DigestStats {
  suggestions_shown: number;
  suggestions_accepted: number;
  time_saved_minutes: number;
  top_apps: Array<{ name: string; count: number }>;
  highlights: string[];
}

// ============================================================================
// Pause Detection
// ============================================================================

export type PauseReason = "meeting" | "lunch" | "coffee" | "away";

export interface PauseData {
  paused: boolean;
  reason?: PauseReason;
  duration_seconds?: number;
  resumed_at?: number;
}

// ============================================================================
// Slash Commands
// ============================================================================

export interface SlashCommand {
  trigger: string;
  label: string;
  description: string;
  icon: string;
  action: (args?: string) => Promise<void>;
}

// ============================================================================
// Quick Actions
// ============================================================================

export type QuickActionType = 
  | "summarize" 
  | "debug" 
  | "improve" 
  | "explain" 
  | "continue";

export interface QuickAction {
  id: QuickActionType;
  icon: string;
  label: string;
  action: () => Promise<void>;
}

// ============================================================================
// Feedback
// ============================================================================

export interface FeedbackData {
  message_id: string;
  helpful: boolean;
  timestamp: number;
  context?: any;
}

// ============================================================================
// Trigger State
// ============================================================================

export type TriggerStateType = 
  | "Observing"
  | "IdleDetected"
  | "ContextConfirmed"
  | "PromptShown"
  | "UserResponded"
  | "Cooldown";

export interface TriggerState {
  state: TriggerStateType;
  reason?: string;
  cooldown_seconds?: number;
  opportunity?: Opportunity;
}

// ============================================================================
// Animation Configs
// ============================================================================

export const SPRING_CONFIG = {
  stiffness: 300,
  damping: 30,
};

export const SOFT_SPRING = {
  stiffness: 200,
  damping: 25,
};

export const GENTLE_SPRING = {
  stiffness: 150,
  damping: 20,
};






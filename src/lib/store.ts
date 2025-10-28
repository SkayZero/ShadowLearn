/**
 * Central Shadow Store
 * Manages global state for ShadowLearn features
 * Syncs with backend via localStorage + Tauri commands
 */

import { invoke } from "@tauri-apps/api/core";

// ============================================================================
// Store State Types
// ============================================================================

interface ShadowStoreState {
  // Dismissed items
  dismissedOpportunities: Set<string>;
  dismissedPills: Set<string>;
  
  // Feedback tracking
  feedbackHistory: Map<string, boolean>; // messageId -> helpful
  
  // Context cache
  lastContext: any | null;
  lastContextTimestamp: number;
  
  // User preferences
  personality: "default" | "mentor" | "buddy" | "pro";
  soundEnabled: boolean;
  
  // Streak tracking
  lastDigestShown: string | null; // Date string
  milestonesShown: Set<number>;
}

// ============================================================================
// Shadow Store Class
// ============================================================================

class ShadowStore {
  private state: ShadowStoreState;
  private readonly STORAGE_KEY = "shadow_store";
  private saveTimeout: number | null = null;

  constructor() {
    this.state = this.loadState();
  }

  // ==========================================================================
  // State Management
  // ==========================================================================

  private loadState(): ShadowStoreState {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      if (!stored) return this.getDefaultState();

      const parsed = JSON.parse(stored);
      
      // Convert arrays back to Sets/Maps
      return {
        ...parsed,
        dismissedOpportunities: new Set(parsed.dismissedOpportunities || []),
        dismissedPills: new Set(parsed.dismissedPills || []),
        feedbackHistory: new Map(parsed.feedbackHistory || []),
        milestonesShown: new Set(parsed.milestonesShown || []),
      };
    } catch (e) {
      console.error("Failed to load shadow store:", e);
      return this.getDefaultState();
    }
  }

  private getDefaultState(): ShadowStoreState {
    return {
      dismissedOpportunities: new Set(),
      dismissedPills: new Set(),
      feedbackHistory: new Map(),
      lastContext: null,
      lastContextTimestamp: 0,
      personality: "default",
      soundEnabled: false,
      lastDigestShown: null,
      milestonesShown: new Set(),
    };
  }

  private saveState() {
    // Debounce saves (wait 500ms after last change)
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
    }

    this.saveTimeout = window.setTimeout(() => {
      try {
        // Convert Sets/Maps to arrays for JSON
        const serializable = {
          ...this.state,
          dismissedOpportunities: Array.from(this.state.dismissedOpportunities),
          dismissedPills: Array.from(this.state.dismissedPills),
          feedbackHistory: Array.from(this.state.feedbackHistory.entries()),
          milestonesShown: Array.from(this.state.milestonesShown),
        };

        localStorage.setItem(this.STORAGE_KEY, JSON.stringify(serializable));
      } catch (e) {
        console.error("Failed to save shadow store:", e);
      }
    }, 500);
  }

  // ==========================================================================
  // Dismissed Items
  // ==========================================================================

  dismissOpportunity(id: string) {
    this.state.dismissedOpportunities.add(id);
    this.saveState();
  }

  isOpportunityDismissed(id: string): boolean {
    return this.state.dismissedOpportunities.has(id);
  }

  dismissPill(id: string) {
    this.state.dismissedPills.add(id);
    this.saveState();
  }

  isPillDismissed(id: string): boolean {
    return this.state.dismissedPills.has(id);
  }

  clearDismissed() {
    this.state.dismissedOpportunities.clear();
    this.state.dismissedPills.clear();
    this.saveState();
  }

  // ==========================================================================
  // Feedback Tracking
  // ==========================================================================

  recordFeedback(messageId: string, helpful: boolean) {
    this.state.feedbackHistory.set(messageId, helpful);
    this.saveState();
  }

  getFeedback(messageId: string): boolean | undefined {
    return this.state.feedbackHistory.get(messageId);
  }

  hasFeedback(messageId: string): boolean {
    return this.state.feedbackHistory.has(messageId);
  }

  // ==========================================================================
  // Context Cache
  // ==========================================================================

  updateContext(context: any) {
    this.state.lastContext = context;
    this.state.lastContextTimestamp = Date.now();
    this.saveState();
  }

  getContext(): any | null {
    // Return cached context if fresh (< 5s old)
    const age = Date.now() - this.state.lastContextTimestamp;
    if (age < 5000) {
      return this.state.lastContext;
    }
    return null;
  }

  // ==========================================================================
  // Personality
  // ==========================================================================

  setPersonality(personality: ShadowStoreState["personality"]) {
    this.state.personality = personality;
    this.saveState();

    // Sync with backend
    invoke("set_personality_mode", { mode: personality }).catch(console.error);
  }

  getPersonality(): ShadowStoreState["personality"] {
    return this.state.personality;
  }

  // ==========================================================================
  // Sound
  // ==========================================================================

  setSoundEnabled(enabled: boolean) {
    this.state.soundEnabled = enabled;
    this.saveState();
  }

  isSoundEnabled(): boolean {
    return this.state.soundEnabled;
  }

  // ==========================================================================
  // Digest & Streaks
  // ==========================================================================

  shouldShowDigest(): boolean {
    const today = new Date().toDateString();
    const lastShown = this.state.lastDigestShown;
    
    if (!lastShown || lastShown !== today) {
      // Check time (after 6pm)
      const now = new Date();
      return now.getHours() >= 18;
    }
    
    return false;
  }

  markDigestShown() {
    this.state.lastDigestShown = new Date().toDateString();
    this.saveState();
  }

  shouldShowMilestone(days: number): boolean {
    return !this.state.milestonesShown.has(days);
  }

  markMilestoneShown(days: number) {
    this.state.milestonesShown.add(days);
    this.saveState();
  }

  // ==========================================================================
  // Debug
  // ==========================================================================

  getState(): ShadowStoreState {
    return { ...this.state };
  }

  reset() {
    this.state = this.getDefaultState();
    this.saveState();
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const shadowStore = new ShadowStore();

// ============================================================================
// React Hooks
// ============================================================================

import { useState } from "react";

export function usePersonality() {
  const [personality, setPersonality] = useState(shadowStore.getPersonality());

  const updatePersonality = (p: ShadowStoreState["personality"]) => {
    shadowStore.setPersonality(p);
    setPersonality(p);
  };

  return [personality, updatePersonality] as const;
}

export function useSoundEnabled() {
  const [enabled, setEnabled] = useState(shadowStore.isSoundEnabled());

  const updateSound = (e: boolean) => {
    shadowStore.setSoundEnabled(e);
    setEnabled(e);
  };

  return [enabled, updateSound] as const;
}


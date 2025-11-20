/**
 * Theme Context
 * Provides theme system based on selected personality
 */

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Personality, Theme, THEMES, getTheme, getThemeCSSVariables } from '../lib/themes';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import { usePlatform } from '../hooks/usePlatform';

interface ThemeContextValue {
  personality: Personality;
  theme: Theme;
  setPersonality: (personality: Personality) => Promise<void>;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [personality, setPersonalityState] = useState<Personality>('aerya');
  const [theme, setTheme] = useState<Theme>(THEMES.aerya);
  const platform = usePlatform();

  // Load personality from backend on mount
  useEffect(() => {
    loadPersonality();

    // Listen for theme changes from other windows
    const unlisten = listen<string>('theme-changed', (event) => {
      const newPersonality = event.payload as Personality;
      console.log(`🎨 Theme changed event received: ${newPersonality}`);

      if (THEMES[newPersonality]) {
        setPersonalityState(newPersonality);
        setTheme(THEMES[newPersonality]);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  // Apply CSS variables when theme changes
  useEffect(() => {
    const root = document.documentElement;
    const body = document.body;
    const cssVars = getThemeCSSVariables(theme);

    // Apply theme-specific variables
    Object.entries(cssVars).forEach(([key, value]) => {
      root.style.setProperty(key, value);
    });

    // Update global glass variables to match theme
    // macOS: Use higher opacity without backdrop-filter to prevent flicker
    // Other platforms: Use lower opacity with backdrop-filter for glassmorphism
    if (platform === 'macos') {
      // macOS: No backdrop-filter, higher opacity (0.92) + gradient for depth
      const color = theme.glass.bg.match(/rgba?\(([^)]+)\)/)?.[1] || '110, 231, 183, 0.92';
      const [r, g, b] = color.split(',').map(v => parseInt(v.trim()));

      root.style.setProperty('--glass-bg', `linear-gradient(135deg, rgba(${r}, ${g}, ${b}, 0.92) 0%, rgba(${r}, ${g}, ${b}, 0.88) 100%)`);
      root.style.setProperty('--glass-backdrop', 'none');
    } else {
      // Windows/Linux: Full glassmorphism with backdrop-filter
      root.style.setProperty('--glass-bg', theme.glass.bg);
      root.style.setProperty('--glass-backdrop', 'blur(40px) saturate(180%)');
    }

    root.style.setProperty('--glass-border', theme.glass.border);
    root.style.setProperty('--glass-shadow', theme.glass.shadow);

    // Update text colors
    root.style.setProperty('--text-primary', theme.text.primary);
    root.style.setProperty('--text-secondary', theme.text.secondary);
    root.style.setProperty('--text-muted', theme.text.muted);

    // Update accent colors
    root.style.setProperty('--accent-primary', theme.accent);
    root.style.setProperty('--accent-light', theme.accentLight);

    // IMPORTANT: Do NOT apply gradient to body for transparent windows
    // Glassmorphic windows need transparent background
    // Only set font and transition
    body.style.fontFamily = theme.font;
    body.style.transition = `background ${theme.transitionSpeed}ms ease-in-out`;

    // Keep body transparent for glassmorphic effect
    body.style.background = 'transparent';
  }, [theme, platform]);

  const loadPersonality = async () => {
    try {
      const savedPersonality = await invoke<string>('get_personality');
      const validPersonality = savedPersonality as Personality;
      
      if (THEMES[validPersonality]) {
        setPersonalityState(validPersonality);
        setTheme(THEMES[validPersonality]);
      }
    } catch (error) {
      console.error('Failed to load personality:', error);
      // Default to aura
    }
  };

  const setPersonality = async (newPersonality: Personality) => {
    try {
      // Save to backend
      await invoke('set_personality', { personality: newPersonality });

      // Update state locally
      setPersonalityState(newPersonality);
      setTheme(getTheme(newPersonality));

      // Broadcast theme change to all other windows
      await emit('theme-changed', newPersonality);

      console.log(`✨ Theme changed to: ${newPersonality} (broadcasted to all windows)`);
    } catch (error) {
      console.error('Failed to save personality:', error);
      throw error;
    }
  };

  return (
    <ThemeContext.Provider value={{ personality, theme, setPersonality }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider');
  }
  return context;
}


/**
 * Opportunity Context - Phase 3A
 * Manages opportunities (mock data for now, real detection in Phase 3B)
 * Listens to Tauri events: opportunity:new
 */

import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Opportunity, OpportunityStatus } from '../types';

// ============================================================================
// Context Types
// ============================================================================

interface OpportunityContextValue {
  opportunities: Opportunity[];
  latestOpportunity: Opportunity | null;
  addOpportunity: (opportunity: Opportunity) => void;
  markAsViewed: (id: string) => void;
  markAsActioned: (id: string) => void;
  markAsIgnored: (id: string) => void;
  getOpportunity: (id: string) => Opportunity | undefined;
  clearAll: () => void;
}

// ============================================================================
// Context Creation
// ============================================================================

const OpportunityContext = createContext<OpportunityContextValue | undefined>(undefined);

// ============================================================================
// Provider Component
// ============================================================================

interface OpportunityProviderProps {
  children: ReactNode;
  maxOpportunities?: number;
}

export function OpportunityProvider({
  children,
  maxOpportunities = 50
}: OpportunityProviderProps) {
  const [opportunities, setOpportunities] = useState<Opportunity[]>([]);

  // Get latest opportunity (most recent pending)
  const latestOpportunity = opportunities
    .filter(opp => opp.status === 'pending')
    .sort((a, b) => b.timestamp - a.timestamp)[0] || null;

  // Add new opportunity
  const addOpportunity = (opportunity: Opportunity) => {
    setOpportunities((prev) => {
      // Add to start (most recent first)
      const updated = [opportunity, ...prev];

      // Trim to max size
      if (updated.length > maxOpportunities) {
        return updated.slice(0, maxOpportunities);
      }

      return updated;
    });
  };

  // Mark as viewed
  const markAsViewed = (id: string) => {
    console.log(`[OpportunityContext] Marking ${id} as viewed`);
    setOpportunities((prev) =>
      prev.map((opp) => {
        if (opp.id === id) {
          console.log(`[OpportunityContext] Status ${opp.status} → viewed`);
          return { ...opp, status: 'viewed' as OpportunityStatus };
        }
        return opp;
      })
    );
  };

  // Mark as actioned
  const markAsActioned = (id: string) => {
    setOpportunities((prev) =>
      prev.map((opp) =>
        opp.id === id ? { ...opp, status: 'actioned' as OpportunityStatus } : opp
      )
    );
  };

  // Mark as ignored
  const markAsIgnored = (id: string) => {
    setOpportunities((prev) =>
      prev.map((opp) =>
        opp.id === id ? { ...opp, status: 'ignored' as OpportunityStatus } : opp
      )
    );
  };

  // Get opportunity by ID
  const getOpportunity = (id: string) => {
    return opportunities.find((opp) => opp.id === id);
  };

  // Clear all opportunities
  const clearAll = () => {
    setOpportunities([]);
  };

  // Listen to Tauri events
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      try {
        unlisten = await listen<Opportunity>('opportunity:new', (event) => {
          console.log('📬 Received opportunity:new event', event.payload);
          addOpportunity(event.payload);
        });
      } catch (error) {
        console.error('Failed to setup opportunity listener:', error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const value: OpportunityContextValue = {
    opportunities,
    latestOpportunity,
    addOpportunity,
    markAsViewed,
    markAsActioned,
    markAsIgnored,
    getOpportunity,
    clearAll,
  };

  return (
    <OpportunityContext.Provider value={value}>
      {children}
    </OpportunityContext.Provider>
  );
}

// ============================================================================
// Custom Hook
// ============================================================================

export function useOpportunities() {
  const context = useContext(OpportunityContext);

  if (context === undefined) {
    throw new Error('useOpportunities must be used within OpportunityProvider');
  }

  return context;
}

import { useState } from 'react';
import { SuggestionBubble } from './components/SuggestionBubble';
import OpportunityToast from './components/OpportunityToast';
import { TriggerBubble } from './components/TriggerBubble';
import { QuickActions } from './components/QuickActions';
import { LearnByDoing } from './components/LearnByDoing';
import { useTrigger } from './hooks/useTrigger';
import './components/SuggestionBubble.css';

interface SuggestionResponse {
  suggestion: {
    id: string;
    artefact_type: string;
  };
  artefact: {
    artefact_type: string;
    file_path: string | null;
    content: string | null;
  };
  context: {
    app: {
      name: string;
    };
  };
}

export default function App() {
  const [currentSuggestion, setCurrentSuggestion] = useState<SuggestionResponse | null>(null);
  const [showLearnByDoing, setShowLearnByDoing] = useState(false);

  // Integrate trigger system with TriggerBubble
  const { triggerContext, showBubble, hideBubble, handleUserInteraction } = useTrigger(
    (ctx) => {
      console.log('🔔 Trigger received:', ctx.app.name);
      // Context is already managed by the hook
    },
    true, // autoStart
    true  // enableSmartPositioning
  );

  // Show Learn by Doing view
  if (showLearnByDoing) {
    return (
      <div className="min-h-screen bg-gray-100 dark:bg-gray-900">
        <button
          onClick={() => setShowLearnByDoing(false)}
          style={{
            position: 'fixed',
            top: '20px',
            right: '20px',
            padding: '10px 20px',
            backgroundColor: '#3b82f6',
            color: '#fff',
            border: 'none',
            borderRadius: '6px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: '600',
            zIndex: 1000,
          }}
        >
          ← Back to Main
        </button>
        <LearnByDoing />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900">
      {/* Trigger Bubble - Point d'entrée principal */}
      <TriggerBubble
        context={triggerContext}
        isVisible={showBubble}
        onHide={hideBubble}
        onUserInteraction={handleUserInteraction}
      />

      {/* Suggestion bubble */}
      {currentSuggestion && (
        <SuggestionBubble
          suggestion={currentSuggestion}
          onClose={() => setCurrentSuggestion(null)}
        />
      )}

      {/* Clueless Phase 1: Opportunity Toast */}
      <OpportunityToast
        onOpenDock={async () => {
          const { invoke } = await import('@tauri-apps/api/core');
          try {
            await invoke('show_window', { windowLabel: 'chat' });
          } catch (e) {
            console.error('Failed to open chat:', e);
          }
        }}
      />

      {/* Quick Actions - Contextual actions */}
      <QuickActions
        context={{
          app: triggerContext?.app.name,
        }}
        onOpenDock={async () => {
          const { invoke } = await import('@tauri-apps/api/core');
          try {
            await invoke('show_window', { windowLabel: 'chat' });
          } catch (e) {
            console.error('Failed to open chat:', e);
          }
        }}
      />

      {/* Learn by Doing Button */}
      <button
        onClick={() => setShowLearnByDoing(true)}
        style={{
          position: 'fixed',
          bottom: '30px',
          right: '30px',
          padding: '16px 24px',
          backgroundColor: '#8b5cf6',
          color: '#fff',
          border: 'none',
          borderRadius: '12px',
          cursor: 'pointer',
          fontSize: '15px',
          fontWeight: '600',
          boxShadow: '0 4px 12px rgba(139, 92, 246, 0.4)',
          zIndex: 1000,
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          transition: 'all 0.2s',
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.transform = 'translateY(-2px)';
          e.currentTarget.style.boxShadow = '0 6px 16px rgba(139, 92, 246, 0.5)';
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.transform = 'translateY(0)';
          e.currentTarget.style.boxShadow = '0 4px 12px rgba(139, 92, 246, 0.4)';
        }}
      >
        📚 Learn by Doing
      </button>
    </div>
  );
}


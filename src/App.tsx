import { useState } from 'react';
import { SuggestionBubble } from './components/SuggestionBubble';
import OpportunityToast from './components/OpportunityToast';
import { TriggerBubble } from './components/TriggerBubble';
import { QuickActions } from './components/QuickActions';
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

  // Integrate trigger system with TriggerBubble
  const { triggerContext, showBubble, hideBubble, handleUserInteraction } = useTrigger(
    (ctx) => {
      console.log('🔔 Trigger received:', ctx.app.name);
      // Context is already managed by the hook
    },
    true, // autoStart
    true  // enableSmartPositioning
  );

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
    </div>
  );
}


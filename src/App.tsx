import { useState } from 'react';
import { SuggestionBubble } from './components/SuggestionBubble';
import OpportunityToast from './components/OpportunityToast';
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

  // TODO: Integrate with trigger system
  // useTrigger(handleTrigger);

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900">
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
    </div>
  );
}


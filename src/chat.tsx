import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import HeaderDraggable from './components/HeaderDraggable';
import WindowManager from './components/WindowManager';
import { TriggerBubble } from './components/TriggerBubble';
import { StatusIndicator } from './components/StatusIndicator';
import { MessageFeedback } from './components/MessageFeedback';
import OpportunityToast from './components/OpportunityToast';
import { SlashCommands } from './components/SlashCommands';
import { QuickActions } from './components/QuickActions';
import { SmartPills } from './components/SmartPills';
import { SmartDock } from './components/SmartDock';
import { DailyDigest } from './components/DailyDigest';
import { StreakTracker } from './components/StreakTracker';
import { PersonalitySelector } from './components/PersonalitySelector';
import { PauseMode } from './components/PauseMode';
import { OpportunityLayer } from './components/OpportunityLayer';
import { HelpModal } from './components/HelpModal';
import { LayoutProvider } from './contexts/LayoutContext';
import { ThemeProvider } from './contexts/ThemeContext';
import useWindowLifecycle from './hooks/useWindowLifecycle';
import useDesktopFocus from './hooks/useDesktopFocus';
import useActivityDetection from './hooks/useActivityDetection';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import type { Opportunity } from './lib';
import './styles/island-globals.css';
import './components/TriggerBubble.css';

interface Message {
  id: number;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

function ChatWindow() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');

  const [isActive, setIsActive] = useState(true);
  const [triggerContext, setTriggerContext] = useState<any>(null);
  const [showBubble, setShowBubble] = useState(false);
  
  // J3: Chat states
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Phase 2: Quick Actions context
  const [currentContext] = useState({
    app: "Cursor",
    selectedText: "",
    url: "",
    language: "typescript",
  });
  
  // Phase 2: Smart Dock
  const [isDockOpen, setIsDockOpen] = useState(false);
  const [dockNearCursor, setDockNearCursor] = useState(false);
  
  // Phase 3: Daily Digest
  const [isDigestOpen, setIsDigestOpen] = useState(false);

  // Opportunity Layer
  const [activeOpportunity, setActiveOpportunity] = useState<Opportunity | null>(null);

  // Help Modal
  const [isHelpOpen, setIsHelpOpen] = useState(false);

  useWindowLifecycle({
    onFocus: () => {},
    onBlur: () => {},
  });

  // DISABLED: useDesktopFocus causes Chat window to become inaccessible on macOS
  // The hook calls invoke('focus_window') which doesn't exist in backend
  // Combined with skipTaskbar:true, this makes the window hide permanently
  // useDesktopFocus({ enabled: true, delay: 150 });
  useActivityDetection(true);

  // Keyboard shortcuts (only work when window is focused)
  useKeyboardShortcuts({
    onToggleDock: () => {
      setIsDockOpen(prev => !prev);
      setDockNearCursor(false);
    },
    onOpenDigest: () => {
      setIsDigestOpen(true);
      if (isDockOpen) setIsDockOpen(false);
    },
    onTogglePause: () => {
      // Pause mode toggle will be implemented with PauseMode component
      console.log('Toggle pause mode');
    },
    onCloseModal: () => {
      if (isDockOpen) setIsDockOpen(false);
      if (isDigestOpen) setIsDigestOpen(false);
      if (isHelpOpen) setIsHelpOpen(false);
    },
  });

  // Écouter les événements trigger du backend
  useEffect(() => {
    const setupListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event');
      const unlisten = await listen('trigger_fired', (event: any) => {
        console.log('🎯 Trigger fired event:', event);
        console.log('🎯 Event payload:', event.payload);
        setTriggerContext(event.payload);
        setShowBubble(true); // Afficher la bulle
      });
      
      console.log('✅ trigger_fired listener registered');
      
      return () => {
        unlisten();
      };
    };
    
    setupListeners();
  }, []);

  const generateMockSuggestion = (appName: string): string => {
    const suggestions: Record<string, string> = {
      'FL Studio': '💡 Astuce FL Studio : Pour ajouter un fade out, sélectionne la région audio, puis Edit → Fade Out. Tu peux ajuster la courbe selon tes préférences.',
      'Visual Studio Code': '💡 Astuce VS Code : Utilise Ctrl+Shift+P (Cmd+Shift+P sur Mac) pour accéder à la palette de commandes et ouvrir rapidement n\'importe quelle fonction.',
      'Cursor': '💡 Astuce Cursor : Utilise Cmd+K pour ouvrir le chat IA et poser des questions sur ton code en contexte.',
      'Blender': '💡 Astuce Blender : Utilise Tab pour basculer entre le mode Edit et le mode Object rapidement.',
      'Figma': '💡 Astuce Figma : Utilise Cmd+D pour dupliquer une sélection rapidement et aligner tes éléments.',
    };
    
    return suggestions[appName] || `💡 Astuce pour ${appName} : Tu sembles bloqué. Veux-tu que je te propose des ressources ou des astuces spécifiques ?`;
  };

  const addMessage = (role: 'user' | 'assistant', content: string) => {
    const newMessage: Message = {
      id: Date.now(),
      role,
      content,
      timestamp: new Date(),
    };
    setMessages((prev) => [...prev, newMessage]);
  };

  const handleSend = async () => {
    if (!inputValue.trim() || isLoading) return;

    const userMessage: Message = {
      id: Date.now(),
      role: 'user',
      content: inputValue,
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    const messageText = inputValue;
    setInputValue('');
    setIsLoading(true);
    setError(null);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const response = await invoke<string>('chat_with_ai', {
        message: messageText,
        includeContext: true,
      });

      const assistantMessage: Message = {
        id: Date.now() + 1,
        role: 'assistant',
        content: response,
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (err: any) {
      console.error('Chat error:', err);
      setError(err || 'Une erreur est survenue');
      
      const errorMessage: Message = {
        id: Date.now() + 1,
        role: 'assistant',
        content: err || 'Impossible de contacter l\'IA. Essayez de relancer l\'application ou utilisez Ollama local.',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const formatTime = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  return (
    <ThemeProvider>
      <LayoutProvider>
        <WindowManager>
          <div className="sl-island">
        <HeaderDraggable
          title="ShadowLearn"
          showMinimize={true}
          rightContent={
            <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
              <button
                onClick={() => setIsHelpOpen(true)}
                style={{
                  padding: '6px 12px',
                  background: 'rgba(135, 206, 235, 0.2)',
                  border: '1px solid rgba(135, 206, 235, 0.5)',
                  borderRadius: '6px',
                  color: 'white',
                  cursor: 'pointer',
                  fontSize: '0.85em',
                  fontWeight: '600',
                  transition: 'all 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = 'rgba(135, 206, 235, 0.3)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = 'rgba(135, 206, 235, 0.2)';
                }}
              >
                ❓ Aide
              </button>
              <button
                onClick={() => setIsActive(!isActive)}
                style={{
                  padding: '6px 12px',
                  background: isActive ? 'rgba(16, 185, 129, 0.2)' : 'rgba(239, 68, 68, 0.2)',
                  border: `1px solid ${isActive ? 'rgba(16, 185, 129, 0.5)' : 'rgba(239, 68, 68, 0.5)'}`,
                  borderRadius: '6px',
                  color: 'white',
                  cursor: 'pointer',
                  fontSize: '0.85em',
                  fontWeight: '600',
                }}
              >
                {isActive ? '✓ Actif' : '✗ Inactif'}
              </button>
            </div>
          }
        >
        </HeaderDraggable>

      <div className="sl-body">
        {/* Opportunity Layer */}
        {activeOpportunity && (
          <OpportunityLayer
            opportunity={activeOpportunity}
            onClose={() => setActiveOpportunity(null)}
            onDiscuss={(text) => {
              // Add the opportunity discussion as a user message
              addMessage('user', text);
              setActiveOpportunity(null);
            }}
            onApply={() => {
              // Add confirmation message
              addMessage('assistant', `✓ Suggestion appliquée avec succès !`);
              setActiveOpportunity(null);
            }}
          />
        )}

        {messages.length === 0 ? (
          <div style={{ 
            display: 'flex', 
            flexDirection: 'column', 
            justifyContent: 'center', 
            alignItems: 'center',
            height: '100%',
            color: 'rgba(255, 255, 255, 0.4)',
            textAlign: 'center',
            padding: '40px',
          }}>
            <div style={{ fontSize: '48px', marginBottom: '16px' }}>🤖</div>
            <h2 style={{ fontSize: '20px', fontWeight: '600', marginBottom: '8px' }}>
              ShadowLearn est prêt
            </h2>
            <p style={{ fontSize: '14px' }}>
              {isActive 
                ? 'Je surveille ton activité et je vais te proposer de l\'aide quand tu en auras besoin.'
                : 'Mets-moi en mode actif pour démarrer.'}
            </p>
          </div>
        ) : (
          <div className="messages-container">
            {messages.map((message) => (
              <div
                key={message.id}
                className={
                  message.role === 'user' ? 'message-user' : 'message-assistant'
                }
              >
                <div>{message.content}</div>
                <div className="message-timestamp">{formatTime(message.timestamp)}</div>
                
                {/* Clueless Phase 1: Message Feedback */}
                {message.role === 'assistant' && (
                  <MessageFeedback 
                    messageId={message.id.toString()}
                    onFeedback={(helpful) => {
                      console.log(`Feedback for message ${message.id}:`, helpful);
                    }}
                  />
                )}
              </div>
            ))}
          </div>
        )}
      </div>

             <footer className="sl-input">
               {error && (
                 <div style={{
                   padding: '8px 12px',
                   marginBottom: '8px',
                   background: 'rgba(251, 191, 36, 0.2)',
                   border: '1px solid rgba(251, 191, 36, 0.5)',
                   borderRadius: '6px',
                   fontSize: '12px',
                   color: 'rgba(251, 191, 36, 0.9)',
                   textAlign: 'center',
                 }}>
                   ⚠️ Réponse locale (Ollama)
                 </div>
               )}
               
               {/* Phase 2: Slash Commands Input */}
               <SlashCommands
                 value={inputValue}
                 onChange={setInputValue}
                 onSubmit={handleSend}
                 onCommandResult={(result) => addMessage('assistant', result)}
                 placeholder={isActive ? "Écris un message ou tape / pour les commandes..." : "Active ShadowLearn pour continuer"}
                 onOpenDigest={() => setIsDigestOpen(true)}
                 onOpenDock={() => setIsDockOpen(true)}
               />
               {isLoading && (
                 <div style={{
                   position: 'absolute',
                   bottom: '16px',
                   right: '80px',
                   fontSize: '12px',
                   color: 'rgba(255,255,255,0.5)',
                 }}>
                   <span style={{ marginRight: '8px' }}>⟳</span>
                   Génération en cours...
                 </div>
               )}
        
        {/* Feedback buttons when suggestion is shown */}
        {messages.some(m => m.content.includes('💡 Astuce')) && (
          <div style={{
            display: 'flex',
            gap: '8px',
            marginTop: '8px',
            justifyContent: 'center'
          }}>
            <button
              onClick={() => {
                addMessage('user', '👍 Utile');
                console.log('✅ Feedback: utile');
                // TODO: Logger feedback dans DB
              }}
              style={{
                padding: '6px 12px',
                background: 'rgba(16, 185, 129, 0.2)',
                border: '1px solid rgba(16, 185, 129, 0.5)',
                borderRadius: '6px',
                color: 'white',
                cursor: 'pointer',
                fontSize: '0.85em',
              }}
            >
              👍 Utile
            </button>
            <button
              onClick={() => {
                addMessage('user', '👎 Pas utile');
                console.log('❌ Feedback: pas utile');
                // TODO: Logger feedback dans DB
              }}
              style={{
                padding: '6px 12px',
                background: 'rgba(239, 68, 68, 0.2)',
                border: '1px solid rgba(239, 68, 68, 0.5)',
                borderRadius: '6px',
                color: 'white',
                cursor: 'pointer',
                fontSize: '0.85em',
              }}
            >
              👎 Pas utile
            </button>
          </div>
        )}
      </footer>
    </div>

      {/* Trigger Bubble */}
      {triggerContext && showBubble && (
        <TriggerBubble
          context={triggerContext}
          isVisible={showBubble}
          onHide={() => {
            console.log('🔴 Hiding bubble');
            setShowBubble(false);
          }}
          onUserInteraction={() => {
            console.log('✅ User interaction');
            setShowBubble(false);
            // Ajouter une carte mock basée sur l'app
            const appName = triggerContext?.app?.name || 'Application';
            const suggestion = generateMockSuggestion(appName);
            addMessage('assistant', suggestion);
          }}
        />
      )}

      {/* Status Indicator (J2) - DISABLED: Causes macOS glassmorphism flicker */}
      {/* setInterval(1000ms) causes constant re-renders that destabilize backdrop-filter */}
      {/* <StatusIndicator /> */}

      {/* Clueless Phase 1: Opportunity Toast */}
      <OpportunityToast
        onOpenDock={() => setIsDockOpen(true)}
        onOpenDigest={() => setIsDigestOpen(true)}
        onOpenChat={(opportunity) => {
          console.log('[Chat] Opening opportunity in chat:', opportunity);
          setActiveOpportunity(opportunity);
        }}
      />

      {/* Clueless Phase 2: Quick Actions */}
      <QuickActions 
        context={currentContext}
        onOpenDock={() => setIsDockOpen(true)}
        onOpenDigest={() => setIsDigestOpen(true)}
      />

      {/* Clueless Phase 2: Smart Pills */}
      <SmartPills context={currentContext} />

      {/* Clueless Phase 2: Smart Dock */}
      <SmartDock
        isOpen={isDockOpen}
        onClose={() => setIsDockOpen(false)}
        nearCursor={dockNearCursor}
      >
        <div style={{ color: "var(--text-primary)", display: "flex", flexDirection: "column", gap: "16px" }}>
          {/* Streak Tracker */}
          <StreakTracker compact={false} />
          
          {/* Personality Selector */}
          <PersonalitySelector 
            compact={false}
            onPersonalityChange={(p) => console.log("Personality changed to:", p)}
          />
          
          {/* Pause Mode */}
          <PauseMode 
            compact={false}
            onPauseChange={(isPaused) => console.log("Pause mode:", isPaused)}
          />
          
          {/* Quick actions */}
          <div style={{ display: "flex", gap: "8px" }}>
            <button
              onClick={() => {
                setIsDigestOpen(true);
                setIsDockOpen(false);
              }}
              style={{
                flex: 1,
                padding: "12px",
                background: "linear-gradient(135deg, rgba(135, 206, 235, 0.3), rgba(16, 185, 129, 0.3))",
                border: "1px solid var(--glass-border)",
                borderRadius: "8px",
                color: "var(--text-primary)",
                cursor: "pointer",
                fontWeight: "600",
                fontSize: "13px",
              }}
            >
              📊 Voir le Digest
            </button>
            <button
              onClick={() => setDockNearCursor(!dockNearCursor)}
              style={{
                padding: "12px 16px",
                background: "rgba(255, 255, 255, 0.05)",
                border: "1px solid var(--glass-border)",
                borderRadius: "8px",
                color: "var(--text-muted)",
                cursor: "pointer",
                fontSize: "13px",
              }}
            >
              {dockNearCursor ? "📍" : "🔒"}
            </button>
          </div>
        </div>
      </SmartDock>

      {/* Clueless Phase 3: Daily Digest */}
      <DailyDigest
        isOpen={isDigestOpen}
        onClose={() => setIsDigestOpen(false)}
      />

      {/* Help Modal */}
      <HelpModal
        isOpen={isHelpOpen}
        onClose={() => setIsHelpOpen(false)}
      />
        </WindowManager>
      </LayoutProvider>
    </ThemeProvider>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ChatWindow />
  </React.StrictMode>
);

# 🌟 SHADOWLEARN "CLUELY+" - FEATURES SIGNATURE 11/10

Tu as raison - ces features sont exactement ce qui transforme un bon produit en produit magique. Voici **tout ce qui fait que Cluely est addictif**, adapté à ShadowLearn, **+ mes suggestions 11/10** :

---

## 📋 FEATURES À IMPLÉMENTER

### ✅ Déjà mentionnées (on les fait toutes)
1. ✅ One-Tap Help Toast
2. ✅ Smart docking near cursor
3. ✅ Daily mini-digest
4. ✅ Slash-commands avec autocomplétion
5. ✅ Feedback binaire instant (👍/👎)

### 🚀 MES SUGGESTIONS 11/10 (Inspirées Cluely + innovations)

6. **🎯 Contextual Quick Actions** : Boutons magiques qui apparaissent selon le contexte
   - *"📋 Résumer ça"* sur long texte détecté
   - *"🐛 Debug ça"* sur stack trace
   - *"✨ Améliorer"* sur code sélectionné
   - *"🔍 Expliquer"* sur terme technique

7. **⚡ Smart Suggestions Pills** : Chips contextuelles flottantes au-dessus de la bulle
   - Apparaissent 2s avant une suggestion complète
   - "Continue X", "Besoin d'aide avec Y?", "Rappel: Z"
   - Swipe-away ou tap-to-expand

8. **🎨 Ambient Mode** : Présence ultra-discrète qui "respire" avec ton travail
   - LED change de rythme selon ton flow state
   - Pulse lent = deep work, pulse rapide = blocked
   - Option "Ne pas déranger" automatique

9. **🧠 Learning Streaks** : Gamification douce sans être intrusive
   - "🔥 5 jours où tu m'as trouvé utile"
   - Célébration discrète milestone (confetti subtil)
   - Unlock features progressivement

10. **📊 Context Cards** : Mini-preview du contexte avant d'ouvrir le dock
    - Hover bulle → voir derniers screenshots analysés
    - "Tu travailles sur [projet] depuis 2h"
    - Preview suggestions sans ouvrir

11. **🎭 Personality Modes** : Ton IA change de style
    - Default (sobre), Mentor (pédago), Buddy (casual), Pro (technique)
    - Détection auto selon contexte ou switch manuel

12. **⏰ Smart Pause** : Detection automatique pauses légitimes
    - Meeting détecté → pause auto
    - Pause café détectée → pas de suggestions
    - Retour bureau → "Re-bienvenue 👋"

---

# 💻 CODE COMPLET - TOUTES LES FEATURES

## 1. 🍞 One-Tap Help Toast

```tsx
// components/OpportunityToast.tsx
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";

interface Opportunity {
  id: string;
  title: string;
  confidence: number;
  preview: string;
}

export default function OpportunityToast({ onOpenDock }: { onOpenDock: () => void }) {
  const [opportunity, setOpportunity] = useState<Opportunity | null>(null);
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());

  useEffect(() => {
    // Listen for opportunity events from Tauri
    const unlisten = window.addEventListener("shadow:opportunity", (e: any) => {
      const opp = e.detail;
      if (!dismissed.has(opp.id) && opp.confidence > 0.7) {
        setOpportunity(opp);
        
        // Auto-dismiss after 10s if no interaction
        setTimeout(() => {
          setOpportunity(null);
        }, 10000);
      }
    });

    return () => {
      // @ts-ignore
      unlisten();
    };
  }, [dismissed]);

  const handleView = async () => {
    if (!opportunity) return;
    
    // Record user accepted
    await invoke("record_opportunity_response", {
      opportunityId: opportunity.id,
      accepted: true,
    }).catch(console.error);
    
    onOpenDock();
    setOpportunity(null);
  };

  const handleDismiss = async () => {
    if (!opportunity) return;
    
    // Record dismissed
    await invoke("record_opportunity_response", {
      opportunityId: opportunity.id,
      accepted: false,
    }).catch(console.error);
    
    setDismissed(prev => new Set([...prev, opportunity.id]));
    setOpportunity(null);
  };

  return (
    <AnimatePresence>
      {opportunity && (
        <motion.div
          initial={{ opacity: 0, y: 20, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: 20, scale: 0.95 }}
          transition={{ 
            type: "spring",
            stiffness: 300,
            damping: 30
          }}
          className="fixed bottom-24 right-6 z-40 max-w-sm"
        >
          <div className="glass rounded-2xl p-4 shadow-xl border border-white/30">
            {/* Header */}
            <div className="flex items-start gap-3 mb-3">
              <motion.div
                animate={{
                  rotate: [0, 10, -10, 10, 0],
                  scale: [1, 1.1, 1],
                }}
                transition={{
                  duration: 0.5,
                  ease: "easeInOut"
                }}
                className="text-2xl"
              >
                💡
              </motion.div>
              <div className="flex-1">
                <h3 className="font-semibold text-gray-900 text-sm">
                  J'ai une idée
                </h3>
                <p className="text-xs text-gray-600 mt-0.5 line-clamp-2">
                  {opportunity.preview}
                </p>
              </div>
            </div>

            {/* Confidence indicator */}
            <div className="mb-3">
              <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
                <span>Confiance</span>
                <span>{Math.round(opportunity.confidence * 100)}%</span>
              </div>
              <div className="h-1 bg-gray-200 rounded-full overflow-hidden">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${opportunity.confidence * 100}%` }}
                  className="h-full bg-gradient-to-r from-emerald-500 to-sky-500"
                />
              </div>
            </div>

            {/* Actions */}
            <div className="flex gap-2">
              <button
                onClick={handleView}
                className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-all hover-lift"
              >
                Voir →
              </button>
              <button
                onClick={handleDismiss}
                className="px-4 py-2 bg-white/50 text-gray-700 rounded-lg text-sm border border-gray-200 hover:bg-white transition-all"
              >
                Ignorer
              </button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
```

**Backend Tauri (src-tauri/src/commands/opportunities.rs)** :

```rust
#[tauri::command]
pub async fn record_opportunity_response(
    opportunity_id: String,
    accepted: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Update learning model
    state.learning_manager.lock().await
        .record_feedback(opportunity_id, accepted)
        .await?;
    
    // Adjust cooldown based on acceptance
    if !accepted {
        let mut sm = state.state_machine.lock().await;
        sm.transition(TriggerEvent::EnterCooldown {
            reason: CooldownReason::UserDismissed,
        })?;
    }
    
    // Record telemetry
    state.telemetry.lock().await
        .record_trigger(accepted)
        .await;
    
    Ok(())
}

// Emit opportunity from trigger loop
app_handle.emit_all("shadow:opportunity", OpportunityPayload {
    id: uuid::Uuid::new_v4().to_string(),
    title: "Aide disponible".into(),
    confidence: 0.85,
    preview: opportunity.detected_task.clone(),
}).ok();
```

---

## 2. 🎯 Smart Docking Near Cursor

```tsx
// utils/smartDocking.ts
interface Position {
  x: number;
  y: number;
}

export function calculateSmartDockPosition(
  cursorPos: Position,
  dockWidth: number,
  dockHeight: number,
  windowWidth: number,
  windowHeight: number
): Position {
  const MARGIN = 24;
  const SNAP_THRESHOLD = 100; // pixels from edge to snap

  let x = cursorPos.x - dockWidth / 2;
  let y = cursorPos.y - dockHeight / 2;

  // Constrain to viewport
  x = Math.max(MARGIN, Math.min(x, windowWidth - dockWidth - MARGIN));
  y = Math.max(MARGIN, Math.min(y, windowHeight - dockHeight - MARGIN));

  // Smart snap to bottom-right if close enough
  const distanceToBottomRight = Math.hypot(
    (windowWidth - dockWidth - MARGIN) - x,
    (windowHeight - dockHeight - MARGIN) - y
  );

  if (distanceToBottomRight < SNAP_THRESHOLD) {
    return {
      x: windowWidth - dockWidth - MARGIN,
      y: windowHeight - dockHeight - MARGIN,
    };
  }

  return { x, y };
}

// Hook pour cursor tracking
export function useCursorPosition() {
  const [position, setPosition] = useState({ x: 0, y: 0 });

  useEffect(() => {
    const handleMove = (e: MouseEvent) => {
      setPosition({ x: e.clientX, y: e.clientY });
    };

    window.addEventListener("mousemove", handleMove);
    return () => window.removeEventListener("mousemove", handleMove);
  }, []);

  return position;
}
```

**ChatDock modifié** :

```tsx
// Dans ChatDock.tsx
import { calculateSmartDockPosition, useCursorPosition } from "../utils/smartDocking";

export default function ChatDock({ open, onClose }: { open: boolean; onClose: () => void }) {
  const cursorPos = useCursorPosition();
  const [dockPosition, setDockPosition] = useState({ x: 0, y: 0 });
  const [isSnapping, setIsSnapping] = useState(false);

  useEffect(() => {
    if (open) {
      const smartPos = calculateSmartDockPosition(
        cursorPos,
        420, // dock width
        640, // dock height
        window.innerWidth,
        window.innerHeight
      );

      setDockPosition(smartPos);

      // Check if snapping
      const isNearEdge = 
        smartPos.x > window.innerWidth - 500 &&
        smartPos.y > window.innerHeight - 700;
      
      setIsSnapping(isNearEdge);
    }
  }, [open, cursorPos]);

  return (
    <AnimatePresence>
      {open && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 bg-black/20 backdrop-blur-sm z-40"
          />

          <motion.div
            initial={{ 
              opacity: 0, 
              scale: 0.8,
              x: cursorPos.x,
              y: cursorPos.y,
            }}
            animate={{ 
              opacity: 1, 
              scale: 1,
              x: dockPosition.x,
              y: dockPosition.y,
            }}
            exit={{ 
              opacity: 0, 
              scale: 0.95,
              x: window.innerWidth - 500,
              y: window.innerHeight - 700,
            }}
            transition={{
              type: "spring",
              stiffness: isSnapping ? 200 : 300,
              damping: isSnapping ? 25 : 30,
            }}
            className="fixed z-50"
            style={{
              width: 420,
              height: 640,
            }}
          >
            {/* Dock content... */}
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
```

---

## 3. 📊 Daily Mini-Digest

```tsx
// components/DailyDigest.tsx
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";

interface DigestStats {
  suggestions_shown: number;
  suggestions_accepted: number;
  time_saved_minutes: number;
  top_apps: Array<{ name: string; count: number }>;
  highlights: string[];
}

export default function DailyDigest() {
  const [show, setShow] = useState(false);
  const [stats, setStats] = useState<DigestStats | null>(null);

  useEffect(() => {
    checkDigestSchedule();
  }, []);

  const checkDigestSchedule = async () => {
    try {
      const config = await invoke<any>("get_config");
      
      // Check if digest enabled and if it's time
      if (!config.digest?.enabled) return;
      
      const lastShown = localStorage.getItem("last_digest_shown");
      const now = new Date();
      const today = now.toDateString();
      
      // Show once per day at 6pm
      if (lastShown !== today && now.getHours() >= 18) {
        const digestStats = await invoke<DigestStats>("get_daily_digest");
        setStats(digestStats);
        setShow(true);
        localStorage.setItem("last_digest_shown", today);
      }
    } catch (e) {
      console.error("Digest check failed:", e);
    }
  };

  const handleClose = () => {
    setShow(false);
  };

  if (!stats) return null;

  return (
    <AnimatePresence>
      {show && (
        <motion.div
          initial={{ opacity: 0, y: 50, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: 50, scale: 0.95 }}
          transition={{ type: "spring", stiffness: 200, damping: 25 }}
          className="fixed bottom-24 right-6 z-50 w-96"
        >
          <div className="glass rounded-2xl p-6 shadow-2xl">
            {/* Header */}
            <div className="flex items-start justify-between mb-4">
              <div>
                <h3 className="text-lg font-bold text-gray-900 flex items-center gap-2">
                  <span>📊</span>
                  <span>Récap du jour</span>
                </h3>
                <p className="text-sm text-gray-600 mt-0.5">
                  Ce que j'ai fait pour toi aujourd'hui
                </p>
              </div>
              <button
                onClick={handleClose}
                className="text-gray-400 hover:text-gray-600 transition-colors"
              >
                ✕
              </button>
            </div>

            {/* Stats Grid */}
            <div className="grid grid-cols-2 gap-3 mb-4">
              <StatCard
                icon="💡"
                value={stats.suggestions_shown}
                label="suggestions"
                color="from-amber-400 to-orange-500"
              />
              <StatCard
                icon="✅"
                value={stats.suggestions_accepted}
                label="acceptées"
                color="from-emerald-400 to-teal-500"
              />
            </div>

            {/* Time saved */}
            {stats.time_saved_minutes > 0 && (
              <motion.div
                initial={{ scale: 0.9, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                transition={{ delay: 0.2 }}
                className="bg-gradient-to-r from-violet-500 to-purple-600 rounded-xl p-4 text-white mb-4"
              >
                <div className="text-3xl font-bold">
                  ~{stats.time_saved_minutes} min
                </div>
                <div className="text-sm opacity-90">
                  de temps gagné estimé ⚡
                </div>
              </motion.div>
            )}

            {/* Top apps */}
            {stats.top_apps.length > 0 && (
              <div className="mb-4">
                <div className="text-xs font-semibold text-gray-500 mb-2">
                  Apps les plus aidées
                </div>
                <div className="space-y-1.5">
                  {stats.top_apps.slice(0, 3).map((app, i) => (
                    <div
                      key={i}
                      className="flex items-center justify-between text-sm"
                    >
                      <span className="text-gray-700">{app.name}</span>
                      <span className="text-gray-500">{app.count}×</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Highlights */}
            {stats.highlights.length > 0 && (
              <div className="mb-4">
                <div className="text-xs font-semibold text-gray-500 mb-2">
                  Moments clés
                </div>
                <div className="space-y-2">
                  {stats.highlights.map((highlight, i) => (
                    <div
                      key={i}
                      className="text-sm text-gray-600 bg-gray-50 rounded-lg p-2"
                    >
                      {highlight}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Footer */}
            <button
              onClick={handleClose}
              className="w-full py-2.5 bg-gray-100 hover:bg-gray-200 rounded-lg text-sm font-medium text-gray-700 transition-colors"
            >
              Fermer
            </button>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function StatCard({ icon, value, label, color }: {
  icon: string;
  value: number;
  label: string;
  color: string;
}) {
  return (
    <div className={`bg-gradient-to-br ${color} rounded-xl p-4 text-white`}>
      <div className="text-2xl mb-1">{icon}</div>
      <div className="text-2xl font-bold">{value}</div>
      <div className="text-xs opacity-90">{label}</div>
    </div>
  );
}
```

**Backend Rust** :

```rust
#[tauri::command]
pub async fn get_daily_digest(
    state: tauri::State<'_, AppState>,
) -> Result<DigestStats, String> {
    let db = &state.db;
    let today_start = chrono::Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
    
    let suggestions_shown = db.count_triggers_since(today_start).await?;
    let suggestions_accepted = db.count_accepted_triggers_since(today_start).await?;
    
    // Estimate time saved (2 min per accepted suggestion)
    let time_saved_minutes = suggestions_accepted * 2;
    
    let top_apps = db.get_top_apps_today(today_start).await?;
    
    let highlights = db.get_highlights_today(today_start).await?;
    
    Ok(DigestStats {
        suggestions_shown,
        suggestions_accepted,
        time_saved_minutes,
        top_apps,
        highlights,
    })
}
```

---

## 4. ⚡ Slash Commands avec Autocomplétion

```tsx
// components/SlashCommandInput.tsx
import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";

interface Command {
  trigger: string;
  label: string;
  description: string;
  icon: string;
  action: (args?: string) => Promise<void>;
}

const COMMANDS: Command[] = [
  {
    trigger: "/expliquer",
    label: "Expliquer",
    description: "Explique le concept ou code sélectionné",
    icon: "💡",
    action: async (args) => {
      // Implementation
    },
  },
  {
    trigger: "/résumer",
    label: "Résumer",
    description: "Résume le texte ou document",
    icon: "📝",
    action: async (args) => {
      // Implementation
    },
  },
  {
    trigger: "/pasclair",
    label: "Pas clair",
    description: "Reformule plus simplement",
    icon: "🤔",
    action: async (args) => {
      // Implementation
    },
  },
  {
    trigger: "/debug",
    label: "Debug",
    description: "Analyse l'erreur ou le bug",
    icon: "🐛",
    action: async (args) => {
      // Implementation
    },
  },
  {
    trigger: "/améliorer",
    label: "Améliorer",
    description: "Suggère des améliorations",
    icon: "✨",
    action: async (args) => {
      // Implementation
    },
  },
  {
    trigger: "/stats",
    label: "Statistiques",
    description: "Affiche tes stats d'utilisation",
    icon: "📊",
    action: async () => {
      // Show stats
    },
  },
];

export default function SlashCommandInput({
  value,
  onChange,
  onSubmit,
}: {
  value: string;
  onChange: (v: string) => void;
  onSubmit: (v: string) => void;
}) {
  const [showCommands, setShowCommands] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [filteredCommands, setFilteredCommands] = useState<Command[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);

  // Detect slash command
  useEffect(() => {
    if (value.startsWith("/")) {
      const query = value.toLowerCase();
      const matches = COMMANDS.filter(cmd =>
        cmd.trigger.toLowerCase().startsWith(query) ||
        cmd.label.toLowerCase().includes(query.slice(1))
      );
      setFilteredCommands(matches);
      setShowCommands(matches.length > 0);
      setSelectedIndex(0);
    } else {
      setShowCommands(false);
    }
  }, [value]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showCommands) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex(i => (i + 1) % filteredCommands.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex(i => (i - 1 + filteredCommands.length) % filteredCommands.length);
    } else if (e.key === "Enter" || e.key === "Tab") {
      e.preventDefault();
      selectCommand(filteredCommands[selectedIndex]);
    } else if (e.key === "Escape") {
      setShowCommands(false);
    }
  };

  const selectCommand = (cmd: Command) => {
    onChange(cmd.trigger + " ");
    setShowCommands(false);
    inputRef.current?.focus();
  };

  const handleSubmitWithCommand = async () => {
    const parts = value.split(" ");
    const cmdTrigger = parts[0];
    const args = parts.slice(1).join(" ");

    const command = COMMANDS.find(c => c.trigger === cmdTrigger);
    
    if (command) {
      await command.action(args);
      onChange("");
    } else {
      onSubmit(value);
    }
  };

  return (
    <div className="relative">
      {/* Command palette */}
      <AnimatePresence>
        {showCommands && (
          <motion.div
            initial={{ opacity: 0, y: 8, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 8, scale: 0.96 }}
            transition={{ duration: 0.15 }}
            className="absolute bottom-full left-0 right-0 mb-2 glass rounded-xl overflow-hidden shadow-xl"
          >
            {filteredCommands.map((cmd, idx) => (
              <button
                key={cmd.trigger}
                onClick={() => selectCommand(cmd)}
                className={`w-full text-left px-4 py-3 flex items-center gap-3 transition-colors ${
                  idx === selectedIndex
                    ? "bg-blue-50 border-l-2 border-blue-500"
                    : "hover:bg-gray-50"
                }`}
              >
                <span className="text-2xl">{cmd.icon}</span>
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-gray-900 text-sm">
                    {cmd.label}
                  </div>
                  <div className="text-xs text-gray-500 truncate">
                    {cmd.description}
                  </div>
                </div>
                <kbd className="px-2 py-1 bg-gray-200 rounded text-[10px] font-mono">
                  {cmd.trigger}
                </kbd>
              </button>
            ))}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Input */}
      <div className="flex gap-2">
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => {
            handleKeyDown(e);
            if (e.key === "Enter" && !showCommands) {
              handleSubmitWithCommand();
            }
          }}
          placeholder="Écris ou tape / pour les commandes..."
          className="flex-1 px-4 py-3 bg-white/50 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 focus:bg-white transition-all"
        />
        <button
          onClick={handleSubmitWithCommand}
          disabled={!value.trim()}
          className="px-5 py-3 bg-blue-600 text-white rounded-xl hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all hover-lift"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
          </svg>
        </button>
      </div>

      {/* Hint */}
      {value === "" && (
        <div className="absolute top-3 left-4 text-gray-400 text-sm pointer-events-none flex items-center gap-2">
          <kbd className="px-1.5 py-0.5 bg-gray-100 rounded text-xs font-mono">/</kbd>
          <span className="text-xs">pour les commandes rapides</span>
        </div>
      )}
    </div>
  );
}
```

---

## 5. 👍👎 Feedback Binaire Instant

```tsx
// components/MessageFeedback.tsx
import { useState } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/tauri";

export function MessageFeedback({ 
  messageId, 
  onFeedback 
}: { 
  messageId: string;
  onFeedback?: (helpful: boolean) => void;
}) {
  const [feedback, setFeedback] = useState<boolean | null>(null);
  const [showThanks, setShowThanks] = useState(false);

  const handleFeedback = async (helpful: boolean) => {
    setFeedback(helpful);
    setShowThanks(true);

    // Record feedback
    try {
      await invoke("record_message_feedback", {
        messageId,
        helpful,
      });
    } catch (e) {
      console.error("Failed to record feedback:", e);
    }

    onFeedback?.(helpful);

    // Hide thanks message after 2s
    setTimeout(() => {
      setShowThanks(false);
    }, 2000);
  };

  if (feedback !== null && showThanks) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        className="flex items-center gap-2 text-sm text-emerald-600 mt-2"
      >
        <span className="text-lg">✓</span>
        <span>Merci pour ton retour !</span>
      </motion.div>
    );
  }

  if (feedback !== null) {
    return null; // Already submitted
  }

  return (
    <div className="flex items-center gap-2 mt-2">
      <span className="text-xs text-gray-500">Cette réponse t'aide ?</span>
      <div className="flex gap-1">
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={() => handleFeedback(true)}
          className="p-1.5 hover:bg-emerald-50 rounded-lg transition-colors group"
          aria-label="Utile"
        >
          <span className="text-lg opacity-40 group-hover:opacity-100 transition-opacity">
            👍
          </span>
        </motion.button>
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={() => handleFeedback(false)}
          className="p-1.5 hover:bg-red-50 rounded-lg transition-colors group"
          aria-label="Pas utile"
        >
          <span className="text-lg opacity-40 group-hover:opacity-100 transition-opacity">
            👎
          </span>
        </motion.button>
      </div>
    </div>
  );
}
```

**Backend Rust** :

```rust
#[tauri::command]
pub async fn record_message_feedback(
    message_id: String,
    helpful: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Store feedback
    state.db.record_feedback(&message_id, helpful).await?;
    
    // Adjust model weights
    state.learning_manager.lock().await
        .adjust_confidence_weights(helpful)
        .await;
    
    // If negative feedback, increase cooldown slightly
    if !helpful {
        let mut sm = state.state_machine.lock().await;
        if let TriggerState::Cooldown { remaining_seconds, reason } = sm.get_current_state() {
            // Add 15s to cooldown
            sm.update_cooldown(*remaining_seconds + 15);
        }
    }
    
    log::info!("Feedback recorded: message={}, helpful={}", message_id, helpful);
    
    Ok(())
}
```

---

## 6. 🎯 Contextual Quick Actions

```tsx
// components/QuickActions.tsx
import { motion, AnimatePresence } from "framer-motion";
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";

interface QuickAction {
  id: string;
  icon: string;
  label: string;
  action: () => Promise<void>;
}

export default function QuickActions() {
  const [actions, setActions] = useState<QuickAction[]>([]);
  const [context, setContext] = useState<any>(null);

  useEffect(() => {
    const checkContext = async () => {
      try {
        const ctx = await invoke<any>("get_current_context");
        setContext(ctx);
        
        // Determine available actions based on context
        const available: QuickAction[] = [];

        if (ctx.detected_long_text) {
          available.push({
            id: "summarize",
            icon: "📋",
            label: "Résumer ça",
            action: async () => {
              await invoke("quick_action_summarize", { context: ctx });
            },
          });
        }

        if (ctx.detected_stack_trace) {
          available.push({
            id: "debug",
            icon: "🐛",
            label: "Debug ça",
            action: async () => {
              await invoke("quick_action_debug", { context: ctx });
            },
          });
        }

        if (ctx.detected_code_selected) {
          available.push({
            id: "improve",
            icon: "✨",
            label: "Améliorer",
            action: async () => {
              await invoke("quick_action_improve", { context: ctx });
            },
          });
        }

        if (ctx.detected_technical_term) {
          available.push({
            id: "explain",
            icon: "🔍",
            label: "Expliquer",
            action: async () => {
              await invoke("quick_action_explain", { 
                term: ctx.detected_technical_term 
              });
            },
          });
        }

        setActions(available);
      } catch (e) {
        console.error("Context check failed:", e);
      }
    };

    const interval = setInterval(checkContext, 3000);
    checkContext();
    
    return () => clearInterval(interval);
  }, []);

  if (actions.length === 0) return null;

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      className="fixed right-6 bottom-28 z-40 flex flex-col gap-2"
    >
      <AnimatePresence>
        {actions.map((action, idx) => (
          <motion.button
            key={action.id}
            initial={{ opacity: 0, x: 20, scale: 0.8 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: 20, scale: 0.8 }}
            transition={{ delay: idx * 0.05 }}
            whileHover={{ scale: 1.05, x: -4 }}
            whileTap={{ scale: 0.95 }}
            onClick={action.action}
            className="glass px-4 py-3 rounded-xl flex items-center gap-2 shadow-lg hover:shadow-xl transition-all"
          >
            <span className="text-xl">{action.icon}</span>
            <span className="font-medium text-sm text-gray-900">
              {action.label}
            </span>
          </motion.button>
        ))}
      </AnimatePresence>
    </motion.div>
  );
}
```

---

## 7. ⚡ Smart Suggestions Pills (Preview)

```tsx
// components/SuggestionPills.tsx
import { motion, AnimatePresence } from "framer-motion";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface SuggestionPill {
  id: string;
  text: string;
  type: "continue" | "help" | "reminder";
}

export default function SuggestionPills({ onExpand }: { onExpand: () => void }) {
  const [pills, setPills] = useState<SuggestionPill[]>([]);
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());

  useEffect(() => {
    const checkSuggestions = async () => {
      try {
        const suggestions = await invoke<SuggestionPill[]>("get_micro_suggestions");
        setPills(suggestions.filter(s => !dismissed.has(s.id)));
      } catch {}
    };

    const interval = setInterval(checkSuggestions, 5000);
    checkSuggestions();
    
    return () => clearInterval(interval);
  }, [dismissed]);

  const handleDismiss = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setDismissed(prev => new Set([...prev, id]));
  };

  return (
    <AnimatePresence>
      {pills.length > 0 && (
        <motion.div
          initial={{ opacity: 0, scale: 0.9, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.9, y: 20 }}
          className="fixed right-6 bottom-28 z-30 flex flex-col gap-2 max-w-xs"
        >
          {pills.map((pill, idx) => (
            <motion.button
              key={pill.id}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 20, scale: 0.8 }}
              transition={{ delay: idx * 0.1 }}
              onClick={onExpand}
              whileHover={{ scale: 1.03, x: -4 }}
              className="glass px-4 py-3 rounded-full flex items-center gap-2 shadow-md hover:shadow-lg transition-all group"
            >
              <TypeIcon type={pill.type} />
              <span className="flex-1 text-sm text-gray-700 text-left">
                {pill.text}
              </span>
              <button
                onClick={(e) => handleDismiss(pill.id, e)}
                className="opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-gray-600 p-1"
              >
                ✕
              </button>
            </motion.button>
          ))}
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function TypeIcon({ type }: { type: SuggestionPill["type"] }) {
  const icons = {
    continue: "▶️",
    help: "💡",
    reminder: "⏰",
  };
  return <span className="text-lg">{icons[type]}</span>;
}
```

---

## 8. 🎨 Ambient Mode (LED breathing)

```tsx
// Dans FloatingBubble.tsx - modifier la LED
function AmbientLED() {
  const [flowState, setFlowState] = useState<"deep" | "normal" | "blocked">("normal");

  useEffect(() => {
    const detectFlow = async () => {
      try {
        const state = await invoke<any>("detect_flow_state");
        setFlowState(state.flow_state);
      } catch {}
    };

    const interval = setInterval(detectFlow, 10000); // Every 10s
    detectFlow();
    
    return () => clearInterval(interval);
  }, []);

  const animationDuration = {
    deep: 3, // Slow breathing = deep work
    normal: 2,
    blocked: 1, // Fast pulse = blocked
  };

  const colors = {
    deep: "bg-emerald-500",
    normal: "bg-sky-500",
    blocked: "bg-amber-500",
  };

  return (
    <motion.div
      animate={{
        scale: [1, 1.2, 1],
        opacity: [0.7, 1, 0.7],
      }}
      transition={{
        duration: animationDuration[flowState],
        repeat: Infinity,
        ease: "easeInOut",
      }}
      className={`w-3 h-3 rounded-full ${colors[flowState]}`}
      style={{
        boxShadow: `0 0 12px currentColor`,
      }}
    />
  );
}
```

**Backend pour flow detection** :

```rust
#[tauri::command]
pub async fn detect_flow_state(
    state: tauri::State<'_, AppState>,
) -> Result<FlowState, String> {
    let ctx = state.context_aggregator.lock().await.get_last_context()?;
    
    // Heuristics for flow state
    let flow_state = if ctx.idle_seconds < 5.0 && ctx.typing_speed > 60 {
        "deep" // Typing fast, no interruptions
    } else if ctx.idle_seconds > 30.0 {
        "blocked" // Long idle = stuck
    } else {
        "normal"
    };
    
    Ok(FlowState {
        flow_state: flow_state.to_string(),
    })
}
```

---

## 9. 🔥 Learning Streaks

```tsx
// components/StreakBadge.tsx
import { motion } from "framer-motion";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface Streak {
  current_days: number;
  longest_days: number;
  milestones_unlocked: string[];
}

export default function StreakBadge() {
  const [streak, setStreak] = useState<Streak | null>(null);
  const [showCelebration, setShowCelebration] = useState(false);

  useEffect(() => {
    const fetchStreak = async () => {
      try {
        const s = await invoke<Streak>("get_streak_data");
        
        // Check if milestone reached
        if (s.current_days > 0 && s.current_days % 5 === 0) {
          if (!localStorage.getItem(`milestone_${s.current_days}_shown`)) {
            setShowCelebration(true);
            localStorage.setItem(`milestone_${s.current_days}_shown`, "true");
            
            setTimeout(() => setShowCelebration(false), 3000);
          }
        }
        
        setStreak(s);
      } catch {}
    };

    fetchStreak();
    const interval = setInterval(fetchStreak, 60000); // Check every minute
    
    return () => clearInterval(interval);
  }, []);

  if (!streak || streak.current_days === 0) return null;

  return (
    <>
      {/* Badge */}
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        className="fixed top-6 right-6 z-40"
      >
        <div className="glass px-4 py-2 rounded-full flex items-center gap-2 shadow-lg">
          <span className="text-xl">🔥</span>
          <div className="text-sm">
            <span className="font-bold text-gray-900">{streak.current_days}</span>
            <span className="text-gray-600 ml-1">
              {streak.current_days === 1 ? "jour" : "jours"}
            </span>
          </div>
        </div>
      </motion.div>

      {/* Celebration */}
      {showCelebration && (
        <motion.div
          initial={{ opacity: 0, scale: 0.8, y: 50 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.8, y: -50 }}
          className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-50"
        >
          <div className="glass px-8 py-6 rounded-2xl text-center shadow-2xl">
            <motion.div
              animate={{
                rotate: [0, 10, -10, 10, 0],
                scale: [1, 1.2, 1],
              }}
              transition={{ duration: 0.5 }}
              className="text-6xl mb-4"
            >
              🎉
            </motion.div>
            <h3 className="text-2xl font-bold text-gray-900 mb-2">
              {streak.current_days} jours !
            </h3>
            <p className="text-gray-600">
              Continue comme ça, tu es au top 🚀
            </p>
          </div>
          
          {/* Confetti effect */}
          <Confetti />
        </motion.div>
      )}
    </>
  );
}

function Confetti() {
  return (
    <div className="fixed inset-0 pointer-events-none">
      {Array.from({ length: 50 }).map((_, i) => (
        <motion.div
          key={i}
          initial={{
            x: "50vw",
            y: "50vh",
            opacity: 1,
          }}
          animate={{
            x: `${Math.random() * 100}vw`,
            y: `${Math.random() * 100}vh`,
            opacity: 0,
            rotate: Math.random() * 360,
          }}
          transition={{
            duration: 1.5 + Math.random(),
            ease: "easeOut",
          }}
          className="absolute w-3 h-3 rounded-full"
          style={{
            background: ["#10b981", "#3b82f6", "#8b5cf6", "#f59e0b"][Math.floor(Math.random() * 4)],
          }}
        />
      ))}
    </div>
  );
}
```

---

## 10. 📊 Context Cards (Preview on Hover)

```tsx
// Dans FloatingBubble.tsx - ajouter hover card
function ContextPreviewCard() {
  const [context, setContext] = useState<any>(null);

  useEffect(() => {
    const fetchContext = async () => {
      try {
        const ctx = await invoke<any>("get_context_preview");
        setContext(ctx);
      } catch {}
    };
    fetchContext();
  }, []);

  if (!context) return null;

  return (
    <motion.div
      initial={{ opacity: 0, y: 10, scale: 0.95 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: 10, scale: 0.95 }}
      className="absolute bottom-full right-0 mb-3 w-80"
    >
      <div className="glass rounded-2xl p-4 shadow-2xl">
        <h4 className="font-semibold text-gray-900 mb-3 text-sm">
          Contexte actuel
        </h4>

        {/* Current app */}
        <div className="flex items-center gap-3 mb-3 p-3 bg-white/50 rounded-xl">
          <span className="text-2xl">💻</span>
          <div className="flex-1 min-w-0">
            <div className="font-medium text-sm text-gray-900">
              {context.app_name}
            </div>
            <div className="text-xs text-gray-500 truncate">
              {context.window_title}
            </div>
          </div>
        </div>

        {/* Work duration */}
        {context.session_duration_minutes > 0 && (
          <div className="flex items-center justify-between text-sm mb-3">
            <span className="text-gray-600">Session en cours</span>
            <span className="font-medium text-gray-900">
              {context.session_duration_minutes} min
            </span>
          </div>
        )}

        {/* Recent screenshots preview */}
        {context.recent_screenshots > 0 && (
          <div className="flex items-center justify-between text-sm mb-3">
            <span className="text-gray-600">Captures récentes</span>
            <span className="font-medium text-gray-900">
              {context.recent_screenshots}
            </span>
          </div>
        )}

        {/* Next suggestion preview */}
        {context.pending_suggestion && (
          <div className="mt-3 pt-3 border-t border-gray-200">
            <div className="text-xs text-gray-500 mb-1">Suggestion prête</div>
            <div className="text-sm text-gray-700 line-clamp-2">
              {context.pending_suggestion}
            </div>
          </div>
        )}
      </div>
    </motion.div>
  );
}
```

---

## 11. 🎭 Personality Modes

```tsx
// components/PersonalitySelector.tsx
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

type Personality = "default" | "mentor" | "buddy" | "pro";

const PERSONALITIES: Record<Personality, {
  name: string;
  description: string;
  icon: string;
  tone: string;
}> = {
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

export default function PersonalitySelector() {
  const [selected, setSelected] = useState<Personality>("default");
  const [expanded, setExpanded] = useState(false);

  const handleSelect = async (personality: Personality) => {
    setSelected(personality);
    setExpanded(false);
    
    try {
      await invoke("set_personality_mode", { mode: personality });
    } catch (e) {
      console.error("Failed to set personality:", e);
    }
  };

  const current = PERSONALITIES[selected];

  return (
    <div className="relative">
      {/* Current personality badge */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="glass px-3 py-2 rounded-full flex items-center gap-2 text-sm hover:shadow-lg transition-all"
      >
        <span className="text-lg">{current.icon}</span>
        <span className="font-medium">{current.name}</span>
        <span className="text-xs text-gray-500">▼</span>
      </button>

      {/* Selector */}
      {expanded && (
        <motion.div
          initial={{ opacity: 0, y: -10, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          className="absolute top-full left-0 mt-2 w-64 glass rounded-xl p-2 shadow-xl z-50"
        >
          {(Object.entries(PERSONALITIES) as [Personality, typeof PERSONALITIES[Personality]][]).map(([key, p]) => (
            <button
              key={key}
              onClick={() => handleSelect(key)}
              className={`w-full text-left px-3 py-3 rounded-lg flex items-start gap-3 transition-colors ${
                selected === key
                  ? "bg-blue-50 border-l-2 border-blue-500"
                  : "hover:bg-gray-50"
              }`}
            >
              <span className="text-2xl">{p.icon}</span>
              <div className="flex-1">
                <div className="font-medium text-sm text-gray-900">
                  {p.name}
                </div>
                <div className="text-xs text-gray-600 mt-0.5">
                  {p.description}
                </div>
              </div>
              {selected === key && (
                <span className="text-blue-500">✓</span>
              )}
            </button>
          ))}
        </motion.div>
      )}
    </div>
  );
}
```

---

## 12. ⏰ Smart Pause Detection

```rust
// src-tauri/src/detection/smart_pause.rs
pub struct SmartPauseDetector {
    last_activity: Instant,
    pause_reason: Option<PauseReason>,
}

#[derive(Debug, Clone)]
pub enum PauseReason {
    Meeting,
    CoffeeBreak,
    LunchBreak,
    Away,
}

impl SmartPauseDetector {
    pub async fn detect_pause(&mut self, ctx: &Context) -> Option<PauseReason> {
        // Meeting detected (Calendar app + Zoom/Teams)
        if ctx.app.name.contains("Calendar") || 
           ctx.app.name.contains("Zoom") ||
           ctx.app.name.contains("Teams") {
            return Some(PauseReason::Meeting);
        }

        // Long idle + lunch hours
        let now = chrono::Local::now();
        if ctx.idle_seconds > 300.0 && (12..=14).contains(&now.hour()) {
            return Some(PauseReason::LunchBreak);
        }

        // Short breaks (5-15 min idle)
        if (300.0..900.0).contains(&ctx.idle_seconds) {
            return Some(PauseReason::CoffeeBreak);
        }

        // Very long idle (>30min)
        if ctx.idle_seconds > 1800.0 {
            return Some(PauseReason::Away);
        }

        None
    }

    pub fn should_suppress_triggers(&self) -> bool {
        self.pause_reason.is_some()
    }
}
```

**Frontend notification** :

```tsx
// components/WelcomeBackToast.tsx
export function WelcomeBackToast({ pauseReason }: { pauseReason: string }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 20 }}
      className="fixed bottom-24 right-6 glass rounded-2xl p-4 shadow-xl"
    >
      <div className="flex items-center gap-3">
        <span className="text-3xl">👋</span>
        <div>
          <div className="font-semibold text-gray-900">Re-bienvenue !</div>
          <div className="text-sm text-gray-600">
            {pauseReason === "meeting" && "Comment s'est passé le meeting ?"}
            {pauseReason === "lunch" && "Bon appétit était ?"}
            {pauseReason === "coffee" && "Prêt à reprendre ?"}
          </div>
        </div>
      </div>
    </motion.div>
  );
}
```

---

## ✅ RÉCAPITULATIF FEATURES 11/10

| Feature | Impact UX | Difficulté | Temps |
|---------|-----------|------------|-------|
| 🍞 One-Tap Toast | ⭐⭐⭐⭐⭐ | Facile | 1h |
| 🎯 Smart Docking | ⭐⭐⭐⭐⭐ | Moyen | 1.5h |
| 📊 Daily Digest | ⭐⭐⭐⭐ | Moyen | 2h |
| ⚡ Slash Commands | ⭐⭐⭐⭐⭐ | Facile | 2h |
| 👍👎 Feedback | ⭐⭐⭐⭐⭐ | Facile | 1h |
| 🎯 Quick Actions | ⭐⭐⭐⭐⭐ | Moyen | 2h |
| ⚡ Suggestion Pills | ⭐⭐⭐⭐ | Moyen | 1.5h |
| 🎨 Ambient Mode | ⭐⭐⭐⭐ | Facile | 1h |
| 🔥 Streaks | ⭐⭐⭐⭐ | Moyen | 2h |
| 📊 Context Cards | ⭐⭐⭐⭐ | Facile | 1h |
| 🎭 Personalities | ⭐⭐⭐ | Moyen | 1.5h |
| ⏰ Smart Pause | ⭐⭐⭐⭐ | Moyen | 2h |

**Total temps : ~18h pour implémenter TOUTES les features 11/10**

---

## 🚀 PLAN D'IMPLÉMENTATION

### Phase 1 (J6 - Quick Wins) : 6h
1. One-Tap Toast
2. Feedback binaire
3. Ambient Mode LED
4. Context Cards hover

### Phase 2 (J7 - Power Features) : 8h
5. Slash Commands
6. Smart Docking
7. Quick Actions
8. Suggestion Pills

### Phase 3 (J8 - Polish) : 4h
9. Daily Digest
10. Streaks
11. Personalities
12. Smart Pause

---

Recommandations ++ (pour atteindre 10/10)

Central Event Bus (Tauri)
→ remplace tous les setInterval par des emit/listen (tauri::Event) pour :

réduire CPU wakeups

synchroniser LED, context, suggestions en temps réel
(= fluidité et autonomie totale du moteur d’événements)

Memory Streamline (Light Store)
→ ajoute un shadow_store.ts global pour centraliser dismissed, feedback, context, etc.
(évite la dispersion entre composants, simplifie sync backend ↔ UI)

Emotional Feedback Layer 💬
→ après un 👍/👎, Shadow réagit brièvement :

👍 → “Parfait 😌”

👎 → “Merci, je ferai mieux la prochaine fois 🤝”
(petit touch humanisant ultra efficace)

Sound Design subtil (optionnel)
→ 3 sons doux :

toast pop (affichage idée)

dock open (fade synth)

message send (soft click)
(Cluely le fait inconsciemment avec micro-feedback auditif)

“Flow HUD” minimal (bonus futur)
→ mini halo discret autour de la bulle selon ton focus (deep/idle/blocked).
(transforme l’assistant en indicateur d’état cognitif — signature visuelle forte)
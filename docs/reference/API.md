# API.md — Référence complète API Tauri

> **Rôle** : Documentation complète des commandes Tauri et événements
> **Public** : Dev frontend/backend, intégrations
> **Importance** : HAUTE — Référence technique complète

**🎯 Ce fichier répond à : "Comment communiquer entre Frontend et Backend ?"**

---

## 📋 Table des matières

1. [Introduction](#introduction)
2. [Communication Frontend → Backend](#communication-frontend--backend)
3. [Communication Backend → Frontend](#communication-backend--frontend)
4. [Référence commandes (130+)](#référence-commandes)
5. [Référence événements](#référence-événements)
6. [Exemples d'usage](#exemples-dusage)

---

## Introduction

ShadowLearn utilise **Tauri v2** pour la communication entre :
- **Frontend** : React/TypeScript (webview)
- **Backend** : Rust (système natif)

**2 patterns de communication** :

```
┌─────────────────────────────────────────────────────┐
│  Frontend (React/TS)    ←→    Backend (Rust)        │
├─────────────────────────────────────────────────────┤
│  invoke('command', args) ──→  #[tauri::command]     │
│  listen('event', handler) ←── app.emit('event')     │
└─────────────────────────────────────────────────────┘
```

---

## Communication Frontend → Backend

### Pattern : `invoke()`

**Frontend** appelle une commande Rust et attend la réponse.

#### Syntaxe TypeScript

```typescript
import { invoke } from '@tauri-apps/api/core';

// Commande sans paramètres
const result = await invoke<ReturnType>('command_name');

// Commande avec paramètres
const result = await invoke<ReturnType>('command_name', {
  param1: value1,
  param2: value2,
});
```

#### Exemple complet

```typescript
// Frontend : Afficher fenêtre Settings
try {
  await invoke('show_window', { windowLabel: 'settings' });
  console.error('✅ Settings window shown');
} catch (error) {
  console.error('❌ Failed to show window:', error);
}
```

#### Backend : Définir une commande

```rust
#[tauri::command]
async fn show_window(
    app_handle: tauri::AppHandle,
    window_label: String,
) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&window_label) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Window '{}' not found", window_label))
    }
}
```

### Gestion des erreurs

**Toutes les commandes Tauri retournent `Result<T, String>`**

```typescript
// ✅ Bonne pratique
try {
  const stats = await invoke<TriggerStats>('get_trigger_stats');
  console.error('Stats:', stats);
} catch (error) {
  console.error('Error getting stats:', error);
}

// ❌ Mauvaise pratique (pas de gestion erreur)
const stats = await invoke('get_trigger_stats'); // Peut crash si erreur
```

---

## Communication Backend → Frontend

### Pattern : `emit()` + `listen()`

**Backend** émet un événement → **Frontend** écoute avec un listener.

#### Backend : Émettre un événement

```rust
// Dans une commande ou logic backend
app_handle.emit("event-name", payload)?;
```

#### Frontend : Écouter un événement

```typescript
import { listen } from '@tauri-apps/api/event';

useEffect(() => {
  let unlisten: UnlistenFn | null = null;

  const setupListener = async () => {
    unlisten = await listen<PayloadType>('event-name', (event) => {
      console.error('Event received:', event.payload);
      // Traiter l'événement
    });
  };

  setupListener();

  return () => {
    if (unlisten) unlisten();
  };
}, []);
```

#### Exemple complet : HUD state change

**Backend émet** (dans `trigger_loop.rs`) :

```rust
app_handle.emit("hud:state-change", json!({
    "state": "opportunity"
}))?;
```

**Frontend écoute** (dans `hud.tsx`) :

```typescript
useEffect(() => {
  const setupListener = async () => {
    const unlisten = await listen<{ state: HUDState }>('hud:state-change', (event) => {
      setState(event.payload.state);
    });
    return unlisten;
  };

  setupListener().then(setUnlisten);

  return () => {
    if (unlisten) unlisten();
  };
}, []);
```

---

## Référence commandes

### 🪟 Gestion des fenêtres

#### `show_window`

Affiche et focus une fenêtre.

**Paramètres** :
- `window_label: String` - Label de la fenêtre (`"chat"`, `"spotlight"`, `"hud"`, `"settings"`)

**Retour** : `Result<(), String>`

**Exemple** :
```typescript
await invoke('show_window', { windowLabel: 'settings' });
```

---

#### `hide_window`

Cache une fenêtre.

**Paramètres** :
- `window_label: String`

**Retour** : `Result<(), String>`

**Exemple** :
```typescript
await invoke('hide_window', { windowLabel: 'chat' });
```

---

#### `toggle_window`

Toggle visibilité d'une fenêtre (show si cachée, hide si visible).

**Paramètres** :
- `label: String`

**Retour** : `Result<(), String>`

**Exemple** :
```typescript
await invoke('toggle_window', { label: 'spotlight' });
```

---

#### `minimize_window`

Minimise une fenêtre.

**Paramètres** :
- `window_label: String`

**Retour** : `Result<(), String>`

---

#### `is_window_visible`

Vérifie si une fenêtre est visible.

**Paramètres** :
- `window_label: String`

**Retour** : `Result<bool, String>`

**Exemple** :
```typescript
const visible = await invoke<boolean>('is_window_visible', {
  windowLabel: 'hud',
});
console.error('HUD visible:', visible);
```

---

#### `ensure_chat_visible`

Force la fenêtre chat visible et focused (sans always_on_top).

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

**Exemple** :
```typescript
await invoke('ensure_chat_visible');
```

---

### 🎯 Triggers & Détection

#### `start_trigger_loop`

Lance la boucle de détection des opportunités.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

**Note** : Lancé automatiquement dans `.setup()`, rarement appelé manuellement.

---

#### `check_should_trigger`

Vérifie si une opportunité devrait être déclenchée maintenant.

**Paramètres** : Aucun

**Retour** : `Result<TriggerDecision, String>`

**Types** :
```typescript
interface TriggerDecision {
  should_trigger: boolean;
  reason: string;
  app_name?: string;
}
```

---

#### `record_trigger_fired`

Enregistre qu'un trigger a été déclenché pour une app.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

---

#### `record_trigger_action`

Enregistre qu'un utilisateur a agi sur un trigger (cliqué).

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `record_trigger_dismissed`

Alias de `record_bubble_dismissed`. Enregistre qu'un trigger a été ignoré.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `record_trigger_ignored`

Enregistre qu'un trigger a été ignoré pour une app spécifique.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

---

#### `get_trigger_stats`

Récupère les statistiques de triggers.

**Paramètres** : Aucun

**Retour** : `Result<TriggerStats, String>`

**Types** :
```typescript
interface TriggerStats {
  total_fires: number;
  total_actions: number;
  total_dismisses: number;
  action_rate: number;
}
```

---

#### `get_extended_trigger_stats`

Récupère statistiques étendues (avec apps mutées, allowlist, etc.).

**Paramètres** : Aucun

**Retour** : `Result<ExtendedTriggerStats, String>`

**Types** :
```typescript
interface ExtendedTriggerStats {
  basic: TriggerStats;
  allowlist: string[];
  muted_apps: string[];
  ignored_triggers: Map<string, number>;
}
```

---

### ⏸️ Snooze & Muting

#### `snooze_triggers`

Met en pause les triggers pour une durée.

**Paramètres** :
- `duration: String` - `"30min"` | `"2h"` | `"today"`

**Retour** : `Result<(), String>`

**Exemple** :
```typescript
await invoke('snooze_triggers', { duration: '2h' });
```

---

#### `unsnooze_triggers`

Réactive les triggers.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `get_snooze_status`

Vérifie si triggers sont en snooze.

**Paramètres** : Aucun

**Retour** : `Result<Option<u64>, String>`

**Note** : Retourne `null` si pas en snooze, sinon timestamp Unix (ms) de fin snooze.

---

#### `mute_app`

Mute les triggers pour une app spécifique.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

**Exemple** :
```typescript
await invoke('mute_app', { appName: 'Slack' });
```

---

#### `unmute_app`

Unmute une app.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

---

#### `add_to_allowlist`

Ajoute une app à l'allowlist (triggers autorisés).

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

---

#### `remove_from_allowlist`

Retire une app de l'allowlist.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

---

### 🧠 Contexte & Détection

#### `capture_context`

Capture le contexte utilisateur actuel (app active, idle time, etc.).

**Paramètres** : Aucun

**Retour** : `Result<Context, String>`

**Types** :
```typescript
interface Context {
  id: string;
  app: {
    name: string;
    window_title?: string;
  };
  clipboard?: string;
  idle_seconds: number;
  timestamp: number;
}
```

---

#### `get_idle_state`

Récupère l'état idle de l'utilisateur.

**Paramètres** : Aucun

**Retour** : `Result<IdleState, String>`

**Types** :
```typescript
interface IdleState {
  is_idle: boolean;
  seconds_idle: number;
  last_activity: number; // timestamp
}
```

---

#### `reset_user_activity`

Reset le timer d'inactivité.

**Paramètres** :
- `activity_type: String` - `"keyboard"` | `"mouse"` | `"scroll"`

**Retour** : `Result<(), String>`

---

### 💾 Persistance & Mémoire

#### `create_conversation`

Crée une nouvelle conversation.

**Paramètres** :
- `title: String`
- `app_context: Option<String>`

**Retour** : `Result<Conversation, String>`

**Types** :
```typescript
interface Conversation {
  id: string;
  title: string;
  app_context?: string;
  created_at: number;
}
```

---

#### `save_message`

Sauvegarde un message dans une conversation.

**Paramètres** :
- `conversation_id: String`
- `role: String` - `"user"` | `"assistant"` | `"system"`
- `content: String`
- `metadata: Option<String>`

**Retour** : `Result<Message, String>`

**Types** :
```typescript
interface Message {
  id: string;
  conversation_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata?: string;
  timestamp: number;
}
```

---

#### `get_recent_conversations`

Récupère les conversations récentes.

**Paramètres** :
- `limit: i32`

**Retour** : `Result<Conversation[], String>`

---

#### `get_conversation_messages`

Récupère les messages d'une conversation.

**Paramètres** :
- `conversation_id: String`

**Retour** : `Result<Message[], String>`

---

#### `get_persistence_stats`

Récupère statistiques de persistance.

**Paramètres** : Aucun

**Retour** : `Result<PersistenceStats, String>`

**Types** :
```typescript
interface PersistenceStats {
  total_conversations: number;
  total_messages: number;
  total_contexts: number;
  db_size_bytes: number;
}
```

---

#### `save_context`

Sauvegarde un contexte capturé.

**Paramètres** :
- `context: CapturedContext`

**Retour** : `Result<(), String>`

---

#### `get_recent_contexts_for_app`

Récupère les contextes récents pour une app.

**Paramètres** :
- `app_name: String`
- `limit: i32`

**Retour** : `Result<CapturedContext[], String>`

---

#### `export_data`

Exporte toutes les données en JSON.

**Paramètres** :
- `file_path: String`

**Retour** : `Result<(), String>`

---

### 🧪 Machine Learning & Personnalisation

#### `record_ml_event`

Enregistre un événement ML pour personnalisation.

**Paramètres** :
- `event_type: String` - `"trigger_fired"` | `"trigger_accepted"` | `"trigger_ignored"` | `"trigger_dismissed"` | `"app_muted"` | `"clipboard_changed"` | `"idle_detected"`
- `app_name: String`
- `context: Option<String>`
- `user_response: Option<String>` - `"accepted"` | `"ignored"` | `"dismissed"` | `"snoozed"`

**Retour** : `Result<(), String>`

---

#### `get_usage_patterns`

Récupère les patterns d'usage appris.

**Paramètres** : Aucun

**Retour** : `Result<UsagePatterns, String>`

**Types** :
```typescript
interface UsagePatterns {
  most_active_hours: number[];
  most_used_apps: string[];
  avg_idle_time: number;
  trigger_acceptance_rate: number;
}
```

---

#### `get_smart_suggestions`

Génère suggestions intelligentes basées sur ML.

**Paramètres** : Aucun

**Retour** : `Result<SmartSuggestions, String>`

**Types** :
```typescript
interface SmartSuggestions {
  recommended_apps: string[];
  apps_to_mute: string[];
  recommended_thresholds: {
    idle_threshold: number;
  };
}
```

---

#### `apply_smart_suggestions`

Applique les suggestions ML (allowlist, mutes).

**Paramètres** :
- `suggestions: SmartSuggestions`

**Retour** : `Result<(), String>`

---

#### `save_ml_patterns`

Sauvegarde patterns ML sur disque.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `load_ml_patterns`

Charge patterns ML depuis disque.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

### 🎓 Learning System (J19)

#### `record_user_feedback`

Enregistre feedback utilisateur sur suggestion.

**Paramètres** :
- `suggestion_id: String`
- `helpful: bool`
- `used: bool`
- `reverted: bool`
- `time_to_flow_ms: Option<i64>`

**Retour** : `Result<f32, String>` (reward score)

---

#### `get_user_trust_level`

Récupère niveau de confiance utilisateur.

**Paramètres** : Aucun

**Retour** : `Result<TrustLevel, String>`

**Types** :
```typescript
interface TrustLevel {
  level: 'low' | 'medium' | 'high';
  score: number; // 0.0 - 1.0
  suggestions_count: number;
  acceptance_rate: number;
}
```

---

#### `get_trust_recommendations`

Récupère recommandations basées sur trust.

**Paramètres** : Aucun

**Retour** : `Result<TrustRecommendations, String>`

---

#### `reset_user_trust`

Reset le trust score.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

### 🔍 Screenshots

#### `capture_screenshot`

Capture screenshot de l'écran actif.

**Paramètres** : Aucun

**Retour** : `Result<ScreenshotResult, String>`

**Types** :
```typescript
interface ScreenshotResult {
  data: string; // base64 JPEG
  path: string;
  size_bytes: number;
  timestamp: number;
}
```

---

#### `check_screenshot_permission`

Vérifie permissions screenshot (macOS).

**Paramètres** : Aucun

**Retour** : `Result<bool, String>`

---

#### `request_screenshot_permission`

Demande permissions screenshot (ouvre System Settings).

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

### 🔐 Permissions & Sécurité (J1-6)

#### `check_permissions`

Vérifie toutes les permissions système.

**Paramètres** : Aucun

**Retour** : `Result<PermissionsStatus, String>`

**Types** :
```typescript
interface PermissionsStatus {
  screen_recording: boolean;
  accessibility: boolean;
}
```

---

#### `request_screen_recording_permission`

Demande permission screen recording (macOS).

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `request_accessibility_permission`

Demande permission accessibility (macOS).

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `check_keychain_status`

Vérifie statut du keychain (crypto).

**Paramètres** : Aucun

**Retour** : `Result<KeychainStatus, String>`

---

### ⚙️ Configuration (J5)

#### `get_config`

Récupère configuration complète.

**Paramètres** : Aucun

**Retour** : `Result<Config, String>`

---

#### `update_config`

Met à jour configuration.

**Paramètres** :
- `config: Config`

**Retour** : `Result<(), String>`

---

#### `get_config_path`

Récupère chemin du fichier config.

**Paramètres** : Aucun

**Retour** : `Result<String, String>`

---

### 🖥️ Screen Monitor

#### `start_screen_monitor`

Lance le monitoring d'écran.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `stop_screen_monitor`

Arrête le monitoring d'écran.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `get_monitor_status`

Récupère statut du monitor.

**Paramètres** : Aucun

**Retour** : `Result<MonitorStatus, String>`

---

### ⌨️ Keyboard Shortcuts

#### `toggle_spotlight`

Toggle la fenêtre Spotlight.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

**Note** : Également accessible via `Cmd+Shift+Y`.

---

#### `get_shortcuts_config`

Récupère config shortcuts.

**Paramètres** : Aucun

**Retour** : `Result<ShortcutsConfig, String>`

---

#### `list_shortcuts`

Liste tous les shortcuts définis.

**Paramètres** : Aucun

**Retour** : `Result<ShortcutDef[], String>`

---

#### `trigger_shortcut_action`

Déclenche une action de shortcut manuellement.

**Paramètres** :
- `action: String`

**Retour** : `Result<(), String>`

---

### 🔒 Privacy Zones

#### `get_privacy_zones_config`

Récupère config privacy zones.

**Paramètres** : Aucun

**Retour** : `Result<PrivacyZonesConfig, String>`

---

#### `add_privacy_zone`

Ajoute une zone de confidentialité (app protégée).

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

---

#### `remove_privacy_zone`

Retire une privacy zone.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<(), String>`

---

#### `set_privacy_zones_enabled`

Active/désactive privacy zones.

**Paramètres** :
- `enabled: bool`

**Retour** : `Result<(), String>`

---

#### `is_app_protected`

Vérifie si une app est protégée.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<bool, String>`

---

### 🎯 Artefact Validation (J20)

#### `validate_artefact`

Valide un artefact avant apprentissage.

**Paramètres** :
- `artefact_path: String`
- `artefact_type: String` - `"blend"` | `"midi"` | `"python"` | `"shader"` | `"json"` | `"text"`

**Retour** : `Result<bool, String>`

---

#### `get_validation_stats`

Récupère stats de validation.

**Paramètres** : Aucun

**Retour** : `Result<ValidationStats, String>`

---

#### `get_validator_status`

Récupère statut du validateur.

**Paramètres** : Aucun

**Retour** : `Result<ValidatorStatus, String>`

---

#### `clear_validation_cache`

Nettoie le cache de validation.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

### 🎨 Artefact Generation (J23)

#### `generate_artifact`

Génère un artefact via ML.

**Paramètres** :
- `domain: String`
- `intent: String`
- `trust_score: f32`
- `idle_time: f32`
- `cluster_id: String`
- `artefact_type: String`

**Retour** : `Result<GeneratedArtifact, String>`

---

#### `get_artifact_stats`

Récupère stats de génération d'artefacts.

**Paramètres** : Aucun

**Retour** : `Result<ArtefactStats, String>`

---

### 💬 Chat LLM (J3)

#### `chat_with_ai`

Envoie message au LLM et récupère réponse.

**Paramètres** :
- `message: String`
- `context: Option<String>`

**Retour** : `Result<String, String>`

---

#### `check_llm_health`

Vérifie santé du LLM.

**Paramètres** : Aucun

**Retour** : `Result<LLMHealthStatus, String>`

---

#### `get_llm_stats`

Récupère stats LLM (tokens, latency).

**Paramètres** : Aucun

**Retour** : `Result<LLMStats, String>`

---

### 🎮 State Machine (J2)

#### `get_trigger_state`

Récupère état actuel de la state machine.

**Paramètres** : Aucun

**Retour** : `Result<String, String>` (state name)

---

#### `get_state_explanation`

Récupère explication de l'état actuel.

**Paramètres** : Aucun

**Retour** : `Result<String, String>`

---

#### `get_state_history`

Récupère historique des transitions d'état.

**Paramètres** : Aucun

**Retour** : `Result<StateTransition[], String>`

---

### 🚀 Feature Flags (J21.5)

#### `get_feature_flags`

Récupère tous les feature flags.

**Paramètres** : Aucun

**Retour** : `FeaturesState`

**Types** :
```typescript
interface FeaturesState {
  idle_detection: boolean;
  screenshot: boolean;
  smart_triggers: boolean;
  telemetry: boolean;
  use_intent_gate: boolean;
}
```

---

#### `enable_feature`

Active un feature flag.

**Paramètres** :
- `feature: String` - `"idle_detection"` | `"screenshot"` | `"smart_triggers"` | `"telemetry"` | `"use_intent_gate"`

**Retour** : `Result<(), String>`

---

#### `disable_feature`

Désactive un feature flag.

**Paramètres** :
- `feature: String`

**Retour** : `Result<(), String>`

---

### 📊 Telemetry & Health

#### `get_health_status`

Vérifie santé du système.

**Paramètres** : Aucun

**Retour** : `Result<HealthStatus, String>`

**Types** :
```typescript
interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  checks: {
    database: boolean;
    triggers: boolean;
    context: boolean;
  };
}
```

---

#### `get_telemetry_stats`

Récupère statistiques télémétrie.

**Paramètres** : Aucun

**Retour** : `Result<TelemetryStats, String>`

---

#### `get_recovery_stats`

Récupère stats de récupération (restarts).

**Paramètres** : Aucun

**Retour** : `Result<RecoveryStats, String>`

---

#### `record_telemetry_event`

Enregistre un événement télémétrie.

**Paramètres** :
- `event_type: String`
- `duration_ms: Option<u64>`

**Retour** : `Result<(), String>`

---

### 🌟 Clueless Features

#### `record_opportunity_response`

Enregistre réponse utilisateur à opportunité.

**Paramètres** :
- `opportunity_id: String`
- `response: String` - `"accepted"` | `"dismissed"` | `"snoozed"`

**Retour** : `Result<(), String>`

---

#### `record_message_feedback`

Enregistre feedback sur message.

**Paramètres** :
- `message_id: String`
- `helpful: bool`

**Retour** : `Result<(), String>`

---

#### `detect_flow_state`

Détecte si utilisateur est en flow state.

**Paramètres** : Aucun

**Retour** : `Result<FlowState, String>`

**Types** :
```typescript
interface FlowState {
  in_flow: bool;
  confidence: number; // 0.0 - 1.0
  duration_minutes: number;
}
```

---

#### `get_context_preview`

Récupère preview du contexte actuel.

**Paramètres** : Aucun

**Retour** : `Result<ContextPreview, String>`

---

#### `get_daily_digest`

Récupère digest quotidien.

**Paramètres** : Aucun

**Retour** : `Result<DailyDigest, String>`

---

#### `record_suggestion_shown`

Enregistre qu'une suggestion a été affichée.

**Paramètres** :
- `suggestion_id: String`

**Retour** : `Result<(), String>`

---

#### `record_suggestion_accepted`

Enregistre qu'une suggestion a été acceptée.

**Paramètres** :
- `suggestion_id: String`

**Retour** : `Result<(), String>`

---

#### `get_micro_suggestions`

Récupère micro suggestions (pills).

**Paramètres** : Aucun

**Retour** : `Result<Pill[], String>`

---

#### `dismiss_pill`

Ferme une pill.

**Paramètres** :
- `pill_id: String`

**Retour** : `Result<(), String>`

---

#### `execute_slash_command`

Exécute une slash command.

**Paramètres** :
- `command: String`

**Retour** : `Result<String, String>`

---

### 🔁 Pattern Recognition (Phase 2.1)

#### `record_user_action`

Enregistre action utilisateur pour apprentissage patterns.

**Paramètres** :
- `action: String`
- `context: String`
- `tags: Vec<String>`

**Retour** : `Result<(), String>`

---

#### `get_next_action_prediction`

Prédit prochaine action probable.

**Paramètres** : Aucun

**Retour** : `Result<ActionPrediction, String>`

---

#### `get_learned_patterns`

Récupère patterns appris.

**Paramètres** : Aucun

**Retour** : `Result<Pattern[], String>`

---

#### `get_patterns_by_tag`

Récupère patterns par tag.

**Paramètres** :
- `tag: String`

**Retour** : `Result<Pattern[], String>`

---

#### `get_all_repetitive_tasks`

Récupère toutes les tâches répétitives détectées.

**Paramètres** : Aucun

**Retour** : `Result<RepetitiveTask[], String>`

---

#### `get_high_priority_repetitive_tasks`

Récupère tâches répétitives haute priorité.

**Paramètres** : Aucun

**Retour** : `Result<RepetitiveTask[], String>`

---

#### `get_pattern_system_stats`

Récupère stats du système de patterns.

**Paramètres** : Aucun

**Retour** : `Result<PatternSystemStats, String>`

---

#### `save_patterns_to_disk`

Sauvegarde patterns sur disque.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `clear_pattern_storage`

Nettoie le stockage patterns.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

### 🎯 Phase 3: Streaks, Personalities, Pause

#### `get_streak`

Récupère streak actuel.

**Paramètres** : Aucun

**Retour** : `Result<Streak, String>`

---

#### `record_activity`

Enregistre activité pour streak.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `get_personality`

Récupère personnalité active.

**Paramètres** : Aucun

**Retour** : `Result<Personality, String>`

---

#### `set_personality`

Change la personnalité.

**Paramètres** :
- `personality: String`

**Retour** : `Result<(), String>`

---

#### `get_pause_state`

Récupère état pause.

**Paramètres** : Aucun

**Retour** : `Result<PauseState, String>`

---

#### `set_pause_state`

Active/désactive pause mode.

**Paramètres** :
- `enabled: bool`

**Retour** : `Result<(), String>`

---

### 📊 Phase 3: Productivity Dashboard

#### `get_productivity_metrics`

Récupère métriques de productivité.

**Paramètres** : Aucun

**Retour** : `Result<ProductivityMetrics, String>`

---

#### `record_productivity_event`

Enregistre événement productivité.

**Paramètres** :
- `event: ProductivityEvent`

**Retour** : `Result<(), String>`

---

#### `record_flow_session_event`

Enregistre session flow.

**Paramètres** :
- `session: FlowSession`

**Retour** : `Result<(), String>`

---

### 🔌 Phase 4: Plugin System

#### `get_all_plugins`

Liste tous les plugins.

**Paramètres** : Aucun

**Retour** : `Result<Plugin[], String>`

---

#### `get_plugin_info`

Récupère info d'un plugin.

**Paramètres** :
- `plugin_id: String`

**Retour** : `Result<PluginInfo, String>`

---

#### `enable_plugin`

Active un plugin.

**Paramètres** :
- `plugin_id: String`

**Retour** : `Result<(), String>`

---

#### `disable_plugin`

Désactive un plugin.

**Paramètres** :
- `plugin_id: String`

**Retour** : `Result<(), String>`

---

#### `uninstall_plugin`

Désinstalle un plugin.

**Paramètres** :
- `plugin_id: String`

**Retour** : `Result<(), String>`

---

#### `reload_plugins`

Recharge tous les plugins.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `get_plugin_stats`

Récupère stats des plugins.

**Paramètres** : Aucun

**Retour** : `Result<PluginStats, String>`

---

#### `execute_plugin_hook`

Exécute un hook de plugin.

**Paramètres** :
- `plugin_id: String`
- `hook_name: String`
- `args: String`

**Retour** : `Result<String, String>`

---

### 🎬 Killer Feature: Shadow Replay

#### `get_replay_events`

Récupère événements replay.

**Paramètres** :
- `start_timestamp: u64`
- `end_timestamp: u64`

**Retour** : `Result<ReplayEvent[], String>`

---

#### `get_replay_sessions`

Récupère sessions replay.

**Paramètres** : Aucun

**Retour** : `Result<ReplaySession[], String>`

---

#### `get_replay_stats`

Récupère stats replay.

**Paramètres** : Aucun

**Retour** : `Result<ReplayStats, String>`

---

#### `start_replay_playback`

Lance playback d'un replay.

**Paramètres** :
- `session_id: String`

**Retour** : `Result<(), String>`

---

#### `stop_replay_playback`

Arrête playback.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `set_replay_speed`

Change vitesse playback.

**Paramètres** :
- `speed: f32` (ex: `1.0`, `2.0`, `0.5`)

**Retour** : `Result<(), String>`

---

#### `get_next_replay_event`

Récupère prochain événement replay.

**Paramètres** : Aucun

**Retour** : `Result<ReplayEvent, String>`

---

#### `get_playback_state`

Récupère état playback.

**Paramètres** : Aucun

**Retour** : `Result<PlaybackState, String>`

---

#### `seek_replay_to`

Seek à un timestamp dans replay.

**Paramètres** :
- `timestamp: u64`

**Retour** : `Result<(), String>`

---

#### `record_replay_suggestion`

Enregistre suggestion dans replay.

**Paramètres** :
- `suggestion: String`

**Retour** : `Result<(), String>`

---

#### `record_replay_flow_session`

Enregistre flow session dans replay.

**Paramètres** :
- `session: FlowSession`

**Retour** : `Result<(), String>`

---

### 🎯 Killer Feature: Focus Mode

#### `get_focus_state`

Récupère état focus.

**Paramètres** : Aucun

**Retour** : `Result<FocusState, String>`

**Types** :
```typescript
interface FocusState {
  active: bool;
  mode: 'deep' | 'flow' | 'light';
  duration_minutes: number;
  blocks_count: number;
}
```

---

#### `get_focus_stats`

Récupère stats focus.

**Paramètres** : Aucun

**Retour** : `Result<FocusStats, String>`

---

#### `get_focus_config`

Récupère config focus.

**Paramètres** : Aucun

**Retour** : `Result<FocusConfig, String>`

---

#### `update_focus_config`

Met à jour config focus.

**Paramètres** :
- `config: FocusConfig`

**Retour** : `Result<(), String>`

---

#### `detect_focus_mode`

Détecte si utilisateur est en focus.

**Paramètres** : Aucun

**Retour** : `Result<bool, String>`

---

#### `should_block_notification`

Vérifie si notification doit être bloquée.

**Paramètres** :
- `app_name: String`

**Retour** : `Result<bool, String>`

---

#### `should_block_trigger`

Vérifie si trigger doit être bloqué.

**Paramètres** : Aucun

**Retour** : `Result<bool, String>`

---

#### `end_focus_session`

Termine session focus.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `get_recent_focus_sessions`

Récupère sessions focus récentes.

**Paramètres** :
- `limit: i32`

**Retour** : `Result<FocusSession[], String>`

---

### 📚 Killer Feature: Learn by Doing

#### `start_workflow_recording`

Lance enregistrement workflow.

**Paramètres** :
- `title: String`

**Retour** : `Result<String, String>` (workflow_id)

---

#### `stop_workflow_recording`

Arrête enregistrement.

**Paramètres** : Aucun

**Retour** : `Result<(), String>`

---

#### `add_workflow_comment`

Ajoute commentaire au workflow actif.

**Paramètres** :
- `comment: String`

**Retour** : `Result<(), String>`

---

#### `generate_workflow_tutorial`

Génère tutoriel depuis workflow.

**Paramètres** :
- `workflow_id: String`

**Retour** : `Result<Tutorial, String>`

---

#### `get_recording_state`

Récupère état enregistrement.

**Paramètres** : Aucun

**Retour** : `Result<RecordingState, String>`

---

#### `get_all_workflows`

Récupère tous les workflows.

**Paramètres** : Aucun

**Retour** : `Result<Workflow[], String>`

---

#### `get_all_tutorials`

Récupère tous les tutoriels.

**Paramètres** : Aucun

**Retour** : `Result<Tutorial[], String>`

---

#### `export_tutorial_as_markdown`

Exporte tutoriel en Markdown.

**Paramètres** :
- `tutorial_id: String`

**Retour** : `Result<String, String>`

---

### 📡 Utilities

#### `broadcast_event`

Broadcast événement custom à toutes fenêtres.

**Paramètres** :
- `event: String`
- `payload: String`

**Retour** : `Result<(), String>`

**Exemple** :
```typescript
await invoke('broadcast_event', {
  event: 'custom:notification',
  payload: JSON.stringify({ message: 'Hello' }),
});
```

---

## Référence événements

### Backend → Frontend Events

**Liste complète des événements émis par le backend** :

| Événement | Payload | Description |
|-----------|---------|-------------|
| `spotlight:show` | `{}` | Spotlight doit s'afficher |
| `spotlight:hide` | `{}` | Spotlight doit se cacher |
| `hud:state-change` | `{ state: HUDState }` | État HUD changé (`"normal"` \| `"opportunity"` \| `"blocked"`) |
| `hud:click` | `{}` | HUD a été cliqué |
| `shortcut-triggered` | `{ action: string }` | Shortcut déclenché |
| `screen-change` | `{ change: ScreenChange }` | Changement écran détecté |
| `shadow:flow_state` | `{ in_flow: bool }` | Flow state détecté |
| `shadow:context_update` | `{ context: Context }` | Contexte mis à jour |
| `shadow:opportunity` | `{ opportunity: Opportunity }` | Opportunité détectée |
| `shadow:micro_suggestion` | `{ pill: Pill }` | Micro suggestion disponible |
| `shadow:sound:play` | `{ sound: string }` | Jouer un son |
| `trigger_fired` | `{ app_name: string }` | Trigger déclenché |

---

### Frontend Event Listeners

**Pattern d'écoute dans React** :

```typescript
import { listen, UnlistenFn } from '@tauri-apps/api/event';

useEffect(() => {
  let unlisten: UnlistenFn | null = null;

  const setupListener = async () => {
    unlisten = await listen<PayloadType>('event-name', (event) => {
      console.error('Received:', event.payload);
      // Traiter événement
    });
  };

  setupListener();

  return () => {
    if (unlisten) unlisten();
  };
}, []);
```

---

## Exemples d'usage

### Exemple 1 : Afficher fenêtre Settings

```typescript
// Frontend
import { invoke } from '@tauri-apps/api/core';

async function showSettings() {
  try {
    await invoke('show_window', { windowLabel: 'settings' });
    console.error('✅ Settings shown');
  } catch (error) {
    console.error('❌ Error showing settings:', error);
  }
}
```

---

### Exemple 2 : Toggle Spotlight

```typescript
import { invoke } from '@tauri-apps/api/core';

async function toggleSpotlight() {
  await invoke('toggle_spotlight');
}
```

---

### Exemple 3 : Écouter changements d'état HUD

```typescript
import { listen } from '@tauri-apps/api/event';
import { useState, useEffect } from 'react';

function useHUDState() {
  const [state, setState] = useState<'normal' | 'opportunity' | 'blocked'>('normal');

  useEffect(() => {
    let unlisten: any = null;

    const setup = async () => {
      unlisten = await listen<{ state: string }>('hud:state-change', (event) => {
        setState(event.payload.state as any);
      });
    };

    setup();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return state;
}
```

---

### Exemple 4 : Capturer screenshot et sauvegarder

```typescript
import { invoke } from '@tauri-apps/api/core';

async function takeScreenshot() {
  try {
    const result = await invoke<ScreenshotResult>('capture_screenshot');
    console.error('Screenshot captured:', result.path);
    console.error('Size:', result.size_bytes, 'bytes');

    // Afficher screenshot
    const img = document.getElementById('screenshot') as HTMLImageElement;
    img.src = `data:image/jpeg;base64,${result.data}`;
  } catch (error) {
    console.error('Screenshot failed:', error);
  }
}
```

---

### Exemple 5 : Enregistrer feedback ML

```typescript
import { invoke } from '@tauri-apps/api/core';

async function recordUserAction(accepted: boolean) {
  const eventType = accepted ? 'trigger_accepted' : 'trigger_ignored';
  const userResponse = accepted ? 'accepted' : 'ignored';

  await invoke('record_ml_event', {
    eventType,
    appName: 'VS Code',
    context: 'Coding TypeScript',
    userResponse,
  });
}
```

---

### Exemple 6 : Récupérer patterns ML et appliquer suggestions

```typescript
import { invoke } from '@tauri-apps/api/core';

async function applyMLSuggestions() {
  // Récupérer suggestions
  const suggestions = await invoke<SmartSuggestions>('get_smart_suggestions');

  console.error('Recommended apps:', suggestions.recommended_apps);
  console.error('Apps to mute:', suggestions.apps_to_mute);

  // Appliquer automatiquement
  await invoke('apply_smart_suggestions', { suggestions });

  console.error('✅ Suggestions applied');
}
```

---

### Exemple 7 : Créer conversation et sauvegarder messages

```typescript
import { invoke } from '@tauri-apps/api/core';

async function createNewChat() {
  // Créer conversation
  const conversation = await invoke<Conversation>('create_conversation', {
    title: 'How to use TypeScript generics',
    appContext: 'VS Code',
  });

  console.error('Conversation created:', conversation.id);

  // Sauvegarder message utilisateur
  await invoke('save_message', {
    conversationId: conversation.id,
    role: 'user',
    content: 'How do I use generics in TypeScript?',
    metadata: null,
  });

  // Sauvegarder réponse assistant
  await invoke('save_message', {
    conversationId: conversation.id,
    role: 'assistant',
    content: 'Generics in TypeScript allow you to...',
    metadata: null,
  });
}
```

---

### Exemple 8 : Workflow Learn by Doing

```typescript
import { invoke } from '@tauri-apps/api/core';

async function recordWorkflow() {
  // Lancer enregistrement
  const workflowId = await invoke<string>('start_workflow_recording', {
    title: 'How to create a React component',
  });

  console.error('Recording workflow:', workflowId);

  // Ajouter commentaire
  await invoke('add_workflow_comment', {
    comment: 'Now I will create the component file',
  });

  // ... l'utilisateur fait des actions ...

  // Arrêter enregistrement
  await invoke('stop_workflow_recording');

  // Générer tutoriel
  const tutorial = await invoke<Tutorial>('generate_workflow_tutorial', {
    workflowId,
  });

  console.error('Tutorial generated:', tutorial.title);

  // Exporter en Markdown
  const markdown = await invoke<string>('export_tutorial_as_markdown', {
    tutorialId: tutorial.id,
  });

  console.error('Markdown:', markdown);
}
```

---

## 🎯 Bonnes pratiques

### ✅ Toujours gérer les erreurs

```typescript
// ✅ BON
try {
  await invoke('command');
} catch (error) {
  console.error('Error:', error);
}

// ❌ MAUVAIS
await invoke('command'); // Peut crash
```

---

### ✅ Toujours unlisten dans cleanup

```typescript
// ✅ BON
useEffect(() => {
  let unlisten: any = null;

  const setup = async () => {
    unlisten = await listen('event', handler);
  };

  setup();

  return () => {
    if (unlisten) unlisten(); // IMPORTANT
  };
}, []);

// ❌ MAUVAIS
useEffect(() => {
  listen('event', handler); // Leak mémoire
}, []);
```

---

### ✅ Typer les retours invoke

```typescript
// ✅ BON
const result = await invoke<TriggerStats>('get_trigger_stats');
console.error(result.total_fires); // TypeScript sait le type

// ❌ MAUVAIS
const result = await invoke('get_trigger_stats');
console.error(result.total_fires); // Type `any`
```

---

### ✅ Ne jamais bloquer l'UI

```typescript
// ✅ BON
const handleClick = async () => {
  setLoading(true);
  try {
    await invoke('long_running_command');
  } finally {
    setLoading(false);
  }
};

// ❌ MAUVAIS
const handleClick = () => {
  invoke('long_running_command'); // Bloque UI
};
```

---

## 🔗 Ressources

- [Tauri v2 Docs - Commands](https://v2.tauri.app/develop/calling-rust/)
- [Tauri v2 Docs - Events](https://v2.tauri.app/develop/inter-process-communication/)
- [SYSTEM_OVERVIEW.md](../../SYSTEM_OVERVIEW.md) - Architecture complète
- [MAINTENANCE.md](../../MAINTENANCE.md) - Quick wins & modifications

---

**💡 Ce fichier est la référence technique complète pour toutes les interactions Frontend ↔ Backend.**

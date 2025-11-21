# SYSTEM_OVERVIEW.md — Architecture & Workflows

> **Rôle** : Vue complète de l'architecture technique et des workflows fonctionnels
> **Public** : Dev qui veut comprendre comment tout fonctionne ensemble
> **Importance** : CRITIQUE — Lire après CONTEXT.md

---

## 📋 Table des matières

1. [Architecture globale](#1-architecture-globale)
2. [Frontend (React/TypeScript)](#2-frontend-reacttypescript)
3. [Backend (Rust/Tauri)](#3-backend-rusttauri)
4. [Communication & Data Flow](#4-communication--data-flow)
5. [Workflows utilisateur](#5-workflows-utilisateur)
6. [Structure détaillée des fichiers](#6-structure-détaillée-des-fichiers)

---

## 1. Architecture globale

### Vue d'ensemble (diagramme)

```
┌─────────────────────────────────────────────────────────────────┐
│                    User (macOS/Windows/Linux)                   │
│                  ↓ Interactions (clicks, shortcuts)              │
└──────────────────────────────┬──────────────────────────────────┘
                               │
        ┌──────────────────────▼───────────────────────┐
        │         Tauri Application Shell              │
        │                                              │
        │  ┌─────────────────────────────────────┐   │
        │  │    Window Manager (4 fenêtres)      │   │
        │  │  ┌──────┬────────┬──────┬─────────┐ │   │
        │  │  │ Main │  Chat  │ HUD  │Spotlight│ │   │
        │  │  │(60px)│(400x600)│(60px)│(600x500)│ │   │
        │  │  └──────┴────────┴──────┴─────────┘ │   │
        │  └─────────────────────────────────────┘   │
        │                                              │
        │  ┌─────────────────────────────────────┐   │
        │  │     Global Shortcuts Manager         │   │
        │  │   (Cmd+Shift+Y, ESC, etc.)          │   │
        │  └─────────────────────────────────────┘   │
        │                                              │
        │  ┌─────────────────────────────────────┐   │
        │  │         Event Bus (IPC)              │   │
        │  │  Frontend ←──→ Backend events        │   │
        │  └─────────────────────────────────────┘   │
        └───────────────┬────────────────────────────┘
                        │
        ┌───────────────▼───────────────────────┐
        │      Frontend (React Webviews)        │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  Components (UI)                 │ │
        │  │  - HUD (ambient LED)             │ │
        │  │  - Spotlight (glassmorphism)     │ │
        │  │  - Chat (messages, input)        │ │
        │  │  - Settings (config UI)          │ │
        │  └──────────────────────────────────┘ │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  Hooks & Contexts                │ │
        │  │  - useTheme (personnalités)      │ │
        │  │  - useHover (optimisations)      │ │
        │  │  - useTrigger                    │ │
        │  └──────────────────────────────────┘ │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  Utils (helpers.ts, etc.)        │ │
        │  └──────────────────────────────────┘ │
        └────────────┬───────────────────────────┘
                     │ invoke() / emit()
        ┌────────────▼───────────────────────────┐
        │     Backend (Rust/Tokio)               │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  Tauri Commands (IPC handlers)   │ │
        │  │  - show_window()                 │ │
        │  │  - toggle_spotlight()            │ │
        │  │  - get_extended_trigger_stats()  │ │
        │  └──────────────────────────────────┘ │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  Shortcuts Manager                │ │
        │  │  - Register global shortcuts     │ │
        │  │  - Handle Cmd+Shift+Y            │ │
        │  │  - Position Spotlight            │ │
        │  └──────────────────────────────────┘ │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  Triggers System                  │ │
        │  │  - Detect opportunities           │ │
        │  │  - Clipboard monitoring           │ │
        │  │  - Pattern detection              │ │
        │  └──────────────────────────────────┘ │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  Storage Layer (SQLite)           │ │
        │  │  - Settings persistence           │ │
        │  │  - Opportunities history          │ │
        │  │  - User preferences               │ │
        │  └──────────────────────────────────┘ │
        │                                        │
        │  ┌──────────────────────────────────┐ │
        │  │  macOS FFI (cocoa)                │ │
        │  │  - NSWindow config                │ │
        │  │  - Fullscreen support             │ │
        │  │  - Window behaviors               │ │
        │  └──────────────────────────────────┘ │
        └────────────────────────────────────────┘
```

### Blocs principaux

| Bloc | Techno | Rôle |
|------|--------|------|
| **Tauri Shell** | Rust | Gestion fenêtres, IPC, OS integration |
| **Frontend** | React 19 + TS | UI components, interactions |
| **Backend** | Rust + Tokio | Business logic, DB, triggers |
| **Storage** | SQLite | Persistence locale |
| **macOS FFI** | cocoa crate | Intégration native macOS |

---

## 2. Frontend (React/TypeScript)

### Structure React

```
src/
├── *.tsx (entry points)
│   ├── main.tsx        # Dashboard (fenêtre main)
│   ├── chat.tsx        # Chat window
│   ├── hud.tsx         # HUD ambient LED
│   ├── spotlight.tsx   # Spotlight popup
│   └── settings.tsx    # Settings window
│
├── components/         # Composants UI réutilisables
│   ├── HeaderDraggable.tsx
│   ├── OpportunityToast.tsx
│   ├── QuickActions.tsx
│   ├── SlashCommands.tsx
│   ├── SmartPills.tsx
│   ├── PersonalitySelector.tsx
│   └── ...
│
├── hooks/              # Custom React hooks
│   ├── useHover.ts          # Gestion hover optimisée
│   ├── useTrigger.ts        # Integration triggers backend
│   ├── useShortcuts.ts      # Shortcuts management
│   └── ...
│
├── contexts/           # React contexts
│   ├── ThemeContext.tsx     # Thèmes & personnalités
│   └── ...
│
├── utils/              # Utilitaires partagés
│   └── helpers.ts           # formatTime, hexToRgba, etc.
│
└── styles/
    └── island-globals.css   # Styles globaux (glassmorphism)
```

### Fenêtres & Entry Points

Chaque fenêtre = 1 fichier HTML + 1 fichier TSX :

| Fenêtre | HTML | TSX | Rôle |
|---------|------|-----|------|
| **Main** | `index.html` | `main.tsx` | Dashboard principal |
| **Chat** | `chat.html` | `chat.tsx` | Interface chat |
| **HUD** | `hud.html` | `hud.tsx` | Ambient LED indicator |
| **Spotlight** | `spotlight.html` | `spotlight.tsx` | Quick decision popup |
| **Settings** | `settings.html` | `settings.tsx` | Configuration |

**Important** : Tous les `.html` doivent être dans `vite.config.ts` :

```typescript
build: {
  rollupOptions: {
    input: {
      main: resolve(__dirname, 'index.html'),
      chat: resolve(__dirname, 'chat.html'),
      hud: resolve(__dirname, 'hud.html'),
      spotlight: resolve(__dirname, 'spotlight.html'),
      settings: resolve(__dirname, 'settings.html'),
    },
  },
},
```

### Composants clés

#### HUD (`src/hud.tsx`)

**Rôle** : Indicateur ambient LED toujours visible

**Features** :
- 60x60px, draggable
- 3 états : normal (vert), opportunity (jaune pulse), blocked (rouge pulse)
- Double-clic → ouvre Spotlight
- Couleurs adaptées au thème (`theme.led.*`)
- Visible même en fullscreen macOS (cocoa FFI côté Rust)

**State management** :
```typescript
const [state, setState] = useState<'normal' | 'opportunity' | 'blocked'>('normal');

// Écoute events backend
listen<{ state: HUDState }>('hud:state-change', (event) => {
  setState(event.payload.state);
});
```

#### Spotlight (`src/spotlight.tsx`)

**Rôle** : Popup décision rapide (600x500px, glassmorphism)

**Features** :
- Position top-center (20% du haut)
- Pas de backdrop dimming (transparent)
- 3 actions : Discuss / View / Ignore
- ESC pour fermer
- Animation Framer Motion

**Ouverture** :
- `Cmd+Shift+Y` (global shortcut)
- Double-clic HUD
- Event `spotlight:show`

#### Chat (`src/chat.tsx`)

**Rôle** : Interface conversation approfondie

**Features** :
- Messages avec markdown
- Context cards
- Slash commands
- Personality selector
- Integration avec opportunités

### Thèmes & Personnalités

**Fichier** : `src/contexts/ThemeContext.tsx`

**Personnalités disponibles** :
- **Orya** : Innovateur créatif (couleur primaire aqua)
- Autres à venir...

**Structure thème** :
```typescript
theme = {
  primary: '#00D9FF',      // Couleur principale
  accent: { ... },          // Couleurs accent
  led: {
    normal: '#4ADE80',     // LED vert (état normal)
    blocked: '#EF4444',    // LED rouge (bloqué)
  },
  glassmorphism: { ... },   // Styles verre
}
```

**Utilisation** :
```typescript
const { theme } = useTheme();
<div style={{ color: theme.led.normal }} />
```

---

## 3. Backend (Rust/Tauri)

### Structure Rust

```
src-tauri/
└── src/
    ├── lib.rs                  # Entry point Tauri
    ├── shortcuts/              # Gestion shortcuts globaux
    │   ├── mod.rs
    │   ├── config.rs          # Définition shortcuts
    │   └── manager.rs         # Logique registration
    ├── triggers/               # Système détection opportunités
    │   ├── mod.rs
    │   ├── trigger_loop.rs    # Boucle détection
    │   └── ...
    ├── storage/                # SQLite persistence
    └── ...
```

### Entry Point (`lib.rs`)

**Sections critiques** :

#### 1. Setup (ligne ~1365)

```rust
.setup(|app| {
    // ⚠️ UN SEUL .setup() — JAMAIS 2 !

    // ESC key handlers pour fenêtres
    setup_escape_handlers(app);

    // Registration shortcuts globaux
    register_global_shortcuts(app);

    // HUD click listener
    setup_hud_listener(app);

    // Configuration macOS
    #[cfg(target_os = "macos")]
    configure_macos_windows(app);

    // Lance trigger loop
    spawn_trigger_loop(app);

    Ok(())
})
```

#### 2. Tauri Commands

Fonctions exposées au frontend via `invoke()` :

```rust
#[tauri::command]
async fn show_window(app_handle: AppHandle, window_label: String) -> Result<(), String>

#[tauri::command]
async fn toggle_spotlight(app_handle: AppHandle) -> Result<bool, String>

#[tauri::command]
async fn get_extended_trigger_stats() -> Result<TriggerStats, String>
```

**Utilisation frontend** :
```typescript
import { invoke } from '@tauri-apps/api/core';

await invoke('show_window', { windowLabel: 'settings' });
const isVisible = await invoke<boolean>('toggle_spotlight');
```

### Shortcuts Manager

**Fichiers** : `src-tauri/src/shortcuts/`

**Workflow** :
1. `config.rs` : Définit shortcuts disponibles
2. `manager.rs` : Register shortcuts au startup
3. Callback → `toggle_spotlight()` → show/hide window

**Code clé** (`manager.rs:160-183`) :
```rust
// Position Spotlight like macOS (top-center, 20% from top)
if let Ok(Some(monitor)) = spotlight_window.current_monitor() {
    let monitor_size = monitor.size();
    let monitor_pos = monitor.position();

    let spotlight_width = 600;
    let spotlight_height = 500;

    let x = monitor_pos.x + (monitor_size.width as i32 - spotlight_width) / 2;
    let y = monitor_pos.y + (monitor_size.height as f64 * 0.20) as i32;

    spotlight_window.set_position(PhysicalPosition::new(x, y))?;
}
```

### macOS FFI (Fullscreen Support)

**Fichier** : `lib.rs:1478-1514`

**Rôle** : Rendre HUD visible même en fullscreen macOS

**Code** :
```rust
#[cfg(target_os = "macos")]
{
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior, NSMainMenuWindowLevel};

    let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
        | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
        | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;

    ns_window.setCollectionBehavior_(behavior);
    ns_window.setLevel_(NSMainMenuWindowLevel + 1);
}
```

**Sans ça** : HUD disparaît quand app en fullscreen

---

## 4. Communication & Data Flow

### Frontend → Backend (invoke)

```typescript
// Frontend appelle commande backend
const { invoke } = await import('@tauri-apps/api/core');
const result = await invoke<ReturnType>('command_name', { param: value });
```

**Exemples** :
```typescript
// Montrer fenêtre
await invoke('show_window', { windowLabel: 'settings' });

// Toggle Spotlight
const isVisible = await invoke<boolean>('toggle_spotlight');

// Get stats
const stats = await invoke<TriggerStats>('get_extended_trigger_stats');
```

### Backend → Frontend (events)

```rust
// Backend émet event
app_handle.emit("event-name", payload)?;
```

```typescript
// Frontend écoute event
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<PayloadType>('event-name', (event) => {
  console.log(event.payload);
});
```

**Exemples** :
```rust
// Backend (Rust)
app_handle.emit("hud:state-change", HUDState { state: "opportunity" })?;
```

```typescript
// Frontend (React)
listen<{ state: string }>('hud:state-change', (event) => {
  setState(event.payload.state);
});
```

### Flow complet (exemple)

```
1. User presse Cmd+Shift+Y
     ↓
2. Rust shortcuts manager détecte
     ↓
3. Appelle toggle_spotlight()
     ↓
4. Rust: window.show() + window.set_focus()
     ↓
5. Rust: emit('spotlight:show', { opportunity })
     ↓
6. Frontend React: listen() reçoit event
     ↓
7. React: setIsVisible(true) + affiche opportunity
     ↓
8. User voit Spotlight avec glassmorphism
```

---

## 5. Workflows utilisateur

### Workflow #1: Détection opportunité → HUD → Spotlight

**Story** :
1. **User code** en fullscreen (FL Studio, VS Code, etc.)
2. **Backend détecte** trigger (copie code, erreur, recherche)
3. **Backend analyse** contexte et crée Opportunity
4. **Backend emit** event `hud:state-change` avec `state: "opportunity"`
5. **HUD pulse jaune** (animation Framer Motion)
6. **User remarque** HUD dans coin écran (pas intrusif)
7. **User double-clic HUD** ou `Cmd+Shift+Y` quand prêt
8. **Spotlight apparaît** top-center (600x500, glassmorphism)
9. **User choisit** : Discuss / View / Ignore
10. **Si Discuss** → Chat s'ouvre avec contexte pré-rempli
11. **Retour coding** sans friction

**Modules impliqués** :
- Backend : `triggers/trigger_loop.rs` (détection)
- Backend : `lib.rs` (emit events)
- Frontend : `hud.tsx` (affichage état)
- Frontend : `spotlight.tsx` (décision)
- Frontend : `chat.tsx` (discussion)

---

### Workflow #2: Settings configuration

**Story** :
1. **User click** bouton "⚙️ Réglages" dans Chat
2. **Frontend invoke** `show_window('settings')`
3. **Backend** trouve fenêtre settings, `show()` + `focus()`
4. **Settings window** apparaît (380x520, séparée)
5. **User modifie** muted apps, allowlist, etc.
6. **Frontend invoke** commandes backend pour save
7. **Backend** persist dans SQLite
8. **Settings ferme** (ESC ou close button)

**Modules impliqués** :
- Frontend : `chat.tsx:275-283` (bouton)
- Backend : `lib.rs:531-544` (show_window command)
- Frontend : `settings.tsx` (UI settings)
- Backend : Storage layer (SQLite)

---

### Workflow #3: Global shortcut → Spotlight

**Story** :
1. **User presse** `Cmd+Shift+Y` (ou `Ctrl+Shift+Y`)
2. **Rust global shortcut** handler déclenché
3. **Backend invoke** `toggle_spotlight()`
4. **Backend check** : Spotlight visible ou caché ?
5. **Si caché** :
   - Calcule position (20% top, center)
   - `window.show()`
   - `window.set_focus()`
   - Emit `spotlight:show`
6. **Si visible** :
   - `window.hide()`
   - Emit `spotlight:hide`
7. **Frontend React** update `isVisible` state
8. **Animation** Framer Motion (fade in/out)

**Modules impliqués** :
- Backend : `shortcuts/manager.rs:130-188` (logique toggle)
- Backend : `shortcuts/manager.rs:160-183` (positioning)
- Frontend : `spotlight.tsx:21-70` (listeners)
- Frontend : `spotlight.tsx:141-164` (animation)

---

### Workflow #4: HUD Drag & Drop

**Story** :
1. **User click+hold** HUD
2. **Frontend détecte** single click (pas double)
3. **Frontend** `setIsDragging(true)`
4. **Frontend invoke** `window.startDragging()`
5. **Tauri** active native window drag
6. **User déplace** souris → HUD suit
7. **User release** → HUD reste à nouvelle position
8. **Position sauvegardée** (TODO: persist per-app)

**Modules impliqués** :
- Frontend : `hud.tsx:49-86` (click detection)
- Tauri : Native `startDragging()` API

---

## 6. Structure détaillée des fichiers

### Où se trouve quoi ?

| Fonctionnalité | Fichier(s) | Lignes approx |
|---------------|------------|---------------|
| **HUD colors** | `src/hud.tsx` | 90-115 |
| **HUD state logic** | `src/hud.tsx` | 49-86 |
| **HUD FFI fullscreen** | `src-tauri/src/lib.rs` | 1478-1514 |
| **Spotlight position** | `src-tauri/src/shortcuts/manager.rs` | 160-183 |
| **Spotlight style** | `src/spotlight.tsx` | 147-161 |
| **Global shortcuts registration** | `src-tauri/src/lib.rs` | 1375-1398 |
| **Shortcut definitions** | `src-tauri/src/shortcuts/config.rs` | Tout le fichier |
| **Settings window config** | `src-tauri/tauri.conf.json` | 50-66 |
| **Themes/personnalités** | `src/contexts/ThemeContext.tsx` | 45-120 |
| **Tauri commands** | `src-tauri/src/lib.rs` | 531-600 |
| **Vite build config** | `vite.config.ts` | 15-25 |

---

## 🧠 Zones critiques (NE PAS CASSER)

**⚠️ Ces zones sont sensibles** — Modifier avec précaution :

| Zone | Fichier | Lignes | Pourquoi critique |
|------|---------|--------|-------------------|
| **Setup Tauri** | `lib.rs` | 1365-1477 | Un seul .setup() autorisé |
| **Shortcuts spawn** | `lib.rs` | 1389-1398 | Utiliser spawn, PAS block_on |
| **HUD double-click** | `hud.tsx` | 49-71 | Logique 300ms fragile |
| **Vite inputs** | `vite.config.ts` | 15-25 | Toutes fenêtres doivent être là |
| **NSWindow FFI** | `lib.rs` | 1486-1496 | Unsafe code, critical pour fullscreen |

---

## 🎯 Prochaine étape

Maintenant que tu comprends l'architecture, voir [MAINTENANCE.md](MAINTENANCE.md) pour savoir **où modifier quoi** concrètement.

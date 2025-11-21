# MAINTENANCE.md — Guide de maintenance quotidien

> **Rôle** : Référence rapide pour modifications courantes
> **Public** : Dev qui travaille sur le projet au quotidien
> **Importance** : TRÈS HAUTE — Fichier le plus consulté

**🎯 Ce fichier répond à : "Je veux modifier X, je fais quoi ?"**

---

## 📋 Table des matières

1. [Quick Wins — Modifications courantes](#quick-wins--modifications-courantes)
2. [Zones critiques (NE PAS CASSER)](#zones-critiques-ne-pas-casser)
3. [Recettes de maintenance](#recettes-de-maintenance)
4. [Conventions de code](#conventions-de-code)
5. [Checklist avant commit](#checklist-avant-commit)

---

## Quick Wins — Modifications courantes

### 🎨 Design & UI

| Je veux... | Fichier | Lignes | Action |
|-----------|---------|--------|--------|
| **Changer couleur LED HUD (normal)** | `src/contexts/ThemeContext.tsx` | 45-80 | Modifier `led.normal: '#4ADE80'` |
| **Changer couleur LED HUD (blocked)** | `src/contexts/ThemeContext.tsx` | 45-80 | Modifier `led.blocked: '#EF4444'` |
| **Modifier taille HUD** | `src-tauri/tauri.conf.json` | 81-82 | Changer `width`/`height` (défaut: 60x60) |
| **Modifier taille Spotlight** | `src-tauri/tauri.conf.json`<br>`src/spotlight.tsx` | 71-72<br>147 | Changer `width`/`height` + style width/height |
| **Changer position Spotlight** | `src-tauri/src/shortcuts/manager.rs` | 173 | Modifier `0.20` (20% du haut) |
| **Modifier border radius Spotlight** | `src/spotlight.tsx` | 155 | Changer `borderRadius: '24px'` |
| **Ajouter une personnalité** | `src/contexts/ThemeContext.tsx` | 45-120 | Dupliquer objet thème, ajouter dans `themes` |
| **Changer glassmorphism** | `src/styles/island-globals.css` | Variables CSS | Modifier `--glass-*` variables |

### ⌨️ Raccourcis & Comportements

| Je veux... | Fichier | Action |
|-----------|---------|--------|
| **Changer raccourci Spotlight** | `src-tauri/src/shortcuts/config.rs` | Modifier `accelerator` dans `ShortcutDef` |
| **Ajouter un nouveau shortcut** | `src-tauri/src/shortcuts/config.rs`<br>`src-tauri/src/shortcuts/manager.rs` | 1. Ajouter def dans config<br>2. Ajouter handler dans manager |
| **Modifier délai double-clic HUD** | `src/hud.tsx` | 54 | Changer `300` (ms) |
| **Changer vitesse pulse HUD** | `src/hud.tsx` | 100-110 | Modifier `pulseSpeed` values |

### 🪟 Fenêtres

| Je veux... | Fichier | Lignes | Action |
|-----------|---------|--------|--------|
| **Ajouter une fenêtre** | Voir [Recette: Nouvelle fenêtre](#recette--ajouter-une-nouvelle-fenêtre) | - | Processus complet ci-dessous |
| **Modifier Settings window size** | `src-tauri/tauri.conf.json` | 53-54 | Changer `width`/`height` |
| **Changer décorations fenêtre** | `src-tauri/tauri.conf.json` | Section window | `decorations: true/false` |
| **Toggle transparent window** | `src-tauri/tauri.conf.json` | Section window | `transparent: true/false` |

### 🔧 Backend & Logic

| Je veux... | Fichier | Action |
|-----------|---------|--------|
| **Ajouter commande Tauri** | `src-tauri/src/lib.rs` | Ajouter `#[tauri::command]` function |
| **Modifier détection opportunité** | `src-tauri/src/triggers/` | Modifier logique dans trigger_loop.rs |
| **Changer SQLite schema** | `src-tauri/src/storage/` | Ajouter migration + modifier structs |

---

## Zones critiques (NE PAS CASSER)

**⚠️ Ces zones nécessitent EXTRÊME attention**

### 🔴 Critique #1: Un seul `.setup()` dans lib.rs

**Fichier** : `src-tauri/src/lib.rs`
**Lignes** : 1365-1477

**RÈGLE ABSOLUE** :
```rust
// ❌ JAMAIS COMME ÇA
.setup(|app| { /* ... */ })
.setup(|app| { /* ... */ })  // Seul le dernier s'exécute !

// ✅ TOUJOURS COMME ÇA
.setup(|app| {
    // Tout dans un seul bloc
    Ok(())
})
```

**Pourquoi** : Tauri n'exécute QUE le dernier `.setup()`. Si tu en mets 2, le premier est ignoré silencieusement.

**Historique** : Ce bug a causé shortcuts jamais enregistrés pendant des heures (voir CONTEXT.md ADR).

---

### 🔴 Critique #2: `spawn()` PAS `block_on()` dans async Tauri

**Fichier** : `src-tauri/src/lib.rs`
**Lignes** : 1389-1398

**RÈGLE** :
```rust
// ❌ JAMAIS
tauri::async_runtime::block_on(async { ... });  // PANIC!

// ✅ TOUJOURS
tauri::async_runtime::spawn(async move { ... });
```

**Pourquoi** : `.setup()` tourne déjà dans runtime Tauri. `block_on()` = "Cannot start runtime from within runtime" panic.

---

### 🔴 Critique #3: Toutes fenêtres dans `vite.config.ts`

**Fichier** : `vite.config.ts`
**Lignes** : 15-25

**RÈGLE** : Chaque `.html` doit être dans `input` :
```typescript
input: {
  main: resolve(__dirname, 'index.html'),
  chat: resolve(__dirname, 'chat.html'),
  spotlight: resolve(__dirname, 'spotlight.html'),
  hud: resolve(__dirname, 'hud.html'),
  settings: resolve(__dirname, 'settings.html'),
}
```

**Pourquoi** : Sans ça, Vite ne build pas le HTML → fenêtre affiche contenu vide.

---

### 🔴 Critique #4: Trigger `idle_seconds` est LEGACY (NE PAS UTILISER)

**Fichier** : `src-tauri/src/triggers/trigger_loop.rs`
**Lignes** : ~50-80 (trigger loop)

**⚠️ RÈGLE ABSOLUE** :

❌ **NE JAMAIS utiliser `idle_seconds > 15` comme trigger principal.**

Le trigger basé uniquement sur l'inactivité utilisateur est **LEGACY** et produit des opportunités **non pertinentes** qui **détruisent l'expérience**.

**Pourquoi** :
- Utilisateur idle ≠ utilisateur bloqué
- Produit des faux positifs massifs
- Interrompt le flow créatif sans raison

**Architecture contractuelle** :
Les vraies opportunités doivent passer par les **patterns définis en Phase 3B** (voir `docs/CONTEXT.md` Section 7) :

1. **Pattern Refacto** : Code répété ≥ 3 fois
2. **Pattern Debug** : Erreur persistante > 60s + 3 tentatives

**Si tu dois modifier triggers** :
1. Lis `docs/CONTEXT.md` Section 7 (Phases 3A/3B)
2. Implémente d'abord Phase 3A (mock data)
3. Puis Phase 3B (patterns intelligents)

**Historique** : Cette décision architecturale est **contractuelle** pour éviter un MVP avec UX cassée.

---

### 🔴 Critique #5: HUD double-click logic

**Fichier** : `src/hud.tsx`
**Lignes** : 49-71

**Zone sensible** : Logique 300ms pour distinguer single-click (drag) vs double-click (open Spotlight)

```typescript
const timeSinceLastClick = now - lastClickRef.current;

if (timeSinceLastClick < 300) {  // ⚠️ Timing fragile
  // Double-click → Spotlight
} else {
  // Single click → Drag
}
```

**Attention** : Si tu changes timing, teste bien drag & double-click.

---

### 🔴 Critique #6: macOS FFI unsafe code

**Fichier** : `src-tauri/src/lib.rs`
**Lignes** : 1486-1496

**Zone** : Code `unsafe` pour NSWindow configuration

```rust
unsafe {
    let behavior = NSWindowCollectionBehavior::...;
    ns_window.setCollectionBehavior_(behavior);
    ns_window.setLevel_(level);
}
```

**Attention** : Ne pas modifier sans comprendre cocoa FFI. Critique pour fullscreen macOS.

---

## Recettes de maintenance

### Recette : Ajouter une nouvelle fenêtre

**Étapes complètes** :

#### 1. Créer fichier HTML
```html
<!-- nouvelle-fenetre.html -->
<!doctype html>
<html lang="fr">
  <head>
    <meta charset="UTF-8" />
    <title>ShadowLearn — Nouvelle Fenêtre</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/nouvelle-fenetre.tsx"></script>
  </body>
</html>
```

#### 2. Créer composant TSX
```typescript
// src/nouvelle-fenetre.tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { ThemeProvider } from './contexts/ThemeContext';

function NouvelleFenetre() {
  return (
    <ThemeProvider>
      <div>
        <h1>Nouvelle Fenêtre</h1>
      </div>
    </ThemeProvider>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <NouvelleFenetre />
  </React.StrictMode>
);
```

#### 3. Ajouter à Vite config
```typescript
// vite.config.ts
input: {
  main: resolve(__dirname, 'index.html'),
  // ... autres fenêtres
  nouvelleFenetre: resolve(__dirname, 'nouvelle-fenetre.html'),  // AJOUTER
}
```

#### 4. Ajouter à Tauri config
```json
// src-tauri/tauri.conf.json, dans "windows": [ ... ]
{
  "label": "nouvelle-fenetre",
  "title": "ShadowLearn — Nouvelle Fenêtre",
  "url": "nouvelle-fenetre.html",
  "width": 400,
  "height": 300,
  "resizable": true,
  "decorations": false,
  "transparent": true,
  "visible": false,
  "center": true
}
```

#### 5. Build et test
```bash
pnpm build
pnpm tauri dev
```

#### 6. Ouvrir depuis frontend
```typescript
import { invoke } from '@tauri-apps/api/core';
await invoke('show_window', { windowLabel: 'nouvelle-fenetre' });
```

**Fichiers modifiés** : 4 fichiers (HTML, TSX, vite.config, tauri.conf)

---

### Recette : Changer les couleurs du HUD

**Scenario** : Tu veux LED bleu au lieu de vert pour état normal

#### 1. Modifier thème Orya
```typescript
// src/contexts/ThemeContext.tsx, ligne ~60
led: {
  normal: '#3B82F6',    // Bleu au lieu de #4ADE80 (vert)
  blocked: '#EF4444',   // Rouge inchangé
},
```

#### 2. Rebuild
```bash
pnpm build
pnpm tauri dev
```

#### 3. Vérifier
- HUD doit être bleu en état normal
- Pulse jaune en état opportunité (utilise normal + opacity)
- Rouge en état blocked

**Fichiers modifiés** : 1 fichier (ThemeContext.tsx)

---

### Recette : Ajouter un nouveau raccourci global

**Scenario** : Tu veux `Cmd+Shift+H` pour toggle HUD

#### 1. Définir shortcut
```rust
// src-tauri/src/shortcuts/config.rs
ShortcutDef {
    id: "toggle-hud",
    label: "Toggle HUD",
    accelerator: if cfg!(target_os = "macos") {
        "Command+Shift+H"
    } else {
        "Ctrl+Shift+H"
    },
}
```

#### 2. Ajouter handler
```rust
// src-tauri/src/shortcuts/manager.rs, dans register_all()
"toggle-hud" => {
    if let Some(hud_window) = app_handle.get_webview_window("hud") {
        let is_visible = hud_window.is_visible()?;
        if is_visible {
            hud_window.hide()?;
        } else {
            hud_window.show()?;
        }
    }
}
```

#### 3. Rebuild et test
```bash
cd src-tauri && cargo build && cd ..
pnpm tauri dev
# Tester Cmd+Shift+H
```

**Fichiers modifiés** : 2 fichiers (config.rs, manager.rs)

---

### Recette : Modifier un thème de personnalité

**Scenario** : Ajuster couleur primaire Orya

#### 1. Modifier couleurs
```typescript
// src/contexts/ThemeContext.tsx
const themes = {
  orya: {
    name: 'Orya',
    primary: '#00E5FF',  // Aqua plus clair
    accent: {
      skyBlue: '#87CEEB',
      emerald: '#50C878',
      // ... autres couleurs
    },
    // ... reste du thème
  },
};
```

#### 2. Test hot reload
```bash
# Si pnpm tauri dev tourne déjà, hot reload automatique
# Sinon: pnpm tauri dev
```

#### 3. Vérifier dans UI
- Chat window: vérifier couleurs accent
- HUD: vérifier LED colors
- Spotlight: vérifier glassmorphism

**Fichiers modifiés** : 1 fichier (ThemeContext.tsx)

---

### Recette : Modifier position Spotlight

**Scenario** : Tu veux Spotlight à 30% du haut au lieu de 20%

#### 1. Modifier calcul position
```rust
// src-tauri/src/shortcuts/manager.rs, ligne ~173
let y = monitor_pos.y + (monitor_size.height as f64 * 0.30) as i32;
//                                                    ^^^^ Changer 0.20 → 0.30
```

#### 2. Rebuild backend
```bash
cd src-tauri && cargo build && cd ..
pnpm tauri dev
```

#### 3. Test
- `Cmd+Shift+Y` pour ouvrir Spotlight
- Vérifier position verticale (doit être plus bas)

**Fichiers modifiés** : 1 fichier (manager.rs)

---

## Conventions de code

### TypeScript / React

**Format** :
- ESLint + Prettier configurés
- Pas de `console.log` en production (seulement `console.error`)
- Hooks custom : préfixe `use` (ex: `useHover`)
- Composants : PascalCase (ex: `HUDIndicator`)

**Structure composant** :
```typescript
import React, { useState } from 'react';

export function MonComposant() {
  const [state, setState] = useState(initial);

  return (
    <div>...</div>
  );
}
```

**Imports organisés** :
```typescript
// 1. React & libraries
import React from 'react';
import { motion } from 'framer-motion';

// 2. Tauri
import { invoke } from '@tauri-apps/api/core';

// 3. Local
import { useTheme } from './contexts/ThemeContext';
import { hexToRgba } from './utils/helpers';
```

---

### Rust / Tauri

**Format** :
- `cargo fmt` avant commit
- `cargo clippy` pour lints
- Pas de `unwrap()` en production (utiliser `?` ou `match`)

**Structure commande Tauri** :
```rust
#[tauri::command]
async fn ma_commande(
    app_handle: tauri::AppHandle,
    param: String,
) -> Result<ReturnType, String> {
    // Logique
    Ok(result)
}
```

**Logging** :
```rust
use tracing::{info, warn, error};

info!("✅ Success message");
warn!("⚠️ Warning message");
error!("❌ Error: {}", err);
```

---

## Checklist avant commit

### ✅ Build & Tests

- [ ] `pnpm tsc --noEmit` passe (pas d'erreurs TypeScript)
- [ ] `pnpm build` réussit (frontend compile)
- [ ] `cargo clippy` passe (pas de warnings Rust)
- [ ] `cargo fmt` appliqué
- [ ] App lance sans crash : `pnpm tauri dev`

### ✅ Code Quality

- [ ] Pas de `console.log` (seulement `console.error`)
- [ ] Pas de code commenté inutile
- [ ] Pas de TODOs sans contexte
- [ ] Imports inutilisés supprimés

### ✅ Fonctionnel

- [ ] Feature testée manuellement
- [ ] Shortcuts globaux fonctionnent (si modifiés)
- [ ] Fenêtres s'affichent correctement (si modifiées)
- [ ] Pas de régression visuelle

### ✅ Documentation

- [ ] Si nouveau feature → Update CONTEXT.md
- [ ] Si nouvelle commande → Update docs/reference/API.md
- [ ] Si modification critique → Update MAINTENANCE.md

### ✅ Git

- [ ] Commit message clair (français)
- [ ] Pas de secrets committés (.env gitignored)
- [ ] Pas de node_modules ou dist/ (via .gitignore)

---

## Fichiers à ne JAMAIS committer

**⚠️ Vérifier .gitignore contient** :
- `node_modules/`
- `dist/`
- `src-tauri/target/`
- `.env`
- `*.log`
- Database SQLite (`*.db`)

---

## 🎯 Prochaines étapes

- **Ajouter une feature** ? Lire [CONTEXT.md](CONTEXT.md) pour décisions passées
- **Débugger** ? Voir [SETUP.md#Troubleshooting](SETUP.md#troubleshooting)
- **Comprendre architecture** ? Voir [SYSTEM_OVERVIEW.md](SYSTEM_OVERVIEW.md)
- **API reference** ? Voir [docs/reference/API.md](docs/reference/API.md)

---

**💡 Ce fichier est ton allié quotidien. Bookmark-le !**

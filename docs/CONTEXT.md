# CONTEXT.md — La mémoire du projet

> **Rôle** : Capturer toute la mémoire des conversations, décisions et raisonnements du projet
> **Public** : Nouveau dev, futur toi, contributeur qui reprend le projet
> **Importance** : **CRITIQUE** — Ce fichier remplace 3 mois de contexte

**🔥 LIS CE FICHIER EN PREMIER** avant de toucher au code. Il contient **TOUT** ce qu'un dev ayant suivi les conversations saurait.

---

## 1. Genèse & Vision produit

### Le problème initial

Les développeurs créatifs (musiciens-codeurs, artistes numériques) travaillent souvent :
- En **fullscreen** (FL Studio, Ableton, VS Code, etc.)
- Dans un **flow créatif profond** qu'on ne peut pas interrompre
- Avec des **moments d'apprentissage potentiels** qu'ils ratent

Les assistants traditionnels (ChatGPT, Copilot) sont **bloquants** :
- Fenêtres pop-up qui cassent le flow
- Demandent de changer de contexte
- Ne comprennent pas qu'on est en mode créatif

### La solution : ShadowLearn

Un **assistant ambient** qui :
1. **Détecte** les opportunités d'apprentissage (copier du code, erreur, recherche, etc.)
2. **Signale discrètement** via un HUD "luciole dans la nuit"
3. **Attend que l'utilisateur décide** (pas d'interruption forcée)
4. **Propose rapidement** via Spotlight (comme macOS)
5. **Approfondit sur demande** via Chat

### Persona cible

**Développeur créatif 25-40 ans** :
- Utilise FL Studio / Ableton / tools créatifs en fullscreen
- Code aussi (VS Code, Cursor, etc.)
- Veut apprendre mais déteste être interrompu
- Aime les interfaces "magiques" mais discrètes
- Sensible à l'esthétique (glassmorphism, animations fluides)

### Philosophie core

**"Luciole dans la nuit"** :
- Toujours présente mais jamais intrusive
- Guide sans forcer
- S'adapte au contexte de l'utilisateur
- Respecte le flow créatif

---

## 2. Décisions de design majeures (et POURQUOI)

### ADR-001: HUD = Ambient LED (pas un bouton)

**Date** : Janvier 2025
**Contexte** : L'utilisateur a besoin d'une présence constante mais non-intrusive
**Décision** : HUD circulaire 60x60px, ambient LED avec états visuels
**Raisons** :
- ✅ "Luciole dans la nuit" = métaphore parfaite
- ✅ Toujours visible sans prendre de place
- ✅ Couleurs adaptées au thème (LED normal/opportunity/blocked)
- ✅ Double-clic pour action = naturel
- ✅ Draggable = personnalisable

**Alternatives rejetées** :
- ❌ Bouton standard : trop "app traditionnelle"
- ❌ Notification système : invisible en fullscreen
- ❌ Menu bar icon : pas assez présent

**Impact** :
- Design unique, mémorable
- Permet de rester visible même en fullscreen
- Nécessite cocoa FFI pour macOS (NSWindowCollectionBehavior)

**Fichiers concernés** :
- `src/hud.tsx` : Composant HUD
- `src/contexts/ThemeContext.tsx` : Couleurs LED par thème
- `src-tauri/src/lib.rs:1470-1514` : Configuration macOS fullscreen

---

### ADR-002: Spotlight top-center, NO backdrop dimming

**Date** : Janvier 2025
**Contexte** : Feedback utilisateur : "je veux voir l'app derrière"
**Décision** :
- Position : **20% du haut, centré** (comme macOS Spotlight)
- Taille : **600x500px fixe** (pas de scroll)
- Background : **transparent** (pas de rgba(0,0,0,0.4))
- BorderRadius : **24px** (bien arrondi)

**Raisons** :
- ✅ Pas de backdrop = voir l'app derrière = workflow fluide
- ✅ Position haute = regard naturel
- ✅ Taille fixe = décision rapide (pas de lecture infinie)
- ✅ Glassmorphism = moderne et léger

**Alternatives rejetées** :
- ❌ Center center : cache trop l'app
- ❌ Backdrop dimming : trop intrusif
- ❌ Modal bloquante : casse le flow

**Impact** :
- Expérience non-bloquante unique
- Utilisateur garde le contrôle visuel
- Workflows créatifs non interrompus

**Fichiers concernés** :
- `src/spotlight.tsx:120-140` : Position et style
- `src-tauri/src/shortcuts/manager.rs:160-183` : Positionnement programmatique

---

### ADR-003: Settings = Fenêtre séparée (PAS modal)

**Date** : Janvier 2025
**Contexte** : Utilisateur : *"les réglages c'est la partie chiante, rendre ça immersif"*
**Décision** : Fenêtre `settings.html` **séparée** du chat, 380x520px
**Raisons** :
- ✅ "Pas une bulle qui apparaît au-dessus du chat"
- ✅ Expérience immersive pour les réglages
- ✅ Peut rester ouverte pendant l'usage
- ✅ Cohérence avec philosophie "jamais de modal bloquante"

**Alternatives rejetées** :
- ❌ Modal au-dessus du chat : "chiante"
- ❌ Onglet dans chat : pas assez immersif

**Impact** :
- Architecture multi-fenêtres (4 fenêtres : main, chat, hud, spotlight, settings)
- Meilleure UX pour configuration

**Fichiers concernés** :
- `src/settings.tsx` : Composant fenêtre
- `settings.html` : Entry point
- `src-tauri/tauri.conf.json:50-66` : Config fenêtre

---

### ADR-004: Tauri v2 (pas Electron)

**Date** : Début projet (2024)
**Contexte** : Besoin desktop natif, faible empreinte mémoire
**Décision** : **Tauri v2** (Rust backend + React frontend)
**Raisons** :
- ✅ **10x plus léger** qu'Electron (~5 MB vs ~50 MB)
- ✅ **Accès natif** : cocoa FFI pour macOS fullscreen
- ✅ **Sécurité Rust** : memory safety
- ✅ **Performance** : pas de Chromium embarqué
- ✅ **Webview système** : moins de RAM

**Alternatives rejetées** :
- ❌ Electron : trop lourd
- ❌ NW.js : moins actif
- ❌ Web app : pas d'accès système

**Conséquences** :
- ✅ Performance native
- ✅ Intégration OS profonde (shortcuts globaux, FFI)
- ⚠️ Setup Rust obligatoire
- ⚠️ Moins de libs que Electron (compensé par qualité)

---

### ADR-005: Raccourci global Cmd+Shift+Y

**Date** : Janvier 2025
**Contexte** : Besoin raccourci accessible mais pas conflictuel
**Décision** : `Cmd+Shift+Y` (macOS) / `Ctrl+Shift+Y` (autres)
**Raisons** :
- ✅ Pas utilisé par apps courantes
- ✅ Proche de `Cmd+Shift+Space` (Spotlight macOS)
- ✅ "Y" = proche de "Yes" = action positive
- ✅ Shift = modificateur fort (évite déclenchements accidentels)

**Alternatives testées** :
- ❌ `Cmd+K` : pris par VS Code
- ❌ `Cmd+J` : pris par apps courantes
- ❌ `Cmd+M` : minimize sur macOS
- ❌ `Cmd+L` : pris par browsers

**Impact** :
- Raccourci mémorisable
- Pas de conflits utilisateur

**Fichiers concernés** :
- `src-tauri/src/shortcuts/config.rs` : Définition shortcuts
- `src-tauri/src/shortcuts/manager.rs` : Logique toggle

---

## 3. Problèmes techniques critiques résolus

### 🐛 Problème #1: Shortcuts jamais enregistrés

**Symptôme** : `Cmd+Shift+Y` ne fonctionnait pas, aucune réaction
**Logs** : "✅ Shortcut manager initialized" mais jamais "🎹 About to register shortcuts"

**Cause racine** : **Duplicate `.setup()` calls** dans `lib.rs`
- `.setup()` à ligne 1101 (avec shortcuts)
- `.setup()` à ligne 1424 (avec window positioning)
- ⚠️ **Tauri exécute SEULEMENT le dernier `.setup()`**

**Solution** :
```rust
// AVANT (2 .setup())
.setup(|app| { /* shortcuts */ })
.setup(|app| { /* windows */ })  // ← Seul celui-ci s'exécute !

// APRÈS (1 seul .setup())
.setup(|app| {
    // shortcuts + windows fusionnés
})
```

**Impact** :
- ✅ Shortcuts fonctionnent maintenant
- ⚠️ **RÈGLE ABSOLUE** : JAMAIS 2 `.setup()` dans lib.rs

**Commit** : `45de946`
**Fichiers** : `src-tauri/src/lib.rs:1365-1477`

---

### 🐛 Problème #2: Runtime panic "Cannot start runtime from within runtime"

**Symptôme** : App crash au démarrage avec panic tokio

**Cause racine** : Utilisation de `block_on()` dans `.setup()` qui tourne déjà dans runtime Tauri

```rust
// AVANT
tauri::async_runtime::block_on(async {
    manager.register_all(&app_handle).await
});  // ← PANIC!

// APRÈS
tauri::async_runtime::spawn(async move {
    manager.register_all(&app_handle).await
});  // ← OK
```

**Leçon** :
- ⚠️ **JAMAIS** `block_on()` dans async Tauri context
- ✅ **TOUJOURS** `spawn()` pour async work dans setup

**Impact** : App stable au démarrage

---

### 🐛 Problème #3: Vite ne buildait pas les fenêtres

**Symptôme** : `dist/` vide après `cargo build`, fenêtres affichaient contenu vide

**Cause racine** : `pnpm build` jamais exécuté (seulement `cargo build`)

**Solution** : Update `clean-build.sh`
```bash
# Build frontend AVANT backend
pnpm build
cd src-tauri && cargo build --release
```

**Impact** :
- ✅ Toutes les HTML générées dans dist/
- ✅ Build reproductible

---

### 🐛 Problème #4: Spotlight/HUD affichaient mauvais contenu

**Symptôme** : Spotlight affichait dashboard au lieu de son UI

**Cause racine** : `spotlight.html` et `hud.html` **absents de `vite.config.ts`**

```typescript
// AVANT
input: {
  main: resolve(__dirname, 'index.html'),
  chat: resolve(__dirname, 'chat.html'),
}

// APRÈS
input: {
  main: resolve(__dirname, 'index.html'),
  chat: resolve(__dirname, 'chat.html'),
  spotlight: resolve(__dirname, 'spotlight.html'),  // AJOUTÉ
  hud: resolve(__dirname, 'hud.html'),              // AJOUTÉ
  settings: resolve(__dirname, 'settings.html'),    // AJOUTÉ
}
```

**Impact** :
- ✅ Chaque fenêtre a son HTML correct
- ⚠️ Toujours ajouter entrées Vite pour nouvelles fenêtres

---

### 🐛 Problème #5: HUD invisible en fullscreen macOS

**Symptôme** : HUD disparaît quand FL Studio ou autre app en fullscreen

**Cause racine** : macOS Fullscreen crée un **Space séparé**, fenêtres normales pas visibles

**Solution** : **cocoa FFI** avec `NSWindowCollectionBehavior`

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

**Impact** :
- ✅ HUD visible sur TOUS les Spaces macOS
- ✅ Reste au-dessus même en fullscreen
- ⚠️ Nécessite dépendances `cocoa` et `objc`

**Fichiers** : `src-tauri/src/lib.rs:1478-1514`

---

## 4. Gotchas techniques ABSOLUS

### ⚠️ RÈGLE #1: Un seul `.setup()` dans lib.rs

**JAMAIS** :
```rust
.setup(|app| { /* ... */ })
.setup(|app| { /* ... */ })  // ❌ Seul celui-ci s'exécute !
```

**TOUJOURS** :
```rust
.setup(|app| {
    // Tout dans un seul bloc
    Ok(())
})
```

---

### ⚠️ RÈGLE #2: `spawn()` pas `block_on()` dans Tauri async

**JAMAIS** :
```rust
tauri::async_runtime::block_on(async { ... });  // ❌ PANIC
```

**TOUJOURS** :
```rust
tauri::async_runtime::spawn(async move { ... });  // ✅ OK
```

---

### ⚠️ RÈGLE #3: Toutes fenêtres dans `vite.config.ts`

Chaque `.html` doit être dans `rollupOptions.input` :

```typescript
input: {
  main: resolve(__dirname, 'index.html'),
  chat: resolve(__dirname, 'chat.html'),
  spotlight: resolve(__dirname, 'spotlight.html'),
  hud: resolve(__dirname, 'hud.html'),
  settings: resolve(__dirname, 'settings.html'),
}
```

---

### ⚠️ RÈGLE #4: NO `console.log` en production

**INTERDIT** : `console.log()` (pollue logs, performance)
**AUTORISÉ** : `console.error()` (debugging errors)

**Convention** :
- Dev : logs temporaires OK
- Avant commit : supprimer tous console.log
- Production : seulement console.error

---

### ⚠️ RÈGLE #5: macOS fullscreen = cocoa FFI obligatoire

Pour fenêtres visibles en fullscreen macOS :

```rust
#[cfg(target_os = "macos")]
{
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
    // Configure window behavior
}
```

**Sans ça** : fenêtre invisible en fullscreen

---

## 5. Architecture decisions

### Pourquoi multi-fenêtres ?

**Décision** : 4 fenêtres séparées (main, chat, hud, spotlight, settings)

**Raisons** :
- ✅ Chaque fenêtre = rôle distinct
- ✅ Peut être positionnée indépendamment
- ✅ Peut avoir styles différents (decorations, transparency)
- ✅ Meilleure performance (rendu séparé)

**Alternative rejetée** : SPA avec routing
- ❌ Moins de contrôle OS-level
- ❌ Animations window moins fluides
- ❌ Pas de drag & drop indépendant

---

### Pourquoi SQLite local (pas cloud) ?

**Décision** : SQLite embarqué

**Raisons** :
- ✅ Privacy : données jamais envoyées ailleurs
- ✅ Offline-first : fonctionne sans internet
- ✅ Performance : pas de latence réseau
- ✅ Simplicité : pas de serveur à maintenir

**Conséquences** :
- ✅ Utilisateur contrôle ses données
- ⚠️ Pas de sync multi-device (volontaire)

---

### Pourquoi React 19 + TypeScript ?

**Raisons** :
- ✅ React 19 : dernières features (use, Server Components si besoin futur)
- ✅ TypeScript : safety, autocomplete, refactoring
- ✅ Framer Motion : animations fluides essentielles au design
- ✅ Écosystème mature : hooks, contexts, libraries

**Alternative considérée** : Svelte
- ❌ Moins de libs desktop
- ❌ Moins de devs familiers

---

## 6. Workflow utilisateur idéal

### Flow complet (vision)

```
1. Utilisateur code en fullscreen (FL Studio, VS Code, etc.)
2. Trigger détecté (copie code, erreur, recherche, etc.)
3. HUD change d'état → pulsation jaune (opportunité)
4. Utilisateur décide quand regarder
5. Double-clic HUD OU Cmd+Shift+Y
6. Spotlight apparaît (top-center, 600x500, glassmorphism)
7. Choix rapide : Discuss / View / Ignore
8. Si Discuss → Chat s'ouvre avec contexte pré-rempli
9. Retour au coding sans friction
```

### États HUD

| État | Couleur | Pulsation | Signification |
|------|---------|-----------|---------------|
| **Normal** | Vert (theme.led.normal) | Aucune | Tout va bien, en attente |
| **Opportunity** | Jaune (theme.led.normal) | Lente (2s) | Opportunité détectée |
| **Blocked** | Rouge (theme.led.blocked) | Rapide (1.5s) | Utilisateur bloqué |

---

## 7. Ce qui est fait vs ce qui reste

### ✅ Fait (Phase 3 complete)

- [x] HUD ambient LED avec états visuels
- [x] HUD visible en fullscreen macOS (cocoa FFI)
- [x] HUD draggable avec position sauvegardée
- [x] HUD double-clic ouvre Spotlight
- [x] Spotlight Cmd+Shift+Y global
- [x] Spotlight position top-center 20%
- [x] Spotlight 600x500 glassmorphism, pas de backdrop
- [x] Settings fenêtre séparée (pas modal)
- [x] Chat fenêtre principale
- [x] Thèmes (Orya, etc.) avec couleurs LED
- [x] Code optimisé (console.log removed, utils/, hooks/)
- [x] Build system (Vite + Tauri)
- [x] Documentation complète

### 🚧 En cours / À faire

#### Priorité HAUTE
- [ ] Système détection opportunités (triggers) — Backend existe, besoin polish
- [ ] Communication HUD ↔ Backend (events) — Partiellement fait
- [ ] Spotlight affiche vraies opportunités — Actuellement mock data
- [ ] Intégration Chat ↔ Spotlight (passer contexte)

#### Priorité MOYENNE
- [ ] Tests E2E (shortcuts, windows, flows)
- [ ] Build automatisé CI/CD
- [ ] Signatures macOS (pour distribution)
- [ ] Persistence settings utilisateur (partiellement fait)

#### Priorité BASSE
- [ ] Windows/Linux support complet
- [ ] Analytics usage (optionnel, privacy-first)
- [ ] Onboarding première utilisation

---

## 8. Où chercher quoi (Quick Reference)

| Je veux... | Fichier(s) à modifier |
|-----------|----------------------|
| **Changer couleurs HUD** | `src/hud.tsx:90-115` + `src/contexts/ThemeContext.tsx:45-80` |
| **Modifier raccourci Spotlight** | `src-tauri/tauri.conf.json` ou `src-tauri/src/shortcuts/config.rs` |
| **Ajouter une personnalité/thème** | `src/contexts/ThemeContext.tsx:45-120` |
| **Changer taille Spotlight** | `src-tauri/tauri.conf.json:71-72` + `src/spotlight.tsx:147` |
| **Modifier fenêtre Settings** | `src-tauri/tauri.conf.json:53-54` + `src/settings.tsx` |
| **Ajouter detection trigger** | `src-tauri/src/triggers/` |
| **Modifier comportement HUD** | `src/hud.tsx` |
| **Changer animations** | `src/spotlight.tsx`, `src/hud.tsx` (Framer Motion) |
| **Build config** | `vite.config.ts`, `src-tauri/tauri.conf.json` |

---

## 9. Lexique technique interne

### Termes métier

- **Opportunité** : Moment détecté où user pourrait apprendre (trigger + contexte)
- **Trigger** : Événement système (clipboard, error, typing pattern, etc.)
- **Pattern** : Séquence d'actions utilisateur qui forme un comportement
- **Context layer** : Informations contextuelles (app, file, code sélectionné, etc.)

### Termes UI

- **HUD** : Heads-Up Display, fenêtre 60x60 toujours visible
- **Spotlight** : Popup décision rapide, inspiration macOS Spotlight
- **Glassmorphism** : Style vitreux avec `backdrop-filter: blur()`
- **Ambient LED** : Concept design du HUD (luciole)

### Termes techniques

- **FFI** : Foreign Function Interface (appels natifs OS depuis Rust)
- **IPC** : Inter-Process Communication (Tauri commands, events)
- **Webview** : Navigateur système embarqué (pas Chromium complet)
- **NSWindow** : Classe fenêtre native macOS (via cocoa)

---

## 10. Future direction / Non tranchés

### Questions ouvertes

1. **Détection opportunités** : Critères exacts pour trigger opportunité ?
   - Actuellement : heuristiques simples
   - Futur : ML ? Patterns ? Feedback utilisateur ?

2. **Multi-device sync** : Vouloir ou pas ?
   - Actuellement : SQLite local seulement
   - Futur : Option cloud optionnelle ?

3. **Marketplace extensions** : Permettre triggers custom ?
   - Actuellement : triggers hardcodés
   - Futur : Plugin system ?

4. **Mobile companion** : App mobile pour notifications ?
   - Actuellement : Desktop seulement
   - Futur : iOS/Android pour alertes ?

### Décisions à prendre

- **Télémétrie** : Collecter analytics anonymes ? (Privacy-first approach nécessaire)
- **Monétisation** : Freemium ? One-time purchase ? Open source ?
- **Distribution** : Mac App Store ? Direct download ? Homebrew ?

---

## 11. Historique décisionnel condensé

### Janvier 2025

**Phase 1: MVP Spotlight + HUD**
- Création architecture multi-fenêtres
- Spotlight première version (rectangle, centered)
- HUD première version (bouton simple)

**Phase 2: Feedback utilisateur**
- Feedback : "le popup rectangle, slideable" → Fix glassmorphism
- Feedback : "je veux voir l'app derrière" → Remove backdrop dimming
- Feedback : "luciole dans la nuit" → Redesign HUD ambient LED

**Phase 3: Optimisations** *(Actuel)*
- Settings fenêtre séparée (user: "pas une bulle chiante")
- Spotlight 600x500 fixe, top-center, rounded
- HUD fullscreen support (cocoa FFI)
- Code cleanup (console.log, utils, hooks)
- Documentation complete

**Phase 4: Production** *(Next)*
- Détection opportunités réelles
- Tests E2E complets
- Build signing
- First beta release

---

## 12. Contribuer à ce fichier

**Ce fichier est vivant.** Quand une décision majeure est prise :

1. **Ajoute un ADR** dans section 2
2. **Documente le problème** dans section 3 si technique
3. **Update "Ce qui est fait"** dans section 7
4. **Ajoute "Où chercher quoi"** dans section 8

**Format ADR** :
```markdown
### ADR-XXX: Titre décision

**Date** : YYYY-MM
**Contexte** : Situation qui a mené à la décision
**Décision** : Ce qui a été décidé
**Raisons** : Pourquoi (bullet points)
**Alternatives rejetées** : Ce qui n'a PAS été choisi
**Impact** : Conséquences de la décision
**Fichiers concernés** : Où c'est implémenté
```

---

**🔥 Ce fichier capture 100% de la mémoire projet. Un dev qui le lit peut reprendre comme s'il avait fait toute la conversation.**

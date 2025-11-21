# 🌑 ShadowLearn

**Assistant d'apprentissage ambient pour développeurs créatifs**

ShadowLearn détecte automatiquement les opportunités d'apprentissage pendant que vous codez, sans jamais interrompre votre flow créatif. Conçu pour les développeurs qui travaillent en fullscreen (FL Studio, VS Code, etc.) et veulent un assistant discret mais toujours présent.

---

## 👁️ Vue d'ensemble (en 30 secondes)

- **ShadowLearn** = assistant d'apprentissage ambient, non-intrusif
- **HUD** = "luciole dans la nuit" (60x60px, toujours visible, adapté au thème)
- **Spotlight** = `Cmd+Shift+Y` (macOS) ou `Ctrl+Shift+Y` (autres)
- **Philosophie** : Pas de backdrop dimming, pas de fenêtres bloquantes
- **3 fenêtres** : HUD (ambient LED), Spotlight (décisions rapides), Chat (discussions approfondies)

---

## 🚀 Quickstart

```bash
# Prérequis: Node 22+, Rust stable, pnpm
git clone <repo>
cd ShadowLearn
pnpm install
pnpm tauri dev
```

---

## 📚 Documentation

### Pour nouveau développeur

**Jour 1 (2h)** — Comprendre le projet :
1. [README.md](README.md) (5 min) ← Vous êtes ici
2. **[CONTEXT.md](docs/CONTEXT.md)** (45 min) ← 🔥 **COMMENCEZ ICI** (toute la mémoire du projet)
3. [SETUP.md](docs/SETUP.md) (30 min)
4. Lancer l'app en dev (30 min)

**Jour 2 (3h)** — Architecture et pratique :
1. [SYSTEM_OVERVIEW.md](docs/SYSTEM_OVERVIEW.md) (1h) — Architecture + workflows
2. [MAINTENANCE.md](docs/MAINTENANCE.md) (30 min) — Où modifier quoi
3. Première modification simple (1h)

**Jour 3+** — Approfondissement :
- [docs/reference/API.md](docs/reference/API.md) — Commandes Tauri
- [docs/reference/SECURITY.md](docs/reference/SECURITY.md) — Privacy & sécurité

---

## 🎯 Next Steps (Priorités actuelles - Jan 2025)

**Phases du système d'opportunités** (voir [docs/CONTEXT.md](docs/CONTEXT.md) Section 7) :

1. ✅ **Phase 1** : Spotlight (UX de base) — FAIT
2. ✅ **Phase 2** : HUD "Luciole" (Ambient LED) — FAIT
3. 🚧 **Phase 3A** : Spotlight avec opportunités MOCK (EN COURS)
   - Store d'opportunités frontend
   - Debug trigger pour tests
   - Actions [Discuter/Voir/Ignorer] fonctionnelles
4. ⏳ **Phase 3B** : Détection intelligente MVP (NEXT)
   - ⚠️ **IMPORTANT** : Désactiver trigger `idle_seconds` (legacy)
   - Pattern Refacto (code répété ≥ 3x)
   - Pattern Debug (erreur persistante)

**Bugs critiques** :
- 🐛 Settings window invisible (logs "shown" mais pas visible)
- ⚠️ ~55 warnings TypeScript à corriger

**Voir la roadmap complète** : [docs/CONTEXT.md](docs/CONTEXT.md) Section 7

---

## 🔑 Glossaire rapide

| Terme | Définition |
|-------|------------|
| **HUD** | Indicateur ambient LED (60x60px) toujours visible, change de couleur selon l'état |
| **Spotlight** | Fenêtre popup (600x500px) style macOS Spotlight, apparaît en top-center |
| **Opportunité** | Moment détecté où l'utilisateur pourrait apprendre quelque chose |
| **Trigger** | Événement qui déclenche une détection (copier du code, erreur, etc.) |
| **Ambient assistant** | Assistant non-intrusif, toujours présent mais jamais bloquant |
| **Luciole** | Design du HUD inspiré d'une luciole dans la nuit |
| **Glassmorphism** | Style visuel avec backdrop blur et transparence |

---

## 🛠️ Stack technique

- **Desktop**: Tauri v2 (Rust + TypeScript)
- **Frontend**: React 19 + Framer Motion + TypeScript
- **Backend**: Rust + Tokio async runtime
- **Storage**: SQLite local
- **Build**: Vite 7 + pnpm
- **Platform**: macOS (primaire), Windows/Linux (secondaire)

---

## ⌨️ Raccourcis clavier

| Raccourci | Action |
|-----------|--------|
| `Cmd+Shift+Y` (macOS)<br>`Ctrl+Shift+Y` (autres) | Toggle Spotlight |
| `Esc` | Fermer Spotlight |
| Double-clic HUD | Ouvrir Spotlight |
| Click + glisser HUD | Déplacer le HUD |

---

## 🎨 Philosophie de design

1. **Non-intrusif** : Jamais de backdrop dimming, jamais de fenêtres modales bloquantes
2. **Toujours accessible** : HUD visible même en fullscreen (cocoa FFI sur macOS)
3. **Contextuellement adapté** : Couleurs et comportement s'adaptent aux thèmes de personnalité
4. **Workflow-first** : L'app s'adapte au flow créatif, pas l'inverse

---

## 📁 Structure du projet

```
ShadowLearn/
├── src/                    # Frontend React/TypeScript
│   ├── components/         # Composants UI
│   ├── hooks/              # Custom hooks React
│   ├── contexts/           # Contexts (Theme, etc.)
│   └── utils/              # Utilitaires partagés
├── src-tauri/              # Backend Rust/Tauri
│   └── src/
│       ├── shortcuts/      # Gestion raccourcis globaux
│       ├── triggers/       # Détection opportunités
│       └── lib.rs          # Entry point Tauri
├── *.html                  # Entry points Vite (chat, hud, spotlight, settings)
└── docs/                   # Documentation
    └── reference/          # Références API/sécurité
```

---

## 🤝 Contribuer

Voir [MAINTENANCE.md](MAINTENANCE.md) pour savoir où modifier quoi.

Pour les conventions de code et tests :
- TypeScript : ESLint + Prettier (pas de console.log en production)
- Rust : clippy + rustfmt
- Commits : Messages clairs en français

---

## 📄 License

[À définir]

---

## 🆘 Besoin d'aide ?

1. **Problème de setup ?** → [SETUP.md](SETUP.md)
2. **Comprendre l'architecture ?** → [SYSTEM_OVERVIEW.md](SYSTEM_OVERVIEW.md)
3. **Modifier une feature ?** → [MAINTENANCE.md](MAINTENANCE.md)
4. **Bug ou question ?** → Ouvrir une issue

---

**Note** : Ce projet évolue rapidement. La documentation est maintenue à jour après chaque changement majeur. Si vous trouvez une incohérence, signalez-la ou créez une PR.

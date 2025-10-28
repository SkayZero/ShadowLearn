# 🌑 ShadowLearn - AI Learning Assistant

ShadowLearn est une application desktop d'apprentissage intelligent basée sur Tauri v2 et React, conçue pour aider les développeurs en analysant leur contexte de travail et en générant des réponses adaptatives.

---

## 🚀 Fonctionnalités Principales

### J21.5: Consolidation & Robustesse
- ✅ **Timeout LLM 30s** avec fallback heuristique
- ✅ **Feature Flags** dynamiques (USE_INTENT_GATE)
- ✅ **Observabilité complète** (cluster_ms, intent_ms, cache_hit_rate)
- ✅ **Bouton Paramètres** ⚙️ pour contrôle à chaud

### J22: Adaptive Prompting Engine
- ✅ **Prompts contextuels** basés sur l'intention détectée
- ✅ **Templates adaptatifs** (Concise, Pedagogical, Creative, Analytical, Empathetic)
- ✅ **Cache intelligent** avec TTL 10 minutes
- ✅ **Trust scaling** pour ajustement dynamique

### Fonctionnalités existantes
- 📸 Capture d'écran contextuelle (J11)
- 🔔 Système de triggers intelligents (J12)
- 💤 Snooze & gestion d'activité (J15)
- 🛡️ Anti-spam & UX optimisée (J16)
- 💾 Persistance SQLite (J17)
- 🧠 Personalisation ML (J18)
- 🎯 User trust scoring (J19)
- ✅ Artefact validation (J20)

---

## 📦 Installation

### Prérequis
- Node.js 22+
- Rust stable
- pnpm

### Installation
```bash
git clone <repo>
cd shadowlearn
pnpm install
```

### Lancement
```bash
# Mode développement avec logs
./run_with_logs.sh

# Dans un autre terminal: monitoring
./monitor_logs.sh
```

---

## 🧪 Tests

### Tests automatiques
```bash
# Test J21.5 (Phase 1)
./test_j21_5.sh

# Test J22 (Phase 3)
./test_j22.sh

# Métriques complètes
./show_j21_metrics.sh
```

### Checklist de validation
```bash
./monitor_j21_5.sh
```

---

## ⚙️ Configuration

### Feature Flags
Via variables d'environnement:
```bash
export SL_USE_INTENT_GATE=true   # Intent Gate
export SL_SMART_TRIGGERS=true    # Smart Triggers
export SL_TELEMETRY=true         # Telemetry
```

Via interface:
1. Cliquer sur ⚙️ dans le header
2. Activer/désactiver les feature flags
3. Observer les changements en temps réel

---

## 📊 Monitoring

### Logs en temps réel
```bash
# Filtrage J21.5 & J22
tail -f /tmp/shadowlearn_dev.log | grep -E "(cluster_ms|intent_ms|ADAPTIVE)"

# Métriques complètes
./show_j21_metrics.sh
```

### Métriques clés
- `cluster_ms`: Latence clustering (< 100ms)
- `intent_ms`: Latence détection (< 2s OpenAI, < 5s Ollama)
- `cache_hit_rate`: Taux de cache (> 40% attendu)
- `trust_score`: Score de confiance utilisateur

---

## 🏗️ Architecture

### Backend (Rust)
- `src/clustering/` - Groupement SimHash
- `src/intent/` - Détection d'intention LLM
- `src/adaptive/` - Prompts adaptatifs
- `src/learning/` - Système d'apprentissage complet
- `src/triggers/` - Déclencheurs intelligents
- `src/persistence/` - Base de données SQLite

### Frontend (React/TypeScript)
- `src/components/SettingsPanel.tsx` - Panel de paramètres
- `src/hooks/` - Hooks React pour triggers, health, etc.

---

## 📚 Documentation

- [J21.5 + J22 Documentation](./docs/J21_5_J22_COMPLETE.md)
- [Guide de test manuel](./manual_test_guide.md)
- [Plan de test complet](./TEST_COMPLETE.md)

---

## 🔧 Développement

### Structure du projet
```
shadowlearn/
├── src/                    # Frontend React
│   ├── components/        # Composants UI
│   ├── hooks/             # Hooks React
│   └── styles/            # Styles CSS
├── src-tauri/             # Backend Rust
│   ├── src/
│   │   ├── adaptive/      # J22: Adaptive Prompting
│   │   ├── clustering/    # J21: Clustering
│   │   ├── intent/        # J21: Intent Detection
│   │   ├── learning/      # Learning System
│   │   └── triggers/      # Trigger System
│   └── Cargo.toml         # Dépendances Rust
└── docs/                  # Documentation

```

### Compilation
```bash
# Backend uniquement
cd src-tauri && cargo build

# Frontend + Backend
pnpm tauri build
```

---

## 🎯 Prochaines étapes

### J23: Artifact Generation
- Génération d' artefacts contextuels
- Validation automatique
- Intégration avec prompts adaptatifs

### J24: Learning Loop
- Feedback utilisateur
- Trust scoring continu
- Ajustement adaptatif

---

## 📝 License

MIT

---

## 🙏 Contribution

Contributions bienvenues ! Voir les [issues](../../issues) pour les tâches en cours.

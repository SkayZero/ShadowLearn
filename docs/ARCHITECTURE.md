# 🏗️ ShadowLearn - Architecture Technique

## Vue d'Ensemble

ShadowLearn est un système d'apprentissage intelligent à 5 couches:

```
┌─────────────────────────────────────────┐
│  5. UI Layer (React + Tauri)           │
│     • ChatWindow, ContextWindow        │
│     • SuggestionBubble                  │
│     • LearningDashboard                 │
└────────────┬────────────────────────────┘
             ↓
┌─────────────────────────────────────────┐
│  4. Orchestration Layer                 │
│     • Process triggers end-to-end       │
│     • Telemetry collection               │
│     • Error handling                     │
└────────────┬────────────────────────────┘
             ↓
┌─────────────────────────────────────────┐
│  3. Learning Layer (Rust)               │
│     • Context → Clustering               │
│     • Intent Detection                   │
│     • Bandit (Thompson Sampling)         │
│     • Trust Scoring                      │
└────────────┬────────────────────────────┘
             ↓
┌─────────────────────────────────────────┐
│  2. Generation Layer                    │
│     • LLM Client (Ollama/OpenAI)        │
│     • Adaptive Prompting                 │
│     • Artefact Generator                 │
│     • Validator                          │
└────────────┬────────────────────────────┘
             ↓
┌─────────────────────────────────────────┐
│  1. Context Engine                       │
│     • OS Observation (App Detection)     │
│     • Clipboard Monitor                  │
│     • Context Aggregation                │
└──────────────────────────────────────────┘
```

---

## 🔑 Composants Clés

### 1. Context Engine (`src-tauri/src/context/`)

**Responsabilité**: Capture le contexte utilisateur

**Modules:**
- `app_detector.rs` - Détection application active
- `clipboard_monitor.rs` - Monitoring presse-papiers
- `aggregator.rs` - Agrégation du contexte

**API Principale:**
```rust
pub struct ContextAggregator {
    app_detector: AppDetector,
    clipboard_monitor: ClipboardMonitor,
}

impl ContextAggregator {
    pub async fn capture() -> Result<Context, Error>;
    pub async fn get_recent_contexts() -> Vec<Context>;
}
```

### 2. Learning Layer (`src-tauri/src/learning/`)

**Responsabilité**: Traitement intelligent du contexte

**Modules:**
- `clustering/` - Groupement SimHash (LSH)
- `intent/` - Détection d'intention (LLM)
- `bandit.rs` - Thompson Sampling
- `trust.rs` - Scoring utilisateur
- `anomaly.rs` - Détection anomalies

**Flow:**
```
Context → Clustering → Intent → Bandit → Trust Weight → Decision
```

### 3. Generation Layer

**Adaptive Prompting** (`src-tauri/src/adaptive/`)
- Prompts contextuels
- Templates adaptatifs
- Cache TTL 10min

**Artefact Generation** (`src-tauri/src/artefact/`)
- Types: Text, Blend, Midi, Shader, Json, Python
- LLM + Fallback
- Validation

### 4. Orchestration (`src-tauri/src/orchestrator.rs`)

**Responsabilité**: Workflow end-to-end

**Flow Complet:**
```rust
pub async fn process_trigger() -> Result<SuggestionResponse> {
    // 1. Capture context
    let ctx = capture_context().await?;
    
    // 2. Check trigger decision
    let decision = check_trigger(&ctx).await?;
    
    // 3. Process context (cluster + intent)
    let processed = process_context(&ctx).await?;
    
    // 4. Select artefact (bandit)
    let artefact_type = select_artefact(&processed).await?;
    
    // 5. Generate resource
    let artefact = generate_resource(&processed, &artefact_type).await?;
    
    // 6. Create suggestion
    let suggestion = create_suggestion(&ctx, &artefact).await?;
    
    // 7. Emit to frontend
    emit("suggestion_ready", &suggestion).await;
    
    Ok(SuggestionResponse { suggestion, artefact, context: ctx })
}
```

### 5. UI Layer

**React Components:**
- `ChatWindow.tsx` - Interface principale
- `ContextWindow.tsx` - Affichage contexte
- `SuggestionBubble.tsx` - Bulle de suggestion
- `LearningDashboard.tsx` - Dashboard dev

**Tauri Commands:**
- `generate_artifact()` - Génération
- `record_artifact_feedback()` - Feedback
- `get_health_status()` - Health
- `get_telemetry_stats()` - Stats

---

## 🔄 Flux de Données

### Pipeline Complet

```
1. Context Capture (3-5ms)
   ↓
2. Trigger Decision (50-100ms)
   ↓
3. Clustering (10-50ms)
   ↓
4. Intent Detection (500-2000ms LLM)
   ↓
5. Bandit Selection (1-5ms)
   ↓
6. Artefact Generation (1000-5000ms LLM)
   ↓
7. Validation (50-500ms)
   ↓
8. Storage + UI (10-20ms)
```

**Latence cible p95**: < 10s

### Outcome Recording

```
User Action → Outcome Record → Learning Update
                                    ↓
                            Trust Score Update
                                    ↓
                            Policy Update (Bandit)
```

---

## 💾 Base de Données

### Structure SQLite

**Tables Principales:**

```sql
-- Contexts
CREATE TABLE contexts (
    id TEXT PRIMARY KEY,
    app_name TEXT,
    domain TEXT,
    timestamp INTEGER,
    metadata TEXT
);

-- Suggestions
CREATE TABLE suggestions (
    id TEXT PRIMARY KEY,
    context_id TEXT,
    artefact_type TEXT,
    prompt_signature TEXT,
    confidence REAL,
    timestamp INTEGER
);

-- Outcomes
CREATE TABLE outcomes (
    id TEXT PRIMARY KEY,
    suggestion_id TEXT,
    used BOOLEAN,
    helpful BOOLEAN,
    reward REAL,
    timestamp INTEGER
);

-- Clusters
CREATE TABLE clusters (
    id TEXT PRIMARY KEY,
    fingerprint TEXT,
    size INTEGER,
    created_at INTEGER,
    updated_at INTEGER
);
```

### Accès

```rust
pub struct DatabaseManager {
    pool: sqlx::SqlitePool,
}

impl DatabaseManager {
    pub async fn store_context(&self, ctx: &Context) -> Result<()>;
    pub async fn get_recent_contexts(&self, limit: usize) -> Vec<Context>;
    pub async fn store_suggestion(&self, sug: &Suggestion) -> Result<()>;
    pub async fn record_outcome(&self, outcome: &Outcome) -> Result<()>;
}
```

---

## 🧪 Tests

### Structure

```
src-tauri/tests/
├── integration/
│   ├── context_capture.rs
│   ├── trigger_decision.rs
│   ├── learning_pipeline.rs
│   └── full_flow.rs
├── unit/
│   ├── clustering.rs
│   ├── bandit.rs
│   └── trust.rs
└── benches/
    ├── context_bench.rs
    └── learning_bench.rs
```

### Exécution

```bash
# Tests unitaires
cargo test --lib

# Tests d'intégration
cargo test --test '*'

# Benchmarks
cargo bench
```

---

## 🔧 Configuration

### Variables d'Environnement

```bash
# LLM Provider
export SL_LLM_PROVIDER=ollama     # ou 'openai'
export SL_LLM_MODEL=llama2       # modèle à utiliser

# Feature Flags
export SL_USE_INTENT_GATE=true    # Validation d'intent
export SL_SMART_TRIGGERS=true     # Triggers intelligents
export SL_TELEMETRY=true          # Collecte métriques

# Logs
export RUST_LOG=info              # Niveau de log
```

### Configuration Runtime

```rust
pub struct Config {
    pub idle_threshold_ms: u64,      // 12000 (12s)
    pub action_cooldown_ms: u64,     // 45000 (45s)
    pub dismiss_cooldown_ms: u64,    // 90000 (90s)
    pub trust_threshold: f32,        // 0.5
    pub max_clusters: usize,         // 1000
}
```

---

## 🚀 Déploiement

### Build

```bash
# macOS (universal)
cargo tauri build --target universal-apple-darwin

# Windows
cargo tauri build --target x86_64-pc-windows-msvc

# Linux
cargo tauri build --target x86_64-unknown-linux-gnu
```

### Distribution

- **macOS**: `.dmg` installer
- **Windows**: `.msi` installer
- **Linux**: `.AppImage` ou `.deb`

---

## 📊 Métriques

### Performance

- **Context Capture**: < 50ms
- **Clustering**: < 100ms
- **Intent Detection**: < 2000ms (LLM)
- **Bandit Selection**: < 10ms
- **Full Flow**: < 10s (p95)

### Ressources

- **RAM**: 50-200MB
- **CPU**: 1-5%
- **Stockage**: 10-100MB (selon usage)

### Qualité

- **Trust Score**: 0.0 - 1.0
- **Success Rate**: % suggestions positives
- **Cache Hit Rate**: > 40%

---

## 🔐 Sécurité

### Permissions

- **Capture d'écran** - Nécessaire pour contexte visuel
- **Accessibilité** - Optionnel pour idle detection
- **Fichiers** - Lecture/écriture artefacts

### Confidentialité

- ✅ Toutes données locales
- ✅ Pas de tracking
- ✅ LLM optionnel
- ✅ Open source

---

## 🛠️ Développement

### Setup

```bash
# Clone
git clone https://github.com/shadowlearn/shadowlearn

# Install dependencies
pnpm install
cd src-tauri
cargo build

# Run dev
pnpm tauri dev
```

### Structure Projet

```
shadowlearn/
├── src/                      # Frontend React
│   ├── components/          # Composants UI
│   ├── hooks/               # Hooks React
│   └── styles/              # CSS
├── src-tauri/src/           # Backend Rust
│   ├── context/             # Context Engine
│   ├── learning/            # Learning Layer
│   ├── orchestration/       # Orchestration
│   ├── telemetry/           # Telemetry
│   └── lib.rs               # Point d'entrée
├── docs/                    # Documentation
└── tests/                   # Tests
```

---

**Version**: 1.0.0  
**Architecture**: 5-layer intelligent learning system


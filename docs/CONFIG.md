# ⚙️ Configuration ShadowLearn

Guide complet de configuration de l'application.

---

## 📁 Variables d'environnement (.env)

### Fichier de configuration

```bash
# Copier le template
cp env.example .env

# Éditer
nano .env  # ou vim, code, etc.
```

### Features

```env
# Idle Detection - Monitore l'inactivité utilisateur
SL_IDLE_ENABLED=1

# Screenshot - Capture d'écran (nécessite permissions)
SL_SCREENSHOT_ENABLED=0  # OFF par défaut

# Smart Triggers - Système de triggers intelligents
# Dépend de: Idle Detection
SL_SMART_TRIGGERS_ENABLED=1

# Telemetry - Métriques de performance (local uniquement)
SL_TELEMETRY=1
```

### Comportement

```env
# Seuil d'inactivité (millisecondes)
SL_IDLE_MS=12000  # 12 secondes

# Cooldown entre triggers
SL_COOLDOWN_MS=45000  # 45 secondes

# Cooldown après dismiss
SL_COOLDOWN_AFTER_DISMISS_MS=90000  # 90 secondes
```

### Screenshot

```env
# Timeout de capture
SL_SCREENSHOT_TIMEOUT_MS=800  # 800ms

# Facteur d'échelle (1-4)
SL_SCREENSHOT_SCALE=3  # Qualité élevée
```

### Logging

```env
# Niveau de log via RUST_LOG
RUST_LOG=info

# Debug d'un module spécifique
RUST_LOG=shadowlearn::health=debug

# Logs détaillés
RUST_LOG=trace
```

---

## 🏗️ Configuration runtime (futur)

### Fichier JSON persistant

**Location** : `~/Library/Application Support/ShadowLearn/config.json`

```json
{
  "features": {
    "idle_detection": true,
    "screenshot": false,
    "smart_triggers": true,
    "telemetry": true
  },
  "behavior": {
    "idle_threshold_ms": 12000,
    "cooldown_ms": 45000,
    "cooldown_after_dismiss_ms": 90000
  },
  "screenshot": {
    "enabled": false,
    "timeout_ms": 800,
    "scale": 3
  }
}
```

**Note** : Cette configuration sera implémentée plus tard avec une UI de settings.

---

## 🔗 Dépendances entre features

### Smart Triggers → Idle Detection

```
SmartTriggers requires IdleDetection

Si IdleDetection disabled:
  → SmartTriggers auto-disabled
  → Log: "Cascading: disabling Smart Triggers"
```

### Activer Smart Triggers

```bash
# 1. Activer Idle Detection d'abord
SL_IDLE_ENABLED=1

# 2. Puis Smart Triggers
SL_SMART_TRIGGERS_ENABLED=1
```

### Vérifier les dépendances

```tsx
// DevTools Console
const state = await invoke('get_features_state')

if (!state.smart_triggers && state.idle_detection) {
  // Peut être réactivé
  await invoke('toggle_feature', {
    feature: 'smart_triggers',
    enabled: true
  })
}
```

---

## 🎛️ Toggle features en runtime

### Via commandes Tauri

```tsx
// Activer une feature
await invoke('toggle_feature', {
  feature: 'screenshot',  // ou 'idle_detection', 'smart_triggers', 'telemetry'
  enabled: true
})

// Désactiver
await invoke('toggle_feature', {
  feature: 'screenshot',
  enabled: false
})

// Get state
const state = await invoke('get_features_state')
console.log(state)
```

### Valeurs de features

```typescript
type Feature = 
  | 'idle_detection'
  | 'screenshot'
  | 'smart_triggers'
  | 'telemetry'
```

---

## 📊 Profils de configuration

### Development (Default)

```env
SL_IDLE_ENABLED=1
SL_SCREENSHOT_ENABLED=0  # Pas de permissions
SL_SMART_TRIGGERS_ENABLED=1
SL_TELEMETRY=1
SL_IDLE_MS=5000  # Plus rapide pour tests
RUST_LOG=debug
```

### Production

```env
SL_IDLE_ENABLED=1
SL_SCREENSHOT_ENABLED=1  # Si permissions accordées
SL_SMART_TRIGGERS_ENABLED=1
SL_TELEMETRY=1
SL_IDLE_MS=12000
RUST_LOG=info
```

### Performance (Low resources)

```env
SL_IDLE_ENABLED=1
SL_SCREENSHOT_ENABLED=0  # Économise CPU/RAM
SL_SMART_TRIGGERS_ENABLED=0
SL_TELEMETRY=0
SL_IDLE_MS=15000  # Polling moins fréquent
SL_IDLE_POLL_MS=1000
```

### Debug

```env
SL_IDLE_ENABLED=1
SL_SCREENSHOT_ENABLED=1
SL_SMART_TRIGGERS_ENABLED=1
SL_TELEMETRY=1
SL_IDLE_MS=5000
RUST_LOG=trace
```

---

## 🔧 Configuration avancée

### Tuning performance

```env
# Polling interval (défaut: 500ms)
SL_IDLE_POLL_MS=500

# Max tentatives de recovery (défaut: 3)
SL_MAX_RECOVERY_ATTEMPTS=3

# Taille du buffer telemetry
SL_TELEMETRY_BUFFER_SIZE=1000
SL_TELEMETRY_HISTOGRAM_SIZE=100
```

### Allowlist applications (futur)

```json
{
  "allowlist": [
    "com.google.Chrome",
    "com.microsoft.VSCode",
    "com.apple.dt.Xcode"
  ],
  "blocklist": [
    "com.apple.loginwindow",
    "com.apple.systempreferences"
  ]
}
```

---

## ✅ Validation de configuration

### Check au démarrage

Logs affichés au lancement :

```
🚀 Starting ShadowLearn...
🚩 Feature flags initialized:
  ├─ Idle Detection: true
  ├─ Screenshot: false
  ├─ Smart Triggers: true
  └─ Telemetry: true
✅ Features enabled: 3/4
```

### Tester la config

```tsx
// 1. Get state
const state = await invoke('get_features_state')

// 2. Verify
console.assert(state.idle_detection === true, 'Idle must be enabled')
console.assert(state.smart_triggers === true, 'Triggers must be enabled')

// 3. Check dependencies
if (state.smart_triggers && !state.idle_detection) {
  console.error('Invalid state: Smart Triggers requires Idle Detection')
}
```

---

## 📝 Exemple complet

### .env pour développement

```env
# ============================================
# ShadowLearn Dev Config
# ============================================

# Features
SL_IDLE_ENABLED=1
SL_SCREENSHOT_ENABLED=0
SL_SMART_TRIGGERS_ENABLED=1
SL_TELEMETRY=1

# Behavior (plus rapide pour tests)
SL_IDLE_MS=5000
SL_COOLDOWN_MS=15000
SL_COOLDOWN_AFTER_DISMISS_MS=30000

# Screenshot
SL_SCREENSHOT_TIMEOUT_MS=500
SL_SCREENSHOT_SCALE=2

# Logging
RUST_LOG=debug

# Advanced
SL_IDLE_POLL_MS=500
```

### Lancer avec cette config

```bash
# Charger le .env
source .env

# Lancer l'app
cargo tauri dev
```

---

## 🚀 Quick Start

```bash
# 1. Copier le template
cp env.example .env

# 2. Activer les features de base
echo "SL_IDLE_ENABLED=1" > .env
echo "SL_SMART_TRIGGERS_ENABLED=1" >> .env
echo "SL_TELEMETRY=1" >> .env
echo "RUST_LOG=info" >> .env

# 3. Lancer
cargo tauri dev
```


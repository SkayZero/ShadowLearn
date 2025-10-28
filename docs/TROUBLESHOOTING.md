# 🔧 Troubleshooting ShadowLearn

Guide de résolution des problèmes courants.

---

## 🚫 Bulle ne s'affiche pas

### Causes possibles

1. **Application pas dans allowlist**
   - Solution : Ajouter l'app dans la configuration
   
2. **Cooldown actif**
   - Attendre 45 secondes entre les triggers
   - Vérifier dans les logs : `Cooldown active`

3. **Idle detection désactivée**
   - Vérifier les feature flags
   - Check `.env` : `SL_IDLE_ENABLED=1`

### Debug

```bash
# Lancer avec logs détaillés
RUST_LOG=info cargo tauri dev

# Chercher dans les logs
# Look for: "🚩 Feature flags initialized"
# Look for: "Idle Detection: true"
```

### Solutions

```bash
# Reset l'état de l'application
rm ~/Library/Application\ Support/ShadowLearn/state.json

# Forcer la réactivation
SL_IDLE_ENABLED=1 cargo tauri dev

# Vérifier le statut des features
# Dans DevTools Console:
await invoke('get_features_state')
```

---

## 📷 Screenshot désactivé

### C'est normal !

Screenshot est **OFF par défaut** car nécessite des permissions système.

### Pour activer

1. **Accorder les permissions système**
   ```
   System Preferences → Privacy & Security → Screen Recording
   ✓ Cocher ShadowLearn
   ```

2. **Activer dans la configuration**
   ```bash
   # Dans .env
   SL_SCREENSHOT_ENABLED=1
   ```

3. **Redémarrer l'application**
   ```bash
   cargo tauri dev
   ```

### Mode fallback

L'app fonctionne **sans screenshot** :
- Bulle s'affiche normalement
- Pas de vignette d'écran
- Toutes les autres features disponibles

---

## ⚡ Latence élevée

### Cibles de performance

- **Trigger → UI** : < 150ms
- **Screenshot** : < 800ms
- **Health check** : < 50ms

### Vérifier les performances

```tsx
// Dans DevTools Console
const stats = await invoke('get_telemetry_stats')
console.log('p95:', stats.global.p95, 'ms')
```

### Solutions

#### 1. Ralentir le polling si CPU élevé

```env
# Dans .env
SL_IDLE_POLL_MS=1000
```

#### 2. Désactiver screenshot

```env
SL_SCREENSHOT_ENABLED=0
```

#### 3. Augmenter le cooldown

```env
SL_COOLDOWN_MS=60000  # 60 secondes
```

---

## 🔴 Features auto-disabled

### Pourquoi ?

Quand un composant fail **3 fois** consécutivement :
- Auto-disable pour stabilité
- Permet à l'app de continuer
- Évite les crash loops

### Messages dans les logs

```
❌ Max restart attempts (3) reached for Idle Detector. Feature will be disabled.
🚫 Idle Detection feature disabled after max restart attempts
⚠️  Cascading: disabling Smart Triggers (depends on Idle Detection)
```

### Re-enable

#### Simple restart
```bash
cargo tauri dev
```

#### Force enable
```bash
SL_IDLE_ENABLED=1 cargo tauri dev
```

### Vérifier l'état

```tsx
// DevTools Console
const state = await invoke('get_features_state')
console.log(state)
// {
//   idle_detection: true,
//   screenshot: false,
//   smart_triggers: true,
//   telemetry: true
// }
```

### Recovery stats

```tsx
// DevTools Console
const recovery = await invoke('get_recovery_stats')
console.log(recovery)
// {
//   idle_detector_restarts: 2,
//   screenshot_restarts: 0,
//   max_restarts: 3
// }
```

---

## 📋 Debug logs

### Niveaux de log

```bash
# Tous les logs
RUST_LOG=debug cargo tauri dev

# Info only (recommandé)
RUST_LOG=info cargo tauri dev

# Errors only
RUST_LOG=error cargo tauri dev

# Module spécifique
RUST_LOG=shadowlearn=debug cargo tauri dev
```

### Localisation des logs

**Console** : Terminal où l'app est lancée

**Fichiers** (future) :
- `~/Library/Logs/ShadowLearn/shadowlearn.log`
- Rotation automatique
- Max 10MB par fichier

---

## 🔍 DevStats ne s'affiche pas

### C'est normal en production !

DevStats est visible **uniquement en mode développement**.

### Pour voir DevStats

```bash
# Mode dev
cargo tauri dev
# ou
pnpm tauri dev
```

### Position

- **Coin bottom-right** de la fenêtre Chat
- **Pliable/dépliable** avec bouton +/-
- **Cachable** avec bouton ×

---

## 🆘 Problèmes non résolus ?

1. **Check les logs** : `RUST_LOG=debug cargo tauri dev`
2. **Vérifier les permissions** : Screen Recording, Accessibility
3. **Reset l'état** : `rm ~/Library/Application\ Support/ShadowLearn/state.json`
4. **Issue GitHub** : Créer une issue avec les logs

### Infos utiles pour report

```bash
# Version
cargo tauri info

# Logs complets
RUST_LOG=trace cargo tauri dev > debug.log 2>&1

# State des features
await invoke('get_features_state')

# Stats recovery
await invoke('get_recovery_stats')

# Stats telemetry
await invoke('get_telemetry_stats')
```


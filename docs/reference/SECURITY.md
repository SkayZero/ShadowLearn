# SECURITY.md — Sécurité & Confidentialité

> **Rôle** : Documentation sécurité, permissions, privacy zones
> **Public** : Dev, auditeurs sécurité, contributeurs
> **Importance** : TRÈS HAUTE — Critique pour confiance utilisateur

**🎯 Ce fichier répond à : "Comment ShadowLearn protège les données sensibles ?"**

---

## 📋 Table des matières

1. [Modèle de menace](#modèle-de-menace)
2. [Architecture sécurité](#architecture-sécurité)
3. [Permissions système](#permissions-système)
4. [Privacy Zones](#privacy-zones)
5. [Gestion des données](#gestion-des-données)
6. [Cryptographie](#cryptographie)
7. [Sécurité réseau](#sécurité-réseau)
8. [Bonnes pratiques](#bonnes-pratiques)
9. [Audit & Compliance](#audit--compliance)
10. [Checklist sécurité](#checklist-sécurité)

---

## Modèle de menace

### Acteurs & Risques

**ShadowLearn est une app locale qui observe l'activité utilisateur**. Voici les risques identifiés :

| Risque | Impact | Mitigation |
|--------|--------|------------|
| **Fuite données sensibles** | CRITIQUE | Privacy zones, pas de network, encryption SQLite |
| **Screenshot apps sensibles** | HAUTE | Privacy zones bloquent screenshot |
| **Clipboard avec passwords** | HAUTE | Filtre regex passwords, pas de log clipboard |
| **Exfiltration données** | CRITIQUE | Pas de network calls, local-only |
| **Malware lecture DB** | MOYENNE | SQLite encryption, keychain stockage |
| **Accès non autorisé** | MOYENNE | Permissions macOS, sandboxing Tauri |

---

### Principes de sécurité

**ShadowLearn suit ces principes ABSOLUS** :

1. ✅ **Local-first** : Aucune donnée envoyée sur réseau (sauf LLM opt-in)
2. ✅ **Privacy by design** : Privacy zones pour apps sensibles (Banking, 1Password, etc.)
3. ✅ **Minimal permissions** : Seulement Accessibility + Screen Recording (macOS)
4. ✅ **Transparent** : User sait toujours ce qui est capturé
5. ✅ **Opt-in** : Features sensibles (screenshot) désactivables
6. ✅ **Encryption at rest** : SQLite database encryptée (quand implémenté)
7. ✅ **No logs** : Pas de logging données sensibles (passwords, API keys)

---

## Architecture sécurité

### Composants & Trust Boundaries

```
┌──────────────────────────────────────────────────┐
│  User Space (Untrusted)                          │
│  ┌────────────────┐                              │
│  │  macOS Apps    │ ← Monitored                  │
│  └────────────────┘                              │
└──────────────────────────────────────────────────┘
                    ↓ (Accessibility API)
┌──────────────────────────────────────────────────┐
│  ShadowLearn (Trusted)                           │
│  ┌──────────────┐  ┌──────────────┐             │
│  │  Frontend    │  │  Backend     │             │
│  │  (WebView)   │←→│  (Rust)      │             │
│  └──────────────┘  └──────────────┘             │
│        ↓                   ↓                     │
│  ┌──────────────┐  ┌──────────────┐             │
│  │  Tauri IPC   │  │  Privacy     │             │
│  │  (Commands)  │  │  Zones       │             │
│  └──────────────┘  └──────────────┘             │
│                           ↓                      │
│                   ┌──────────────┐               │
│                   │  SQLite DB   │               │
│                   │  (Encrypted) │               │
│                   └──────────────┘               │
└──────────────────────────────────────────────────┘
                    ↓ (Optional, User Consent)
┌──────────────────────────────────────────────────┐
│  External (Network)                              │
│  ┌──────────────┐                                │
│  │  LLM API     │ (OpenAI, Anthropic)            │
│  └──────────────┘                                │
└──────────────────────────────────────────────────┘
```

### Trust Boundaries

**Frontend (WebView)** :
- ❌ Ne peut PAS accéder directement au système
- ❌ Ne peut PAS lire fichiers sans permission
- ✅ Communique UNIQUEMENT via Tauri commands (whitelist)

**Backend (Rust)** :
- ✅ Accès système via Accessibility + Screen Recording
- ✅ Gère Privacy Zones (filtrage apps sensibles)
- ✅ Accès SQLite database

**IPC (Tauri)** :
- ✅ Whitelist stricte de commandes
- ✅ Validation paramètres
- ✅ Pas de commandes dangereuses exposées (ex: `exec`, `fs:write` sans validation)

---

## Permissions système

### macOS Permissions

ShadowLearn nécessite **2 permissions macOS** :

#### 1. Accessibility Permission

**Pourquoi** : Permet de lire app active + window title.

**Ce qui est accessible** :
- ✅ Nom de l'app active (`com.apple.Terminal`)
- ✅ Titre de la fenêtre (`~/Documents`)
- ❌ Contenu fenêtre (pas d'access au texte)

**Vérification** :
```typescript
import { invoke } from '@tauri-apps/api/core';

const permissions = await invoke<PermissionsStatus>('check_permissions');
console.error('Accessibility:', permissions.accessibility);
```

**Demander permission** :
```typescript
await invoke('request_accessibility_permission');
// Ouvre System Settings > Privacy & Security > Accessibility
```

---

#### 2. Screen Recording Permission

**Pourquoi** : Permet de capturer screenshots.

**Ce qui est accessible** :
- ✅ Screenshot de l'écran actif (si feature activée)
- ❌ PAS de screenshot si Privacy Zone active

**Vérification** :
```typescript
const hasPermission = await invoke<boolean>('check_screenshot_permission');
console.error('Screen Recording:', hasPermission);
```

**Demander permission** :
```typescript
await invoke('request_screenshot_permission');
// Ouvre System Settings > Privacy & Security > Screen Recording
```

---

### Feature Flags Sécurité

**Feature flags permettent de désactiver fonctions sensibles** :

```typescript
interface FeaturesState {
  idle_detection: boolean;      // Détection inactivité
  screenshot: boolean;          // 🔴 Capture screenshot (SENSIBLE)
  smart_triggers: boolean;      // Triggers automatiques
  telemetry: boolean;          // Télémétrie usage
  use_intent_gate: boolean;    // Intent gate pour LLM
}
```

**Désactiver screenshot** :
```typescript
await invoke('disable_feature', { feature: 'screenshot' });
```

**Fichier** : `src-tauri/src/features/mod.rs`

---

## Privacy Zones

### Concept

**Privacy Zones = Apps protégées où ShadowLearn n'observe RIEN**.

**Exemples d'apps sensibles** :
- Banking apps (Chase, Bank of America)
- Password managers (1Password, LastPass, Bitwarden)
- Browsers en mode incognito
- Apps santé (Health, Therapy apps)

---

### Fonctionnement

**Quand une app est dans Privacy Zone** :

1. ✅ **Triggers bloqués** - Aucune opportunité affichée
2. ✅ **Screenshot bloqué** - `capture_screenshot` retourne erreur
3. ✅ **Context non capturé** - App name + window title = `null`
4. ✅ **Clipboard ignoré** - Pas de monitoring clipboard
5. ✅ **HUD = État "blocked"** - LED rouge

---

### Configuration

**Ajouter app à Privacy Zone** :

```typescript
await invoke('add_privacy_zone', { appName: '1Password' });
```

**Retirer app** :

```typescript
await invoke('remove_privacy_zone', { appName: '1Password' });
```

**Vérifier si app protégée** :

```typescript
const isProtected = await invoke<boolean>('is_app_protected', {
  appName: 'com.apple.Safari',
});
console.error('Safari protected:', isProtected);
```

**Toggle global Privacy Zones** :

```typescript
await invoke('set_privacy_zones_enabled', { enabled: true });
```

---

### Apps par défaut

**Privacy Zones activées par défaut pour** :

```rust
// src-tauri/src/privacy/mod.rs
const DEFAULT_PROTECTED_APPS: &[&str] = &[
    "1Password",
    "LastPass",
    "Bitwarden",
    "com.apple.keychainaccess",
    "Banking",
    "Chase",
    "com.apple.private.browsing", // Safari Private
];
```

**User peut ajouter/retirer apps via Settings**.

---

### Enforcement

**Backend vérifie Privacy Zone à chaque capture** :

```rust
// src-tauri/src/privacy/mod.rs
impl PrivacyZoneManager {
    pub fn is_app_protected(&self, app_name: &str) -> bool {
        if !self.config.enabled {
            return false; // Privacy zones désactivées
        }

        self.config.protected_apps.contains(&app_name.to_string())
    }
}
```

**Utilisé dans** :
- `src-tauri/src/context/aggregator.rs` - Filtrage contexte
- `src-tauri/src/screenshot/mod.rs` - Blocage screenshot
- `src-tauri/src/triggers/trigger_loop.rs` - Blocage triggers

---

## Gestion des données

### SQLite Database

**Emplacement** :
- macOS : `~/.local/share/com.shadowlearn.app/shadowlearn.db`
- Linux : `~/.local/share/ShadowLearn/shadowlearn.db`
- Windows : `%APPDATA%\com.shadowlearn.app\shadowlearn.db`

**Contenu** :
```sql
-- Conversations LLM
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    app_context TEXT,
    created_at INTEGER NOT NULL
);

-- Messages
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL, -- "user" | "assistant" | "system"
    content TEXT NOT NULL,
    metadata TEXT,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Contextes capturés
CREATE TABLE captured_contexts (
    id TEXT PRIMARY KEY,
    app_name TEXT NOT NULL,
    window_title TEXT,
    clipboard TEXT,
    idle_seconds INTEGER NOT NULL,
    timestamp INTEGER NOT NULL
);

-- ML patterns
CREATE TABLE ml_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    app_name TEXT NOT NULL,
    context TEXT,
    user_response TEXT,
    timestamp INTEGER NOT NULL
);
```

---

### Données sensibles

**⚠️ Ces données peuvent être sensibles** :

| Donnée | Sensibilité | Mitigation |
|--------|-------------|------------|
| `window_title` | HAUTE | Privacy zones filtrent |
| `clipboard` | CRITIQUE | Regex filtre passwords, opt-in |
| `screenshot` | CRITIQUE | Privacy zones bloquent, opt-in |
| `messages.content` | HAUTE | Pas d'envoi réseau (sauf LLM opt-in) |

---

### Filtrage Clipboard

**Regex pour détecter données sensibles** :

```rust
// src-tauri/src/context/aggregator.rs
const SENSITIVE_PATTERNS: &[&str] = &[
    r"(?i)password[:=]\s*\S+",         // password=abc123
    r"(?i)api[_-]?key[:=]\s*\S+",     // api_key=sk-...
    r"(?i)secret[:=]\s*\S+",          // secret=xyz
    r"(?i)token[:=]\s*\S+",           // token=eyJ...
    r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b", // emails
    r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b", // credit cards
];

fn is_sensitive_clipboard(text: &str) -> bool {
    SENSITIVE_PATTERNS.iter().any(|pattern| {
        Regex::new(pattern).unwrap().is_match(text)
    })
}
```

**Si clipboard sensible détecté** :
- ✅ Clipboard = `null` dans contexte
- ✅ Log warning : `"⚠️ Sensitive clipboard data filtered"`
- ✅ Pas de sauvegarde dans DB

---

### Logging Policy

**❌ NE JAMAIS logger** :
- Clipboard content
- Window titles complets (seulement app name dans certains logs)
- Message content utilisateur
- API keys, tokens, passwords

**✅ Logger seulement** :
- Event types (`"trigger_fired"`, `"screenshot_captured"`)
- App names (sauf si Privacy Zone)
- Timestamps, counts, stats
- Errors (sans données sensibles)

**Exemple BON** :
```rust
info!("✅ Trigger fired for app: {}", app_name);
```

**Exemple MAUVAIS** :
```rust
// ❌ NE JAMAIS FAIRE
error!("Error processing clipboard: {}", clipboard_content);
```

---

## Cryptographie

### Keychain (macOS)

**ShadowLearn utilise macOS Keychain pour stocker secrets** :

**Commandes** :
```typescript
// Vérifier keychain status
const status = await invoke<KeychainStatus>('check_keychain_status');
console.error('Keychain available:', status.available);
```

**Fichier** : `src-tauri/src/crypto/keymanager.rs`

**Ce qui est stocké** :
- API keys LLM (si utilisateur configure)
- Secrets plugins (Phase 4)

**Implémentation** :
```rust
use security_framework::passwords::{get_generic_password, set_generic_password};

pub fn store_api_key(service: &str, account: &str, key: &str) -> Result<(), String> {
    set_generic_password(service, account, key.as_bytes())
        .map_err(|e| e.to_string())
}

pub fn get_api_key(service: &str, account: &str) -> Result<String, String> {
    let password = get_generic_password(service, account)
        .map_err(|e| e.to_string())?;
    String::from_utf8(password.to_vec()).map_err(|e| e.to_string())
}
```

---

### SQLite Encryption

**🚧 TODO : Implémenter SQLite encryption avec SQLCipher**.

**Plan** :
1. Utiliser `sqlx` avec feature `sqlcipher`
2. Générer clé encryption via macOS Keychain
3. Encrypt database at rest

**Référence** :
- [SQLCipher](https://www.zetetic.net/sqlcipher/)
- Clé stockée dans Keychain : `com.shadowlearn.app.db_key`

---

## Sécurité réseau

### Politique réseau

**Par défaut : AUCUN appel réseau**.

**ShadowLearn est 100% local-only**, sauf :

1. ✅ **LLM API** (opt-in utilisateur)
   - OpenAI API
   - Anthropic API
   - Uniquement si utilisateur configure API key

2. ❌ **Pas de télémétrie externe**
3. ❌ **Pas d'analytics tiers**
4. ❌ **Pas de crash reporting externe**

---

### LLM API Calls

**Quand utilisateur active LLM** :

**Données envoyées** :
- Message utilisateur
- Contexte app (nom app seulement, pas window title)
- Conversation history (si multi-turn)

**Données JAMAIS envoyées** :
- Screenshots
- Clipboard content
- Window titles
- Apps dans Privacy Zones

**Code** :
```rust
// src-tauri/src/chat/mod.rs
pub async fn chat_with_ai(
    message: String,
    context: Option<String>,
) -> Result<String, String> {
    // Vérifier opt-in utilisateur
    if !config.llm_enabled {
        return Err("LLM disabled by user".into());
    }

    // Filtrer données sensibles du contexte
    let safe_context = filter_sensitive_context(context);

    // Appel API
    let response = llm_client
        .send_message(message, safe_context)
        .await?;

    Ok(response)
}
```

**Fichier config** : `~/.config/shadowlearn/config.json`
```json
{
  "llm_enabled": false,
  "llm_provider": "openai",
  "llm_api_key": "stored-in-keychain"
}
```

---

### HTTPS Only

**Si LLM activé** :
- ✅ Tous appels réseau en HTTPS
- ✅ Certificate pinning (TODO)
- ✅ Timeout 30s
- ✅ Retry 3x avec backoff

---

## Bonnes pratiques

### Pour développeurs

#### ✅ DO

1. **Toujours vérifier Privacy Zones** avant capture
   ```rust
   if privacy_manager.is_app_protected(&app_name) {
       return Err("App is in privacy zone".into());
   }
   ```

2. **Toujours filtrer clipboard** avec regex sensitive patterns

3. **Utiliser feature flags** pour désactiver fonctions sensibles
   ```rust
   if !feature_flags.is_enabled(Feature::Screenshot) {
       return Err("Screenshot disabled".into());
   }
   ```

4. **Logger sans données sensibles**
   ```rust
   info!("Context captured for app: {}", app_name);
   // PAS: info!("Context: {:?}", context);
   ```

5. **Valider paramètres Tauri commands**
   ```rust
   #[tauri::command]
   fn show_window(window_label: String) -> Result<(), String> {
       if window_label.is_empty() {
           return Err("Invalid window label".into());
       }
       // ...
   }
   ```

---

#### ❌ DON'T

1. ❌ **NE JAMAIS logger clipboard/passwords/API keys**

2. ❌ **NE JAMAIS bypass Privacy Zones**

3. ❌ **NE JAMAIS exposer commandes dangereuses** (`fs:write`, `shell:exec`)

4. ❌ **NE JAMAIS envoyer screenshots sur réseau** sans opt-in explicite

5. ❌ **NE JAMAIS stocker API keys en plaintext**
   ```rust
   // ❌ MAUVAIS
   let api_key = "sk-abc123";

   // ✅ BON
   let api_key = keychain::get_api_key("openai", "user")?;
   ```

---

### Pour utilisateurs

#### 🔒 Sécuriser ShadowLearn

1. **Configurer Privacy Zones** pour apps sensibles (Banking, Password managers)

2. **Vérifier feature flags** - Désactiver screenshot si pas nécessaire

3. **Limiter permissions macOS** - Retirer Accessibility si app non utilisée

4. **Exporter données régulièrement**
   ```typescript
   await invoke('export_data', { filePath: '/backup/shadowlearn-backup.json' });
   ```

5. **Vérifier config file**
   ```bash
   cat ~/.config/shadowlearn/config.json
   ```

---

## Audit & Compliance

### Audit Checklist

**Pour audit sécurité, vérifier** :

- [ ] Privacy Zones actives par défaut
- [ ] Clipboard filtering fonctionne (test regex)
- [ ] Screenshot bloqué si Privacy Zone
- [ ] Permissions macOS demandées correctement
- [ ] Pas de logs clipboard/passwords
- [ ] Feature flags respectés
- [ ] SQLite database permissions : `chmod 600`
- [ ] Config file permissions : `chmod 600`
- [ ] Keychain utilisé pour API keys
- [ ] HTTPS only pour LLM calls
- [ ] Pas de télémétrie externe sans opt-in

---

### Tests sécurité

**Fichiers de tests** :
- `src-tauri/src/privacy/tests.rs` - Privacy Zones
- `src-tauri/src/context/tests.rs` - Clipboard filtering
- `src-tauri/src/permissions/tests.rs` - Permissions checking

**Lancer tests** :
```bash
cd src-tauri
cargo test --all-features
```

---

### Compliance GDPR

**ShadowLearn est GDPR-compliant par design** :

| Principe GDPR | Implémentation |
|---------------|----------------|
| **Minimisation données** | Seulement app name + idle time capturés |
| **Droit à l'oubli** | Commande `export_data` + suppression manuelle DB |
| **Transparence** | Docs complètes sur données capturées |
| **Sécurité** | SQLite encryption, Keychain, Privacy Zones |
| **Consentement** | Feature flags + opt-in LLM |

---

### Threat Model Update

**Dernière révision** : 2025-01-21

**Changements récents** :
- Ajout Privacy Zones (J5)
- Clipboard filtering regex
- Feature flags pour screenshot

**Prochaines révisions** :
- SQLite encryption (J25)
- Certificate pinning LLM API
- Audit externe sécurité

---

## Checklist sécurité

### Pour release

Avant chaque release, vérifier :

#### 🔐 Permissions

- [ ] Permissions macOS documentées dans README
- [ ] Dialogs permissions clairs (pas de surprise)
- [ ] Permissions minimales (pas de Camera, Microphone, Location)

#### 🛡️ Privacy

- [ ] Privacy Zones activées par défaut
- [ ] Apps sensibles dans liste par défaut
- [ ] Screenshot feature désactivable
- [ ] Clipboard filtering actif

#### 💾 Données

- [ ] SQLite database permissions `600`
- [ ] Config file permissions `600`
- [ ] Pas de logs données sensibles
- [ ] Export data fonctionne

#### 🔑 Crypto

- [ ] API keys dans Keychain (pas plaintext)
- [ ] SQLite encryption (TODO J25)
- [ ] HTTPS only pour réseau

#### 📡 Réseau

- [ ] Pas de télémétrie par défaut
- [ ] LLM opt-in seulement
- [ ] Pas d'analytics tiers
- [ ] Certificate validation HTTPS

#### 🧪 Tests

- [ ] Tests Privacy Zones passent
- [ ] Tests clipboard filtering passent
- [ ] Tests permissions passent
- [ ] Audit sécurité manuel fait

---

## 🔗 Ressources

- [Tauri Security Best Practices](https://v2.tauri.app/security/)
- [macOS Permissions Guide](https://developer.apple.com/documentation/security)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [GDPR Compliance](https://gdpr.eu/)
- [SQLCipher Docs](https://www.zetetic.net/sqlcipher/)

---

**🔒 ShadowLearn prend la sécurité et la confidentialité au sérieux. Si vous découvrez une vulnérabilité, contactez : security@shadowlearn.app**

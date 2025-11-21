# SETUP.md — Installation & Configuration

> **Rôle** : Guide complet pour setup l'environnement de développement
> **Public** : Nouveau dev, contributeur
> **Importance** : HAUTE — Nécessaire pour lancer le projet

---

## 📋 Table des matières

1. [TL;DR — Démarrage en 5 minutes](#tldr--démarrage-en-5-minutes)
2. [Setup complet](#setup-complet)
3. [Variables d'environnement](#variables-denvironnement)
4. [Commandes utiles](#commandes-utiles)
5. [Troubleshooting](#troubleshooting)

---

## TL;DR — Démarrage en 5 minutes

**Prérequis rapides** : Node 22+, Rust stable, pnpm

```bash
# 1. Clone
git clone <repo>
cd ShadowLearn

# 2. Install dépendances
pnpm install

# 3. Lance en dev
pnpm tauri dev

# ✅ L'app devrait démarrer avec :
# - Fenêtre chat (main)
# - HUD (petit cercle en haut à droite)
# - Spotlight accessible via Cmd+Shift+Y
```

**Si ça fonctionne pas** → Voir [Troubleshooting](#troubleshooting)

---

## Setup complet

### 1. Prérequis système

#### Node.js & pnpm

```bash
# Node 22+ requis
node --version  # >= 22.0.0

# Installer pnpm si besoin
npm install -g pnpm

# Vérifier pnpm
pnpm --version  # >= 8.0.0
```

#### Rust

```bash
# Installer Rust (rustup recommandé)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Vérifier installation
rustc --version  # >= 1.75.0
cargo --version  # >= 1.75.0

# Update Rust si besoin
rustup update stable
```

#### Dépendances système (macOS)

```bash
# Tauri nécessite ces dépendances macOS
# Normalement déjà présentes sur macOS récent

# Si build errors, installer Xcode Command Line Tools
xcode-select --install
```

#### Dépendances système (Linux)

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel

# Arch
sudo pacman -S webkit2gtk \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    appmenu-gtk-module \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    libvips
```

#### Dépendances système (Windows)

```powershell
# Installer Visual Studio C++ Build Tools
# https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Installer WebView2 (normalement préinstallé sur Windows 11)
# https://developer.microsoft.com/microsoft-edge/webview2/
```

---

### 2. Installation du projet

```bash
# Clone le repo
git clone <repo-url>
cd ShadowLearn

# Installer dépendances frontend
pnpm install

# Les dépendances Rust sont gérées par Cargo automatiquement
```

---

### 3. Configuration

#### Base de données SQLite

```bash
# La DB SQLite est créée automatiquement au premier lancement
# Emplacement: ~/.local/share/com.shadowlearn.app/shadowlearn.db (Linux/macOS)
#           ou: %APPDATA%\com.shadowlearn.app\shadowlearn.db (Windows)

# Pas de migration manuelle nécessaire pour l'instant
```

#### Permissions macOS

Sur macOS, pour que les shortcuts globaux fonctionnent :

1. **System Settings** > **Privacy & Security** > **Accessibility**
2. Ajouter **Terminal.app** (ou votre terminal)
3. Redémarrer le terminal
4. Lancer `pnpm tauri dev`

---

### 4. Lancement

#### Mode développement

```bash
# Lance app avec hot reload
pnpm tauri dev

# Logs détaillés
RUST_LOG=debug pnpm tauri dev

# Lance seulement frontend (sans Tauri)
pnpm dev
```

#### Build production

```bash
# Build frontend
pnpm build

# Build app complète (frontend + backend)
pnpm tauri build

# L'app compilée sera dans:
# - macOS: src-tauri/target/release/bundle/macos/
# - Linux: src-tauri/target/release/bundle/appimage/
# - Windows: src-tauri/target/release/bundle/msi/
```

#### Scripts utiles

```bash
# Linter TypeScript
pnpm lint

# Format code
pnpm format

# Tests (quand implémentés)
pnpm test

# Type check
pnpm tsc --noEmit

# Clean build artifacts
./clean-build.sh  # macOS/Linux
```

---

## Variables d'environnement

### Optionnelles

| Variable | Rôle | Valeur par défaut |
|----------|------|-------------------|
| `RUST_LOG` | Niveau de logs Rust | `info` |
| `TAURI_DEBUG` | Active debug Tauri | `false` |
| `DATABASE_PATH` | Chemin custom DB | Auto (voir ci-dessus) |

### Fichier `.env` (si nécessaire)

```bash
# Créer .env à la racine (optionnel)
RUST_LOG=debug
TAURI_DEBUG=1
```

**⚠️ Le fichier `.env` est gitignored** — Ne jamais committer de secrets.

---

## Commandes utiles

### Développement

```bash
# Lancer dev avec logs
RUST_LOG=debug pnpm tauri dev

# Rebuild seulement frontend
pnpm build

# Clean complet et rebuild
./clean-build.sh && pnpm tauri dev
```

### Debugging

```bash
# Inspecter webview (Chrome DevTools)
# Menu: View > Developer > Developer Tools
# Ou: Right-click > Inspect Element

# Logs Rust backend
RUST_LOG=trace pnpm tauri dev

# Logs SQLite queries
RUST_LOG=sqlx=debug pnpm tauri dev
```

### Base de données

```bash
# Ouvrir DB SQLite directement
sqlite3 ~/.local/share/com.shadowlearn.app/shadowlearn.db

# Voir tables
.tables

# Voir schema
.schema

# Quitter
.quit
```

---

## Troubleshooting

### Problème: `pnpm tauri dev` ne démarre pas

**Symptômes** : Erreur compilation Rust ou fenêtres n'apparaissent pas

**Solutions** :

1. **Vérifier Rust à jour**
   ```bash
   rustup update stable
   ```

2. **Clean cache Cargo**
   ```bash
   cd src-tauri
   cargo clean
   cd ..
   ```

3. **Rebuild frontend**
   ```bash
   pnpm build
   ```

4. **Script clean-build**
   ```bash
   ./clean-build.sh
   ```

---

### Problème: Shortcuts globaux ne fonctionnent pas (macOS)

**Symptômes** : `Cmd+Shift+Y` ne fait rien

**Solutions** :

1. **Permissions Accessibility**
   - System Settings > Privacy & Security > Accessibility
   - Ajouter Terminal.app
   - Redémarrer terminal

2. **Vérifier logs**
   ```bash
   RUST_LOG=debug pnpm tauri dev
   # Chercher: "🎹" ou "shortcut" dans logs
   ```

3. **Tester avec autre app fermée**
   - Parfois conflit avec autre app utilisant même shortcut
   - Fermer VS Code, browsers, etc.

---

### Problème: HUD invisible

**Symptômes** : App lance mais pas de petit cercle HUD

**Solutions** :

1. **Vérifier fenêtre HUD créée**
   ```bash
   # Dans logs, chercher "✅ Found HUD window"
   ```

2. **Rebuild frontend**
   ```bash
   pnpm build
   pnpm tauri dev
   ```

3. **Vérifier hud.html existe**
   ```bash
   ls dist/hud.html
   # Doit exister après pnpm build
   ```

---

### Problème: Erreur "Cannot find module @rollup/rollup-*"

**Symptômes** : Erreur pnpm build sur module rollup manquant

**Solution** :

```bash
# Supprimer node_modules et réinstaller
rm -rf node_modules package-lock.json
pnpm install
```

---

### Problème: Fenêtre Settings n'apparaît pas

**Symptômes** : Click sur "⚙️ Réglages" mais rien ne se passe

**Solutions** :

1. **Vérifier logs**
   ```bash
   # Chercher "Window 'settings' shown" dans logs
   ```

2. **Vérifier settings.html buildé**
   ```bash
   ls dist/settings.html
   ```

3. **Rebuild si manquant**
   ```bash
   pnpm build
   ```

4. **Vérifier config Tauri**
   ```bash
   # Dans src-tauri/tauri.conf.json
   # Chercher label: "settings"
   ```

---

### Problème: Build error gdk-sys ou pango-sys (Linux)

**Symptômes** : Erreur "The system library `gdk-3.0` was not found"

**Solution** :

```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev libpango1.0-dev

# Fedora
sudo dnf install gtk3-devel pango-devel

# Arch
sudo pacman -S gtk3 pango
```

---

### Problème: Performance lente en dev

**Symptômes** : App lag, hot reload lent

**Solutions** :

1. **Désactiver sourcemaps**
   ```typescript
   // vite.config.ts
   build: {
     sourcemap: false,
   }
   ```

2. **Limiter watchers**
   ```bash
   # Fermer apps qui watchent beaucoup de fichiers
   ```

3. **Build release pour tester perfs**
   ```bash
   pnpm tauri build
   # Tester l'app compilée
   ```

---

### Problème: TypeScript errors après pull

**Symptômes** : `pnpm tsc` montre errors

**Solution** :

```bash
# Reinstaller dépendances
pnpm install

# Clean TypeScript cache
rm -rf node_modules/.cache

# Rebuild
pnpm build
```

---

## Environnements de test

### Test sur macOS Fullscreen

1. Ouvrir FL Studio / autre app fullscreen
2. Lancer ShadowLearn
3. Vérifier HUD reste visible
4. Tester `Cmd+Shift+Y` fonctionne

### Test shortcuts

```bash
# Lancer avec logs shortcut
RUST_LOG=shadowlearn::shortcuts=debug pnpm tauri dev

# Tester toutes les combinaisons
# Vérifier logs pour confirmation
```

---

## Ressources

- [Tauri v2 Docs](https://v2.tauri.app)
- [React Docs](https://react.dev)
- [Vite Docs](https://vitejs.dev)
- [Rust Book](https://doc.rust-lang.org/book/)

---

**🎯 Prochaine étape** : Une fois le setup OK, lire [SYSTEM_OVERVIEW.md](SYSTEM_OVERVIEW.md) pour comprendre l'architecture.

#!/bin/bash

echo "🧹 Nettoyage complet des caches..."

# Nettoyer le cache Vite
echo "📦 Suppression du cache Vite..."
rm -rf node_modules/.vite
rm -rf dist

# Nettoyer le cache Rust
echo "🦀 Suppression du cache Rust..."
rm -rf src-tauri/target

echo "✨ Caches nettoyés!"
echo ""
echo "🔨 Rebuild complet..."

# Pull les derniers changements
echo "📥 Pull des derniers changements..."
git pull origin claude/shadowlearn-learn-by-doing-01VmoEeKGsDfqGZBzYueyAdn

# Build frontend avec Vite
echo "⚡ Build frontend (Vite)..."
pnpm build

# Build backend avec Cargo
echo "🚀 Build backend (Rust)..."
cd src-tauri
cargo build --release
cd ..

echo ""
echo "✅ Build terminé!"
echo ""
echo "Pour lancer l'app:"
echo "  ./src-tauri/target/release/shadowlearn"

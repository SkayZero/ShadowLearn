#!/bin/bash

# Script de monitoring des logs pour ShadowLearn
# Ce script capture les logs du frontend (Vite) et du backend (Rust/Tauri)
# et les affiche en temps réel dans un terminal dédié

LOG_DIR="./logs"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
FRONTEND_LOG="$LOG_DIR/frontend_$TIMESTAMP.log"
BACKEND_LOG="$LOG_DIR/backend_$TIMESTAMP.log"
COMBINED_LOG="$LOG_DIR/combined_$TIMESTAMP.log"

# Créer le répertoire de logs s'il n'existe pas
mkdir -p "$LOG_DIR"

echo "==================================================="
echo "  ShadowLearn - Monitoring de Logs en Temps Réel"
echo "==================================================="
echo ""
echo "📁 Logs sauvegardés dans: $LOG_DIR"
echo "🖥️  Frontend log: $FRONTEND_LOG"
echo "🦀 Backend log:  $BACKEND_LOG"
echo "📊 Combined log: $COMBINED_LOG"
echo ""
echo "Appuyez sur Ctrl+C pour arrêter le monitoring"
echo "==================================================="
echo ""

# Fonction pour nettoyer les codes ANSI
strip_ansi() {
    sed 's/\x1b\[[0-9;]*m//g'
}

# Créer un fichier combiné en temps réel
tail -f "$FRONTEND_LOG" "$BACKEND_LOG" 2>/dev/null | while IFS= read -r line; do
    echo "[$(date +'%H:%M:%S')] $line" | tee -a "$COMBINED_LOG"
done &

TAIL_PID=$!

# Fonction de nettoyage à la sortie
cleanup() {
    echo ""
    echo "🛑 Arrêt du monitoring..."
    kill $TAIL_PID 2>/dev/null
    echo "✅ Logs disponibles dans: $LOG_DIR"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Garder le script actif
wait $TAIL_PID

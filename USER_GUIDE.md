# 📖 ShadowLearn - Guide Utilisateur

## Bienvenue dans ShadowLearn

ShadowLearn est un assistant d'apprentissage intelligent qui observe votre travail et vous propose des suggestions proactives pour améliorer votre productivité.

---

## 🚀 Première Utilisation

### Installation

**macOS:**
```bash
# Option 1: Via le fichier .dmg
1. Téléchargez ShadowLearn.dmg
2. Ouvrez le fichier .dmg
3. Glissez ShadowLearn vers Applications

# Option 2: Via Homebrew
brew install shadowlearn
```

**Windows:**
```bash
# Via l'installer .msi
1. Téléchargez ShadowLearn.msi
2. Double-cliquez sur le fichier
3. Suivez les instructions d'installation
```

**Linux:**
```bash
# Via AppImage
1. Téléchargez ShadowLearn.AppImage
2. chmod +x ShadowLearn.AppImage
3. ./ShadowLearn.AppImage
```

---

## 🎯 Démarrage Rapide

### 1. Permissions Système

Lors du premier lancement, ShadowLearn demandera plusieurs permissions:

- **📸 Capture d'écran** : Pour observer votre contexte de travail
- **⌨️ Accessibilité** : Pour détecter l'inactivité (optionnel)
- **📋 Presse-papiers** : Pour capturer le contexte (optionnel)

**Important**: Ces permissions sont nécessaires pour que ShadowLearn fonctionne correctement.

### 2. Applications Autorisées

Par défaut, ShadowLearn observe:
- ✅ **Visual Studio Code** / Cursor
- ✅ **Blender**
- ✅ **FL Studio**
- ✅ **Figma**
- ✅ **Chrome** / Safari

**Personnaliser:**
1. Ouvrir le bouton Paramètres ⚙️
2. Aller dans "Applications"
3. Ajouter/retirer des applications

### 3. Commencer à Travailler

1. **Lancez une application autorisée** (ex: VSCode)
2. **Travaillez normalement**
3. Après **12 secondes d'inactivité**, ShadowLearn vous proposera des suggestions
4. **Une bulle apparaîtra** avec des suggestions basées sur votre contexte

---

## 💡 Fonctionnalités Principales

### 🎯 Suggestions Proactives

ShadowLearn détecte quand vous êtes potentiellement bloqué et vous propose:

- **🎵 Patterns MIDI** - Si vous travaillez dans FL Studio
- **🐍 Scripts Python** - Pour vos projets de développement
- **📄 Configurations JSON** - Pour vos workflows
- **📚 Tutoriels** - Pour apprendre de nouvelles choses

### 🧠 Apprentissage Intelligent

ShadowLearn s'améliore au fil du temps:

- **Feedback ❤️/💔** - Indiquez si une suggestion est utile
- **Apprentissage** - Le système adapte ses suggestions
- **Trust Score** - ShadowLearn ajuste son niveau de confiance
- **Filtrage automatique** - Réduit le bruit et les suggestions inutiles

### ⚙️ Contrôles

#### Bouton Paramètres ⚙️
- Activer/désactiver des features
- Ajuster les cooldowns
- Voir les statistiques

#### Bouton Artefact 🛠️
- Générer des artefacts manuellement
- Voir les statistiques de génération
- Tester différents types d'artefacts

#### Bouton Snooze 💤
- Mettre en pause les suggestions
- Durées: 30min, 2h, Aujourd'hui
- Reprendre les suggestions quand vous voulez

---

## 🎨 Interface

### Chat Window
La fenêtre principale pour interagir avec ShadowLearn:
- Cliquez sur **❤️** pour indiquer qu'une suggestion est utile
- Cliquez sur **💔** pour indiquer qu'elle ne l'est pas
- Copiez les artefacts pour les utiliser dans votre projet

### Context Window
Affiche le contexte actuel:
- Application active
- Temps d'inactivité
- État du clipboard
- Statistiques de performance

---

## 🔧 Configuration Avancée

### Feature Flags

Dans le bouton Paramètres ⚙️, vous pouvez activer/désactiver:

- **📊 Idle Detection** - Détection d'inactivité
- **📸 Screenshot** - Capture d'écran
- **🧠 Smart Triggers** - Triggers intelligents
- **📈 Telemetry** - Collecte de métriques
- **🎯 Intent Gate** - Validation d'intention

### Variables d'Environnement

Pour une configuration avancée:

```bash
# Utiliser Ollama (LLM local)
export SL_LLM_PROVIDER=ollama
export SL_LLM_MODEL=llama2

# Ou utiliser OpenAI
export SL_LLM_PROVIDER=openai
export OPENAI_API_KEY=sk-...

# Désactiver Intent Gate
export SL_USE_INTENT_GATE=false
```

---

## 🐛 Dépannage

### Les suggestions n'apparaissent pas

1. **Vérifiez les permissions**:
   - Système > Paramètres > Capture d'écran
   - Accès autorisé à ShadowLearn

2. **Vérifiez l'application**:
   - L'application est-elle dans la liste autorisée ?
   - Vérifiez dans ⚙️ > Applications

3. **Vérifiez les cooldowns**:
   - Avez-vous fermé/dimissé récemment ?
   - Attendez 45-90s selon votre dernière interaction

### Les artefacts ne s'ouvrent pas

1. **Vérifiez le chemin**:
   - Les fichiers sont dans `~/Library/Application Support/ShadowLearn/artefacts/`

2. **Permissions**:
   - Les fichiers doivent être lisibles
   - Essayez de les ouvrir manuellement

### Performance lente

1. **Réduisez la taille de la DB**:
   - ⚙️ > Data Manager > Cleanup

2. **Désactivez des features**:
   - ⚙️ > Feature Flags > Désactiver ce qui n'est pas nécessaire

3. **Redémarrez ShadowLearn**

---

## 📊 Statistiques

### Télémétrie

Dans le bouton Paramètres ⚙️ > Télémétrie:
- **Events** - Nombre d'événements enregistrés
- **Average Latency** - Temps de réponse moyen
- **Success Rate** - Taux de succès des suggestions
- **Memory Usage** - Utilisation mémoire

### Apprentissage

Dans le bouton Artefact 🛠️ > Statistiques:
- **Total Generated** - Total d'artefacts générés
- **Successful** - Artefacts validés
- **Failed** - Artefacts échoués
- **Average Time** - Temps moyen de génération

---

## 🔒 Confidentialité

ShadowLearn est conçu pour être **100% privé**:

- ✅ **Toutes les données sont locales** - Stockées sur votre machine
- ✅ **Aucun tracking** - Pas de surveillance de votre activité
- ✅ **Optionnel LLM cloud** - Vous pouvez utiliser Ollama (local)
- ✅ **Open Source** - Code disponible sur GitHub

### Où sont stockées les données ?

**macOS:**
```
~/Library/Application Support/ShadowLearn/
├── database.sqlite          # Base de données
├── contexts/                 # Contextes capturés
└── artefacts/               # Artefacts générés
```

**Windows:**
```
%APPDATA%/ShadowLearn/
├── database.sqlite
├── contexts/
└── artefacts/
```

**Linux:**
```
~/.local/share/ShadowLearn/
├── database.sqlite
├── contexts/
└── artefacts/
```

---

## ❓ FAQ

### Q: ShadowLearn consomme-t-il beaucoup de ressources ?
**R:** Non, ShadowLearn est optimisé pour être léger:
- RAM: ~50-100MB
- CPU: <5% en moyenne
- Stockage: ~10-50MB selon l'usage

### Q: Puis-je utiliser ShadowLearn sans Internet ?
**R:** Oui, avec Ollama (voir INSTALL_OLLAMA.md)
- Installez Ollama
- Téléchargez un modèle local
- ShadowLearn fonctionnera entièrement hors ligne

### Q: Comment désactiver temporairement ShadowLearn ?
**R:** Utilisez le bouton Snooze 💤
- 30 minutes
- 2 heures
- Aujourd'hui

### Q: Les suggestions sont-elles trop fréquentes ?
**R:** Ajustez les cooldowns dans ⚙️ > Paramètres
- Action cooldown: 45s (par défaut)
- Dismiss cooldown: 90s (par défaut)

---

## 📚 Ressources

- **Documentation**: `/docs/README.md`
- **Installation Ollama**: `/docs/INSTALL_OLLAMA.md`
- **Architecture**: `/docs/ARCHITECTURE.md`
- **Contribution**: `/docs/CONTRIBUTING.md`

---

## 🆘 Support

- **Issues GitHub**: https://github.com/shadowlearn/issues
- **Discussions**: https://github.com/shadowlearn/discussions
- **Email**: support@shadowlearn.dev

---

**Version**: 1.0.0  
**Dernière mise à jour**: Octobre 2025


# Phase 3A Testing Guide

## 🧪 Flow complet : Trigger → HUD → Spotlight → Action

### Prérequis

1. Build frontend : `pnpm build`
2. Lancer app : `pnpm tauri dev`

### Test Steps

#### 1️⃣ Trigger Mock Opportunity

**Action** : Dans la fenêtre Chat, utiliser le panneau debug en bas à droite (🧪 Phase 3A Debug).

**Boutons disponibles** :
- 🔧 Refacto Pattern
- 🐛 Debug Assistance
- 📚 Learning Tip
- 💡 Quick Tip

**Attendu** :
- ✅ Message de succès : "✅ Triggered {type} opportunity"
- ✅ Logs console : `[Debug] Triggered mock opportunity: {type}`

---

#### 2️⃣ HUD Pulse

**Attendu** :
- ✅ HUD (petit cercle 60x60) change de couleur : Jaune
- ✅ HUD pulse lentement (animation 2s)
- ✅ Console logs : `🔔 HUD pulse received: {state: "opportunity"}`
- ✅ Counter opportunités +1

**Délai avant retour normal** : 30 secondes (auto-reset)

---

#### 3️⃣ Ouvrir Spotlight

**Action** : Appuyer sur `Cmd+Shift+Y` (macOS) ou `Ctrl+Shift+Y` (autres)
**OU** : Double-clic sur le HUD

**Attendu** :
- ✅ Fenêtre Spotlight s'ouvre (600x500, centre-haut de l'écran)
- ✅ Affichage de l'opportunité :
  - Badge avec emoji selon type (🔧/🐛/📚/💡)
  - Titre : ex. "Code répété détecté"
  - Description
  - Contexte (app, file, line, code snippet)
  - Confiance : ex. "85% confiance"
- ✅ 3 boutons actions visibles : **[✓ Voir]** / **[💬 Discuter]** / **[✕]**

---

#### 4️⃣ Action : Voir

**Action** : Cliquer sur **[✓ Voir]**

**Attendu** :
- ✅ Modal s'affiche DANS le Spotlight (pas nouvelle fenêtre)
- ✅ Détails complets :
  - ID
  - Type
  - Confiance
  - Status : "viewed"
  - Timestamp
- ✅ Bouton **[✕]** pour fermer modal
- ✅ Spotlight reste ouvert

**Action** : Fermer modal

**Attendu** :
- ✅ Retour à vue normale Spotlight

---

#### 5️⃣ Action : Discuter

**Action** : Cliquer sur **[💬 Discuter]**

**Attendu** :
- ✅ Spotlight se ferme
- ✅ Fenêtre Chat s'ouvre et prend le focus
- ✅ Console logs : Event `chat:prefill` émis avec :
  ```json
  {
    "opportunityId": "mock_refacto_1234567890",
    "context": { ... }
  }
  ```
- ✅ Status opportunité → "actioned"

**Note** : Le prefill du chat n'est pas encore implémenté (Phase 3A+), mais l'événement est émis.

---

#### 6️⃣ Action : Ignorer

**Action** : Trigger nouvelle opportunité → Ouvrir Spotlight → Cliquer sur **[✕]**

**Attendu** :
- ✅ Spotlight se ferme immédiatement
- ✅ Status opportunité → "ignored"
- ✅ HUD retourne en état normal (vert)

---

#### 7️⃣ Test avec plusieurs opportunités

**Action** :
1. Trigger 3 opportunités de types différents (sans ouvrir Spotlight)
2. Ouvrir Spotlight

**Attendu** :
- ✅ Spotlight affiche LA PLUS RÉCENTE opportunité "pending"
- ✅ Pas d'affichage des opportunités ignorées/actioned

---

### 🐛 Debugging

#### Logs utiles

```bash
# Frontend (browser console)
🔔 HUD pulse received: {state: "opportunity"}
📬 Received opportunity:new event {...}
[Debug] Triggered mock opportunity: refacto

# Backend (terminal Rust logs avec RUST_LOG=debug)
🧪 Triggering mock opportunity: refacto
✅ Mock opportunity emitted: mock_refacto_1234567890
✅ HUD pulse event emitted
```

#### Si Spotlight n'affiche rien

1. Vérifier console : "📬 Received opportunity:new event"
2. Vérifier `latestOpportunity` dans React DevTools
3. Vérifier que OpportunityProvider wraps SpotlightWindow

#### Si HUD ne pulse pas

1. Vérifier console : "🔔 HUD pulse received"
2. Vérifier event listener dans hud.tsx (ligne ~42)
3. Vérifier émission Rust : `app.emit("hud:pulse", ...)`

#### Si commande trigger_mock_opportunity échoue

1. Vérifier compilation Rust : `cd src-tauri && cargo check`
2. Vérifier handler enregistré : `lib.rs` ligne 1361 (dans invoke_handler)

---

### ✅ Critères de succès Phase 3A

- [ ] Trigger 4 types d'opportunités (refacto, debug, learn, tip)
- [ ] HUD pulse jaune à chaque trigger
- [ ] Spotlight affiche opportunité avec données complètes
- [ ] Action "Voir" → Modal détails
- [ ] Action "Discuter" → Chat s'ouvre + event émis
- [ ] Action "Ignorer" → Opportunité ignorée + Spotlight ferme
- [ ] Pas de crash, pas d'erreur console bloquante
- [ ] Flow complet < 5 secondes (trigger → action)

---

### 🚀 Prochaine étape

**Après validation Phase 3A** → Phase 3B : Détection intelligente réelle

**À implémenter** :
1. File watcher (notify crate)
2. Pattern Refacto (regex sliding window)
3. Pattern Debug (heuristics comportementales)
4. Désactiver trigger `idle_seconds` legacy

**À supprimer** :
- `DebugOpportunities.tsx` component
- Import dans `chat.tsx`
- Remplacer mock triggers par vraies détections

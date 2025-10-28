# 🧪 Test OpportunityToast - Fix useEvent

## 🎯 Objectif
Valider que le toast "J'ai une idée" s'affiche correctement après le fix du hook `useEvent`.

---

## 📝 Changements Appliqués

### 1. `useEvent` Hook (src/lib/eventBus.ts)
✅ **Ajout d'un listener hybride** :
- Écoute événements **Tauri** (backend)
- Écoute événements **DOM** (tests manuels)
- Logs exhaustifs pour debug

### 2. OpportunityToast (src/components/OpportunityToast.tsx)
✅ **Ajout `data-testid="opportunity-toast"`** pour tests

---

## 🧪 Test Manuel 1 : Événement DOM (Immédiat)

### Étapes
1. Lancer l'app :
```bash
cd /Users/syloh/Desktop/shadowlearn
pnpm tauri dev
```

2. Ouvrir DevTools (Cmd+Option+I)

3. Dans la **Console**, coller et exécuter :
```javascript
// Test manuel d'événement
window.dispatchEvent(new CustomEvent('shadow:opportunity', {
  detail: {
    id: 'test-manual-1',
    title: 'Test Manuel',
    confidence: 0.95,
    preview: 'Ceci est un test manuel du toast'
  }
}));

// Attendre 1s puis vérifier
setTimeout(() => {
  const toast = document.querySelector('[data-testid="opportunity-toast"]');
  console.log('✅ Toast trouvé :', toast !== null);
  if (toast) {
    console.log('✅ Toast visible à l\'écran');
  } else {
    console.error('❌ Toast NOT FOUND');
  }
}, 1000);
```

### Résultat Attendu
- ✅ Dans Console : Log `[useEvent] ✅ DOM event received: shadow:opportunity`
- ✅ Toast apparaît en **bas à droite** de l'écran
- ✅ Toast affiche "J'ai une idée"
- ✅ Preview : "Ceci est un test manuel du toast"
- ✅ Confiance : 95%
- ✅ Boutons "Voir →" et "Ignorer" visibles

### Résultat Réel
**À REMPLIR APRÈS TEST** :
- [ ] Toast visible ?
- [ ] Texte correct ?
- [ ] Boutons fonctionnels ?
- Console logs :
```
[Copier les logs ici]
```

---

## 🧪 Test Manuel 2 : Événement Backend (Après idle)

### Étapes
1. Laisser l'app idle pendant **30 secondes**
2. Le backend devrait émettre un trigger
3. Observer si toast apparaît

### Résultat Attendu
- ✅ Dans Console : Log `[useEvent] ✅ Tauri event received: shadow:opportunity`
- ✅ Toast apparaît avec vraies données backend
- ✅ App + window title corrects

### Résultat Réel
**À REMPLIR APRÈS TEST** :
- [ ] Toast visible après idle ?
- [ ] Données backend correctes ?
- Console logs :
```
[Copier les logs ici]
```

---

## 🧪 Test Manuel 3 : Interactions

### Test 3.1 : Bouton "Voir"
1. Déclencher toast (manuel ou idle)
2. Cliquer sur **"Voir →"**

**Attendu** :
- ✅ Toast disparaît
- ✅ Dock s'ouvre (si `onOpenDock` connecté)
- ✅ Backend enregistre acceptance

**Résultat** :
- [ ] Toast disparaît ?
- [ ] Dock s'ouvre ?

---

### Test 3.2 : Bouton "Ignorer"
1. Déclencher toast
2. Cliquer sur **"Ignorer"**

**Attendu** :
- ✅ Toast disparaît
- ✅ Backend enregistre rejet
- ✅ Même toast ne réapparaît pas

**Résultat** :
- [ ] Toast disparaît ?
- [ ] Ne réapparaît pas ?

---

### Test 3.3 : Auto-dismiss (10s)
1. Déclencher toast
2. Ne rien faire pendant 10 secondes

**Attendu** :
- ✅ Toast disparaît automatiquement après 10s

**Résultat** :
- [ ] Auto-dismiss fonctionne ?

---

## 🐛 Debug si Échec

### Si toast ne s'affiche pas

#### 1. Vérifier logs useEvent
Dans Console, chercher :
```
[useEvent] Setting up listeners for: shadow:opportunity
[useEvent] ✅ DOM listener registered
[useEvent] ✅ Tauri listener registered
```

Si **absent** → Hook pas monté, vérifier que `OpportunityToast` est rendu dans `chat.tsx`

---

#### 2. Vérifier composant monté
```javascript
// Dans console
const toast = document.querySelector('[data-testid="opportunity-toast"]');
console.log('Composant OpportunityToast monté ?', toast !== null);

// Si null : composant pas rendu
// Vérifier chat.tsx ligne ~370 : <OpportunityToast />
```

---

#### 3. Vérifier condition confidence
Le toast ne s'affiche que si `confidence > 0.7`

Test avec confidence faible (ne devrait PAS s'afficher) :
```javascript
window.dispatchEvent(new CustomEvent('shadow:opportunity', {
  detail: {
    id: 'test-low-conf',
    confidence: 0.5,  // < 0.7 → PAS affiché
    preview: 'Low confidence'
  }
}));
```

---

#### 4. Vérifier dismissed
Le toast ne s'affiche pas si déjà dismissed

Clear dismissed :
```javascript
// Dans console
localStorage.removeItem('shadow_store');
location.reload();
```

---

## 📊 Checklist Validation

### Must-Have (Bloquer si non OK)
- [ ] **Test 1** : Toast s'affiche avec événement DOM manuel
- [ ] **Test 2** : Toast s'affiche après idle backend
- [ ] **Test 3.1** : Bouton "Voir" fonctionne
- [ ] **Test 3.2** : Bouton "Ignorer" fonctionne

### Should-Have
- [ ] **Test 3.3** : Auto-dismiss 10s fonctionne
- [ ] Console logs propres (pas d'erreurs)
- [ ] Backend tracking enregistré

---

## ✅ Critères de Succès

**OpportunityToast est considéré FIXÉ si** :
1. ✅ Toast s'affiche avec événement DOM manuel
2. ✅ Toast s'affiche avec événement backend (idle)
3. ✅ Les 2 boutons fonctionnent
4. ✅ Pas d'erreur console

**Si tous OK** → ✅ Marquer TODO `stab_useevent` comme `completed`

---

## 🔄 Prochaine Étape

Si OpportunityToast OK → **Priority 2** : TriggerBubble + StreakTracker

---

## 📝 Notes de Test

[Espace pour notes additionnelles pendant les tests]



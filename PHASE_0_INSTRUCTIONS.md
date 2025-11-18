# 🚀 Phase 0: Audit & Stabilisation - Instructions

## ✅ Étape 1: Installation des Dépendances de Test

```bash
cd /Users/syloh/Desktop/shadowlearn

# Installer les dépendances de test
pnpm install

# Vérifier que Vitest fonctionne
pnpm test --run
```

**Attendu**: Les 3 tests devraient s'exécuter (peuvent échouer au début, c'est normal).

---

## 📝 Étape 2: Audit Manuel (2h)

### Lancer l'Application

```bash
# Terminal 1: Backend logs
cd /Users/syloh/Desktop/shadowlearn
pnpm tauri dev 2>&1 | tee audit_logs.txt
```

### Checklist d'Audit

Ouvrir `AUDIT_REPORT.md` et compléter chaque section :

1. **Pour chaque feature** :
   - [ ] Suivre les "Test Steps"
   - [ ] Cocher les "Observations"
   - [ ] Noter tout bug dans "Issues Found"
   - [ ] Marquer le status: ✅ OK / ⚠️ Partial / ❌ Broken

2. **Console Browser** :
   - Ouvrir DevTools (Cmd+Option+I)
   - Noter toutes les erreurs rouges
   - Noter tous les warnings jaunes importants
   - Copier dans `AUDIT_REPORT.md` → "Console Errors"

3. **Logs Rust** :
   - Regarder le terminal backend
   - Noter toutes les lignes `ERROR` ou `WARN`
   - Copier dans `AUDIT_REPORT.md` → "Rust Logs"

4. **Performance Baseline** :
   ```javascript
   // Dans console browser
   performance.getEntriesByType('measure')
   ```
   - Noter les latences observées
   - Compléter dans `AUDIT_REPORT.md` → "Performance Baseline"

---

## 📊 Étape 3: Exporter les Métriques

### Dans la Console Browser

```javascript
// Importer les fonctions de monitoring
import { exportMetricsCSV, getPerformanceReport } from './src/hooks/usePerfMarks';

// Exporter CSV
const csv = exportMetricsCSV();
console.log(csv);
// Copier le CSV et sauver dans perf_baseline.csv

// Rapport de performance
const report = getPerformanceReport();
console.log(report);
```

---

## 🐛 Étape 4: Classifier les Bugs

Dans `AUDIT_REPORT.md`, séparer les bugs en :

### Critiques (Bloquants)
- Crashes
- Features totalement cassées
- Erreurs backend qui empêchent l'utilisation

### Mineurs (Non-bloquants)
- UI glitches
- Animations pas parfaites
- Performance sous-optimale mais acceptable

---

## 🎯 Étape 5: Décision GO/NO-GO

Compléter la section "Decision" dans `AUDIT_REPORT.md` :

- **GO (Phase 1)** si :
  - < 5 bugs critiques
  - ≥ 8/12 features au moins partiellement fonctionnelles
  - Pas de crash systématique

- **NO-GO (Stabiliser d'abord)** si :
  - ≥ 5 bugs critiques
  - < 6/12 features fonctionnelles
  - Crashes fréquents

---

## 📋 Livrables Phase 0

À la fin de cette phase, tu dois avoir :

1. ✅ `AUDIT_REPORT.md` - Complété avec toutes les sections
2. ✅ `audit_logs.txt` - Logs backend du run d'audit
3. ✅ `perf_baseline.csv` - Métriques de performance actuelles
4. ✅ Liste prioritisée des bugs à fixer

---

## 🔄 Prochaine Étape

Si **GO** → Passer à Phase 1 (Fix & Validate)  
Si **NO-GO** → Fixer bugs critiques en premier

---

## 💡 Tips d'Audit

### Tester Systématiquement
- Ne pas assumer qu'une feature marche
- Tester toutes les interactions (click, hover, keyboard)
- Tester edge cases (ex: spammer les boutons)

### Observer les Details
- Animations saccadées ?
- Textes mal alignés ?
- Couleurs incohérentes ?
- Sons pas synchronisés ?

### Noter TOUT
- Mieux avoir trop d'info que pas assez
- Screenshots pour bugs visuels
- Vidéos pour bugs d'animation

### Mesurer Objectivement
- Utiliser DevTools Performance tab
- Noter FPS pendant animations
- Mesurer memory usage (10min de run)

---

## 🚨 Erreurs Communes à Vérifier

### Frontend
- [ ] `data-testid` manquants sur composants
- [ ] Event listeners pas nettoyés (memory leaks)
- [ ] Props pas typées correctement
- [ ] State updates après unmount
- [ ] Re-renders excessifs

### Backend
- [ ] Commands Tauri pas enregistrées
- [ ] Events pas émis correctement
- [ ] Mutex deadlocks
- [ ] Panic non gérés
- [ ] Memory leaks Rust

### Integration
- [ ] Events frontend/backend désynchronisés
- [ ] Payload types incompatibles
- [ ] Timeouts trop courts
- [ ] Race conditions

---

## ⏱️ Timeline Phase 0

- **Setup (30min)** : Installation + lancement
- **Audit Features (1h)** : Tester les 12 features
- **Performance (15min)** : Baseline metrics
- **Rapport (15min)** : Compléter AUDIT_REPORT.md

**Total : ~2h**

---

## 📞 Support

Si bloqué sur un aspect technique, consulter :
- `docs/ARCHITECTURE.md` - Architecture générale
- `ROADMAP_SUITE.md` - Plan global
- `Next_Steps.md` - Plan hybride détaillé

---

Bonne chance pour l'audit ! 🚀





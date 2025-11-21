# Review du Plan Dev - ShadowLearn

**Date** : 2025-01-21
**Reviewer** : Claude
**Status** : ✅ Approuvé avec ajustements

---

## Phase 3A - Spotlight Mock Data (2-3 jours)

### ✅ Approuvé tel quel
- Store d'opportunités frontend
- Commande Rust `trigger_mock_opportunity`
- UI Spotlight avec 3 actions
- Flow HUD → Spotlight

### ⚠️ Ajustements requis

**Commandes Tauri manquantes** :

```typescript
// ❌ N'existe pas
invoke('prefill_chat_context', { ... })
invoke('open_in_editor', { ... })

// ✅ Utiliser à la place
const handleDiscuss = async (opp: Opportunity) => {
  markAsActioned(opp.id);
  await invoke('show_window', { windowLabel: 'chat' });
  emit('chat:prefill', { opportunityId: opp.id, context: opp }); // Event au lieu de command
  toggleSpotlight(false);
};

const handleView = async (opp: Opportunity) => {
  markAsViewed(opp.id);
  // Pour MVP : juste afficher détails dans modal
  setShowDetails(true);
  // open_in_editor à implémenter post-MVP
};
```

**Action** : Simplifier actions pour MVP, ne pas bloquer sur commandes manquantes.

---

## Phase 3B - Détection Intelligente (⏰ 2 semaines, pas 1)

### ✅ Approuvé : Désactivation idle_seconds

```rust
// LEGACY TRIGGER DISABLED - See CONTEXT.md Section 7
// if context.idle_seconds > 15 { ... }
```

### ⚠️ Pattern Refacto : Simplifier pour MVP

**Proposé** : AST parsing complet
**Problème** : Trop complexe, supporte multi-langages (TS, Rust, Python)

**Recommandation MVP** :

```rust
// Version MVP : Regex + heuristiques
pub fn detect_repeated_patterns(content: &str) -> Vec<RepeatedPattern> {
    let lines: Vec<&str> = content.lines().collect();
    let mut patterns = HashMap::new();

    // Sliding window de 5-10 lignes
    for window in lines.windows(5) {
        let normalized = normalize_whitespace(window.join("\n"));
        *patterns.entry(normalized).or_insert(0) += 1;
    }

    patterns.into_iter()
        .filter(|(_, count)| *count >= 3)
        .map(|(pattern, count)| RepeatedPattern { pattern, count })
        .collect()
}
```

**Évolution post-MVP** : Ajouter tree-sitter pour parsing précis.

---

### ⚠️ Pattern Debug : Simplifier détection erreurs

**Proposé** : Parser compiler output
**Problème** : Varie par IDE (VS Code, Cursor, Terminal)

**Recommandation MVP** : Heuristiques comportementales

```rust
pub struct DebugDetector {
    active_sessions: HashMap<String, DebugSession>,
}

struct DebugSession {
    file: String,
    focus_start: Instant,
    last_save: Instant,
    edit_count: u32,
}

impl DebugDetector {
    // Détecter debug session par heuristiques :
    // - Focus sur même fichier > 2 min
    // - Pas de sauvegarde récente
    // - Éditions rapides (>10 en 30s)
    pub fn detect_debug_session(&mut self, context: &Context) -> Option<Opportunity> {
        let session = self.active_sessions.entry(context.file.clone()).or_insert(...);

        let focus_duration = session.focus_start.elapsed().as_secs();
        let time_since_save = session.last_save.elapsed().as_secs();

        if focus_duration > 120 && time_since_save > 60 && session.edit_count > 10 {
            return Some(Opportunity {
                title: "Tu sembles bloqué sur ce fichier",
                description: format!("Focus depuis {}s sans sauvegarde", focus_duration),
                type_: "debug",
                confidence: 0.75,
                ...
            });
        }
        None
    }
}
```

**Évolution post-MVP** : Ajouter log parsing (VS Code console, terminal output).

---

### 🚨 File System Watcher manquant

**Requis pour** : Pattern Refacto (détecter quand fichier sauvegardé)

**À implémenter** : `src-tauri/src/monitor/file_watcher.rs` (3-4h)

```rust
use notify::{Watcher, RecursiveMode, Event};

pub fn watch_active_file(app_handle: AppHandle, path: &Path) -> Result<(), String> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    tokio::spawn(async move {
        for event in rx {
            if let Event::Modify(_) = event {
                app_handle.emit("file:saved", path).ok();
            }
        }
    });

    Ok(())
}
```

**Action** : Implémenter file watcher **avant** Pattern Refacto.

---

## Bugs Critiques (1-2 jours)

### ✅ Approuvé
- Settings window : Hypothèse `center()` correcte
- TS warnings : Approche par batch correcte

### 💡 Ajout recommandé

**Avant de fixer** :
```bash
# Lister warnings
pnpm tsc --noEmit > ts-warnings.txt

# Catégoriser
grep "unused" ts-warnings.txt | wc -l
grep "'any'" ts-warnings.txt | wc -l
```

**Après fix** : Documenter dans CONTEXT.md Section 3 (Problèmes résolus).

---

## Post-3B : Priorisation

### ✅ FAIRE (Priorité HAUTE)

**1. Persistence (2 jours)**
```sql
CREATE TABLE opportunities (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    type TEXT NOT NULL,
    confidence REAL NOT NULL,
    context_json TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    status TEXT NOT NULL,
    user_feedback INTEGER
);
```

**2. Analytics & Feedback (3 jours)**
```typescript
// Après action Discuter/Voir
<div className="feedback-prompt">
  Cette opportunité était utile ?
  <button onClick={() => sendFeedback(opp.id, 2)}>👍</button>
  <button onClick={() => sendFeedback(opp.id, 0)}>👎</button>
</div>
```

**3. 1-2 nouveaux patterns MAX**
- Pattern Clipboard (copier-coller StackOverflow)
- OU Pattern Documentation (usage API inconnue)
- **PAS 4-6 patterns d'un coup** → Valider approche d'abord

---

### ⏰ REPORTER (Post-validation MVP)

**LLM Integration** : Complexe, dépendance externe, opt-in privacy
**i18n** : Pas urgent si target français d'abord
**Collaboration / Plugins** : Vision 6+ mois

---

## 🚨 Ce qui manque au plan

### 1. Tests E2E (À ajouter après Phase 3A)

```bash
# Playwright ou Tauri WebDriver
pnpm add -D @playwright/test
```

**Tests critiques** :
- Cmd+Shift+Y → Spotlight s'ouvre
- Double-clic HUD → Spotlight s'ouvre
- Action "Discuter" → Chat s'ouvre avec contexte
- Action "Ignorer" → Opportunité dismissed

---

### 2. Performance Monitoring

**Patterns qui tournent en loop = risque ralentissement**

```rust
use std::time::Instant;

pub fn run_pattern_analysis(&self) -> Result<Opportunity, String> {
    let start = Instant::now();
    let result = self.analyze();
    let duration = start.elapsed();

    if duration.as_millis() > 100 {
        warn!("Pattern {} too slow: {}ms", self.name, duration.as_millis());
    }

    result
}
```

---

### 3. Error Recovery (Circuit Breaker)

**Si pattern crash, ne pas bloquer toute l'app**

```rust
pub struct PatternExecutor {
    error_counts: HashMap<String, u32>,
    disabled_patterns: HashSet<String>,
}

impl PatternExecutor {
    pub fn execute_pattern(&mut self, pattern: &dyn Pattern) -> Result<(), String> {
        if self.disabled_patterns.contains(&pattern.name()) {
            return Err("Pattern temporarily disabled".into());
        }

        match pattern.run() {
            Ok(result) => {
                self.error_counts.insert(pattern.name(), 0);
                Ok(result)
            }
            Err(e) => {
                let count = self.error_counts.entry(pattern.name()).or_insert(0);
                *count += 1;

                if *count >= 3 {
                    warn!("Disabling pattern {} after 3 errors", pattern.name());
                    self.disabled_patterns.insert(pattern.name());
                }

                Err(e)
            }
        }
    }
}
```

---

## Timeline révisée

| Phase | Proposé | Révisé | Raison |
|-------|---------|--------|--------|
| Phase 3A | 2-3 jours | 2-3 jours | ✅ OK |
| Bugs | 1-2 jours | 1-2 jours | ✅ OK |
| Phase 3B | 1 semaine | **2 semaines** | File watcher + patterns simplifiés + tests |
| Post-3B | Beaucoup | **1 semaine** | Persistence + Analytics seulement |

**Total MVP complet** : ~4 semaines (au lieu de 2-3)

---

## Checklist avant démarrage Phase 3A

- [ ] Clarifier `prefill_chat_context` → Utiliser events au lieu de command
- [ ] Clarifier `open_in_editor` → Reporter post-MVP ou simplifier
- [ ] Designer UI Spotlight (maquette Figma ou wireframe)
- [ ] Vérifier mock data structure finale

## Checklist avant démarrage Phase 3B

- [ ] Phase 3A terminée + testée manuellement
- [ ] File watcher implémenté (3-4h)
- [ ] Patterns simplifiés validés (regex au lieu d'AST)
- [ ] Tests E2E Phase 3A passent

## Checklist avant Post-3B

- [ ] Phase 3B validée avec utilisateurs réels
- [ ] Au moins 20 opportunités réelles déclenchées
- [ ] Taux d'action > 30% (sinon ajuster patterns)
- [ ] Performance patterns < 100ms

---

## Validation finale

**Questions pour le dev** :

1. ✅ Tu comprends pourquoi simplifier Refacto (regex) et Debug (heuristiques) pour MVP ?
2. ⚠️ File watcher : Tu peux l'implémenter avant Pattern Refacto ?
3. ⚠️ Commandes manquantes : Tu utilises events au lieu ?
4. 💡 Timeline 4 semaines : Réaliste pour toi ?

**Si oui à tout** → 🚀 **GO Phase 3A !**

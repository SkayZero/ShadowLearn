# Screen Monitor - Documentation

## Vue d'ensemble

Le **Screen Monitor** est un système de monitoring intelligent qui :
- Capture l'écran toutes les 5 secondes
- Détecte les changements significatifs (>15% de différence)
- Analyse le contenu avec Claude Vision (optionnel)
- Génère des suggestions automatiques

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Screen Monitor                        │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │ ScreenMonitor│─▶│ChangeDetector│─▶│ VisionClient  │ │
│  │   (Loop 5s)  │  │ (Hash Diff)  │  │ (Claude API)  │ │
│  └──────────────┘  └──────────────┘  └───────────────┘ │
│         │                  │                  │         │
│         ▼                  ▼                  ▼         │
│  ┌──────────────────────────────────────────────────┐  │
│  │          Emit "screen-change" Event              │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React)                      │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │useScreenMonit│─▶│useEventBus   │─▶│ScreenMonitor  │ │
│  │   or Hook    │  │  (Listen)    │  │    Bubble     │ │
│  └──────────────┘  └──────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Configuration

### Backend (Rust)

```rust
// src-tauri/src/lib.rs
let monitor_config = monitor::MonitorConfig {
    interval_secs: 5,              // Capture toutes les 5s
    similarity_threshold: 0.85,     // 85% de similarité = pas de changement
    use_vision: true,              // Activer Claude Vision
    enabled: true,                  // Activer le monitoring
};
```

### Variables d'environnement

Pour utiliser Claude Vision, définissez :

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

## Utilisation

### Backend Commands (Tauri)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Démarrer le monitoring
await invoke('start_screen_monitor');

// Arrêter le monitoring
await invoke('stop_screen_monitor');

// Vérifier le statut
const isRunning = await invoke<boolean>('get_monitor_status');

// Reset le détecteur (utile après changement d'app)
await invoke('reset_monitor_detector');
```

### Frontend (React Hooks)

```tsx
import { useScreenMonitor } from './hooks/useScreenMonitor';
import { ScreenMonitorBubble } from './components/ScreenMonitorBubble';

function App() {
  const {
    isMonitoring,
    latestChange,
    startMonitoring,
    stopMonitoring
  } = useScreenMonitor();

  return (
    <div>
      <button onClick={startMonitoring}>
        Start Monitoring
      </button>

      <button onClick={stopMonitoring}>
        Stop Monitoring
      </button>

      {/* Auto-affiche les suggestions */}
      <ScreenMonitorBubble autoDismissSeconds={30} />

      {/* Afficher la dernière capture */}
      {latestChange && (
        <div>
          <p>Last change: {new Date(latestChange.timestamp * 1000).toLocaleString()}</p>
          {latestChange.analysis && <p>Suggestion: {latestChange.analysis}</p>}
        </div>
      )}
    </div>
  );
}
```

## Événements

### Backend → Frontend

**Event:** `screen-change`

**Payload:**
```typescript
{
  timestamp: number;         // Unix timestamp (seconds)
  image_path: string;        // Chemin vers le fichier screenshot
  image_base64: string;      // Image en base64
  analysis: string | null;   // Suggestion de Claude Vision (si activée)
}
```

### Frontend → Frontend

**Event:** `shadow:suggestion`

**Payload:**
```typescript
{
  id: string;               // "screen-{timestamp}"
  type: "screen-monitor";
  text: string;             // Le texte de la suggestion
  timestamp: number;
}
```

## Détection de Changement

Le système utilise un **Average Hash (aHash)** pour détecter les changements :

1. Screenshot capturé
2. Redimensionné à 8x8 pixels (grayscale)
3. Hash binaire généré (64 bits)
4. Comparaison avec le hash précédent
5. Si similarité < 85% → Changement détecté

### Avantages de aHash
- ✅ Ultra rapide (< 1ms)
- ✅ Résistant aux légères variations
- ✅ Détecte les changements structurels
- ✅ Peu de faux positifs

## Claude Vision Analysis

Quand `use_vision: true`, chaque changement détecté est envoyé à Claude Vision pour analyse.

**Prompt utilisé:**
```
Analyze this screenshot and suggest helpful actions the user might want to take.

Focus on:
1. What application or task is the user working on?
2. What could be automated or improved?
3. Are there any learning opportunities?

Respond with 1-3 concise, actionable suggestions.
```

**Modèle:** `claude-3-haiku-20240307` (rapide + économique)

**Timeout:** 30 secondes

## Performance

### Optimisations appliquées

1. **Screenshot Capture**
   - Compression JPEG 50%
   - Redimensionnement à 720px max
   - Capture asynchrone (tokio::spawn_blocking)

2. **Change Detection**
   - Hash perceptuel en < 1ms
   - Pas de comparaison pixel par pixel
   - Cache du dernier hash

3. **Vision Analysis**
   - Utilise Haiku (le plus rapide)
   - Timeout 30s avec retry
   - Optionnel (désactivé par défaut)

### Benchmarks typiques

| Opération            | Durée   |
|---------------------|---------|
| Screenshot capture  | 50-100ms|
| Hash calculation    | < 1ms   |
| Vision analysis     | 1-3s    |
| Total (sans Vision) | ~100ms  |
| Total (avec Vision) | ~1.5s   |

## Cas d'usage

### 1. Apprentissage automatique
Détecte quand l'utilisateur bloque sur un problème et suggère de l'aide.

### 2. Automation
Identifie les tâches répétitives et propose des scripts/raccourcis.

### 3. Productivité
Suggère de meilleures pratiques ou outils selon le contexte.

### 4. Documentation
Génère automatiquement de la documentation basée sur les actions visuelles.

## Sécurité & Confidentialité

- ⚠️ Les screenshots sont stockés temporairement dans `/tmp`
- ⚠️ Les images sont envoyées à l'API Anthropic si `use_vision: true`
- ✅ Possibilité de désactiver complètement via `enabled: false`
- ✅ Possibilité d'utiliser uniquement la détection locale (sans Vision)

**Recommandations:**
- Utilisez `use_vision: false` pour les données sensibles
- Ajoutez une allowlist/blocklist d'applications
- Chiffrez les logs si nécessaire

## Roadmap

- [ ] OCR local (Tesseract) comme alternative à Vision
- [ ] Filtrage par application (allowlist/blocklist)
- [ ] Historique des changements détectés
- [ ] Métriques de productivité
- [ ] Export des suggestions pour analytics
- [ ] Mode "Focus" (pause automatique)

## Dépannage

### Le monitoring ne démarre pas

1. Vérifier les permissions screen recording (macOS)
2. Vérifier les logs : `info!("🎬 Starting screen monitor")`
3. Tester la capture manuelle : `await invoke('capture_screenshot')`

### Claude Vision ne fonctionne pas

1. Vérifier `ANTHROPIC_API_KEY` est définie
2. Vérifier les logs : `warn!("⚠️ Claude Vision client init failed")`
3. Tester la connexion API manuellement

### Trop de faux positifs

1. Augmenter `similarity_threshold` (ex: 0.90)
2. Augmenter `interval_secs` (ex: 10)
3. Utiliser `reset_monitor_detector()` après changement d'app

## Exemples

### Mode "Focus" avec pause automatique

```tsx
function FocusMode() {
  const { startMonitoring, stopMonitoring } = useScreenMonitor();
  const { isPaused } = usePauseDetection();

  useEffect(() => {
    if (isPaused) {
      stopMonitoring(); // Pause pendant les breaks
    } else {
      startMonitoring();
    }
  }, [isPaused]);
}
```

### Filtrage par application

```tsx
function AppFilteredMonitor() {
  const { latestChange } = useScreenMonitor();
  const allowedApps = ['VS Code', 'Blender', 'Ableton'];

  useEffect(() => {
    if (latestChange && !allowedApps.some(app =>
      latestChange.analysis?.includes(app)
    )) {
      console.log('Ignoring change - not in allowed apps');
      return;
    }

    // Process suggestion...
  }, [latestChange]);
}
```

## Contribution

Le code source est dans :
- Backend: `src-tauri/src/monitor/`
- Frontend: `src/hooks/useScreenMonitor.ts` + `src/components/ScreenMonitorBubble.tsx`

Pour contribuer :
1. Fork le repo
2. Créer une branche feature
3. Tester localement
4. Soumettre une PR

## Licence

MIT

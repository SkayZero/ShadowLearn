use super::app_detector::{ActiveApp, AppDetector};
use super::clipboard_monitor::ClipboardMonitor;
use super::errors::ContextError;
use super::idle_detector::{ActivityType, IdleDetector, IdleState};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::debug;
use uuid::Uuid;

/// Résultat d'un peek rapide (< 10ms)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeekResult {
    pub app: ActiveApp,
    pub idle_seconds: f64,
}

/// Contexte agrégé capturé à un instant T
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub app: ActiveApp,
    pub clipboard: Option<String>,
    pub idle_seconds: f64,
    pub timestamp: u64,
    pub capture_duration_ms: u64,
}

/// Agrégateur de contexte avec cache fast-path
pub struct ContextAggregator {
    app_detector: AppDetector,
    clipboard_monitor: ClipboardMonitor,
    idle_detector: IdleDetector,
    last_capture: Option<Instant>,
}

impl ContextAggregator {
    pub fn new() -> Result<Self, ContextError> {
        Ok(Self {
            app_detector: AppDetector::new(),
            clipboard_monitor: ClipboardMonitor::new()?,
            idle_detector: IdleDetector::new(),
            last_capture: None,
        })
    }

    /// Peek rapide (< 10ms) : app + idle uniquement
    /// Utilisé pour les checks de trigger sans overhead
    pub fn peek(&mut self) -> Result<PeekResult, ContextError> {
        Ok(PeekResult {
            app: self.app_detector.get_active_app()?,
            idle_seconds: self.idle_detector.get_idle_seconds(),
        })
    }

    /// Capture complète du contexte (100-300ms)
    /// Utilisé uniquement quand un trigger est validé
    pub async fn capture(&mut self) -> Result<Context, ContextError> {
        let start = Instant::now();

        // Get active app (with cache)
        let app = self.app_detector.get_active_app()?;

        // Get clipboard (only if changed)
        let clipboard = self.clipboard_monitor.get_recent_content();

        // Idle detection
        let idle_seconds = self.idle_detector.get_idle_seconds();

        let capture_duration_ms = start.elapsed().as_millis() as u64;

        let context = Context {
            id: Uuid::new_v4().to_string(),
            app,
            clipboard,
            idle_seconds,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            capture_duration_ms,
        };

        self.last_capture = Some(Instant::now());

        debug!(
            "Context captured in {}ms: app={}, clipboard={}",
            capture_duration_ms,
            context.app.name,
            context.clipboard.is_some()
        );

        Ok(context)
    }

    /// Reset l'activité utilisateur avec type (appelé sur mouvement souris/clavier)
    pub fn reset_user_activity(&mut self, activity_type: ActivityType) {
        self.idle_detector.reset_activity(activity_type);
    }

    /// Récupère l'état complet de l'idle (OS + local + effective)
    pub fn get_idle_state(&self) -> IdleState {
        self.idle_detector.get_idle_state()
    }

    /// Get the last captured context for preview purposes
    pub fn get_last_context(&mut self) -> Result<Context, ContextError> {
        // Use peek to get current context quickly
        let peek = self.peek()?;
        Ok(Context {
            id: Uuid::new_v4().to_string(),
            app: peek.app,
            clipboard: None,
            idle_seconds: peek.idle_seconds,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            capture_duration_ms: 0,
        })
    }
}

impl Default for ContextAggregator {
    fn default() -> Self {
        Self::new().expect("Failed to initialize context aggregator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_aggregator_creation() {
        let result = ContextAggregator::new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_capture() {
        let mut aggregator = ContextAggregator::new().unwrap();
        let result = aggregator.capture().await;

        match result {
            Ok(ctx) => {
                assert!(!ctx.id.is_empty());
                assert!(!ctx.app.name.is_empty());
                assert!(ctx.idle_seconds >= 0.0);
                assert!(ctx.capture_duration_ms < 100); // Should be fast
                info!("Context captured successfully: {:?}", ctx);
            }
            Err(e) => {
                println!("Context capture error (expected on CI): {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_multiple_captures() {
        let mut aggregator = ContextAggregator::new().unwrap();

        // First capture
        if let Ok(ctx1) = aggregator.capture().await {
            // Second capture (should use cache)
            if let Ok(ctx2) = aggregator.capture().await {
                assert_ne!(ctx1.id, ctx2.id); // Different IDs
                assert!(ctx2.capture_duration_ms <= ctx1.capture_duration_ms * 2);
                // Should be similar
            }
        }
    }

    #[tokio::test]
    async fn test_has_recent_capture() {
        let mut aggregator = ContextAggregator::new().unwrap();

        assert!(!aggregator.has_recent_capture(Duration::from_secs(1)));

        if aggregator.capture().await.is_ok() {
            assert!(aggregator.has_recent_capture(Duration::from_secs(1)));
        }
    }

    #[tokio::test]
    async fn test_capture_latency() {
        let mut aggregator = ContextAggregator::new().unwrap();

        // Measure p95 latency over multiple captures
        let mut latencies = Vec::new();

        for _ in 0..20 {
            let start = Instant::now();
            if aggregator.capture().await.is_ok() {
                let latency = start.elapsed().as_millis() as u64;
                latencies.push(latency);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        if !latencies.is_empty() {
            latencies.sort_unstable();
            let p95_index = (latencies.len() as f64 * 0.95) as usize;
            let p95 = latencies[p95_index.min(latencies.len() - 1)];

            info!("Capture latency p95: {}ms", p95);
            assert!(p95 < 50, "p95 latency too high: {}ms", p95);
        }
    }

    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn test_memory_footprint_500_captures() {
        use std::process::Command;

        let mut aggregator = ContextAggregator::new().unwrap();

        // Get initial memory
        let pid = std::process::id();
        let mem_before = get_process_rss_mb(pid);

        info!("RSS before captures: {} MB", mem_before);

        // Perform 500 captures
        for i in 0..500 {
            if i % 100 == 0 {
                info!("Captured {}/500...", i);
            }
            let _ = aggregator.capture().await;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        // Get final memory
        let mem_after = get_process_rss_mb(pid);
        let mem_delta = mem_after - mem_before;

        info!(
            "RSS after 500 captures: {} MB (delta: {} MB)",
            mem_after, mem_delta
        );

        // Allow 30MB total RSS
        assert!(mem_after < 30.0, "RSS too high: {} MB", mem_after);
    }
}

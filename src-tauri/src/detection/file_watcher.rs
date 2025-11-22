/**
 * File Watcher for Phase 3B
 *
 * Monitors file changes in the project directory and triggers
 * pattern detection when files are modified.
 */

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

use super::patterns::{detect_debug_pattern, detect_refacto_pattern};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    watch_path: PathBuf,
}

impl FileWatcher {
    /// Create a new FileWatcher for the given project directory
    pub fn new(project_path: PathBuf) -> Result<Self, notify::Error> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    error!("Failed to send file watch event: {}", e);
                }
            },
            Config::default()
                .with_poll_interval(Duration::from_secs(1))
                .with_compare_contents(true),
        )?;

        info!("📁 FileWatcher created for: {:?}", project_path);

        Ok(FileWatcher {
            _watcher: watcher,
            receiver: rx,
            watch_path: project_path,
        })
    }

    /// Start watching the project directory
    pub fn start(&mut self) -> Result<(), notify::Error> {
        self._watcher
            .watch(&self.watch_path, RecursiveMode::Recursive)?;
        info!("👀 FileWatcher started watching: {:?}", self.watch_path);
        Ok(())
    }

    /// Process file change events and trigger pattern detection
    pub fn process_events(&self, app: &AppHandle) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                Ok(event) => {
                    if self.should_process_event(&event) {
                        self.handle_file_change(app, &event);
                    }
                }
                Err(e) => {
                    warn!("File watch error: {}", e);
                }
            }
        }
    }

    /// Check if we should process this event
    fn should_process_event(&self, event: &Event) -> bool {
        // Only process modify events
        matches!(event.kind, EventKind::Modify(_))
            && event.paths.iter().any(|path| {
                // Only watch source code files
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
                        matches!(
                            ext,
                            "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "cpp"
                                | "c" | "h"
                        )
                    })
                    .unwrap_or(false)
                    // Ignore node_modules, target, dist, etc.
                    && !path.to_str().unwrap_or("").contains("node_modules")
                    && !path.to_str().unwrap_or("").contains("target")
                    && !path.to_str().unwrap_or("").contains("dist")
                    && !path.to_str().unwrap_or("").contains(".git")
            })
    }

    /// Handle a file change event by running pattern detection
    fn handle_file_change(&self, app: &AppHandle, event: &Event) {
        for path in &event.paths {
            info!("📝 File modified: {:?}", path);

            // Read file content
            if let Ok(content) = std::fs::read_to_string(path) {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                // Run refacto pattern detection
                if let Some(opportunity) = detect_refacto_pattern(&content, file_name) {
                    info!("🔧 Refacto pattern detected in {}", file_name);
                    if let Err(e) = app.emit("opportunity:new", &opportunity) {
                        error!("Failed to emit refacto opportunity: {}", e);
                    } else {
                        // Emit HUD pulse
                        if let Err(e) = app.emit("hud:pulse", serde_json::json!({"state": "opportunity"})) {
                            error!("Failed to emit HUD pulse: {}", e);
                        }
                    }
                }

                // Run debug pattern detection
                if let Some(opportunity) = detect_debug_pattern(&content, file_name) {
                    info!("🐛 Debug pattern detected in {}", file_name);
                    if let Err(e) = app.emit("opportunity:new", &opportunity) {
                        error!("Failed to emit debug opportunity: {}", e);
                    } else {
                        // Emit HUD pulse
                        if let Err(e) = app.emit("hud:pulse", serde_json::json!({"state": "opportunity"})) {
                            error!("Failed to emit HUD pulse: {}", e);
                        }
                    }
                }
            }
        }
    }
}

/// Start file watcher in background task
pub fn start_file_watcher(app: AppHandle, project_path: PathBuf) {
    tokio::spawn(async move {
        match FileWatcher::new(project_path) {
            Ok(mut watcher) => {
                if let Err(e) = watcher.start() {
                    error!("Failed to start file watcher: {}", e);
                    return;
                }

                info!("✅ File watcher started successfully");

                // Process events in loop
                loop {
                    watcher.process_events(&app);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
            Err(e) => {
                error!("Failed to create file watcher: {}", e);
            }
        }
    });
}

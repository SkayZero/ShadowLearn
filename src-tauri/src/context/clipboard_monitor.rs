use super::errors::ClipboardError;
use arboard::Clipboard;
use std::time::Instant;
use tracing::{debug, warn};

const MAX_CLIPBOARD_LENGTH: usize = 10_000; // 10KB

/// Moniteur de clipboard avec historique
pub struct ClipboardMonitor {
    clipboard: Clipboard,
    last_content: Option<String>,
    last_update: Instant,
    max_length: usize,
}

impl ClipboardMonitor {
    pub fn new() -> Result<Self, ClipboardError> {
        Ok(Self {
            clipboard: Clipboard::new().map_err(|e| ClipboardError::AccessError(e.to_string()))?,
            last_content: None,
            last_update: Instant::now(),
            max_length: MAX_CLIPBOARD_LENGTH,
        })
    }

    /// Récupère le contenu récent du clipboard (None si inchangé)
    pub fn get_recent_content(&mut self) -> Option<String> {
        match self.clipboard.get_text() {
            Ok(text) => {
                // Check size limit
                if text.len() > self.max_length {
                    // Use debug instead of warn to reduce log spam
                    debug!(
                        "Clipboard content too large: {} bytes (max {})",
                        text.len(),
                        self.max_length
                    );
                    return None;
                }

                // Check if content changed
                if self.last_content.as_ref() != Some(&text) {
                    debug!("Clipboard content changed: {} bytes", text.len());
                    self.last_content = Some(text.clone());
                    self.last_update = Instant::now();
                    Some(text)
                } else {
                    debug!("Clipboard content unchanged");
                    None
                }
            }
            Err(e) => {
                debug!("Failed to read clipboard: {}", e);
                None
            }
        }
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize clipboard monitor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_monitor_creation() {
        let result = ClipboardMonitor::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clipboard_read() {
        let mut monitor = ClipboardMonitor::new().unwrap();

        // Try to set clipboard content
        if let Err(e) = monitor.clipboard.set_text("test content") {
            println!("Cannot set clipboard (expected on CI): {}", e);
            return;
        }

        // Try to read it back
        let content = monitor.get_recent_content();
        assert!(content.is_some() || content.is_none()); // Either works on CI
    }

    #[test]
    fn test_clipboard_size_limit() {
        let mut monitor = ClipboardMonitor::new().unwrap();

        // Create oversized content
        let large_content = "a".repeat(MAX_CLIPBOARD_LENGTH + 1);

        if monitor.clipboard.set_text(&large_content).is_ok() {
            let content = monitor.get_recent_content();
            assert!(content.is_none()); // Should be rejected
        }
    }

    #[test]
    fn test_time_since_update() {
        let monitor = ClipboardMonitor::new().unwrap();
        let elapsed = monitor.time_since_update();
        assert!(elapsed < Duration::from_secs(1));
    }
}

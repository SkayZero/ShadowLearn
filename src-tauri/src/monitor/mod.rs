pub mod screen_monitor;
pub mod change_detector;
pub mod commands;
pub mod vision_client;
pub mod ocr_client;

pub use screen_monitor::{ScreenMonitor, MonitorConfig, ScreenChange};
pub use change_detector::ChangeDetector;
pub use vision_client::ClaudeVisionClient;
pub use ocr_client::{LocalOCR, OCRResult, DetectedPattern};

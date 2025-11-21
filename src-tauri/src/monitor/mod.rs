pub mod screen_monitor;
pub mod change_detector;
pub mod commands;
pub mod vision_client;
pub mod ocr_client;
pub mod smart_cache;

pub use screen_monitor::{ScreenMonitor, MonitorConfig};
pub use ocr_client::{OCRResult, DetectedPattern};

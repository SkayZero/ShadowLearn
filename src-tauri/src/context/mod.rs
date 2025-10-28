pub mod aggregator;
pub mod app_detector;
pub mod clipboard_monitor;
pub mod errors;
pub mod idle_detector;
pub mod preview;

pub use aggregator::{Context, ContextAggregator};
pub use idle_detector::{ActivityType, IdleState};

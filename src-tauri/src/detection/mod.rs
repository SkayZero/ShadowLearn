/**
 * Phase 3B: Real Opportunity Detection
 *
 * This module handles intelligent detection of learning opportunities
 * based on file changes and code patterns.
 */

pub mod file_watcher;
pub mod patterns;

pub use file_watcher::FileWatcher;
pub use patterns::{detect_refacto_pattern, detect_debug_pattern};

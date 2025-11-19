/**
 * Pattern Recognition Module
 * Phase 2.1 - ML-based workflow pattern learning and prediction
 */

pub mod learning;
pub mod prediction;
pub mod repetition;
pub mod storage;
pub mod commands;

pub use learning::PatternLearner;
pub use prediction::ActionPredictor;
pub use repetition::RepetitionDetector;
pub use storage::PatternStorage;

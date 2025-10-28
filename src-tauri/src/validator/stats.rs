use crate::validator::ArtefactType;
use crate::validator::ValidationResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Statistics tracking for validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub valid_count: u64,
    pub invalid_count: u64,
    pub error_count: u64,
    pub skipped_count: u64,
    pub timeout_count: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub by_type: HashMap<String, TypeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStats {
    pub count: u64,
    pub valid: u64,
    pub invalid: u64,
    pub errors: u64,
    pub skipped: u64,
    pub timeouts: u64,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
}

impl ValidationStats {
    pub fn new() -> Self {
        Self {
            total_validations: 0,
            cache_hits: 0,
            cache_misses: 0,
            valid_count: 0,
            invalid_count: 0,
            error_count: 0,
            skipped_count: 0,
            timeout_count: 0,
            total_duration_ms: 0,
            average_duration_ms: 0.0,
            by_type: HashMap::new(),
        }
    }

    pub fn record_validation(
        &mut self,
        artefact_type: ArtefactType,
        result: &ValidationResult,
        duration: Duration,
    ) {
        self.total_validations += 1;
        self.total_duration_ms += duration.as_millis() as u64;
        self.average_duration_ms = self.total_duration_ms as f64 / self.total_validations as f64;

        // Update global counters
        match result {
            ValidationResult::Valid => self.valid_count += 1,
            ValidationResult::Invalid(_) => self.invalid_count += 1,
            ValidationResult::Error(_) => self.error_count += 1,
            ValidationResult::Skipped(_) => self.skipped_count += 1,
        }

        // Update type-specific stats
        let type_name = format!("{:?}", artefact_type);
        let type_stats = self.by_type.entry(type_name).or_insert_with(TypeStats::new);

        type_stats.count += 1;
        type_stats.total_duration_ms += duration.as_millis() as u64;
        type_stats.average_duration_ms =
            type_stats.total_duration_ms as f64 / type_stats.count as f64;

        match result {
            ValidationResult::Valid => type_stats.valid += 1,
            ValidationResult::Invalid(_) => type_stats.invalid += 1,
            ValidationResult::Error(_) => type_stats.errors += 1,
            ValidationResult::Skipped(_) => type_stats.skipped += 1,
        }
    }

    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn record_timeout(&mut self) {
        self.timeout_count += 1;
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.total_validations == 0 {
            return 0.0;
        }
        self.valid_count as f64 / self.total_validations as f64
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        let total_cache_operations = self.cache_hits + self.cache_misses;
        if total_cache_operations == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total_cache_operations as f64
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl TypeStats {
    fn new() -> Self {
        Self {
            count: 0,
            valid: 0,
            invalid: 0,
            errors: 0,
            skipped: 0,
            timeouts: 0,
            total_duration_ms: 0,
            average_duration_ms: 0.0,
        }
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.valid as f64 / self.count as f64
    }
}

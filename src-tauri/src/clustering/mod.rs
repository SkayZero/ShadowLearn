#![allow(unused_imports)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

pub mod fingerprint;
pub mod manager;

use fingerprint::*;
use manager::*;

/// Main clustering system for grouping similar contexts
#[derive(Debug)]
#[allow(dead_code)] // J21.5: Will be used in J22
#[allow(dead_code)]
pub struct ClusteringSystem {
    cluster_manager: ClusterManager,
    fingerprint_generator: FingerprintGenerator,
    stats: ClusteringStats,
}

#[allow(dead_code)]
impl ClusteringSystem {
    pub fn new() -> Self {
        Self {
            cluster_manager: ClusterManager::new(),
            fingerprint_generator: FingerprintGenerator::new(),
            stats: ClusteringStats::new(),
        }
    }

    /// Process a context and find/create appropriate cluster
    pub async fn process_context(
        &mut self,
        ctx: &crate::context::aggregator::Context,
    ) -> Result<ProcessedContext, String> {
        let start_time = Instant::now();

        // Generate fingerprint
        let fingerprint = self.fingerprint_generator.generate(ctx);

        // Find or create cluster
        let cluster_id = self.cluster_manager.find_or_create_cluster(&fingerprint);

        // Update cluster with new data
        self.cluster_manager
            .update_cluster(&cluster_id, fingerprint.simhash);

        // Record stats
        let duration = start_time.elapsed();
        self.stats.record_processing(duration);

        debug!(
            "[CLUSTER] Processed context: cluster={}, duration={:?}",
            cluster_id, duration
        );

        Ok(ProcessedContext {
            context: ctx.clone(),
            fingerprint,
            cluster_id,
            processing_time_ms: duration.as_millis() as u64,
        })
    }

    /// Get cluster information
    pub fn get_cluster_info(&self, cluster_id: &str) -> Option<&Cluster> {
        self.cluster_manager.get_cluster_info(cluster_id)
    }

    /// Get clustering statistics
    pub fn get_stats(&self) -> &ClusteringStats {
        &self.stats
    }

    /// Cleanup old clusters
    pub fn cleanup_old_clusters(&mut self, max_age_days: i64) {
        self.cluster_manager.cleanup_old_clusters(max_age_days);
        info!(
            "[CLUSTER] Cleaned up clusters older than {} days",
            max_age_days
        );
    }

    /// Get all clusters (for debugging/monitoring)
    pub fn get_all_clusters(&self) -> Vec<&Cluster> {
        self.cluster_manager.get_all_clusters()
    }

    /// Get cluster count
    pub fn get_cluster_count(&self) -> usize {
        self.cluster_manager.cluster_count()
    }
}

/// Processed context with clustering information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProcessedContext {
    pub context: crate::context::aggregator::Context,
    pub fingerprint: ContextFingerprint,
    pub cluster_id: String,
    pub processing_time_ms: u64,
}

/// Clustering statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ClusteringStats {
    pub total_contexts_processed: u64,
    pub total_clusters_created: u64,
    pub total_clusters_updated: u64,
    pub average_processing_time_ms: f64,
    pub total_processing_time_ms: u64,
    pub cache_hit_rate: f64,
    pub similarity_distribution: HashMap<String, u64>, // "0.8-0.9", "0.9-1.0", etc.
}

#[allow(dead_code)]
impl ClusteringStats {
    fn new() -> Self {
        Self {
            total_contexts_processed: 0,
            total_clusters_created: 0,
            total_clusters_updated: 0,
            average_processing_time_ms: 0.0,
            total_processing_time_ms: 0,
            cache_hit_rate: 0.0,
            similarity_distribution: HashMap::new(),
        }
    }

    fn record_processing(&mut self, duration: Duration) {
        self.total_contexts_processed += 1;
        self.total_processing_time_ms += duration.as_millis() as u64;
        self.average_processing_time_ms =
            self.total_processing_time_ms as f64 / self.total_contexts_processed as f64;
    }

    fn record_cluster_created(&mut self) {
        self.total_clusters_created += 1;
    }

    fn record_cluster_updated(&mut self) {
        self.total_clusters_updated += 1;
    }

    fn record_similarity(&mut self, similarity: f32) {
        let bucket = if similarity >= 0.95 {
            "0.95-1.00"
        } else if similarity >= 0.90 {
            "0.90-0.95"
        } else if similarity >= 0.85 {
            "0.85-0.90"
        } else if similarity >= 0.80 {
            "0.80-0.85"
        } else {
            "0.00-0.80"
        };

        *self
            .similarity_distribution
            .entry(bucket.to_string())
            .or_insert(0) += 1;
    }
}

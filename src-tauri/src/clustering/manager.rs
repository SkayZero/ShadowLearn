#![allow(unused_imports)]
#![allow(dead_code)]
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::clustering::fingerprint::{ContextFingerprint, FingerprintGenerator};

/// Cluster manager with LRU cache and similarity-based clustering
#[derive(Debug)]
#[allow(dead_code)]
pub struct ClusterManager {
    clusters: LruCache<String, Cluster>,
    similarity_threshold: f32,
    max_clusters: usize,
    stats: ClusterManagerStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Cluster {
    pub id: String,
    pub centroid: u64,
    pub count: usize,
    pub domain: String,
    pub created_at_ms: u64,
    pub last_updated_ms: u64,
    pub similarity_scores: Vec<f32>, // For debugging/analysis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ClusterManagerStats {
    pub total_clusters_created: u64,
    pub total_clusters_updated: u64,
    pub total_clusters_evicted: u64,
    pub average_cluster_size: f64,
    pub similarity_threshold: f32,
}

#[allow(dead_code)]
impl ClusterManager {
    pub fn new() -> Self {
        Self {
            clusters: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            similarity_threshold: 0.85,
            max_clusters: 1000,
            stats: ClusterManagerStats {
                total_clusters_created: 0,
                total_clusters_updated: 0,
                total_clusters_evicted: 0,
                average_cluster_size: 0.0,
                similarity_threshold: 0.85,
            },
        }
    }

    /// Find existing cluster or create new one
    pub fn find_or_create_cluster(&mut self, fp: &ContextFingerprint) -> String {
        // Search existing clusters for similarity
        let mut best_match: Option<(String, f32)> = None;

        for (cluster_id, cluster) in self.clusters.iter() {
            let similarity = FingerprintGenerator::similarity(fp.simhash, cluster.centroid);

            // Bonus for same domain
            let domain_bonus = if fp.domain == cluster.domain {
                0.05
            } else {
                0.0
            };
            let adjusted_similarity = (similarity + domain_bonus).min(1.0);

            if adjusted_similarity >= self.similarity_threshold {
                if let Some((_, best_sim)) = best_match {
                    if adjusted_similarity > best_sim {
                        best_match = Some((cluster_id.clone(), adjusted_similarity));
                    }
                } else {
                    best_match = Some((cluster_id.clone(), adjusted_similarity));
                }
            }
        }

        // Return best match if found
        if let Some((cluster_id, similarity)) = best_match {
            debug!(
                "[CLUSTER] Found matching cluster: {} (similarity: {:.3})",
                cluster_id, similarity
            );
            return cluster_id;
        }

        // Create new cluster
        let cluster_id = format!("cluster_{}", &Uuid::new_v4().to_string()[..8]);
        let cluster = Cluster {
            id: cluster_id.clone(),
            centroid: fp.simhash,
            count: 1,
            domain: fp.domain.clone(),
            created_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            last_updated_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            similarity_scores: vec![1.0], // Perfect similarity to itself
        };

        self.clusters.put(cluster_id.clone(), cluster);
        self.stats.total_clusters_created += 1;

        info!(
            "[CLUSTER] Created new cluster: {} (domain: {})",
            cluster_id, fp.domain
        );
        cluster_id
    }

    /// Update cluster with new fingerprint
    pub fn update_cluster(&mut self, cluster_id: &str, new_simhash: u64) {
        if let Some(cluster) = self.clusters.get_mut(cluster_id) {
            // Update centroid using weighted average
            cluster.count += 1;
            let alpha = 1.0 / cluster.count as f32;

            // Weighted average of simhashes (bit-wise)
            let mut new_centroid = 0u64;
            for i in 0..64 {
                let old_bit = (cluster.centroid >> i) & 1;
                let new_bit = (new_simhash >> i) & 1;

                let weighted =
                    (old_bit as f32 * (1.0 - alpha) + new_bit as f32 * alpha).round() as u64;
                new_centroid |= weighted << i;
            }

            cluster.centroid = new_centroid;
            cluster.last_updated_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            // Track similarity for analysis
            let similarity = FingerprintGenerator::similarity(new_simhash, cluster.centroid);
            cluster.similarity_scores.push(similarity);

            // Keep only last 100 similarity scores
            if cluster.similarity_scores.len() > 100 {
                cluster.similarity_scores.remove(0);
            }

            self.stats.total_clusters_updated += 1;

            debug!(
                "[CLUSTER] Updated cluster: {} (count: {}, similarity: {:.3})",
                cluster_id, cluster.count, similarity
            );
        }
    }

    /// Get cluster information
    pub fn get_cluster_info(&self, cluster_id: &str) -> Option<&Cluster> {
        self.clusters.peek(cluster_id)
    }

    /// Get all clusters (for debugging/monitoring)
    pub fn get_all_clusters(&self) -> Vec<&Cluster> {
        self.clusters.iter().map(|(_, cluster)| cluster).collect()
    }

    /// Cleanup old clusters
    pub fn cleanup_old_clusters(&mut self, max_age_days: i64) {
        let cutoff_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            - (max_age_days as u64 * 86400 * 1000);

        let old_clusters: Vec<String> = self
            .clusters
            .iter()
            .filter(|(_, cluster)| cluster.created_at_ms < cutoff_ms)
            .map(|(id, _)| id.clone())
            .collect();

        let old_count = old_clusters.len();
        for cluster_id in old_clusters {
            self.clusters.pop(&cluster_id);
            self.stats.total_clusters_evicted += 1;
            warn!("[CLUSTER] Evicted old cluster: {}", cluster_id);
        }

        if old_count > 0 {
            info!("[CLUSTER] Evicted {} old clusters", old_count);
        }
    }

    /// Get cluster manager statistics
    pub fn get_stats(&self) -> &ClusterManagerStats {
        &self.stats
    }

    /// Get cluster count
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    /// Check if cluster exists
    pub fn has_cluster(&self, cluster_id: &str) -> bool {
        self.clusters.contains(cluster_id)
    }

    /// Merge similar clusters (advanced feature)
    pub fn merge_similar_clusters(&mut self, similarity_threshold: f32) -> usize {
        let mut merged_count = 0;
        let cluster_ids: Vec<String> = self.clusters.iter().map(|(id, _)| id.clone()).collect();

        for i in 0..cluster_ids.len() {
            for j in (i + 1)..cluster_ids.len() {
                let id1 = &cluster_ids[i];
                let id2 = &cluster_ids[j];

                if let (Some(cluster1), Some(cluster2)) =
                    (self.clusters.peek(id1), self.clusters.peek(id2))
                {
                    let similarity =
                        FingerprintGenerator::similarity(cluster1.centroid, cluster2.centroid);

                    if similarity >= similarity_threshold {
                        // Merge cluster2 into cluster1
                        if let Some(mut cluster1) = self.clusters.pop(id1) {
                            if let Some(cluster2) = self.clusters.pop(id2) {
                                cluster1.count += cluster2.count;
                                cluster1
                                    .similarity_scores
                                    .extend(cluster2.similarity_scores);

                                // Update centroid
                                let alpha = cluster2.count as f32 / cluster1.count as f32;
                                let mut new_centroid = 0u64;
                                for k in 0..64 {
                                    let bit1 = (cluster1.centroid >> k) & 1;
                                    let bit2 = (cluster2.centroid >> k) & 1;
                                    let weighted =
                                        (bit1 as f32 * (1.0 - alpha) + bit2 as f32 * alpha).round()
                                            as u64;
                                    new_centroid |= weighted << k;
                                }
                                cluster1.centroid = new_centroid;

                                self.clusters.put(id1.clone(), cluster1);
                                merged_count += 1;

                                info!(
                                    "[CLUSTER] Merged clusters: {} + {} (similarity: {:.3})",
                                    id1, id2, similarity
                                );
                            }
                        }
                    }
                }
            }
        }

        merged_count
    }
}

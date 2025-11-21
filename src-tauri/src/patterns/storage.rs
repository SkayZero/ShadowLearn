/**
 * Pattern Storage System
 * Persistent storage for learned patterns and predictions
 */

use super::learning::WorkflowPattern;
use super::repetition::RepetitiveTask;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use tracing::{info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PatternDatabase {
    patterns: Vec<WorkflowPattern>,
    repetitive_tasks: Vec<RepetitiveTask>,
    metadata: DatabaseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseMetadata {
    version: String,
    last_updated: String,
    total_patterns: usize,
    total_tasks: usize,
}

pub struct PatternStorage {
    storage_path: PathBuf,
}

impl PatternStorage {
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        let storage_path = app_dir.join("patterns.json");

        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        Ok(Self { storage_path })
    }

    /// Save patterns to disk
    pub fn save_patterns(&self, patterns: &[WorkflowPattern]) -> Result<(), String> {
        debug!("💾 Saving {} patterns to disk", patterns.len());

        let mut db = self.load_database().unwrap_or_else(|_| PatternDatabase {
            patterns: Vec::new(),
            repetitive_tasks: Vec::new(),
            metadata: DatabaseMetadata {
                version: "1.0".to_string(),
                last_updated: chrono::Utc::now().to_rfc3339(),
                total_patterns: 0,
                total_tasks: 0,
            },
        });

        db.patterns = patterns.to_vec();
        db.metadata.total_patterns = patterns.len();
        db.metadata.last_updated = chrono::Utc::now().to_rfc3339();

        self.write_database(&db)?;

        info!("✅ Saved {} patterns to disk", patterns.len());
        Ok(())
    }

    /// Load patterns from disk
    pub fn load_patterns(&self) -> Result<Vec<WorkflowPattern>, String> {
        debug!("📖 Loading patterns from disk");

        let db = self.load_database()?;

        info!("✅ Loaded {} patterns from disk", db.patterns.len());
        Ok(db.patterns)
    }

    /// Save repetitive tasks to disk
    pub fn save_tasks(&self, tasks: &[RepetitiveTask]) -> Result<(), String> {
        debug!("💾 Saving {} repetitive tasks to disk", tasks.len());

        let mut db = self.load_database().unwrap_or_else(|_| PatternDatabase {
            patterns: Vec::new(),
            repetitive_tasks: Vec::new(),
            metadata: DatabaseMetadata {
                version: "1.0".to_string(),
                last_updated: chrono::Utc::now().to_rfc3339(),
                total_patterns: 0,
                total_tasks: 0,
            },
        });

        db.repetitive_tasks = tasks.to_vec();
        db.metadata.total_tasks = tasks.len();
        db.metadata.last_updated = chrono::Utc::now().to_rfc3339();

        self.write_database(&db)?;

        info!("✅ Saved {} repetitive tasks to disk", tasks.len());
        Ok(())
    }

    /// Load repetitive tasks from disk
    pub fn load_tasks(&self) -> Result<Vec<RepetitiveTask>, String> {
        debug!("📖 Loading repetitive tasks from disk");

        let db = self.load_database()?;

        info!("✅ Loaded {} repetitive tasks from disk", db.repetitive_tasks.len());
        Ok(db.repetitive_tasks)
    }

    /// Clear all stored data
    pub fn clear(&self) -> Result<(), String> {
        info!("🗑️ Clearing pattern storage");

        if self.storage_path.exists() {
            fs::remove_file(&self.storage_path)
                .map_err(|e| format!("Failed to clear storage: {}", e))?;
        }

        Ok(())
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> Result<StorageStats, String> {
        let db = self.load_database()?;

        let file_size = fs::metadata(&self.storage_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(StorageStats {
            total_patterns: db.patterns.len(),
            total_tasks: db.repetitive_tasks.len(),
            file_size_bytes: file_size,
            last_updated: db.metadata.last_updated,
        })
    }

    /// Load complete database
    fn load_database(&self) -> Result<PatternDatabase, String> {
        if !self.storage_path.exists() {
            return Err("Storage file does not exist".to_string());
        }

        let contents = fs::read_to_string(&self.storage_path)
            .map_err(|e| format!("Failed to read storage file: {}", e))?;

        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse storage file: {}", e))
    }

    /// Write complete database
    fn write_database(&self, db: &PatternDatabase) -> Result<(), String> {
        let contents = serde_json::to_string_pretty(db)
            .map_err(|e| format!("Failed to serialize database: {}", e))?;

        fs::write(&self.storage_path, contents)
            .map_err(|e| format!("Failed to write storage file: {}", e))?;

        Ok(())
    }

    /// Export patterns to JSON string (for debugging/sharing)
    pub fn export_json(&self) -> Result<String, String> {
        let db = self.load_database()?;
        serde_json::to_string_pretty(&db)
            .map_err(|e| format!("Failed to export JSON: {}", e))
    }

    /// Import patterns from JSON string
    pub fn import_json(&self, json: &str) -> Result<(), String> {
        let db: PatternDatabase = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse import JSON: {}", e))?;

        self.write_database(&db)?;

        info!("✅ Imported {} patterns and {} tasks",
            db.patterns.len(), db.repetitive_tasks.len());

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_patterns: usize,
    pub total_tasks: usize,
    pub file_size_bytes: u64,
    pub last_updated: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = PatternStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Initially should fail to load
        assert!(storage.load_patterns().is_err());

        // Save empty patterns
        storage.save_patterns(&[]).unwrap();

        // Should now load successfully
        let patterns = storage.load_patterns().unwrap();
        assert_eq!(patterns.len(), 0);

        // Clear storage
        storage.clear().unwrap();
        assert!(storage.load_patterns().is_err());
    }

    #[test]
    fn test_stats() {
        let temp_dir = TempDir::new().unwrap();
        let storage = PatternStorage::new(temp_dir.path().to_path_buf()).unwrap();

        storage.save_patterns(&[]).unwrap();

        let stats = storage.get_stats().unwrap();
        assert_eq!(stats.total_patterns, 0);
        assert!(stats.file_size_bytes > 0);
    }
}

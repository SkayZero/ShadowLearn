pub mod compression;
pub mod database;
pub mod models;

pub use compression::CompressionManager;
pub use database::DatabaseManager;
pub use models::*;

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Gestionnaire principal de persistance
pub struct PersistenceManager {
    database: Arc<Mutex<DatabaseManager>>,
    compression: CompressionManager,
}

impl PersistenceManager {
    /// Initialise le gestionnaire de persistance
    pub async fn new() -> Result<Self, String> {
        info!("🗄️ Initializing PersistenceManager...");

        let database = DatabaseManager::new().await?;
        let compression = CompressionManager::new();

        let manager = Self {
            database: Arc::new(Mutex::new(database)),
            compression,
        };

        info!("✅ PersistenceManager initialized successfully");
        Ok(manager)
    }

    /// Crée une nouvelle conversation
    pub async fn create_conversation(
        &self,
        title: String,
        app_context: Option<String>,
    ) -> Result<Conversation, String> {
        let conversation = Conversation {
            id: Uuid::new_v4().to_string(),
            title,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            app_context,
            message_count: 0,
            is_archived: false,
        };

        let db = self.database.lock().await;
        let result = db.save_conversation(&conversation).await?;

        if result.success {
            debug!("💾 Created conversation: {}", conversation.id);
            Ok(conversation)
        } else {
            Err(result.error.unwrap_or("Unknown error".to_string()))
        }
    }

    /// Sauvegarde un message dans une conversation
    pub async fn save_message(
        &self,
        conversation_id: &str,
        role: MessageRole,
        content: String,
        metadata: Option<String>,
    ) -> Result<Message, String> {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            conversation_id: conversation_id.to_string(),
            role,
            content,
            created_at: Utc::now(),
            metadata,
        };

        let db = self.database.lock().await;
        let result = db.save_message(&message).await?;

        if result.success {
            debug!(
                "💾 Saved message: {} in conversation: {}",
                message.id, conversation_id
            );
            Ok(message)
        } else {
            Err(result.error.unwrap_or("Unknown error".to_string()))
        }
    }

    /// Récupère les conversations récentes
    pub async fn get_recent_conversations(&self, limit: i32) -> Result<Vec<Conversation>, String> {
        let db = self.database.lock().await;
        db.get_recent_conversations(limit).await
    }

    /// Récupère les messages d'une conversation
    pub async fn get_conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<Message>, String> {
        let db = self.database.lock().await;
        db.get_conversation_messages(conversation_id).await
    }

    /// Sauvegarde un contexte capturé
    pub async fn save_context(&self, context: CapturedContext) -> Result<(), String> {
        // Compresser les données de screenshot si nécessaire
        let mut compressed_context = context;
        if let Some(screenshot_data) = &compressed_context.screenshot_data {
            if self.compression.should_compress(screenshot_data.len()) {
                match self.compression.compress_to_base64(screenshot_data) {
                    Ok(compressed) => {
                        let original_len = screenshot_data.len();
                        compressed_context.screenshot_data = Some(compressed.clone());
                        debug!(
                            "📦 Compressed screenshot data: {} -> {} bytes",
                            original_len,
                            compressed.len()
                        );
                    }
                    Err(e) => {
                        error!("❌ Failed to compress screenshot: {}", e);
                        // Continuer sans compression
                    }
                }
            }
        }

        // Sauvegarder le contexte dans la base de données
        let db = self.database.lock().await;
        db.save_context(&compressed_context).await?;
        debug!("💾 Context saved: {}", compressed_context.id);
        Ok(())
    }

    /// Récupère les contextes récents pour une app
    pub async fn get_recent_contexts_for_app(
        &self,
        app_name: &str,
        limit: i32,
    ) -> Result<Vec<CapturedContext>, String> {
        let db = self.database.lock().await;
        let contexts = db.get_recent_contexts_for_app(app_name, limit).await?;
        debug!(
            "🔍 Fetched {} contexts for app: {}",
            contexts.len(),
            app_name
        );
        Ok(contexts)
    }

    /// Obtient les statistiques de persistance
    pub async fn get_stats(&self) -> Result<PersistenceStats, String> {
        let db = self.database.lock().await;
        db.get_persistence_stats().await
    }

    /// Exporte toutes les données vers un fichier JSON
    pub async fn export_data(&self, file_path: &str) -> Result<(), String> {
        info!("📤 Exporting data to: {}", file_path);

        let conversations = self.get_recent_conversations(1000).await?;
        let mut export_data = serde_json::Map::new();

        // Exporter les conversations avec leurs messages
        let mut conversations_data = Vec::new();
        for conversation in conversations {
            let messages = self.get_conversation_messages(&conversation.id).await?;
            let conversation_data = serde_json::json!({
                "conversation": conversation,
                "messages": messages
            });
            conversations_data.push(conversation_data);
        }

        export_data.insert(
            "conversations".to_string(),
            serde_json::Value::Array(conversations_data),
        );
        export_data.insert(
            "export_timestamp".to_string(),
            serde_json::Value::String(Utc::now().to_rfc3339()),
        );

        let json_data = serde_json::to_string_pretty(&export_data)
            .map_err(|e| format!("Failed to serialize export data: {}", e))?;

        std::fs::write(file_path, json_data)
            .map_err(|e| format!("Failed to write export file: {}", e))?;

        info!("✅ Data exported successfully");
        Ok(())
    }

    /// Obtenir une référence à la base de données
    pub fn get_database(&self) -> Arc<Mutex<DatabaseManager>> {
        self.database.clone()
    }
}

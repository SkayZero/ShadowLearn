use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Représente une conversation complète
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub app_context: Option<String>, // App qui a déclenché la conversation
    pub message_count: i32,
    pub is_archived: bool,
}

/// Représente un message dans une conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON metadata (trigger context, etc.)
}

/// Rôle d'un message
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "message_role", rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Contexte capturé lors d'un trigger
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CapturedContext {
    pub id: String,
    pub conversation_id: Option<String>, // Lié à une conversation si applicable
    pub app_name: String,
    pub app_bundle_id: String,
    pub window_title: String,
    pub clipboard_content: Option<String>,
    pub idle_seconds: f64,
    pub screenshot_data: Option<String>, // Base64 compressed
    pub created_at: DateTime<Utc>,
    pub trigger_reason: String,   // "idle_ok", "cooldown_ok", etc.
    pub capture_duration_ms: i64, // Durée de capture en millisecondes
}

/// Statistiques de persistance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceStats {
    pub total_conversations: i64,
    pub total_messages: i64,
    pub total_contexts: i64,
    pub database_size_bytes: i64,
    pub last_backup_at: Option<DateTime<Utc>>,
    pub compression_ratio: Option<f64>,
}

/// Résultat d'une opération de persistance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub operation_time_ms: u64,
}

impl<T> PersistenceResult<T> {
    pub fn success(data: T, operation_time_ms: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            operation_time_ms,
        }
    }

    pub fn error(error: String, operation_time_ms: u64) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            operation_time_ms,
        }
    }
}

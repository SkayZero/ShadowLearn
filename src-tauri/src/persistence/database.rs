use chrono::Utc;
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::persistence::models::*;

/// Gestionnaire de base de données SQLite pour la persistance
pub struct DatabaseManager {
    pool: SqlitePool,
    db_path: PathBuf,
}

impl DatabaseManager {
    /// Initialise la base de données et crée les tables
    pub async fn new() -> Result<Self, String> {
        // Utiliser une base de données en mémoire pour tester
        let database_url = "sqlite::memory:";

        info!("🗄️ Initializing in-memory database");

        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?;

        let manager = Self {
            pool,
            db_path: std::path::PathBuf::from(":memory:"),
        };

        // Créer les tables
        manager.create_tables().await?;

        info!("✅ Database initialized successfully");
        Ok(manager)
    }

    /// Obtient le chemin de la base de données
    fn get_database_path() -> Result<PathBuf, String> {
        // Utiliser un chemin temporaire pour tester
        let temp_dir = std::env::temp_dir();
        let shadowlearn_dir = temp_dir.join("ShadowLearn");
        std::fs::create_dir_all(&shadowlearn_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        Ok(shadowlearn_dir.join("shadowlearn.db"))
    }

    /// Crée toutes les tables nécessaires
    async fn create_tables(&self) -> Result<(), String> {
        debug!("Creating database tables...");

        // Exécuter les migrations dans l'ordre
        self.run_migration("001_initial_schema.sql").await?;
        self.run_migration("002_add_trust_tables.sql").await?;

        debug!("✅ Database tables created successfully");
        Ok(())
    }

    /// Exécuter une migration SQL
    async fn run_migration(&self, migration_file: &str) -> Result<(), String> {
        debug!("Running migration: {}", migration_file);

        let migration_sql = match migration_file {
            "001_initial_schema.sql" => include_str!("migrations/001_initial_schema.sql"),
            "002_add_trust_tables.sql" => include_str!("migrations/002_add_trust_tables.sql"),
            _ => return Err(format!("Unknown migration file: {}", migration_file)),
        };

        // Diviser les requêtes par ';' et les exécuter
        for statement in migration_sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() && !statement.starts_with("--") {
                debug!("Executing statement: {}", statement);
                sqlx::query(statement)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| {
                        format!(
                            "Failed to execute migration statement in {}: {} - Error: {}",
                            migration_file, statement, e
                        )
                    })?;
                debug!("Statement executed successfully");
            }
        }

        debug!("✅ Migration {} completed", migration_file);
        Ok(())
    }

    /// Sauvegarde une nouvelle conversation
    pub async fn save_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<PersistenceResult<Conversation>, String> {
        let start = std::time::Instant::now();

        let result = sqlx::query(
            r#"
            INSERT INTO conversations (id, title, created_at, updated_at, app_context, message_count, is_archived)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&conversation.id)
        .bind(&conversation.title)
        .bind(conversation.created_at.to_rfc3339())
        .bind(conversation.updated_at.to_rfc3339())
        .bind(&conversation.app_context)
        .bind(conversation.message_count)
        .bind(conversation.is_archived)
        .execute(&self.pool)
        .await;

        let operation_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(_) => {
                debug!("💾 Conversation saved: {}", conversation.id);
                Ok(PersistenceResult::success(
                    conversation.clone(),
                    operation_time,
                ))
            }
            Err(e) => {
                error!("❌ Failed to save conversation: {}", e);
                Ok(PersistenceResult::error(e.to_string(), operation_time))
            }
        }
    }

    /// Sauvegarde un message
    pub async fn save_message(
        &self,
        message: &Message,
    ) -> Result<PersistenceResult<Message>, String> {
        let start = std::time::Instant::now();

        let result = sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, role, content, created_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(&message.conversation_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(message.created_at.to_rfc3339())
        .bind(&message.metadata)
        .execute(&self.pool)
        .await;

        let operation_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(_) => {
                // Mettre à jour le compteur de messages dans la conversation
                self.update_conversation_message_count(&message.conversation_id)
                    .await?;

                debug!("💾 Message saved: {}", message.id);
                Ok(PersistenceResult::success(message.clone(), operation_time))
            }
            Err(e) => {
                error!("❌ Failed to save message: {}", e);
                Ok(PersistenceResult::error(e.to_string(), operation_time))
            }
        }
    }

    /// Met à jour le compteur de messages d'une conversation
    async fn update_conversation_message_count(&self, conversation_id: &str) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE conversations 
            SET message_count = (
                SELECT COUNT(*) FROM messages WHERE conversation_id = ?
            ),
            updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(conversation_id)
        .bind(Utc::now().to_rfc3339())
        .bind(conversation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update message count: {}", e))?;

        Ok(())
    }

    /// Récupère les conversations récentes
    pub async fn get_recent_conversations(&self, limit: i32) -> Result<Vec<Conversation>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, created_at, updated_at, app_context, message_count, is_archived
            FROM conversations
            WHERE is_archived = FALSE
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch conversations: {}", e))?;

        let conversations = rows
            .into_iter()
            .map(|row| Conversation {
                id: row.get("id"),
                title: row.get("title"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .unwrap()
                .with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .unwrap()
                .with_timezone(&Utc),
                app_context: row.get("app_context"),
                message_count: row.get("message_count"),
                is_archived: row.get("is_archived"),
            })
            .collect();

        Ok(conversations)
    }

    /// Récupère les messages d'une conversation
    pub async fn get_conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<Message>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, conversation_id, role, content, created_at, metadata
            FROM messages
            WHERE conversation_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch messages: {}", e))?;

        let messages = rows
            .into_iter()
            .map(|row| Message {
                id: row.get("id"),
                conversation_id: row.get("conversation_id"),
                role: match row.get::<String, _>("role").as_str() {
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "system" => MessageRole::System,
                    _ => MessageRole::User,
                },
                content: row.get("content"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .unwrap()
                .with_timezone(&Utc),
                metadata: row.get("metadata"),
            })
            .collect();

        Ok(messages)
    }

    /// Obtient les statistiques de persistance
    pub async fn get_persistence_stats(&self) -> Result<PersistenceStats, String> {
        let conversations_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM conversations")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to get conversations count: {}", e))?;

        let messages_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to get messages count: {}", e))?;

        let contexts_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM captured_contexts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to get contexts count: {}", e))?;

        let db_size = std::fs::metadata(&self.db_path)
            .map_err(|e| format!("Failed to get database size: {}", e))?
            .len() as i64;

        Ok(PersistenceStats {
            total_conversations: conversations_count.0,
            total_messages: messages_count.0,
            total_contexts: contexts_count.0,
            database_size_bytes: db_size,
            last_backup_at: None,    // TODO: Implémenter les backups
            compression_ratio: None, // TODO: Calculer le ratio de compression
        })
    }

    /// Récupère les contextes récents pour une app
    pub async fn get_recent_contexts_for_app(
        &self,
        app_name: &str,
        limit: i32,
    ) -> Result<Vec<CapturedContext>, String> {
        let contexts = sqlx::query_as::<_, CapturedContext>(
            "SELECT * FROM captured_contexts WHERE app_name = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(app_name)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch contexts for app {}: {}", app_name, e))?;

        Ok(contexts)
    }

    /// Sauvegarde un contexte capturé
    pub async fn save_context(&self, context: &CapturedContext) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO captured_contexts (
                id, conversation_id, app_name, app_bundle_id, window_title,
                clipboard_content, idle_seconds, screenshot_data, created_at, trigger_reason, capture_duration_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&context.id)
        .bind(&context.conversation_id)
        .bind(&context.app_name)
        .bind(&context.app_bundle_id)
        .bind(&context.window_title)
        .bind(&context.clipboard_content)
        .bind(context.idle_seconds)
        .bind(&context.screenshot_data)
        .bind(context.created_at)
        .bind(&context.trigger_reason)
        .bind(context.capture_duration_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save context: {}", e))?;

        Ok(())
    }

    /// Obtenir le trust d'un utilisateur
    pub async fn get_user_trust(
        &self,
        device_id: &str,
    ) -> Result<crate::learning::trust::UserTrust, String> {
        let row = sqlx::query(
            "SELECT id, device_id, pos, neg, trust, quarantine, last_updated, created_at FROM user_trust WHERE device_id = ?"
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get user trust: {}", e))?;

        match row {
            Some(row) => Ok(crate::learning::trust::UserTrust {
                id: row.get::<String, _>("id"),
                device_id: row.get::<String, _>("device_id"),
                pos: row.get::<f32, _>("pos"),
                neg: row.get::<f32, _>("neg"),
                trust: row.get::<f32, _>("trust"),
                quarantine: row.get::<i32, _>("quarantine") != 0,
                last_updated: std::time::Instant::now(), // TODO: Convert from timestamp
                created_at: std::time::Instant::now(),   // TODO: Convert from timestamp
            }),
            None => Err("User trust not found".to_string()),
        }
    }

    /// Créer un nouveau trust utilisateur
    pub async fn create_user_trust(
        &self,
        trust: &crate::learning::trust::UserTrust,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO user_trust (id, device_id, pos, neg, trust, quarantine, last_updated, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&trust.id)
        .bind(&trust.device_id)
        .bind(trust.pos)
        .bind(trust.neg)
        .bind(trust.trust)
        .bind(if trust.quarantine { 1 } else { 0 })
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create user trust: {}", e))?;

        Ok(())
    }

    /// Mettre à jour le trust d'un utilisateur
    pub async fn update_user_trust(
        &self,
        trust: &crate::learning::trust::UserTrust,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE user_trust SET pos = ?, neg = ?, trust = ?, quarantine = ?, last_updated = ? WHERE device_id = ?"
        )
        .bind(trust.pos)
        .bind(trust.neg)
        .bind(trust.trust)
        .bind(if trust.quarantine { 1 } else { 0 })
        .bind(chrono::Utc::now())
        .bind(&trust.device_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update user trust: {}", e))?;

        Ok(())
    }

    /// Sauvegarder un événement de trust
    pub async fn save_trust_event(
        &self,
        event: &crate::learning::trust::TrustEvent,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO trust_events (id, device_id, reward, timestamp) VALUES (?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&event.device_id)
        .bind(event.reward)
        .bind(chrono::Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save trust event: {}", e))?;

        Ok(())
    }

    /// Obtenir les événements de trust récents
    pub async fn get_recent_trust_events(
        &self,
        device_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::learning::trust::TrustEvent>, String> {
        let rows = sqlx::query(
            "SELECT id, device_id, reward, timestamp FROM trust_events WHERE device_id = ? ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(device_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get recent trust events: {}", e))?;

        let events = rows
            .into_iter()
            .map(|row| crate::learning::trust::TrustEvent {
                id: row.get::<String, _>("id"),
                device_id: row.get::<String, _>("device_id"),
                reward: row.get::<f32, _>("reward"),
                timestamp: std::time::Instant::now(), // TODO: Convert from timestamp
            })
            .collect();

        Ok(events)
    }

    /// Obtenir l'historique des rewards récents
    pub async fn get_recent_rewards(
        &self,
        device_id: &str,
        limit: usize,
    ) -> Result<Vec<f32>, String> {
        let rows = sqlx::query(
            "SELECT reward FROM trust_events WHERE device_id = ? ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(device_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get recent rewards: {}", e))?;

        let rewards = rows
            .into_iter()
            .map(|row| row.get::<f32, _>("reward"))
            .collect();
        Ok(rewards)
    }

    /// Stocker un outcome
    pub async fn store_outcome(
        &self,
        outcome_id: &str,
        suggestion_id: &str,
        used: bool,
        helpful: bool,
        reverted: bool,
        time_to_flow_ms: i64,
        reward: f32,
        cluster_id: &str,
        artefact_type: &str,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO outcomes (id, suggestion_id, used, helpful, reverted, time_to_flow_ms, reward, cluster_id, artefact_type, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(outcome_id)
        .bind(suggestion_id)
        .bind(if used { 1 } else { 0 })
        .bind(if helpful { 1 } else { 0 })
        .bind(if reverted { 1 } else { 0 })
        .bind(time_to_flow_ms)
        .bind(reward)
        .bind(cluster_id)
        .bind(artefact_type)
        .bind(chrono::Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store outcome: {}", e))?;

        Ok(())
    }

    /// Obtenir les outcomes récents
    pub async fn get_recent_outcomes(
        &self,
        _device_id: &str,
        limit: usize,
    ) -> Result<Vec<crate::learning::reward::Outcome>, String> {
        let rows = sqlx::query(
            "SELECT used, helpful, reverted, time_to_flow_ms FROM outcomes ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get recent outcomes: {}", e))?;

        let outcomes = rows
            .into_iter()
            .map(|row| {
                let used = row.get::<i32, _>("used") != 0;
                if used {
                    crate::learning::reward::Outcome::Used {
                        helpful: row.get::<i32, _>("helpful") != 0,
                        reverted: row.get::<i32, _>("reverted") != 0,
                        time_to_flow: {
                            let ms = row.get::<i64, _>("time_to_flow_ms");
                            if ms > 0 {
                                Some(std::time::Duration::from_millis(ms as u64))
                            } else {
                                None
                            }
                        },
                    }
                } else {
                    crate::learning::reward::Outcome::Ignored
                }
            })
            .collect();

        Ok(outcomes)
    }
}

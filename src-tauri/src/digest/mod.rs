/**
 * Daily Digest Module
 * Generates daily statistics and highlights
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestStats {
    pub suggestions_shown: u32,
    pub suggestions_accepted: u32,
    pub time_saved_minutes: u32,
    pub top_apps: Vec<AppUsage>,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    pub name: String,
    pub count: u32,
}

pub struct DigestManager {
    suggestions_shown: u32,
    suggestions_accepted: u32,
    app_usage: std::collections::HashMap<String, u32>,
}

impl DigestManager {
    pub fn new() -> Self {
        Self {
            suggestions_shown: 0,
            suggestions_accepted: 0,
            app_usage: std::collections::HashMap::new(),
        }
    }

    pub fn record_suggestion_shown(&mut self, app_name: &str) {
        self.suggestions_shown += 1;
        *self.app_usage.entry(app_name.to_string()).or_insert(0) += 1;
    }

    pub fn record_suggestion_accepted(&mut self) {
        self.suggestions_accepted += 1;
    }

    pub fn get_digest(&self) -> DigestStats {
        // Calculate time saved (2 minutes per accepted suggestion)
        let time_saved_minutes = self.suggestions_accepted * 2;

        // Get top 3 apps
        let mut top_apps: Vec<AppUsage> = self
            .app_usage
            .iter()
            .map(|(name, count)| AppUsage {
                name: name.clone(),
                count: *count,
            })
            .collect();
        top_apps.sort_by(|a, b| b.count.cmp(&a.count));
        top_apps.truncate(3);

        // Generate highlights
        let mut highlights = Vec::new();

        if self.suggestions_accepted > 0 {
            let acceptance_rate = (self.suggestions_accepted as f32
                / self.suggestions_shown as f32
                * 100.0) as u32;
            highlights.push(format!(
                "Tu as accepté {}% des suggestions 🎯",
                acceptance_rate
            ));
        }

        if time_saved_minutes >= 10 {
            highlights.push(format!(
                "{} min gagnées sur du debugging",
                time_saved_minutes
            ));
        }

        if self.suggestions_accepted >= 5 {
            highlights.push("Ton meilleur jour cette semaine 🚀".to_string());
        }

        DigestStats {
            suggestions_shown: self.suggestions_shown,
            suggestions_accepted: self.suggestions_accepted,
            time_saved_minutes,
            top_apps,
            highlights,
        }
    }

    pub fn reset_daily(&mut self) {
        self.suggestions_shown = 0;
        self.suggestions_accepted = 0;
        self.app_usage.clear();
    }
}

#[tauri::command]
pub async fn get_daily_digest(
    digest_manager: State<'_, Arc<Mutex<DigestManager>>>,
) -> Result<DigestStats, String> {
    let manager = digest_manager.lock().await;
    Ok(manager.get_digest())
}

#[tauri::command]
pub async fn record_suggestion_shown(
    app_name: String,
    digest_manager: State<'_, Arc<Mutex<DigestManager>>>,
) -> Result<(), String> {
    let mut manager = digest_manager.lock().await;
    manager.record_suggestion_shown(&app_name);
    Ok(())
}

#[tauri::command]
pub async fn record_suggestion_accepted(
    digest_manager: State<'_, Arc<Mutex<DigestManager>>>,
) -> Result<(), String> {
    let mut manager = digest_manager.lock().await;
    manager.record_suggestion_accepted();
    Ok(())
}


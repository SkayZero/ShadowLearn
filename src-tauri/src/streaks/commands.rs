use super::{StreakData, StreakManager};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// Get current streak data
#[tauri::command]
pub async fn get_streak(
    streak_manager: State<'_, Arc<Mutex<StreakManager>>>,
) -> Result<StreakData, String> {
    let manager = streak_manager.lock().await;
    Ok(manager.get_streak())
}

/// Record user activity to update streak
#[tauri::command]
pub async fn record_activity(
    streak_manager: State<'_, Arc<Mutex<StreakManager>>>,
) -> Result<(), String> {
    let mut manager = streak_manager.lock().await;
    manager.record_activity();
    Ok(())
}



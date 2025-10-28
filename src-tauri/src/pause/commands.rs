use super::{PauseManager, PauseState};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// Get current pause state
#[tauri::command]
pub async fn get_pause_state(
    pause_manager: State<'_, Arc<Mutex<PauseManager>>>,
) -> Result<PauseState, String> {
    let manager = pause_manager.lock().await;
    Ok(manager.get_pause_state())
}

/// Set pause state
#[tauri::command]
pub async fn set_pause_state(
    active: bool,
    duration_minutes: Option<u32>,
    pause_manager: State<'_, Arc<Mutex<PauseManager>>>,
) -> Result<(), String> {
    let mut manager = pause_manager.lock().await;
    manager.set_pause_state(active, duration_minutes);
    Ok(())
}



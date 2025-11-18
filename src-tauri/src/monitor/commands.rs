use super::{MonitorConfig, ScreenMonitor};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Start the screen monitor
#[tauri::command]
pub async fn start_screen_monitor(
    app: tauri::AppHandle,
    monitor: tauri::State<'_, Arc<Mutex<ScreenMonitor>>>,
) -> Result<(), String> {
    info!("🎬 start_screen_monitor command called");
    let mon = monitor.lock().await;
    mon.start(app).await;
    Ok(())
}

/// Stop the screen monitor
#[tauri::command]
pub async fn stop_screen_monitor(
    monitor: tauri::State<'_, Arc<Mutex<ScreenMonitor>>>,
) -> Result<(), String> {
    info!("🛑 stop_screen_monitor command called");
    let mon = monitor.lock().await;
    mon.stop().await;
    Ok(())
}

/// Get monitor status
#[tauri::command]
pub async fn get_monitor_status(
    monitor: tauri::State<'_, Arc<Mutex<ScreenMonitor>>>,
) -> Result<bool, String> {
    let mon = monitor.lock().await;
    Ok(mon.is_running().await)
}

/// Reset the change detector
#[tauri::command]
pub async fn reset_monitor_detector(
    monitor: tauri::State<'_, Arc<Mutex<ScreenMonitor>>>,
) -> Result<(), String> {
    info!("🔄 reset_monitor_detector command called");
    let mon = monitor.lock().await;
    mon.reset_detector().await;
    Ok(())
}

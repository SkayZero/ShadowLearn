use super::{PrivacyZone, PrivacyZoneManager, PrivacyZonesConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Get privacy zones configuration
#[tauri::command]
pub async fn get_privacy_zones_config(
    privacy: tauri::State<'_, Arc<Mutex<PrivacyZoneManager>>>,
) -> Result<PrivacyZonesConfig, String> {
    let manager = privacy.lock().await;
    Ok(manager.config().clone())
}

/// Add a new privacy zone
#[tauri::command]
pub async fn add_privacy_zone(
    privacy: tauri::State<'_, Arc<Mutex<PrivacyZoneManager>>>,
    zone: PrivacyZone,
) -> Result<(), String> {
    info!("🔒 add_privacy_zone command called");
    let mut manager = privacy.lock().await;
    manager.add_zone(zone);
    Ok(())
}

/// Remove a privacy zone
#[tauri::command]
pub async fn remove_privacy_zone(
    privacy: tauri::State<'_, Arc<Mutex<PrivacyZoneManager>>>,
    zone: PrivacyZone,
) -> Result<bool, String> {
    info!("🔒 remove_privacy_zone command called");
    let mut manager = privacy.lock().await;
    Ok(manager.remove_zone(&zone))
}

/// Enable or disable privacy zones
#[tauri::command]
pub async fn set_privacy_zones_enabled(
    privacy: tauri::State<'_, Arc<Mutex<PrivacyZoneManager>>>,
    enabled: bool,
) -> Result<(), String> {
    info!("🔒 set_privacy_zones_enabled: {}", enabled);
    let mut manager = privacy.lock().await;
    manager.set_enabled(enabled);
    Ok(())
}

/// Check if a specific app is protected
#[tauri::command]
pub async fn is_app_protected(
    privacy: tauri::State<'_, Arc<Mutex<PrivacyZoneManager>>>,
    app_name: String,
) -> Result<bool, String> {
    let manager = privacy.lock().await;
    Ok(manager.is_app_protected(&app_name))
}

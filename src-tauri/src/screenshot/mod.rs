pub mod capturer;
pub mod errors;
pub mod permissions;

pub use capturer::ScreenshotCapturer;
pub use errors::{PermissionStatus, ScreenshotError};
pub use permissions::open_system_preferences;

use base64::{engine::general_purpose, Engine as _};
use std::sync::Mutex;
use tauri::Manager;
use tracing::{error, info};

/// Global capturer instance (lazy init)
static CAPTURER: once_cell::sync::Lazy<Mutex<Option<ScreenshotCapturer>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Initialise le capturer (appelé au démarrage)
pub fn init_capturer() -> Result<(), ScreenshotError> {
    let mut capturer = CAPTURER.lock().unwrap();

    if capturer.is_none() {
        *capturer = Some(ScreenshotCapturer::new()?);
        info!("✅ ScreenshotCapturer initialized");
    }

    Ok(())
}

/// Tauri command: Capture screenshot (OPTIMIZED)
#[tauri::command]
pub async fn capture_screenshot(app: tauri::AppHandle) -> Result<CaptureResult, String> {
    info!("📸 capture_screenshot command called");
    let start = std::time::Instant::now();

    // OPTIMIZED: Hide windows
    let windows_to_hide = ["chat", "context"];
    for label in &windows_to_hide {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }

    // OPTIMIZED: Short delay, just enough for compositor
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Spawn blocking pour pas bloquer l'async runtime
    let start_clone = start;
    let result = tokio::task::spawn_blocking(move || {
        let mut capturer_guard = CAPTURER.lock().unwrap();

        // Init lazy si pas encore fait
        if capturer_guard.is_none() {
            *capturer_guard = Some(ScreenshotCapturer::new().map_err(|e| e.to_string())?);
        }

        let capturer = capturer_guard.as_mut().unwrap();

        // OPTIMIZED: Check permissions cached (no repeated checks)
        let check_start = std::time::Instant::now();
        let status = capturer.check_permissions();
        info!("⏱️ Permission check: {}ms", check_start.elapsed().as_millis());
        
        if status == PermissionStatus::Denied {
            error!("❌ Screenshot permission denied");
            return Err(
                "Screen recording permission denied. Please grant access in System Preferences."
                    .into(),
            );
        }

        // Capture
        let capture_start = std::time::Instant::now();
        let path = capturer.capture_active_screen().map_err(|e| {
            error!("❌ Capture failed: {}", e);
            e.to_string()
        })?;
        info!("⏱️ Screen capture: {}ms", capture_start.elapsed().as_millis());

        // OPTIMIZED: Only read file once
        let read_start = std::time::Instant::now();
        let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
        let file_size = bytes.len();
        info!("⏱️ File read: {}ms ({} bytes)", read_start.elapsed().as_millis(), file_size);
        
        let encode_start = std::time::Instant::now();
        let base64_data = general_purpose::STANDARD.encode(&bytes);
        info!("⏱️ Base64 encode: {}ms ({} chars)", encode_start.elapsed().as_millis(), base64_data.len());

        info!(
            "✅ Screenshot: {} bytes → {} base64 chars (total {}ms)",
            file_size,
            base64_data.len(),
            start_clone.elapsed().as_millis()
        );

        Ok(CaptureResult {
            data: base64_data,
            path: path.to_string_lossy().to_string(),
            size_bytes: file_size,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    // Restore windows after capture
    for label in &windows_to_hide {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.show();
        }
    }

    info!("🎯 Total screenshot duration: {}ms", start.elapsed().as_millis());
    result
}

#[derive(serde::Serialize)]
pub struct CaptureResult {
    pub data: String, // base64
    pub path: String,
    pub size_bytes: usize,
}

/// Tauri command: Check permissions
#[tauri::command]
pub fn check_screenshot_permission() -> PermissionStatus {
    let mut capturer_guard = CAPTURER.lock().unwrap();

    // Init lazy si pas encore fait
    if capturer_guard.is_none() {
        match ScreenshotCapturer::new() {
            Ok(c) => *capturer_guard = Some(c),
            Err(e) => {
                error!("Failed to init capturer for permission check: {}", e);
                return PermissionStatus::Unknown;
            }
        }
    }

    capturer_guard.as_mut().unwrap().check_permissions()
}

/// Tauri command: Request permissions (open settings)
#[tauri::command]
pub fn request_screenshot_permission() -> Result<(), String> {
    open_system_preferences().map_err(|e| e.to_string())
}

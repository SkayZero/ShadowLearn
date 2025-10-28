use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub screen_recording: bool,
    pub accessibility: bool,
    pub all_granted: bool,
}

pub struct PermissionChecker;

impl PermissionChecker {
    pub fn check_all() -> PermissionStatus {
        let sr = Self::check_screen_recording();
        let ax = Self::check_accessibility();
        PermissionStatus {
            screen_recording: sr,
            accessibility: ax,
            all_granted: sr && ax,
        }
    }

    #[cfg(target_os = "macos")]
    fn check_screen_recording() -> bool {
        match screenshots::Screen::all() {
            Ok(s) => s
                .first()
                .and_then(|scr| scr.capture().ok())
                .is_some(),
            Err(_) => false,
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn check_screen_recording() -> bool {
        true
    }

    #[cfg(target_os = "macos")]
    fn check_accessibility() -> bool {
        unsafe { AXIsProcessTrustedWithOptions(std::ptr::null()) }
    }

    #[cfg(not(target_os = "macos"))]
    fn check_accessibility() -> bool {
        true
    }

    pub fn open_screen_recording_settings() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn open_accessibility_settings() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
}

#[tauri::command]
pub fn check_permissions() -> PermissionStatus {
    PermissionChecker::check_all()
}

#[tauri::command]
pub fn request_screen_recording_permission() -> Result<(), String> {
    PermissionChecker::open_screen_recording_settings()
}

#[tauri::command]
pub fn request_accessibility_permission() -> Result<(), String> {
    PermissionChecker::open_accessibility_settings()
}

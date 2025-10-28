use super::errors::{AppDetectionError, TCCStatus};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::debug;

/// Information sur l'application active
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveApp {
    pub bundle_id: String,
    pub name: String,
    pub window_title: String,
    pub pid: u32,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcc_status: Option<TCCStatus>,
}

/// Détecteur d'application active avec cache
pub struct AppDetector {
    last_app: Option<ActiveApp>,
    last_check: Instant,
    cache_ttl: Duration,
}

impl AppDetector {
    pub fn new() -> Self {
        Self {
            last_app: None,
            last_check: Instant::now(),
            cache_ttl: Duration::from_millis(150),
        }
    }

    /// Récupère l'application active avec cache fast-path
    pub fn get_active_app(&mut self) -> Result<ActiveApp, AppDetectionError> {
        // Fast-path: si cache valide et app inchangée
        if let Some(ref app) = self.last_app {
            let elapsed = self.last_check.elapsed();
            if elapsed < self.cache_ttl {
                debug!("Cache hit: returning cached app (age: {:?})", elapsed);
                return Ok(app.clone());
            }
        }

        // Cache miss ou expiré: fetch depuis l'OS
        let app = self.fetch_active_app()?;
        self.last_app = Some(app.clone());
        self.last_check = Instant::now();

        Ok(app)
    }

    /// Fetch l'app active depuis l'OS (pas de cache)
    fn fetch_active_app(&self) -> Result<ActiveApp, AppDetectionError> {
        #[cfg(target_os = "macos")]
        #[allow(unexpected_cfgs)]
        {
            self.get_active_app_macos()
        }

        #[cfg(target_os = "windows")]
        {
            self.get_active_app_windows()
        }

        #[cfg(target_os = "linux")]
        {
            self.get_active_app_linux()
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(AppDetectionError::NotAvailable(
                "Unsupported OS".to_string(),
            ))
        }
    }

    // ============================================
    // macOS Implementation
    // ============================================
    #[cfg(target_os = "macos")]
    #[allow(unexpected_cfgs)]
    fn get_active_app_macos(&self) -> Result<ActiveApp, AppDetectionError> {
        use cocoa::base::{id, nil};
        use objc::{class, msg_send, sel, sel_impl};
        use std::ffi::CStr;

        unsafe {
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            let app: id = msg_send![workspace, frontmostApplication];

            if app == nil {
                return Err(AppDetectionError::NoActiveWindow);
            }

            // Bundle ID
            let bundle_id_ns: id = msg_send![app, bundleIdentifier];
            let bundle_id = if bundle_id_ns != nil {
                let bundle_id_ptr: *const i8 = msg_send![bundle_id_ns, UTF8String];
                CStr::from_ptr(bundle_id_ptr).to_string_lossy().into_owned()
            } else {
                "unknown".to_string()
            };

            // App name
            let app_name_ns: id = msg_send![app, localizedName];
            let name = if app_name_ns != nil {
                let app_name_ptr: *const i8 = msg_send![app_name_ns, UTF8String];
                CStr::from_ptr(app_name_ptr).to_string_lossy().into_owned()
            } else {
                "Unknown".to_string()
            };

            // PID
            let pid: i32 = msg_send![app, processIdentifier];

            // Window title (basic via NSWorkspace - pas toujours fiable)
            let window_title = self
                .get_window_title_macos()
                .unwrap_or_else(|_| format!("{} Window", name));

            // TCC status
            let tcc_status = self.check_tcc_status_macos();

            Ok(ActiveApp {
                bundle_id,
                name,
                window_title,
                pid: pid as u32,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                tcc_status: Some(tcc_status),
            })
        }
    }

    #[cfg(target_os = "macos")]
    fn get_window_title_macos(&self) -> Result<String, AppDetectionError> {
        // TODO: Implémenter via Accessibility API (AX)
        // Pour l'instant, retourne un placeholder
        Ok("Window".to_string())
    }

    #[cfg(target_os = "macos")]
    fn check_tcc_status_macos(&self) -> TCCStatus {
        // TODO: Vérifier via CGPreflightScreenCaptureAccess
        // Pour l'instant, retourne Unknown
        TCCStatus::Unknown
    }

    // ============================================
    // Windows Implementation
    // ============================================
    #[cfg(target_os = "windows")]
    fn get_active_app_windows(&self) -> Result<ActiveApp, AppDetectionError> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
        };

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 {
                return Err(AppDetectionError::NoActiveWindow);
            }

            // Window title
            let mut title_buf = [0u16; 512];
            let title_len = GetWindowTextW(hwnd, &mut title_buf);
            let window_title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

            // PID
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));

            if pid == 0 {
                return Err(AppDetectionError::Windows(
                    "Failed to get process ID".to_string(),
                ));
            }

            // Process name
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
                .map_err(|e| AppDetectionError::Windows(format!("OpenProcess failed: {}", e)))?;

            let mut name_buf = [0u16; 512];
            let mut name_len = name_buf.len() as u32;
            QueryFullProcessImageNameW(process_handle, 0, &mut name_buf, &mut name_len).map_err(
                |e| AppDetectionError::Windows(format!("QueryFullProcessImageNameW failed: {}", e)),
            )?;

            let full_path = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            let name = std::path::Path::new(&full_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            Ok(ActiveApp {
                bundle_id: format!("windows.{}", pid),
                name,
                window_title,
                pid,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                tcc_status: None,
            })
        }
    }

    // ============================================
    // Linux Implementation
    // ============================================
    #[cfg(target_os = "linux")]
    fn get_active_app_linux(&self) -> Result<ActiveApp, AppDetectionError> {
        // Try xdotool first
        if let Ok(app) = self.get_active_app_linux_xdotool() {
            return Ok(app);
        }

        warn!("xdotool not available, trying EWMH fallback");

        // Fallback to EWMH (X11)
        if let Ok(app) = self.get_active_app_linux_ewmh() {
            return Ok(app);
        }

        // Last resort: generic unknown
        Err(AppDetectionError::NotAvailable(
            "No window detection method available (install xdotool or wmctrl)".to_string(),
        ))
    }

    #[cfg(target_os = "linux")]
    fn get_active_app_linux_xdotool(&self) -> Result<ActiveApp, AppDetectionError> {
        use std::process::Command;

        // Check if xdotool is available
        if which::which("xdotool").is_err() {
            return Err(AppDetectionError::NotAvailable(
                "xdotool not found".to_string(),
            ));
        }

        // Get window ID
        let window_id_output = Command::new("xdotool")
            .args(&["getactivewindow"])
            .output()
            .map_err(|e| {
                AppDetectionError::Linux(format!("xdotool getactivewindow failed: {}", e))
            })?;

        if !window_id_output.status.success() {
            return Err(AppDetectionError::Linux(
                "xdotool getactivewindow returned error".to_string(),
            ));
        }

        let window_id = String::from_utf8_lossy(&window_id_output.stdout)
            .trim()
            .to_string();

        // Get window title
        let title_output = Command::new("xdotool")
            .args(&["getwindowname", &window_id])
            .output()
            .map_err(|e| {
                AppDetectionError::Linux(format!("xdotool getwindowname failed: {}", e))
            })?;

        let window_title = String::from_utf8_lossy(&title_output.stdout)
            .trim()
            .to_string();

        // Get PID
        let pid_output = Command::new("xdotool")
            .args(&["getwindowpid", &window_id])
            .output()
            .map_err(|e| AppDetectionError::Linux(format!("xdotool getwindowpid failed: {}", e)))?;

        let pid: u32 = String::from_utf8_lossy(&pid_output.stdout)
            .trim()
            .parse()
            .unwrap_or(0);

        // Get process name from /proc
        let name = if pid > 0 {
            std::fs::read_to_string(format!("/proc/{}/comm", pid))
                .unwrap_or_else(|_| "Unknown".to_string())
                .trim()
                .to_string()
        } else {
            "Unknown".to_string()
        };

        Ok(ActiveApp {
            bundle_id: format!("linux.{}", window_id),
            name,
            window_title,
            pid,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tcc_status: None,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_active_app_linux_ewmh(&self) -> Result<ActiveApp, AppDetectionError> {
        // TODO: Implémenter EWMH via x11rb
        // Pour l'instant, retourne une erreur
        Err(AppDetectionError::NotAvailable(
            "EWMH support not yet implemented".to_string(),
        ))
    }
}

impl Default for AppDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_detector_creation() {
        let detector = AppDetector::new();
        assert_eq!(detector.cache_ttl, Duration::from_millis(150));
        assert!(detector.last_app.is_none());
    }

    #[test]
    fn test_app_detection() {
        let mut detector = AppDetector::new();
        let result = detector.get_active_app();

        // Should succeed on supported platforms
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
        {
            match result {
                Ok(app) => {
                    assert!(!app.name.is_empty());
                    assert!(!app.bundle_id.is_empty());
                    assert!(app.pid > 0);
                }
                Err(e) => {
                    // Acceptable errors on CI/headless
                    println!("App detection error (expected on CI): {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_cache_behavior() {
        let mut detector = AppDetector::new();

        // First call - cache miss
        if let Ok(app1) = detector.get_active_app() {
            // Second call within TTL - should use cache
            if let Ok(app2) = detector.get_active_app() {
                assert_eq!(app1.bundle_id, app2.bundle_id);
                assert_eq!(app1.name, app2.name);
            }
        }
    }
}

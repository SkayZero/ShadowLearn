use super::errors::ScreenshotError;
use tracing::info;

/// Ouvre les paramètres système pour les permissions de capture d'écran
pub fn open_system_preferences() -> Result<(), ScreenshotError> {
    #[cfg(target_os = "macos")]
    {
        info!("🔓 Opening macOS System Preferences → Privacy → Screen Recording");

        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()
            .map_err(ScreenshotError::IoError)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        info!("🔓 Opening Windows Settings → Privacy → Screen capture");

        // Sur Windows, il faut utiliser explorer.exe avec l'URI ms-settings
        std::process::Command::new("explorer.exe")
            .arg("ms-settings:privacy-screencapture")
            .spawn()
            .map_err(|e| ScreenshotError::IoError(e))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        warn!("⚠️ Linux doesn't have system-wide screen capture permissions");

        // Sur Linux, pas de permission globale, mais on peut tenter d'ouvrir les paramètres
        // Dépend du DE (GNOME, KDE, etc.)

        // Tentative GNOME Settings
        if let Ok(_) = std::process::Command::new("gnome-control-center")
            .arg("privacy")
            .spawn()
        {
            return Ok(());
        }

        // Tentative KDE Settings
        if let Ok(_) = std::process::Command::new("systemsettings5").spawn() {
            return Ok(());
        }

        warn!("Could not open system settings on Linux");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run manually (opens system preferences)
    fn test_open_preferences() {
        let result = open_system_preferences();
        assert!(result.is_ok());
    }
}

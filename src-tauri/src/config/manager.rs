use super::types::AppConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};
use dirs;

pub struct ConfigManager {
    config: Arc<Mutex<AppConfig>>,
    config_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub period: String, // "never" | "1h" | "24h" | "7d" | "30d"
}

impl ConfigManager {
    pub fn new() -> Result<Self, String> {
        let config_file = Self::get_config_file_path()?;
        let config = Self::load_or_create_config(&config_file)?;
        
        info!("✅ ConfigManager initialized: {:?}", config_file);
        
        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            config_file,
        })
    }

    fn get_config_file_path() -> Result<PathBuf, String> {
        let app_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?
            .join("ShadowLearn");

        std::fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app directory: {}", e))?;

        Ok(app_dir.join("config.json"))
    }

    fn load_or_create_config(path: &PathBuf) -> Result<AppConfig, String> {
        if !path.exists() {
            debug!("No config file found, using defaults");
            return Ok(AppConfig::default());
        }

        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let config: AppConfig = match serde_json::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to parse config, using defaults: {}", e);
                AppConfig::default()
            }
        };

        Ok(config)
    }

    fn save_config(&self) -> Result<(), String> {
        let config_guard = self.config.lock().unwrap();
        let contents = serde_json::to_string_pretty(&*config_guard)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&self.config_file, contents)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        debug!("Config saved to {:?}", self.config_file);
        Ok(())
    }

    pub fn get_config(&self) -> Result<AppConfig, String> {
        Ok(self.config.lock().unwrap().clone())
    }

    pub fn update_config(&self, new_config: AppConfig) -> Result<(), String> {
        *self.config.lock().unwrap() = new_config;
        self.save_config()
    }

    pub fn get_config_path(&self) -> PathBuf {
        self.config_file.clone()
    }
}

#[tauri::command]
pub async fn get_config(
    manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>,
) -> Result<AppConfig, String> {
    let manager_guard = manager.lock().unwrap();
    manager_guard.get_config()
}

#[tauri::command]
pub async fn update_config(
    config: AppConfig,
    manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>,
) -> Result<(), String> {
    let manager_guard = manager.lock().unwrap();
    manager_guard.update_config(config)
}

#[tauri::command]
pub async fn get_config_path(
    manager: tauri::State<'_, Arc<Mutex<ConfigManager>>>,
) -> Result<String, String> {
    let manager_guard = manager.lock().unwrap();
    Ok(manager_guard.get_config_path().to_string_lossy().to_string())
}


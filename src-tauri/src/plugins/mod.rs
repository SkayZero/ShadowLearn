/**
 * Plugin System Module
 * Extensible plugin architecture for ShadowLearn
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub mod loader;
pub mod runtime;
pub mod types;

pub use types::{Plugin, PluginConfig, PluginHook, PluginManifest, PluginMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
    pub hooks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub enabled_plugins: usize,
    pub total_hooks: usize,
    pub plugin_directory: String,
}

pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
    plugin_dir: PathBuf,
    hooks: HashMap<String, Vec<String>>, // hook_name -> [plugin_ids]
}

impl PluginManager {
    pub fn new() -> Result<Self, String> {
        let plugin_dir = Self::get_plugin_directory()?;

        // Create plugin directory if it doesn't exist
        if !plugin_dir.exists() {
            fs::create_dir_all(&plugin_dir)
                .map_err(|e| format!("Failed to create plugin directory: {}", e))?;
        }

        Ok(Self {
            plugins: HashMap::new(),
            plugin_dir,
            hooks: HashMap::new(),
        })
    }

    fn get_plugin_directory() -> Result<PathBuf, String> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| "Could not find data directory".to_string())?;
        Ok(data_dir.join("ShadowLearn").join("plugins"))
    }

    pub fn load_all_plugins(&mut self) -> Result<usize, String> {
        info!("🔌 Loading plugins from {:?}", self.plugin_dir);

        if !self.plugin_dir.exists() {
            info!("📁 Plugin directory doesn't exist, skipping plugin load");
            return Ok(0);
        }

        let entries = fs::read_dir(&self.plugin_dir)
            .map_err(|e| format!("Failed to read plugin directory: {}", e))?;

        let mut loaded_count = 0;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            match self.load_plugin(&path) {
                Ok(plugin_id) => {
                    info!("✅ Loaded plugin: {}", plugin_id);
                    loaded_count += 1;
                }
                Err(e) => {
                    error!("❌ Failed to load plugin at {:?}: {}", path, e);
                }
            }
        }

        info!("🎉 Loaded {} plugins", loaded_count);
        Ok(loaded_count)
    }

    fn load_plugin(&mut self, plugin_path: &Path) -> Result<String, String> {
        let manifest_path = plugin_path.join("plugin.json");

        if !manifest_path.exists() {
            return Err("plugin.json not found".to_string());
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        // Validate plugin
        if manifest.metadata.id.is_empty() {
            return Err("Plugin ID cannot be empty".to_string());
        }

        let plugin = Plugin {
            id: manifest.metadata.id.clone(),
            metadata: manifest.metadata,
            config: manifest.config,
            enabled: true,
            path: plugin_path.to_path_buf(),
        };

        // Register hooks
        for hook in &plugin.config.hooks {
            self.hooks
                .entry(hook.name.clone())
                .or_insert_with(Vec::new)
                .push(plugin.id.clone());
        }

        let plugin_id = plugin.id.clone();
        self.plugins.insert(plugin_id.clone(), plugin);

        Ok(plugin_id)
    }

    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        plugin.enabled = true;
        info!("✅ Enabled plugin: {}", plugin_id);
        Ok(())
    }

    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        plugin.enabled = false;
        info!("🔇 Disabled plugin: {}", plugin_id);
        Ok(())
    }

    pub fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin = self.plugins.remove(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        // Remove from hooks registry
        for hook_plugins in self.hooks.values_mut() {
            hook_plugins.retain(|id| id != plugin_id);
        }

        // Delete plugin directory
        if plugin.path.exists() {
            fs::remove_dir_all(&plugin.path)
                .map_err(|e| format!("Failed to delete plugin directory: {}", e))?;
        }

        info!("🗑️ Uninstalled plugin: {}", plugin_id);
        Ok(())
    }

    pub fn execute_hook(&self, hook_name: &str, context: &str) -> Vec<String> {
        let mut results = Vec::new();

        if let Some(plugin_ids) = self.hooks.get(hook_name) {
            for plugin_id in plugin_ids {
                if let Some(plugin) = self.plugins.get(plugin_id) {
                    if !plugin.enabled {
                        continue;
                    }

                    // Find the hook configuration
                    if let Some(hook) = plugin.config.hooks.iter().find(|h| h.name == hook_name) {
                        info!("🪝 Executing hook '{}' for plugin '{}'", hook_name, plugin_id);

                        // Execute the hook action
                        match runtime::execute_hook_action(&plugin.path, &hook.action, context) {
                            Ok(result) => results.push(result),
                            Err(e) => {
                                error!("❌ Hook execution failed for {}: {}", plugin_id, e);
                            }
                        }
                    }
                }
            }
        }

        results
    }

    pub fn get_all_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .map(|p| PluginInfo {
                id: p.id.clone(),
                name: p.metadata.name.clone(),
                version: p.metadata.version.clone(),
                author: p.metadata.author.clone(),
                description: p.metadata.description.clone(),
                enabled: p.enabled,
                hooks: p.config.hooks.iter().map(|h| h.name.clone()).collect(),
            })
            .collect()
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginInfo> {
        self.plugins.get(plugin_id).map(|p| PluginInfo {
            id: p.id.clone(),
            name: p.metadata.name.clone(),
            version: p.metadata.version.clone(),
            author: p.metadata.author.clone(),
            description: p.metadata.description.clone(),
            enabled: p.enabled,
            hooks: p.config.hooks.iter().map(|h| h.name.clone()).collect(),
        })
    }

    pub fn get_stats(&self) -> PluginStats {
        let enabled_count = self.plugins.values().filter(|p| p.enabled).count();
        let total_hooks: usize = self.plugins.values()
            .map(|p| p.config.hooks.len())
            .sum();

        PluginStats {
            total_plugins: self.plugins.len(),
            enabled_plugins: enabled_count,
            total_hooks,
            plugin_directory: self.plugin_dir.to_string_lossy().to_string(),
        }
    }

    pub fn reload_plugins(&mut self) -> Result<usize, String> {
        info!("🔄 Reloading all plugins...");

        // Clear existing plugins and hooks
        self.plugins.clear();
        self.hooks.clear();

        // Reload from disk
        self.load_all_plugins()
    }
}

// Tauri Commands
#[tauri::command]
pub async fn get_all_plugins(
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<PluginInfo>, String> {
    let manager = plugin_manager.lock().await;
    Ok(manager.get_all_plugins())
}

#[tauri::command]
pub async fn get_plugin_info(
    plugin_id: String,
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<PluginInfo, String> {
    let manager = plugin_manager.lock().await;
    manager.get_plugin(& plugin_id)
        .ok_or_else(|| format!("Plugin not found: {}", plugin_id))
}

#[tauri::command]
pub async fn enable_plugin(
    plugin_id: String,
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let mut manager = plugin_manager.lock().await;
    manager.enable_plugin(&plugin_id)
}

#[tauri::command]
pub async fn disable_plugin(
    plugin_id: String,
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let mut manager = plugin_manager.lock().await;
    manager.disable_plugin(&plugin_id)
}

#[tauri::command]
pub async fn uninstall_plugin(
    plugin_id: String,
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    let mut manager = plugin_manager.lock().await;
    manager.uninstall_plugin(&plugin_id)
}

#[tauri::command]
pub async fn reload_plugins(
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<usize, String> {
    let mut manager = plugin_manager.lock().await;
    manager.reload_plugins()
}

#[tauri::command]
pub async fn get_plugin_stats(
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<PluginStats, String> {
    let manager = plugin_manager.lock().await;
    Ok(manager.get_stats())
}

#[tauri::command]
pub async fn execute_plugin_hook(
    hook_name: String,
    context: String,
    plugin_manager: State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<String>, String> {
    let manager = plugin_manager.lock().await;
    Ok(manager.execute_hook(&hook_name, &context))
}

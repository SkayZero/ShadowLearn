use super::types::{PluginManifest, PluginMetadata, PluginConfig, PluginHook, HookAction};
use std::fs;
use std::path::Path;

pub fn load_manifest(plugin_path: &Path) -> Result<PluginManifest, String> {
    let manifest_path = plugin_path.join("plugin.json");

    if !manifest_path.exists() {
        return Err("plugin.json not found".to_string());
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let manifest: PluginManifest = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    validate_manifest(&manifest)?;

    Ok(manifest)
}

fn validate_manifest(manifest: &PluginManifest) -> Result<(), String> {
    if manifest.metadata.id.is_empty() {
        return Err("Plugin ID cannot be empty".to_string());
    }

    if manifest.metadata.name.is_empty() {
        return Err("Plugin name cannot be empty".to_string());
    }

    if manifest.metadata.version.is_empty() {
        return Err("Plugin version cannot be empty".to_string());
    }

    // Validate hook names
    for hook in &manifest.config.hooks {
        if hook.name.is_empty() {
            return Err("Hook name cannot be empty".to_string());
        }
    }

    Ok(())
}

pub fn create_example_plugin(plugin_dir: &Path, plugin_id: &str) -> Result<(), String> {
    let plugin_path = plugin_dir.join(plugin_id);

    if plugin_path.exists() {
        return Err(format!("Plugin directory already exists: {:?}", plugin_path));
    }

    fs::create_dir_all(&plugin_path)
        .map_err(|e| format!("Failed to create plugin directory: {}", e))?;

    // Create example manifest
    let manifest = PluginManifest {
        metadata: PluginMetadata {
            id: plugin_id.to_string(),
            name: format!("Example Plugin: {}", plugin_id),
            version: "1.0.0".to_string(),
            author: "ShadowLearn".to_string(),
            description: "An example plugin".to_string(),
            homepage: None,
            repository: None,
        },
        config: PluginConfig {
            hooks: vec![
                PluginHook {
                    name: "on_suggestion".to_string(),
                    description: "Triggered when a suggestion is shown".to_string(),
                    action: HookAction::Script {
                        command: "on_suggestion.sh".to_string(),
                        args: vec![],
                    },
                },
            ],
            permissions: vec!["notifications".to_string()],
            settings: None,
        },
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    fs::write(plugin_path.join("plugin.json"), manifest_json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    // Create example script
    let script_content = r#"#!/bin/bash
# Example plugin hook script
echo "Plugin executed with context: $SHADOWLEARN_CONTEXT"
"#;

    fs::write(plugin_path.join("on_suggestion.sh"), script_content)
        .map_err(|e| format!("Failed to write script: {}", e))?;

    // Make script executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script_path = plugin_path.join("on_suggestion.sh");
        let mut perms = fs::metadata(&script_path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

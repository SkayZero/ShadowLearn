use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub hooks: Vec<PluginHook>,
    pub permissions: Vec<String>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    pub name: String,
    pub description: String,
    pub action: HookAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HookAction {
    Script {
        command: String,
        args: Vec<String>,
    },
    Function {
        module: String,
        function: String,
    },
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub id: String,
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    pub enabled: bool,
    pub path: PathBuf,
}

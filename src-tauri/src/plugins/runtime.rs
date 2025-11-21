use super::types::HookAction;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};

pub fn execute_hook_action(
    plugin_path: &Path,
    action: &HookAction,
    context: &str,
) -> Result<String, String> {
    match action {
        HookAction::Script { command, args } => {
            execute_script(plugin_path, command, args, context)
        }
        HookAction::Function { module, function } => {
            // For now, we'll just return a placeholder
            // In a full implementation, this would use a scripting engine like rhai or lua
            Ok(format!(
                "Function hook: {}::{} (context: {})",
                module, function, context
            ))
        }
    }
}

fn execute_script(
    plugin_path: &Path,
    command: &str,
    args: &[String],
    context: &str,
) -> Result<String, String> {
    let script_path = plugin_path.join(command);

    if !script_path.exists() {
        return Err(format!("Script not found: {:?}", script_path));
    }

    info!("🚀 Executing script: {:?}", script_path);

    let mut cmd = Command::new(&script_path);
    cmd.args(args);
    cmd.env("SHADOWLEARN_CONTEXT", context);
    cmd.env("SHADOWLEARN_PLUGIN_PATH", plugin_path);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Script execution failed: {}", stderr);
        return Err(format!("Script failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout.trim().to_string())
}

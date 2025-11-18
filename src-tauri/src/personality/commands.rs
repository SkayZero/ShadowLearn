use super::{Personality, PersonalityManager};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// Get current personality
#[tauri::command]
pub async fn get_personality(
    personality_manager: State<'_, Arc<Mutex<PersonalityManager>>>,
) -> Result<Personality, String> {
    let manager = personality_manager.lock().await;
    Ok(manager.get_personality())
}

/// Set personality
#[tauri::command]
pub async fn set_personality(
    personality: String,
    personality_manager: State<'_, Arc<Mutex<PersonalityManager>>>,
) -> Result<(), String> {
    let p = match personality.to_lowercase().as_str() {
        "aerya" => Personality::Aerya,
        "aura" => Personality::Aura,
        "spark" => Personality::Spark,
        "nova" => Personality::Nova,
        "kai" => Personality::Kai,
        "echo" => Personality::Echo,
        "void" => Personality::Void,
        _ => return Err(format!("Invalid personality: {}", personality)),
    };
    
    let mut manager = personality_manager.lock().await;
    manager.set_personality(p);
    Ok(())
}



use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod commands;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseState {
    pub active: bool,
    pub duration_minutes: Option<u32>,
    pub paused_until: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct PauseManager {
    state: PauseState,
}

impl PauseManager {
    pub fn new() -> Self {
        Self {
            state: PauseState {
                active: false,
                duration_minutes: None,
                paused_until: None,
            },
        }
    }

    pub fn set_pause_state(&mut self, active: bool, duration_minutes: Option<u32>) {
        self.state.active = active;
        self.state.duration_minutes = duration_minutes;
        
        if active {
            if let Some(duration) = duration_minutes {
                self.state.paused_until = Some(chrono::Utc::now() + chrono::Duration::minutes(duration as i64));
            } else {
                self.state.paused_until = None;
            }
        } else {
            self.state.paused_until = None;
        }
        
        tracing::info!("Pause state changed: active={}, duration={:?}", active, duration_minutes);
    }

    pub fn get_pause_state(&self) -> PauseState {
        // Check if pause has expired
        if let Some(until) = self.state.paused_until {
            if chrono::Utc::now() > until {
                return PauseState {
                    active: false,
                    duration_minutes: None,
                    paused_until: None,
                };
            }
        }
        
        self.state.clone()
    }

    pub fn is_paused(&self) -> bool {
        self.get_pause_state().active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_manager() {
        let mut manager = PauseManager::new();
        assert!(!manager.is_paused());
        
        manager.set_pause_state(true, Some(30));
        assert!(manager.is_paused());
        
        manager.set_pause_state(false, None);
        assert!(!manager.is_paused());
    }
}




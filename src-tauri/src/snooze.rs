use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnoozeDuration {
    ThirtyMinutes,
    TwoHours,
    UntilToday, // Jusqu'à minuit
}

impl SnoozeDuration {
    pub fn to_duration(&self) -> Duration {
        match self {
            SnoozeDuration::ThirtyMinutes => Duration::from_secs(30 * 60),
            SnoozeDuration::TwoHours => Duration::from_secs(2 * 60 * 60),
            SnoozeDuration::UntilToday => {
                // Calculer le temps jusqu'à minuit
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let seconds_in_day = 24 * 60 * 60;
                let seconds_since_midnight = now % seconds_in_day;
                let seconds_until_midnight = seconds_in_day - seconds_since_midnight;
                Duration::from_secs(seconds_until_midnight)
            }
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SnoozeDuration::ThirtyMinutes => "30 minutes",
            SnoozeDuration::TwoHours => "2 hours",
            SnoozeDuration::UntilToday => "Until midnight",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnoozeState {
    pub snoozed_until: Option<u64>, // Unix timestamp
}

pub struct SnoozeManager {
    state: SnoozeState,
    state_file: PathBuf,
}

impl SnoozeManager {
    pub fn new() -> Result<Self, String> {
        let state_file = Self::get_state_file_path()?;
        let state = Self::load_state(&state_file)?;

        Ok(Self { state, state_file })
    }

    fn get_state_file_path() -> Result<PathBuf, String> {
        let app_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?
            .join("ShadowLearn");

        // Créer le dossier s'il n'existe pas
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("Failed to create app directory: {}", e))?;

        Ok(app_dir.join("snooze_state.json"))
    }

    fn load_state(path: &PathBuf) -> Result<SnoozeState, String> {
        if !path.exists() {
            debug!("No snooze state file found, using default");
            return Ok(SnoozeState::default());
        }

        let contents =
            fs::read_to_string(path).map_err(|e| format!("Failed to read snooze state: {}", e))?;

        serde_json::from_str(&contents)
            .map_err(|e| {
                warn!("Failed to parse snooze state, using default: {}", e);
                Ok(SnoozeState::default())
            })
            .unwrap_or_else(|_: Result<SnoozeState, String>| Ok(SnoozeState::default()))
    }

    fn save_state(&self) -> Result<(), String> {
        let contents = serde_json::to_string_pretty(&self.state)
            .map_err(|e| format!("Failed to serialize snooze state: {}", e))?;

        fs::write(&self.state_file, contents)
            .map_err(|e| format!("Failed to write snooze state: {}", e))?;

        debug!("Snooze state saved to {:?}", self.state_file);
        Ok(())
    }

    /// Active le snooze pour une durée donnée
    pub fn snooze(&mut self, duration: SnoozeDuration) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("System time error: {}", e))?
            .as_secs();

        let snooze_duration = duration.to_duration().as_secs();
        let snoozed_until = now + snooze_duration;

        self.state.snoozed_until = Some(snoozed_until);
        self.save_state()?;

        info!(
            "😴 Snoozed for {} (until {})",
            duration.label(),
            snoozed_until
        );
        Ok(())
    }

    /// Annule le snooze
    pub fn unsnooze(&mut self) -> Result<(), String> {
        if self.state.snoozed_until.is_some() {
            self.state.snoozed_until = None;
            self.save_state()?;
            info!("⏰ Snooze cancelled");
        }
        Ok(())
    }

    /// Vérifie si l'app est actuellement snoozed
    pub fn is_snoozed(&self) -> bool {
        if let Some(snoozed_until) = self.state.snoozed_until {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now < snoozed_until {
                return true;
            } else {
                // Snooze expiré, mais on ne le clear pas ici (immutable)
                debug!("Snooze expired (was until {})", snoozed_until);
            }
        }
        false
    }

    /// Récupère le timestamp du snooze (None si pas snoozed)
    pub fn get_snooze_until(&self) -> Option<u64> {
        if self.is_snoozed() {
            self.state.snoozed_until
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snooze_duration() {
        assert!(SnoozeDuration::ThirtyMinutes.to_duration().as_secs() == 30 * 60);
        assert!(SnoozeDuration::TwoHours.to_duration().as_secs() == 2 * 60 * 60);
    }
}

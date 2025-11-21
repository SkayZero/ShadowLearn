// Storage module for persisting replay events
// Future implementation: Save/load events from disk

use super::ReplayEvent;

pub fn save_events_to_disk(_events: &[ReplayEvent]) -> Result<(), String> {
    // TODO: Implement persistence
    Ok(())
}

pub fn load_events_from_disk() -> Result<Vec<ReplayEvent>, String> {
    // TODO: Implement loading
    Ok(Vec::new())
}

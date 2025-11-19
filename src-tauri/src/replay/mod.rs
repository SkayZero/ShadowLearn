/**
 * Shadow Replay Module
 * Records and replays user activity timeline
 */

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub mod storage;
pub mod player;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub id: String,
    pub timestamp: i64,
    pub event_type: EventType,
    pub app_name: String,
    pub description: String,
    pub metadata: serde_json::Value,
    pub screenshot_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    Suggestion {
        suggestion_text: String,
        accepted: bool,
    },
    FlowSession {
        duration_minutes: u32,
        quality_score: f32,
    },
    AppSwitch {
        from_app: String,
        to_app: String,
    },
    Screenshot {
        analysis: Option<String>,
    },
    PatternDetected {
        pattern_name: String,
        confidence: f32,
    },
    Interruption {
        source: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySession {
    pub date: String,
    pub start_time: i64,
    pub end_time: i64,
    pub event_count: usize,
    pub duration_minutes: u32,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStats {
    pub total_events: usize,
    pub total_sessions: usize,
    pub oldest_event: Option<i64>,
    pub newest_event: Option<i64>,
    pub events_by_type: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub current_index: usize,
    pub speed: f32,
    pub events_remaining: usize,
}

pub struct ReplayManager {
    events: VecDeque<ReplayEvent>,
    max_events: usize,
    current_playback_index: usize,
    is_playing: bool,
    playback_speed: f32,
}

impl ReplayManager {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_events,
            current_playback_index: 0,
            is_playing: false,
            playback_speed: 1.0,
        }
    }

    pub fn record_event(&mut self, event: ReplayEvent) {
        info!("📼 Recording event: {} at {}", event.description, event.timestamp);

        // Add to front of queue (most recent first)
        self.events.push_front(event);

        // Trim if exceeding max
        while self.events.len() > self.max_events {
            self.events.pop_back();
        }
    }

    pub fn record_suggestion(&mut self, app_name: &str, suggestion_text: &str, accepted: bool) {
        let event = ReplayEvent {
            id: format!("suggestion_{}", Utc::now().timestamp_millis()),
            timestamp: Utc::now().timestamp(),
            event_type: EventType::Suggestion {
                suggestion_text: suggestion_text.to_string(),
                accepted,
            },
            app_name: app_name.to_string(),
            description: if accepted {
                format!("✅ Accepted suggestion: {}", truncate(suggestion_text, 50))
            } else {
                format!("❌ Dismissed suggestion: {}", truncate(suggestion_text, 50))
            },
            metadata: serde_json::json!({ "accepted": accepted }),
            screenshot_path: None,
        };

        self.record_event(event);
    }

    pub fn record_flow_session(&mut self, app_name: &str, duration_minutes: u32, quality_score: f32) {
        let event = ReplayEvent {
            id: format!("flow_{}", Utc::now().timestamp_millis()),
            timestamp: Utc::now().timestamp(),
            event_type: EventType::FlowSession {
                duration_minutes,
                quality_score,
            },
            app_name: app_name.to_string(),
            description: format!("🧘 Flow session in {} for {}min (quality: {:.0}%)",
                app_name, duration_minutes, quality_score * 100.0),
            metadata: serde_json::json!({
                "duration_minutes": duration_minutes,
                "quality_score": quality_score
            }),
            screenshot_path: None,
        };

        self.record_event(event);
    }

    pub fn record_app_switch(&mut self, from_app: &str, to_app: &str) {
        let event = ReplayEvent {
            id: format!("app_switch_{}", Utc::now().timestamp_millis()),
            timestamp: Utc::now().timestamp(),
            event_type: EventType::AppSwitch {
                from_app: from_app.to_string(),
                to_app: to_app.to_string(),
            },
            app_name: to_app.to_string(),
            description: format!("🔄 Switched from {} to {}", from_app, to_app),
            metadata: serde_json::json!({
                "from": from_app,
                "to": to_app
            }),
            screenshot_path: None,
        };

        self.record_event(event);
    }

    pub fn record_screenshot(&mut self, app_name: &str, screenshot_path: &str, analysis: Option<String>) {
        let event = ReplayEvent {
            id: format!("screenshot_{}", Utc::now().timestamp_millis()),
            timestamp: Utc::now().timestamp(),
            event_type: EventType::Screenshot {
                analysis: analysis.clone(),
            },
            app_name: app_name.to_string(),
            description: format!("📸 Screenshot captured in {}", app_name),
            metadata: serde_json::json!({ "analysis": analysis }),
            screenshot_path: Some(screenshot_path.to_string()),
        };

        self.record_event(event);
    }

    pub fn get_events_for_date(&self, date: &str) -> Vec<ReplayEvent> {
        self.events
            .iter()
            .filter(|e| {
                let event_date = DateTime::from_timestamp(e.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                event_date == date
            })
            .cloned()
            .collect()
    }

    pub fn get_events_for_range(&self, start: i64, end: i64) -> Vec<ReplayEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    pub fn get_recent_events(&self, limit: usize) -> Vec<ReplayEvent> {
        self.events
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_all_sessions(&self) -> Vec<ReplaySession> {
        let mut sessions = Vec::new();
        let mut current_date: Option<String> = None;
        let mut current_session: Option<(String, i64, i64, Vec<String>)> = None;
        let mut event_count = 0;

        for event in self.events.iter().rev() {
            let event_date = DateTime::from_timestamp(event.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default();

            if current_date.as_ref() != Some(&event_date) {
                // Save previous session
                if let Some((date, start, end, highlights)) = current_session.take() {
                    let duration = ((end - start) / 60) as u32;
                    sessions.push(ReplaySession {
                        date,
                        start_time: start,
                        end_time: end,
                        event_count,
                        duration_minutes: duration,
                        highlights,
                    });
                }

                // Start new session
                current_date = Some(event_date.clone());
                current_session = Some((event_date, event.timestamp, event.timestamp, Vec::new()));
                event_count = 1;
            } else {
                // Update current session
                if let Some((_, _, ref mut end, ref mut highlights)) = current_session {
                    *end = event.timestamp;
                    event_count += 1;

                    // Add highlights (flow sessions and accepted suggestions)
                    match &event.event_type {
                        EventType::FlowSession { duration_minutes, .. } if *duration_minutes >= 30 => {
                            highlights.push(format!("🧘 {}min flow session", duration_minutes));
                        }
                        EventType::Suggestion { accepted: true, suggestion_text } => {
                            highlights.push(format!("✅ {}", truncate(suggestion_text, 40)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Save last session
        if let Some((date, start, end, highlights)) = current_session {
            let duration = ((end - start) / 60) as u32;
            sessions.push(ReplaySession {
                date,
                start_time: start,
                end_time: end,
                event_count,
                duration_minutes: duration,
                highlights,
            });
        }

        sessions.reverse();
        sessions
    }

    pub fn get_stats(&self) -> ReplayStats {
        let mut events_by_type = std::collections::HashMap::new();

        for event in &self.events {
            let type_name = match &event.event_type {
                EventType::Suggestion { .. } => "suggestion",
                EventType::FlowSession { .. } => "flow_session",
                EventType::AppSwitch { .. } => "app_switch",
                EventType::Screenshot { .. } => "screenshot",
                EventType::PatternDetected { .. } => "pattern_detected",
                EventType::Interruption { .. } => "interruption",
            };

            *events_by_type.entry(type_name.to_string()).or_insert(0) += 1;
        }

        ReplayStats {
            total_events: self.events.len(),
            total_sessions: self.get_all_sessions().len(),
            oldest_event: self.events.back().map(|e| e.timestamp),
            newest_event: self.events.front().map(|e| e.timestamp),
            events_by_type,
        }
    }

    pub fn start_playback(&mut self) -> Result<(), String> {
        if self.events.is_empty() {
            return Err("No events to replay".to_string());
        }

        self.is_playing = true;
        self.current_playback_index = 0;
        info!("▶️ Starting playback");
        Ok(())
    }

    pub fn stop_playback(&mut self) {
        self.is_playing = false;
        info!("⏸️ Stopped playback");
    }

    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.clamp(0.5, 10.0);
        info!("⏩ Playback speed set to {}x", self.playback_speed);
    }

    pub fn next_event(&mut self) -> Option<ReplayEvent> {
        if !self.is_playing || self.current_playback_index >= self.events.len() {
            return None;
        }

        let event = self.events.get(self.current_playback_index).cloned();
        self.current_playback_index += 1;

        if self.current_playback_index >= self.events.len() {
            self.is_playing = false;
            info!("⏹️ Playback finished");
        }

        event
    }

    pub fn get_playback_state(&self) -> PlaybackState {
        PlaybackState {
            is_playing: self.is_playing,
            current_index: self.current_playback_index,
            speed: self.playback_speed,
            events_remaining: if self.current_playback_index < self.events.len() {
                self.events.len() - self.current_playback_index
            } else {
                0
            },
        }
    }

    pub fn seek_to(&mut self, index: usize) -> Result<(), String> {
        if index >= self.events.len() {
            return Err(format!("Index {} out of bounds (max: {})", index, self.events.len()));
        }

        self.current_playback_index = index;
        Ok(())
    }

    pub fn clear_all_events(&mut self) {
        self.events.clear();
        self.current_playback_index = 0;
        self.is_playing = false;
        info!("🗑️ Cleared all replay events");
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// Tauri Commands
#[tauri::command]
pub async fn get_replay_events(
    date: Option<String>,
    limit: Option<usize>,
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<Vec<ReplayEvent>, String> {
    let manager = replay_manager.lock().await;

    if let Some(date_str) = date {
        Ok(manager.get_events_for_date(&date_str))
    } else {
        Ok(manager.get_recent_events(limit.unwrap_or(100)))
    }
}

#[tauri::command]
pub async fn get_replay_sessions(
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<Vec<ReplaySession>, String> {
    let manager = replay_manager.lock().await;
    Ok(manager.get_all_sessions())
}

#[tauri::command]
pub async fn get_replay_stats(
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<ReplayStats, String> {
    let manager = replay_manager.lock().await;
    Ok(manager.get_stats())
}

#[tauri::command]
pub async fn start_replay_playback(
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<(), String> {
    let mut manager = replay_manager.lock().await;
    manager.start_playback()
}

#[tauri::command]
pub async fn stop_replay_playback(
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<(), String> {
    let mut manager = replay_manager.lock().await;
    manager.stop_playback();
    Ok(())
}

#[tauri::command]
pub async fn set_replay_speed(
    speed: f32,
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<(), String> {
    let mut manager = replay_manager.lock().await;
    manager.set_playback_speed(speed);
    Ok(())
}

#[tauri::command]
pub async fn get_next_replay_event(
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<Option<ReplayEvent>, String> {
    let mut manager = replay_manager.lock().await;
    Ok(manager.next_event())
}

#[tauri::command]
pub async fn get_playback_state(
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<PlaybackState, String> {
    let manager = replay_manager.lock().await;
    Ok(manager.get_playback_state())
}

#[tauri::command]
pub async fn seek_replay_to(
    index: usize,
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<(), String> {
    let mut manager = replay_manager.lock().await;
    manager.seek_to(index)
}

#[tauri::command]
pub async fn record_replay_suggestion(
    app_name: String,
    suggestion_text: String,
    accepted: bool,
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<(), String> {
    let mut manager = replay_manager.lock().await;
    manager.record_suggestion(&app_name, &suggestion_text, accepted);
    Ok(())
}

#[tauri::command]
pub async fn record_replay_flow_session(
    app_name: String,
    duration_minutes: u32,
    quality_score: f32,
    replay_manager: State<'_, Arc<Mutex<ReplayManager>>>,
) -> Result<(), String> {
    let mut manager = replay_manager.lock().await;
    manager.record_flow_session(&app_name, duration_minutes, quality_score);
    Ok(())
}

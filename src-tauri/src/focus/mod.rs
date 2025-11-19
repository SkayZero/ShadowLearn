/**
 * Focus Mode Module
 * Detects deep work and automatically blocks notifications
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusState {
    pub is_in_focus: bool;
    pub focus_start_time: Option<i64>,
    pub focus_duration_minutes: u32,
    pub notifications_blocked: u32,
    pub focus_quality_score: f32,
    pub current_app: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub id: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub duration_minutes: u32,
    pub app_name: String,
    pub quality_score: f32,
    pub interruptions: u32,
    pub notifications_blocked: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusStats {
    pub total_sessions: usize,
    pub total_focus_time_minutes: u32,
    pub total_notifications_blocked: u32,
    pub average_session_duration: u32,
    pub best_session_duration: u32,
    pub current_streak_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusConfig {
    pub enabled: bool,
    pub auto_detect: bool,
    pub min_duration_minutes: u32,
    pub quality_threshold: f32,
    pub block_notifications: bool,
    pub block_triggers: bool,
    pub whitelist_apps: Vec<String>,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_detect: true,
            min_duration_minutes: 15,
            quality_threshold: 0.7,
            block_notifications: true,
            block_triggers: true,
            whitelist_apps: vec!["VS Code".to_string(), "Blender".to_string(), "Ableton".to_string()],
        }
    }
}

pub struct FocusManager {
    config: FocusConfig,
    current_session: Option<FocusSession>,
    sessions: Vec<FocusSession>,
    notifications_blocked_count: u32,
    last_activity_time: Option<Instant>,
    focus_indicators: FocusIndicators,
}

#[derive(Debug, Clone)]
struct FocusIndicators {
    consecutive_keyboard_events: u32,
    consecutive_mouse_events: u32,
    app_switch_count: u32,
    last_app: String,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            config: FocusConfig::default(),
            current_session: None,
            sessions: Vec::new(),
            notifications_blocked_count: 0,
            last_activity_time: None,
            focus_indicators: FocusIndicators {
                consecutive_keyboard_events: 0,
                consecutive_mouse_events: 0,
                app_switch_count: 0,
                last_app: String::new(),
            },
        }
    }

    pub fn update_config(&mut self, config: FocusConfig) {
        self.config = config;
        info!("🎯 Focus config updated");
    }

    pub fn get_config(&self) -> FocusConfig {
        self.config.clone()
    }

    pub fn detect_focus(&mut self, app_name: &str) -> bool {
        if !self.config.enabled || !self.config.auto_detect {
            return false;
        }

        // Check if app is in whitelist (focus-worthy apps)
        let is_focus_app = self.config.whitelist_apps.iter().any(|wa| app_name.contains(wa));

        // Update indicators
        self.last_activity_time = Some(Instant::now());

        if app_name != self.focus_indicators.last_app {
            self.focus_indicators.app_switch_count += 1;
            self.focus_indicators.last_app = app_name.to_string();
        }

        // Calculate focus score
        let focus_score = self.calculate_focus_score(is_focus_app);

        // Check if we should enter focus mode
        let should_enter_focus = focus_score >= self.config.quality_threshold && is_focus_app;

        if should_enter_focus && self.current_session.is_none() {
            self.start_focus_session(app_name);
            return true;
        } else if !should_enter_focus && self.current_session.is_some() {
            self.end_focus_session();
            return false;
        }

        self.current_session.is_some()
    }

    fn calculate_focus_score(&self, is_focus_app: bool) -> f32 {
        let mut score = 0.0;

        // Base score from focus app
        if is_focus_app {
            score += 0.5;
        }

        // Low app switching indicates focus
        if self.focus_indicators.app_switch_count < 3 {
            score += 0.2;
        }

        // Consistent activity indicates focus
        if self.focus_indicators.consecutive_keyboard_events > 10 {
            score += 0.2;
        }

        // Minimal mouse movement indicates focus (deep keyboard work)
        if self.focus_indicators.consecutive_mouse_events < 5 {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn start_focus_session(&mut self, app_name: &str) {
        let now = chrono::Utc::now().timestamp();

        let session = FocusSession {
            id: format!("focus_{}", now),
            start_time: now,
            end_time: None,
            duration_minutes: 0,
            app_name: app_name.to_string(),
            quality_score: self.config.quality_threshold,
            interruptions: 0,
            notifications_blocked: 0,
        };

        self.current_session = Some(session);
        self.notifications_blocked_count = 0;

        info!("🧘 Focus session started in {}", app_name);
    }

    fn end_focus_session(&mut self) {
        if let Some(mut session) = self.current_session.take() {
            let now = chrono::Utc::now().timestamp();
            session.end_time = Some(now);
            session.duration_minutes = ((now - session.start_time) / 60) as u32;
            session.notifications_blocked = self.notifications_blocked_count;

            // Only save sessions longer than minimum duration
            if session.duration_minutes >= self.config.min_duration_minutes {
                self.sessions.push(session.clone());
                info!("✅ Focus session ended: {}min (quality: {:.0}%)",
                    session.duration_minutes, session.quality_score * 100.0);
            } else {
                info!("⏭️ Focus session too short, not saved");
            }

            self.notifications_blocked_count = 0;
        }
    }

    pub fn should_block_notification(&mut self) -> bool {
        if !self.config.enabled || !self.config.block_notifications {
            return false;
        }

        let in_focus = self.current_session.is_some();

        if in_focus {
            self.notifications_blocked_count += 1;
            if let Some(ref mut session) = self.current_session {
                session.notifications_blocked += 1;
            }
            info!("🔕 Notification blocked during focus");
        }

        in_focus
    }

    pub fn should_block_trigger(&self) -> bool {
        if !self.config.enabled || !self.config.block_triggers {
            return false;
        }

        self.current_session.is_some()
    }

    pub fn record_keyboard_activity(&mut self) {
        self.focus_indicators.consecutive_keyboard_events += 1;
        self.focus_indicators.consecutive_mouse_events = 0; // Reset mouse
    }

    pub fn record_mouse_activity(&mut self) {
        self.focus_indicators.consecutive_mouse_events += 1;
        // Don't reset keyboard - typing + occasional mouse is ok
    }

    pub fn record_interruption(&mut self) {
        if let Some(ref mut session) = self.current_session {
            session.interruptions += 1;
            session.quality_score = (session.quality_score - 0.05).max(0.0);
            info!("⚠️ Interruption recorded");
        }
    }

    pub fn get_focus_state(&self) -> FocusState {
        if let Some(ref session) = self.current_session {
            let now = chrono::Utc::now().timestamp();
            let duration = ((now - session.start_time) / 60) as u32;

            FocusState {
                is_in_focus: true,
                focus_start_time: Some(session.start_time),
                focus_duration_minutes: duration,
                notifications_blocked: self.notifications_blocked_count,
                focus_quality_score: session.quality_score,
                current_app: session.app_name.clone(),
            }
        } else {
            FocusState {
                is_in_focus: false,
                focus_start_time: None,
                focus_duration_minutes: 0,
                notifications_blocked: 0,
                focus_quality_score: 0.0,
                current_app: String::new(),
            }
        }
    }

    pub fn get_stats(&self) -> FocusStats {
        let total_focus_time: u32 = self.sessions.iter().map(|s| s.duration_minutes).sum();
        let total_notifications: u32 = self.sessions.iter().map(|s| s.notifications_blocked).sum();
        let avg_duration = if self.sessions.is_empty() {
            0
        } else {
            total_focus_time / self.sessions.len() as u32
        };
        let best_duration = self.sessions.iter().map(|s| s.duration_minutes).max().unwrap_or(0);

        FocusStats {
            total_sessions: self.sessions.len(),
            total_focus_time_minutes: total_focus_time,
            total_notifications_blocked: total_notifications,
            average_session_duration: avg_duration,
            best_session_duration: best_duration,
            current_streak_days: self.calculate_streak(),
        }
    }

    fn calculate_streak(&self) -> u32 {
        // Calculate consecutive days with focus sessions
        let mut streak = 0;
        let mut current_date = chrono::Utc::now().date_naive();

        for session in self.sessions.iter().rev() {
            let session_date = chrono::DateTime::from_timestamp(session.start_time, 0)
                .map(|dt| dt.date_naive())
                .unwrap_or_default();

            if session_date == current_date {
                // Same day, continue
                continue;
            } else if session_date == current_date - chrono::Duration::days(1) {
                // Previous day, increment streak
                streak += 1;
                current_date = session_date;
            } else {
                // Gap in streak, stop
                break;
            }
        }

        // Include today if we have a session
        if self.sessions.iter().any(|s| {
            chrono::DateTime::from_timestamp(s.start_time, 0)
                .map(|dt| dt.date_naive())
                .unwrap_or_default() == chrono::Utc::now().date_naive()
        }) {
            streak += 1;
        }

        streak
    }

    pub fn get_recent_sessions(&self, limit: usize) -> Vec<FocusSession> {
        self.sessions.iter().rev().take(limit).cloned().collect()
    }

    pub fn force_end_focus(&mut self) {
        self.end_focus_session();
        info!("⏹️ Focus mode manually ended");
    }
}

// Tauri Commands
#[tauri::command]
pub async fn get_focus_state(
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<FocusState, String> {
    let manager = focus_manager.lock().await;
    Ok(manager.get_focus_state())
}

#[tauri::command]
pub async fn get_focus_stats(
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<FocusStats, String> {
    let manager = focus_manager.lock().await;
    Ok(manager.get_stats())
}

#[tauri::command]
pub async fn get_focus_config(
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<FocusConfig, String> {
    let manager = focus_manager.lock().await;
    Ok(manager.get_config())
}

#[tauri::command]
pub async fn update_focus_config(
    config: FocusConfig,
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<(), String> {
    let mut manager = focus_manager.lock().await;
    manager.update_config(config);
    Ok(())
}

#[tauri::command]
pub async fn detect_focus_mode(
    app_name: String,
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<bool, String> {
    let mut manager = focus_manager.lock().await;
    Ok(manager.detect_focus(&app_name))
}

#[tauri::command]
pub async fn should_block_notification(
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<bool, String> {
    let mut manager = focus_manager.lock().await;
    Ok(manager.should_block_notification())
}

#[tauri::command]
pub async fn should_block_trigger(
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<bool, String> {
    let manager = focus_manager.lock().await;
    Ok(manager.should_block_trigger())
}

#[tauri::command]
pub async fn end_focus_session(
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<(), String> {
    let mut manager = focus_manager.lock().await;
    manager.force_end_focus();
    Ok(())
}

#[tauri::command]
pub async fn get_recent_focus_sessions(
    limit: usize,
    focus_manager: State<'_, Arc<Mutex<FocusManager>>>,
) -> Result<Vec<FocusSession>, String> {
    let manager = focus_manager.lock().await;
    Ok(manager.get_recent_sessions(limit))
}

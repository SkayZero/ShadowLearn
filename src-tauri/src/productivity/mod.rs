/**
 * Productivity Module
 * Tracks comprehensive productivity metrics, trends, and insights
 */

use chrono::{DateTime, Utc, Datelike, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub today: DayMetrics,
    pub week: WeekMetrics,
    pub trends: TrendData,
    pub insights: Vec<Insight>,
    pub flow_sessions: Vec<FlowSession>,
    pub top_productive_hours: Vec<ProductiveHour>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayMetrics {
    pub date: String,
    pub suggestions_shown: u32,
    pub suggestions_accepted: u32,
    pub acceptance_rate: f32,
    pub time_saved_minutes: u32,
    pub flow_time_minutes: u32,
    pub interruptions: u32,
    pub top_apps: Vec<AppMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekMetrics {
    pub week_number: u32,
    pub total_suggestions: u32,
    pub total_accepted: u32,
    pub total_time_saved: u32,
    pub total_flow_time: u32,
    pub daily_breakdown: Vec<DayMetrics>,
    pub best_day: Option<String>,
    pub improvement_vs_last_week: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub acceptance_rate_trend: Vec<TrendPoint>,
    pub flow_time_trend: Vec<TrendPoint>,
    pub productivity_score_trend: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub date: String,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: String,
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub impact: ImpactLevel,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightCategory {
    Achievement,
    Pattern,
    Improvement,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSession {
    pub start_time: i64,
    pub end_time: i64,
    pub duration_minutes: u32,
    pub app_name: String,
    pub quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductiveHour {
    pub hour: u32,
    pub productivity_score: f32,
    pub flow_sessions: u32,
    pub acceptance_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetric {
    pub name: String,
    pub usage_count: u32,
    pub acceptance_rate: f32,
    pub time_saved: u32,
}

pub struct ProductivityManager {
    daily_data: HashMap<String, DayData>,
    flow_sessions: Vec<FlowSession>,
    hourly_data: HashMap<u32, HourlyData>,
}

#[derive(Debug, Clone)]
struct DayData {
    date: String,
    suggestions_shown: u32,
    suggestions_accepted: u32,
    flow_time_minutes: u32,
    interruptions: u32,
    app_usage: HashMap<String, AppData>,
}

#[derive(Debug, Clone)]
struct AppData {
    count: u32,
    accepted: u32,
}

#[derive(Debug, Clone)]
struct HourlyData {
    suggestions_shown: u32,
    suggestions_accepted: u32,
    flow_sessions: u32,
}

impl ProductivityManager {
    pub fn new() -> Self {
        Self {
            daily_data: HashMap::new(),
            flow_sessions: Vec::new(),
            hourly_data: HashMap::new(),
        }
    }

    fn get_date_key(date: &DateTime<Utc>) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    pub fn record_suggestion(&mut self, app_name: &str, accepted: bool) {
        let now = Utc::now();
        let date_key = Self::get_date_key(&now);
        let hour = now.hour();

        // Update daily data
        let day_data = self.daily_data.entry(date_key.clone()).or_insert(DayData {
            date: date_key,
            suggestions_shown: 0,
            suggestions_accepted: 0,
            flow_time_minutes: 0,
            interruptions: 0,
            app_usage: HashMap::new(),
        });

        day_data.suggestions_shown += 1;
        if accepted {
            day_data.suggestions_accepted += 1;
        }

        let app_data = day_data
            .app_usage
            .entry(app_name.to_string())
            .or_insert(AppData { count: 0, accepted: 0 });
        app_data.count += 1;
        if accepted {
            app_data.accepted += 1;
        }

        // Update hourly data
        let hourly = self.hourly_data.entry(hour).or_insert(HourlyData {
            suggestions_shown: 0,
            suggestions_accepted: 0,
            flow_sessions: 0,
        });
        hourly.suggestions_shown += 1;
        if accepted {
            hourly.suggestions_accepted += 1;
        }
    }

    pub fn record_flow_session(&mut self, app_name: &str, duration_minutes: u32, quality_score: f32) {
        let now = Utc::now();
        let date_key = Self::get_date_key(&now);
        let hour = now.hour();

        let session = FlowSession {
            start_time: now.timestamp() - (duration_minutes as i64 * 60),
            end_time: now.timestamp(),
            duration_minutes,
            app_name: app_name.to_string(),
            quality_score,
        };

        self.flow_sessions.push(session);

        // Update daily flow time
        if let Some(day_data) = self.daily_data.get_mut(&date_key) {
            day_data.flow_time_minutes += duration_minutes;
        }

        // Update hourly flow sessions
        if let Some(hourly) = self.hourly_data.get_mut(&hour) {
            hourly.flow_sessions += 1;
        }
    }

    pub fn record_interruption(&mut self) {
        let now = Utc::now();
        let date_key = Self::get_date_key(&now);

        if let Some(day_data) = self.daily_data.get_mut(&date_key) {
            day_data.interruptions += 1;
        }
    }

    pub fn get_productivity_metrics(&self) -> ProductivityMetrics {
        let today = self.get_today_metrics();
        let week = self.get_week_metrics();
        let trends = self.get_trends();
        let insights = self.generate_insights(&today, &week);
        let top_productive_hours = self.get_top_productive_hours();

        // Get recent flow sessions (last 7 days)
        let seven_days_ago = Utc::now() - Duration::days(7);
        let recent_flow_sessions: Vec<FlowSession> = self
            .flow_sessions
            .iter()
            .filter(|s| s.start_time >= seven_days_ago.timestamp())
            .cloned()
            .collect();

        ProductivityMetrics {
            today,
            week,
            trends,
            insights,
            flow_sessions: recent_flow_sessions,
            top_productive_hours,
        }
    }

    fn get_today_metrics(&self) -> DayMetrics {
        let now = Utc::now();
        let date_key = Self::get_date_key(&now);

        if let Some(day_data) = self.daily_data.get(&date_key) {
            self.day_data_to_metrics(day_data)
        } else {
            self.empty_day_metrics(&date_key)
        }
    }

    fn get_week_metrics(&self) -> WeekMetrics {
        let now = Utc::now();
        let week_number = now.iso_week().week();

        let mut daily_breakdown = Vec::new();
        let mut total_suggestions = 0;
        let mut total_accepted = 0;
        let mut total_time_saved = 0;
        let mut total_flow_time = 0;
        let mut best_day_score = 0.0;
        let mut best_day_date: Option<String> = None;

        // Get last 7 days
        for i in 0..7 {
            let date = now - Duration::days(i);
            let date_key = Self::get_date_key(&date);

            let day_metrics = if let Some(day_data) = self.daily_data.get(&date_key) {
                self.day_data_to_metrics(day_data)
            } else {
                self.empty_day_metrics(&date_key)
            };

            total_suggestions += day_metrics.suggestions_shown;
            total_accepted += day_metrics.suggestions_accepted;
            total_time_saved += day_metrics.time_saved_minutes;
            total_flow_time += day_metrics.flow_time_minutes;

            // Calculate day score for best day
            let day_score = (day_metrics.acceptance_rate
                + (day_metrics.flow_time_minutes as f32 / 60.0) * 10.0
                - day_metrics.interruptions as f32) as f32;

            if day_score > best_day_score {
                best_day_score = day_score;
                best_day_date = Some(date_key.clone());
            }

            daily_breakdown.push(day_metrics);
        }

        // TODO: Calculate improvement vs last week (needs historical data)
        let improvement_vs_last_week = 0.0;

        WeekMetrics {
            week_number,
            total_suggestions,
            total_accepted,
            total_time_saved,
            total_flow_time,
            daily_breakdown,
            best_day: best_day_date,
            improvement_vs_last_week,
        }
    }

    fn get_trends(&self) -> TrendData {
        let now = Utc::now();
        let mut acceptance_rate_trend = Vec::new();
        let mut flow_time_trend = Vec::new();
        let mut productivity_score_trend = Vec::new();

        // Generate trends for last 30 days
        for i in 0..30 {
            let date = now - Duration::days(i);
            let date_key = Self::get_date_key(&date);

            if let Some(day_data) = self.daily_data.get(&date_key) {
                let acceptance_rate = if day_data.suggestions_shown > 0 {
                    (day_data.suggestions_accepted as f32 / day_data.suggestions_shown as f32) * 100.0
                } else {
                    0.0
                };

                let productivity_score = acceptance_rate
                    + (day_data.flow_time_minutes as f32 / 10.0)
                    - (day_data.interruptions as f32 * 2.0);

                acceptance_rate_trend.push(TrendPoint {
                    date: date_key.clone(),
                    value: acceptance_rate,
                });

                flow_time_trend.push(TrendPoint {
                    date: date_key.clone(),
                    value: day_data.flow_time_minutes as f32,
                });

                productivity_score_trend.push(TrendPoint {
                    date: date_key,
                    value: productivity_score,
                });
            }
        }

        // Reverse to get chronological order
        acceptance_rate_trend.reverse();
        flow_time_trend.reverse();
        productivity_score_trend.reverse();

        TrendData {
            acceptance_rate_trend,
            flow_time_trend,
            productivity_score_trend,
        }
    }

    fn generate_insights(&self, today: &DayMetrics, week: &WeekMetrics) -> Vec<Insight> {
        let mut insights = Vec::new();

        // Achievement: High acceptance rate
        if today.acceptance_rate >= 80.0 {
            insights.push(Insight {
                id: "high-acceptance".to_string(),
                category: InsightCategory::Achievement,
                title: "Excellent jour !".to_string(),
                description: format!("{}% de taux d'acceptation aujourd'hui", today.acceptance_rate as u32),
                impact: ImpactLevel::High,
                action: Some("Continue comme ça !".to_string()),
            });
        }

        // Achievement: Flow state
        if today.flow_time_minutes >= 120 {
            insights.push(Insight {
                id: "flow-master".to_string(),
                category: InsightCategory::Achievement,
                title: "Flow Master 🧘".to_string(),
                description: format!("{}h en flow state aujourd'hui", today.flow_time_minutes / 60),
                impact: ImpactLevel::High,
                action: None,
            });
        }

        // Pattern: Best time of day
        if let Some(best_hour) = self.get_best_productive_hour() {
            insights.push(Insight {
                id: "best-hour".to_string(),
                category: InsightCategory::Pattern,
                title: "Ta meilleure heure".to_string(),
                description: format!("Tu es plus productif vers {}h", best_hour),
                impact: ImpactLevel::Medium,
                action: Some("Planifie tes tâches importantes à ce moment".to_string()),
            });
        }

        // Warning: Too many interruptions
        if today.interruptions > 10 {
            insights.push(Insight {
                id: "interruptions".to_string(),
                category: InsightCategory::Warning,
                title: "Beaucoup d'interruptions".to_string(),
                description: format!("{} interruptions aujourd'hui", today.interruptions),
                impact: ImpactLevel::Medium,
                action: Some("Active le mode Focus pour réduire les distractions".to_string()),
            });
        }

        // Improvement: Week over week
        if week.improvement_vs_last_week > 10.0 {
            insights.push(Insight {
                id: "week-improvement".to_string(),
                category: InsightCategory::Improvement,
                title: "Progression !".to_string(),
                description: format!("+{:.0}% vs semaine dernière", week.improvement_vs_last_week),
                impact: ImpactLevel::High,
                action: None,
            });
        }

        insights
    }

    fn get_top_productive_hours(&self) -> Vec<ProductiveHour> {
        let mut hours: Vec<ProductiveHour> = self
            .hourly_data
            .iter()
            .map(|(hour, data)| {
                let acceptance_rate = if data.suggestions_shown > 0 {
                    (data.suggestions_accepted as f32 / data.suggestions_shown as f32) * 100.0
                } else {
                    0.0
                };

                let productivity_score = acceptance_rate + (data.flow_sessions as f32 * 10.0);

                ProductiveHour {
                    hour: *hour,
                    productivity_score,
                    flow_sessions: data.flow_sessions,
                    acceptance_rate,
                }
            })
            .collect();

        hours.sort_by(|a, b| b.productivity_score.partial_cmp(&a.productivity_score).unwrap());
        hours.truncate(5);
        hours
    }

    fn get_best_productive_hour(&self) -> Option<u32> {
        self.hourly_data
            .iter()
            .max_by_key(|(_, data)| {
                let acceptance_rate = if data.suggestions_shown > 0 {
                    (data.suggestions_accepted as f32 / data.suggestions_shown as f32) * 100.0
                } else {
                    0.0
                };
                (acceptance_rate + (data.flow_sessions as f32 * 10.0)) as u32
            })
            .map(|(hour, _)| *hour)
    }

    fn day_data_to_metrics(&self, day_data: &DayData) -> DayMetrics {
        let acceptance_rate = if day_data.suggestions_shown > 0 {
            (day_data.suggestions_accepted as f32 / day_data.suggestions_shown as f32) * 100.0
        } else {
            0.0
        };

        let time_saved_minutes = day_data.suggestions_accepted * 2;

        let mut top_apps: Vec<AppMetric> = day_data
            .app_usage
            .iter()
            .map(|(name, data)| AppMetric {
                name: name.clone(),
                usage_count: data.count,
                acceptance_rate: if data.count > 0 {
                    (data.accepted as f32 / data.count as f32) * 100.0
                } else {
                    0.0
                },
                time_saved: data.accepted * 2,
            })
            .collect();

        top_apps.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        top_apps.truncate(5);

        DayMetrics {
            date: day_data.date.clone(),
            suggestions_shown: day_data.suggestions_shown,
            suggestions_accepted: day_data.suggestions_accepted,
            acceptance_rate,
            time_saved_minutes,
            flow_time_minutes: day_data.flow_time_minutes,
            interruptions: day_data.interruptions,
            top_apps,
        }
    }

    fn empty_day_metrics(&self, date_key: &str) -> DayMetrics {
        DayMetrics {
            date: date_key.to_string(),
            suggestions_shown: 0,
            suggestions_accepted: 0,
            acceptance_rate: 0.0,
            time_saved_minutes: 0,
            flow_time_minutes: 0,
            interruptions: 0,
            top_apps: Vec::new(),
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_productivity_metrics(
    productivity_manager: State<'_, Arc<Mutex<ProductivityManager>>>,
) -> Result<ProductivityMetrics, String> {
    let manager = productivity_manager.lock().await;
    Ok(manager.get_productivity_metrics())
}

#[tauri::command]
pub async fn record_productivity_event(
    event_type: String,
    app_name: String,
    accepted: bool,
    productivity_manager: State<'_, Arc<Mutex<ProductivityManager>>>,
) -> Result<(), String> {
    let mut manager = productivity_manager.lock().await;

    match event_type.as_str() {
        "suggestion" => manager.record_suggestion(&app_name, accepted),
        "interruption" => manager.record_interruption(),
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub async fn record_flow_session_event(
    app_name: String,
    duration_minutes: u32,
    quality_score: f32,
    productivity_manager: State<'_, Arc<Mutex<ProductivityManager>>>,
) -> Result<(), String> {
    let mut manager = productivity_manager.lock().await;
    manager.record_flow_session(&app_name, duration_minutes, quality_score);
    Ok(())
}

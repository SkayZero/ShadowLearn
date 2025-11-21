use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod commands;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakData {
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_days: u32,
    pub last_activity: DateTime<Utc>,
    pub streak_broken: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String, // YYYY-MM-DD
    pub interactions: u32,
    pub time_active_minutes: u32,
}

pub struct StreakManager {
    activity_log: HashMap<String, DailyActivity>,
}

impl StreakManager {
    pub fn new() -> Self {
        Self {
            activity_log: HashMap::new(),
        }
    }

    pub fn record_activity(&mut self) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        
        self.activity_log
            .entry(today.clone())
            .and_modify(|activity| {
                activity.interactions += 1;
            })
            .or_insert(DailyActivity {
                date: today,
                interactions: 1,
                time_active_minutes: 1,
            });
    }

    pub fn calculate_streak(&self) -> StreakData {
        let today = Utc::now();
        let mut current_streak = 0;
        let mut longest_streak = 0;
        let mut temp_streak = 0;
        let mut total_days = 0;
        let mut last_activity = today;
        let mut streak_broken = false;

        // Get sorted dates
        let mut dates: Vec<_> = self.activity_log.keys().collect();
        dates.sort();
        dates.reverse(); // Most recent first

        // Calculate current streak
        for (i, date_str) in dates.iter().enumerate() {
            if let Ok(date) = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date_str)) {
                let date = date.with_timezone(&Utc);
                
                if i == 0 {
                    // Check if today or yesterday
                    let days_diff = (today.date_naive() - date.date_naive()).num_days();
                    
                    if days_diff == 0 {
                        current_streak = 1;
                        temp_streak = 1;
                        last_activity = date;
                    } else if days_diff == 1 {
                        current_streak = 1;
                        temp_streak = 1;
                        last_activity = date;
                    } else {
                        streak_broken = true;
                        break;
                    }
                } else {
                    // Check consecutive days
                    if let Ok(prev_date) = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", dates[i - 1])) {
                        let prev_date = prev_date.with_timezone(&Utc);
                        let days_diff = (prev_date.date_naive() - date.date_naive()).num_days();
                        
                        if days_diff == 1 {
                            current_streak += 1;
                            temp_streak += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        // Calculate longest streak
        for date_str in dates.iter() {
            if self.activity_log.contains_key(*date_str) {
                temp_streak += 1;
                longest_streak = longest_streak.max(temp_streak);
                total_days += 1;
            } else {
                temp_streak = 0;
            }
        }

        StreakData {
            current_streak,
            longest_streak: longest_streak.max(current_streak),
            total_days: self.activity_log.len() as u32,
            last_activity,
            streak_broken,
        }
    }

    pub fn get_streak(&self) -> StreakData {
        self.calculate_streak()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streak_calculation() {
        let mut manager = StreakManager::new();
        manager.record_activity();
        
        let streak = manager.get_streak();
        assert_eq!(streak.current_streak, 1);
        assert_eq!(streak.total_days, 1);
    }
}




#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSpan {
    pub name: String,
    pub duration_ms: u64,
    pub tags: HashMap<String, String>,
    pub timestamp: i64,
}

pub struct TelemetryCollector {
    spans: Arc<Mutex<Vec<CompletedSpan>>>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_span(&self, name: &str) -> Span {
        Span {
            name: name.to_string(),
            start: Instant::now(),
            tags: HashMap::new(),
            collector: self.spans.clone(),
        }
    }

    pub async fn get_recent_spans(&self, limit: usize) -> Vec<CompletedSpan> {
        let spans = self.spans.lock().await;
        spans.iter().rev().take(limit).cloned().collect()
    }

    pub async fn clear(&self) {
        let mut spans = self.spans.lock().await;
        spans.clear();
    }
}

pub struct Span {
    name: String,
    start: Instant,
    tags: HashMap<String, String>,
    collector: Arc<Mutex<Vec<CompletedSpan>>>,
}

impl Span {
    pub fn set_tag(&mut self, key: &str, value: impl ToString) {
        self.tags.insert(key.to_string(), value.to_string());
    }

    pub fn finish(self) {
        let completed = CompletedSpan {
            name: self.name,
            duration_ms: self.start.elapsed().as_millis() as u64,
            tags: self.tags,
            timestamp: chrono::Utc::now().timestamp(),
        };

        tokio::spawn(async move {
            self.collector.lock().await.push(completed);
        });
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}


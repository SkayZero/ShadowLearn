pub mod collector;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Types d'événements de télémétrie
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    IdleCheck,
    ScreenshotCapture,
    HealthCheck,
    ComponentRestart,
    WindowToggle,
    MessageSent,
}

impl EventType {
    pub fn name(&self) -> &'static str {
        match self {
            EventType::IdleCheck => "idle_check",
            EventType::ScreenshotCapture => "screenshot_capture",
            EventType::HealthCheck => "health_check",
            EventType::ComponentRestart => "component_restart",
            EventType::WindowToggle => "window_toggle",
            EventType::MessageSent => "message_sent",
        }
    }
}

/// Événement de télémétrie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub timestamp: u64,
    pub event_type: EventType,
    pub duration_ms: Option<u64>,
    pub metadata: Option<String>,
}

impl TelemetryEvent {
    pub fn new(event_type: EventType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            event_type,
            duration_ms: None,
            metadata: None,
        }
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

/// Histogramme pour calculer les percentiles
#[derive(Debug, Clone)]
struct Histogram {
    samples: Vec<u64>,
    max_samples: usize,
}

impl Histogram {
    fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    fn add_sample(&mut self, value: u64) {
        if self.samples.len() >= self.max_samples {
            // Remove oldest sample (FIFO)
            self.samples.remove(0);
        }
        self.samples.push(value);
    }

    fn percentile(&self, p: f64) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }

        let mut sorted = self.samples.clone();
        sorted.sort_unstable();

        let index = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
        Some(sorted[index])
    }

    fn count(&self) -> usize {
        self.samples.len()
    }

    fn avg(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }

        let sum: u64 = self.samples.iter().sum();
        Some(sum as f64 / self.samples.len() as f64)
    }
}

/// Ensemble d'histogrammes pour différents types d'événements
#[derive(Debug, Clone)]
struct HistogramSet {
    idle_check: Histogram,
    screenshot_capture: Histogram,
    health_check: Histogram,
    component_restart: Histogram,
    window_toggle: Histogram,
    message_sent: Histogram,
}

impl HistogramSet {
    fn new(max_samples: usize) -> Self {
        Self {
            idle_check: Histogram::new(max_samples),
            screenshot_capture: Histogram::new(max_samples),
            health_check: Histogram::new(max_samples),
            component_restart: Histogram::new(max_samples),
            window_toggle: Histogram::new(max_samples),
            message_sent: Histogram::new(max_samples),
        }
    }

    fn get_histogram_mut(&mut self, event_type: EventType) -> &mut Histogram {
        match event_type {
            EventType::IdleCheck => &mut self.idle_check,
            EventType::ScreenshotCapture => &mut self.screenshot_capture,
            EventType::HealthCheck => &mut self.health_check,
            EventType::ComponentRestart => &mut self.component_restart,
            EventType::WindowToggle => &mut self.window_toggle,
            EventType::MessageSent => &mut self.message_sent,
        }
    }
}

/// Système de télémétrie
pub struct Telemetry {
    events: Arc<Mutex<VecDeque<TelemetryEvent>>>,
    histograms: Arc<Mutex<HistogramSet>>,
    max_events: usize,
}

impl Telemetry {
    pub fn new(max_events: usize, max_samples: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::with_capacity(max_events))),
            histograms: Arc::new(Mutex::new(HistogramSet::new(max_samples))),
            max_events,
        }
    }

    /// Enregistre un événement
    pub fn record_event(&self, event: TelemetryEvent) {
        // Ajouter à la liste des événements
        {
            let mut events = self.events.lock().unwrap();
            if events.len() >= self.max_events {
                events.pop_front();
            }
            events.push_back(event.clone());
        }

        // Mettre à jour l'histogramme si l'événement a une durée
        if let Some(duration_ms) = event.duration_ms {
            let mut histograms = self.histograms.lock().unwrap();
            histograms
                .get_histogram_mut(event.event_type)
                .add_sample(duration_ms);
        }

        // Only log non-health-check events to avoid spam
        if event.event_type != EventType::HealthCheck {
            println!(
                "📊 Telemetry: {} ({}ms)",
                event.event_type.name(),
                event.duration_ms.unwrap_or(0)
            );
        }
    }

    /// Récupère les statistiques de télémétrie
    pub fn get_stats(&self) -> TelemetryStats {
        let histograms = self.histograms.lock().unwrap();
        let events = self.events.lock().unwrap();

        // Calculer les statistiques globales (tous les événements avec durée)
        let all_durations: Vec<u64> = events.iter().filter_map(|e| e.duration_ms).collect();

        let global_stats = if !all_durations.is_empty() {
            let mut sorted = all_durations.clone();
            sorted.sort_unstable();

            let p50_index = ((0.5) * (sorted.len() as f64 - 1.0)).round() as usize;
            let p95_index = ((0.95) * (sorted.len() as f64 - 1.0)).round() as usize;
            let p99_index = ((0.99) * (sorted.len() as f64 - 1.0)).round() as usize;

            PercentileStats {
                p50: sorted.get(p50_index).copied(),
                p95: sorted.get(p95_index).copied(),
                p99: sorted.get(p99_index).copied(),
                avg: Some(sorted.iter().sum::<u64>() as f64 / sorted.len() as f64),
                count: sorted.len(),
            }
        } else {
            PercentileStats::default()
        };

        TelemetryStats {
            global: global_stats,
            idle_check: Self::get_percentile_stats(&histograms.idle_check),
            screenshot_capture: Self::get_percentile_stats(&histograms.screenshot_capture),
            health_check: Self::get_percentile_stats(&histograms.health_check),
            component_restart: Self::get_percentile_stats(&histograms.component_restart),
            window_toggle: Self::get_percentile_stats(&histograms.window_toggle),
            message_sent: Self::get_percentile_stats(&histograms.message_sent),
            total_events: events.len(),
        }
    }

    fn get_percentile_stats(histogram: &Histogram) -> PercentileStats {
        PercentileStats {
            p50: histogram.percentile(50.0),
            p95: histogram.percentile(95.0),
            p99: histogram.percentile(99.0),
            avg: histogram.avg(),
            count: histogram.count(),
        }
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new(1000, 100)
    }
}

/// Statistiques de percentiles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PercentileStats {
    pub p50: Option<u64>,
    pub p95: Option<u64>,
    pub p99: Option<u64>,
    pub avg: Option<f64>,
    pub count: usize,
}

/// Statistiques globales de télémétrie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryStats {
    pub global: PercentileStats,
    pub idle_check: PercentileStats,
    pub screenshot_capture: PercentileStats,
    pub health_check: PercentileStats,
    pub component_restart: PercentileStats,
    pub window_toggle: PercentileStats,
    pub message_sent: PercentileStats,
    pub total_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_percentiles() {
        let mut histogram = Histogram::new(100);

        // Add samples: 1, 2, 3, ..., 100
        for i in 1..=100 {
            histogram.add_sample(i);
        }

        assert_eq!(histogram.percentile(50.0), Some(50));
        assert_eq!(histogram.percentile(95.0), Some(95));
        assert_eq!(histogram.percentile(99.0), Some(99));
        assert_eq!(histogram.avg(), Some(50.5));
    }

    #[test]
    fn test_histogram_max_samples() {
        let mut histogram = Histogram::new(10);

        for i in 1..=20 {
            histogram.add_sample(i);
        }

        assert_eq!(histogram.count(), 10);
        // Should keep last 10 samples (11-20)
        assert_eq!(histogram.percentile(50.0), Some(15));
    }

    #[test]
    fn test_telemetry_record_event() {
        let telemetry = Telemetry::new(100, 50);

        let event = TelemetryEvent::new(EventType::HealthCheck).with_duration(42);

        telemetry.record_event(event);

        let stats = telemetry.get_stats();
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.health_check.count, 1);
        assert_eq!(stats.health_check.p50, Some(42));
    }

    #[test]
    fn test_telemetry_multiple_events() {
        let telemetry = Telemetry::new(100, 50);

        for i in 1..=10 {
            let event = TelemetryEvent::new(EventType::IdleCheck).with_duration(i * 10);
            telemetry.record_event(event);
        }

        let stats = telemetry.get_stats();
        assert_eq!(stats.total_events, 10);
        assert_eq!(stats.idle_check.count, 10);
        assert_eq!(stats.idle_check.p50, Some(50));
    }

    #[test]
    fn test_recent_events() {
        let telemetry = Telemetry::new(100, 50);

        for i in 1..=5 {
            let event = TelemetryEvent::new(EventType::MessageSent).with_duration(i * 10);
            telemetry.record_event(event);
        }

        let recent = telemetry.get_recent_events(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].duration_ms, Some(50)); // Most recent first
        assert_eq!(recent[1].duration_ms, Some(40));
        assert_eq!(recent[2].duration_ms, Some(30));
    }

    #[test]
    fn test_telemetry_reset() {
        let telemetry = Telemetry::new(100, 50);

        let event = TelemetryEvent::new(EventType::HealthCheck).with_duration(42);
        telemetry.record_event(event);

        telemetry.reset();

        let stats = telemetry.get_stats();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.health_check.count, 0);
    }
}

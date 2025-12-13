//! Resolution Time Tracking (v0.0.487).
//!
//! Tracks how long it takes to resolve user requests.
//! Provides statistics on fastest and slowest resolutions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A recorded resolution with timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRecord {
    /// Unix timestamp when request started
    pub start_time: u64,
    /// Unix timestamp when resolved
    pub end_time: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Brief description of the request
    pub description: String,
    /// Category of the request
    pub category: Option<String>,
    /// Whether resolution was successful
    pub successful: bool,
    /// Whether it required escalation
    pub escalated: bool,
}

impl ResolutionRecord {
    /// Create a new resolution record
    pub fn new(start_time: u64, end_time: u64, description: &str) -> Self {
        let duration_ms = (end_time - start_time) * 1000;
        Self {
            start_time,
            end_time,
            duration_ms,
            description: description.to_string(),
            category: None,
            successful: true,
            escalated: false,
        }
    }

    /// Create from millisecond timestamps
    pub fn from_ms(start_ms: u64, end_ms: u64, description: &str) -> Self {
        Self {
            start_time: start_ms / 1000,
            end_time: end_ms / 1000,
            duration_ms: end_ms.saturating_sub(start_ms),
            description: description.to_string(),
            category: None,
            successful: true,
            escalated: false,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Mark as failed
    pub fn mark_failed(mut self) -> Self {
        self.successful = false;
        self
    }

    /// Mark as escalated
    pub fn mark_escalated(mut self) -> Self {
        self.escalated = true;
        self
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> f64 {
        self.duration_ms as f64 / 1000.0
    }

    /// Get human-readable duration
    pub fn duration_human(&self) -> String {
        format_duration_ms(self.duration_ms)
    }
}

/// Resolution time statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolutionTimeTracker {
    /// Total resolutions tracked
    pub total_resolutions: u64,
    /// Total time spent (ms)
    pub total_time_ms: u64,
    /// Successful resolutions
    pub successful_count: u64,
    /// Failed resolutions
    pub failed_count: u64,
    /// Escalated resolutions
    pub escalated_count: u64,
    /// Fastest resolution
    pub fastest: Option<ResolutionRecord>,
    /// Slowest resolution
    pub slowest: Option<ResolutionRecord>,
    /// Recent resolutions (last 20)
    pub recent: Vec<ResolutionRecord>,
    /// Resolution times by category
    pub by_category: HashMap<String, CategoryStats>,
}

/// Statistics for a category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStats {
    /// Number of resolutions
    pub count: u64,
    /// Total time (ms)
    pub total_ms: u64,
    /// Fastest (ms)
    pub fastest_ms: u64,
    /// Slowest (ms)
    pub slowest_ms: u64,
}

impl CategoryStats {
    /// Get average time (ms)
    pub fn average_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_ms as f64 / self.count as f64
        }
    }
}

impl ResolutionTimeTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a resolution
    pub fn record(&mut self, record: ResolutionRecord) {
        self.total_resolutions += 1;
        self.total_time_ms += record.duration_ms;

        if record.successful {
            self.successful_count += 1;
        } else {
            self.failed_count += 1;
        }

        if record.escalated {
            self.escalated_count += 1;
        }

        // Update fastest
        if self
            .fastest
            .as_ref()
            .map(|f| record.duration_ms < f.duration_ms)
            .unwrap_or(true)
        {
            self.fastest = Some(record.clone());
        }

        // Update slowest
        if self
            .slowest
            .as_ref()
            .map(|s| record.duration_ms > s.duration_ms)
            .unwrap_or(true)
        {
            self.slowest = Some(record.clone());
        }

        // Update category stats
        if let Some(category) = &record.category {
            let stats = self.by_category.entry(category.clone()).or_default();
            stats.count += 1;
            stats.total_ms += record.duration_ms;
            if stats.fastest_ms == 0 || record.duration_ms < stats.fastest_ms {
                stats.fastest_ms = record.duration_ms;
            }
            if record.duration_ms > stats.slowest_ms {
                stats.slowest_ms = record.duration_ms;
            }
        }

        // Add to recent, keep last 20
        self.recent.push(record);
        if self.recent.len() > 20 {
            self.recent.remove(0);
        }
    }

    /// Record a simple resolution
    pub fn record_simple(&mut self, start_ms: u64, end_ms: u64, description: &str) {
        let record = ResolutionRecord::from_ms(start_ms, end_ms, description);
        self.record(record);
    }

    /// Get average resolution time (ms)
    pub fn average_ms(&self) -> f64 {
        if self.total_resolutions == 0 {
            0.0
        } else {
            self.total_time_ms as f64 / self.total_resolutions as f64
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_resolutions == 0 {
            0.0
        } else {
            self.successful_count as f64 / self.total_resolutions as f64 * 100.0
        }
    }

    /// Get escalation rate
    pub fn escalation_rate(&self) -> f64 {
        if self.total_resolutions == 0 {
            0.0
        } else {
            self.escalated_count as f64 / self.total_resolutions as f64 * 100.0
        }
    }

    /// Get time range (fastest, slowest) in ms
    pub fn time_range(&self) -> Option<(u64, u64)> {
        match (&self.fastest, &self.slowest) {
            (Some(f), Some(s)) => Some((f.duration_ms, s.duration_ms)),
            _ => None,
        }
    }

    /// Get fastest category
    pub fn fastest_category(&self) -> Option<(&str, f64)> {
        self.by_category
            .iter()
            .filter(|(_, s)| s.count > 0)
            .min_by(|a, b| {
                a.1.average_ms()
                    .partial_cmp(&b.1.average_ms())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(k, s)| (k.as_str(), s.average_ms()))
    }

    /// Get slowest category
    pub fn slowest_category(&self) -> Option<(&str, f64)> {
        self.by_category
            .iter()
            .filter(|(_, s)| s.count > 0)
            .max_by(|a, b| {
                a.1.average_ms()
                    .partial_cmp(&b.1.average_ms())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(k, s)| (k.as_str(), s.average_ms()))
    }
}

/// Format milliseconds as human-readable duration
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3600000 {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = ms / 3600000;
        let mins = (ms % 3600000) / 60000;
        format!("{}h {}m", hours, mins)
    }
}

/// Resolution time summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionTimeSummary {
    /// Total resolutions
    pub total: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average time (ms)
    pub avg_ms: f64,
    /// Fastest (ms)
    pub fastest_ms: u64,
    /// Slowest (ms)
    pub slowest_ms: u64,
    /// Escalation rate percentage
    pub escalation_rate: f64,
}

impl ResolutionTimeTracker {
    /// Generate summary
    pub fn summary(&self) -> ResolutionTimeSummary {
        ResolutionTimeSummary {
            total: self.total_resolutions,
            success_rate: self.success_rate(),
            avg_ms: self.average_ms(),
            fastest_ms: self.fastest.as_ref().map(|f| f.duration_ms).unwrap_or(0),
            slowest_ms: self.slowest.as_ref().map(|s| s.duration_ms).unwrap_or(0),
            escalation_rate: self.escalation_rate(),
        }
    }
}

/// Format resolution time stats for display
pub fn format_resolution_times(tracker: &ResolutionTimeTracker) -> String {
    let mut output = String::new();

    output.push_str("Resolution Time Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if tracker.total_resolutions == 0 {
        output.push_str("No resolutions recorded yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Resolutions: {} ({:.1}% success)\n",
        tracker.total_resolutions,
        tracker.success_rate()
    ));
    output.push_str(&format!(
        "Average Time:      {}\n",
        format_duration_ms(tracker.average_ms() as u64)
    ));
    output.push_str(&format!(
        "Escalation Rate:   {:.1}%\n\n",
        tracker.escalation_rate()
    ));

    if let Some(fastest) = &tracker.fastest {
        output.push_str("Fastest Resolution:\n");
        output.push_str(&format!(
            "  {} - \"{}\"\n\n",
            fastest.duration_human(),
            if fastest.description.len() > 40 {
                format!("{}...", &fastest.description[..37])
            } else {
                fastest.description.clone()
            }
        ));
    }

    if let Some(slowest) = &tracker.slowest {
        output.push_str("Slowest Resolution:\n");
        output.push_str(&format!(
            "  {} - \"{}\"\n",
            slowest.duration_human(),
            if slowest.description.len() > 40 {
                format!("{}...", &slowest.description[..37])
            } else {
                slowest.description.clone()
            }
        ));
    }

    output
}

/// Format compact resolution time info
pub fn format_resolution_times_compact(tracker: &ResolutionTimeTracker) -> String {
    if tracker.total_resolutions == 0 {
        return "No resolutions yet".to_string();
    }

    let fastest = tracker.fastest.as_ref().map(|f| f.duration_ms).unwrap_or(0);
    let slowest = tracker.slowest.as_ref().map(|s| s.duration_ms).unwrap_or(0);

    format!(
        "{} resolutions, avg {}, range {}–{}",
        tracker.total_resolutions,
        format_duration_ms(tracker.average_ms() as u64),
        format_duration_ms(fastest),
        format_duration_ms(slowest)
    )
}

/// Generate fun fact about resolution times
pub fn resolution_time_fun_fact(tracker: &ResolutionTimeTracker) -> Option<String> {
    if tracker.total_resolutions < 5 {
        return None;
    }

    let avg_secs = tracker.average_ms() / 1000.0;
    let fastest_ms = tracker.fastest.as_ref().map(|f| f.duration_ms).unwrap_or(0);
    let slowest_ms = tracker.slowest.as_ref().map(|s| s.duration_ms).unwrap_or(0);

    let facts = vec![
        format!(
            "Average resolution takes {:.1} seconds - {} making instant coffee!",
            avg_secs,
            if avg_secs < 30.0 { "faster than" } else { "slower than" }
        ),
        format!(
            "Fastest fix was {} - blink and you'd miss it!",
            format_duration_ms(fastest_ms)
        ),
        format!(
            "Longest resolution took {} - that was a tough one!",
            format_duration_ms(slowest_ms)
        ),
        format!(
            "Success rate is {:.1}% - {}!",
            tracker.success_rate(),
            if tracker.success_rate() > 90.0 {
                "excellent reliability"
            } else if tracker.success_rate() > 70.0 {
                "pretty good"
            } else {
                "room for improvement"
            }
        ),
    ];

    let index = (tracker.total_resolutions as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about resolution times
pub fn is_resolution_time_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "resolution time",
        "how long",
        "fastest resolution",
        "slowest resolution",
        "average time",
        "time to resolve",
        "response time",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_record_new() {
        let record = ResolutionRecord::new(1000, 1005, "Install vim");
        assert_eq!(record.duration_ms, 5000);
        assert!(record.successful);
        assert!(!record.escalated);
    }

    #[test]
    fn test_resolution_record_from_ms() {
        let record = ResolutionRecord::from_ms(1000, 3500, "Quick fix");
        assert_eq!(record.duration_ms, 2500);
    }

    #[test]
    fn test_duration_human() {
        let record = ResolutionRecord::from_ms(0, 2500, "Test");
        assert_eq!(record.duration_human(), "2.5s");

        let record2 = ResolutionRecord::from_ms(0, 65000, "Test");
        assert!(record2.duration_human().contains("m"));
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(2500), "2.5s");
        assert_eq!(format_duration_ms(65000), "1m 5s");
        assert_eq!(format_duration_ms(3700000), "1h 1m");
    }

    #[test]
    fn test_tracker_record() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "Fast fix");
        tracker.record_simple(0, 5000, "Slow fix");
        tracker.record_simple(0, 2500, "Medium fix");

        assert_eq!(tracker.total_resolutions, 3);
        assert!(tracker.fastest.is_some());
        assert!(tracker.slowest.is_some());
    }

    #[test]
    fn test_fastest_slowest() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 500, "Fast");
        tracker.record_simple(0, 10000, "Slow");

        assert_eq!(tracker.fastest.as_ref().unwrap().duration_ms, 500);
        assert_eq!(tracker.slowest.as_ref().unwrap().duration_ms, 10000);
    }

    #[test]
    fn test_average() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "One");
        tracker.record_simple(0, 3000, "Two");

        assert_eq!(tracker.average_ms(), 2000.0);
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = ResolutionTimeTracker::new();

        let success = ResolutionRecord::from_ms(0, 1000, "Success");
        let fail = ResolutionRecord::from_ms(0, 1000, "Fail").mark_failed();

        tracker.record(success);
        tracker.record(fail);

        assert_eq!(tracker.success_rate(), 50.0);
    }

    #[test]
    fn test_escalation_rate() {
        let mut tracker = ResolutionTimeTracker::new();

        let normal = ResolutionRecord::from_ms(0, 1000, "Normal");
        let escalated = ResolutionRecord::from_ms(0, 1000, "Escalated").mark_escalated();

        tracker.record(normal);
        tracker.record(escalated);

        assert_eq!(tracker.escalation_rate(), 50.0);
    }

    #[test]
    fn test_category_stats() {
        let mut tracker = ResolutionTimeTracker::new();

        let r1 = ResolutionRecord::from_ms(0, 1000, "Package 1").with_category("package");
        let r2 = ResolutionRecord::from_ms(0, 3000, "Package 2").with_category("package");
        let r3 = ResolutionRecord::from_ms(0, 500, "Service").with_category("service");

        tracker.record(r1);
        tracker.record(r2);
        tracker.record(r3);

        assert_eq!(tracker.by_category.get("package").unwrap().count, 2);
        assert_eq!(tracker.by_category.get("service").unwrap().count, 1);
    }

    #[test]
    fn test_recent_limit() {
        let mut tracker = ResolutionTimeTracker::new();

        for i in 0..25 {
            tracker.record_simple(0, i * 100, &format!("Record {}", i));
        }

        assert_eq!(tracker.recent.len(), 20);
    }

    #[test]
    fn test_summary() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "One");
        tracker.record_simple(0, 2000, "Two");

        let summary = tracker.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.success_rate, 100.0);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 1000, "Test");
        tracker.record_simple(0, 2000, "Test");

        let output = format_resolution_times_compact(&tracker);
        assert!(output.contains("2 resolutions"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = ResolutionTimeTracker::new();

        for i in 0..10 {
            tracker.record_simple(0, (i + 1) * 1000, &format!("Task {}", i));
        }

        let fact = resolution_time_fun_fact(&tracker);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_resolution_time_query() {
        assert!(is_resolution_time_query("what's the average resolution time"));
        assert!(is_resolution_time_query("fastest resolution"));
        assert!(is_resolution_time_query("how long does it take"));

        assert!(!is_resolution_time_query("install vim"));
        assert!(!is_resolution_time_query("status"));
    }

    #[test]
    fn test_time_range() {
        let mut tracker = ResolutionTimeTracker::new();

        tracker.record_simple(0, 500, "Fast");
        tracker.record_simple(0, 5000, "Slow");

        let range = tracker.time_range();
        assert!(range.is_some());
        let (min, max) = range.unwrap();
        assert_eq!(min, 500);
        assert_eq!(max, 5000);
    }
}

//! Resolution time statistics and tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::record::ResolutionRecord;

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

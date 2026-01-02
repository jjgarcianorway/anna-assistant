//! Boot Time Tracker Implementation
//!
//! Core tracking logic for recording and analyzing boot times.

use crate::boot_time_tracking::types::{BootRecord, BootTimeTracker, BootTrend};

impl BootTimeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new boot time
    pub fn record(&mut self, record: BootRecord) {
        // Update fastest/slowest
        match self.fastest_boot_secs {
            Some(fastest) if record.boot_time_secs < fastest => {
                self.fastest_boot_secs = Some(record.boot_time_secs);
            }
            None => self.fastest_boot_secs = Some(record.boot_time_secs),
            _ => {}
        }

        match self.slowest_boot_secs {
            Some(slowest) if record.boot_time_secs > slowest => {
                self.slowest_boot_secs = Some(record.boot_time_secs);
            }
            None => self.slowest_boot_secs = Some(record.boot_time_secs),
            _ => {}
        }

        // Track problem services
        for slow in &record.slow_services {
            *self.problem_services.entry(slow.name.clone()).or_insert(0) += 1;
        }

        self.records.push(record);
    }

    /// Get the most recent boot record
    pub fn latest(&self) -> Option<&BootRecord> {
        self.records.last()
    }

    /// Get the previous boot record
    pub fn previous(&self) -> Option<&BootRecord> {
        if self.records.len() >= 2 {
            Some(&self.records[self.records.len() - 2])
        } else {
            None
        }
    }

    /// Calculate the change from previous boot
    pub fn change_from_previous(&self) -> Option<f64> {
        match (self.latest(), self.previous()) {
            (Some(latest), Some(prev)) => Some(latest.boot_time_secs - prev.boot_time_secs),
            _ => None,
        }
    }

    /// Get the trend over recent boots
    pub fn trend(&self, window: usize) -> BootTrend {
        if self.records.len() < 2 {
            return BootTrend::Stable;
        }

        let records: Vec<_> = self.records.iter().rev().take(window).collect();
        if records.len() < 2 {
            return BootTrend::Stable;
        }

        let recent_avg: f64 = records.iter().take(window / 2).map(|r| r.boot_time_secs).sum::<f64>()
            / (window / 2).max(1) as f64;
        let older_avg: f64 = records.iter().skip(window / 2).map(|r| r.boot_time_secs).sum::<f64>()
            / records.len().saturating_sub(window / 2).max(1) as f64;

        let diff = recent_avg - older_avg;
        if diff > 1.0 {
            BootTrend::Slower
        } else if diff < -1.0 {
            BootTrend::Faster
        } else {
            BootTrend::Stable
        }
    }

    /// Average boot time
    pub fn average_boot_secs(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.records.iter().map(|r| r.boot_time_secs).sum::<f64>() / self.records.len() as f64
    }

    /// Number of boots recorded
    pub fn boot_count(&self) -> usize {
        self.records.len()
    }

    /// Top services that slow boot (by occurrence count)
    pub fn top_slow_services(&self, limit: usize) -> Vec<(&str, u32)> {
        let mut services: Vec<_> = self.problem_services.iter().collect();
        services.sort_by(|a, b| b.1.cmp(a.1));
        services.into_iter().take(limit).map(|(k, v)| (k.as_str(), *v)).collect()
    }

    /// Recent boot times for display
    pub fn recent_boots(&self, limit: usize) -> Vec<&BootRecord> {
        self.records.iter().rev().take(limit).collect()
    }
}

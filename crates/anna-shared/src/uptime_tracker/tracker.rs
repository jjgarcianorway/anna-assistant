//! Uptime statistics tracker.

use serde::{Deserialize, Serialize};
use super::record::UptimeRecord;

/// Uptime statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UptimeTracker {
    /// Installation timestamp
    pub installed_at: u64,
    /// Current session start
    pub current_session_start: Option<u64>,
    /// Total uptime in seconds
    pub total_uptime_secs: u64,
    /// Number of sessions
    pub session_count: u64,
    /// Clean shutdown count
    pub clean_shutdowns: u64,
    /// Crash/unclean shutdown count
    pub crashes: u64,
    /// Longest session (seconds)
    pub longest_session_secs: u64,
    /// Shortest session (seconds)
    pub shortest_session_secs: u64,
    /// Recent sessions (last 10)
    pub recent_sessions: Vec<UptimeRecord>,
}

impl UptimeTracker {
    /// Create new tracker with installation time
    pub fn new(installed_at: u64) -> Self {
        Self {
            installed_at,
            shortest_session_secs: u64::MAX,
            ..Default::default()
        }
    }

    /// Start a new session
    pub fn start_session(&mut self, timestamp: u64) {
        self.current_session_start = Some(timestamp);
        self.session_count += 1;
    }

    /// End current session
    pub fn end_session(&mut self, timestamp: u64, clean: bool) {
        if let Some(start) = self.current_session_start.take() {
            let duration = timestamp.saturating_sub(start);
            self.total_uptime_secs += duration;

            if clean {
                self.clean_shutdowns += 1;
            } else {
                self.crashes += 1;
            }

            // Update longest/shortest
            if duration > self.longest_session_secs {
                self.longest_session_secs = duration;
            }
            if duration < self.shortest_session_secs && duration > 0 {
                self.shortest_session_secs = duration;
            }

            // Record session
            let mut record = UptimeRecord::start(start);
            record.end(timestamp, clean);

            self.recent_sessions.push(record);
            if self.recent_sessions.len() > 10 {
                self.recent_sessions.remove(0);
            }
        }
    }

    /// Get current session duration (if running)
    pub fn current_session_duration(&self, now: u64) -> Option<u64> {
        self.current_session_start.map(|start| now.saturating_sub(start))
    }

    /// Get days since installation
    pub fn days_since_install(&self, now: u64) -> u64 {
        (now.saturating_sub(self.installed_at)) / 86400
    }

    /// Get total uptime in hours
    pub fn total_uptime_hours(&self) -> f64 {
        self.total_uptime_secs as f64 / 3600.0
    }

    /// Get total uptime in days
    pub fn total_uptime_days(&self) -> f64 {
        self.total_uptime_secs as f64 / 86400.0
    }

    /// Get average session duration
    pub fn avg_session_duration(&self) -> f64 {
        if self.session_count == 0 {
            0.0
        } else {
            self.total_uptime_secs as f64 / self.session_count as f64
        }
    }

    /// Get uptime percentage since install
    pub fn uptime_percentage(&self, now: u64) -> f64 {
        let total_time = now.saturating_sub(self.installed_at);
        if total_time == 0 {
            0.0
        } else {
            self.total_uptime_secs as f64 / total_time as f64 * 100.0
        }
    }

    /// Get clean shutdown rate
    pub fn clean_shutdown_rate(&self) -> f64 {
        let total = self.clean_shutdowns + self.crashes;
        if total == 0 {
            100.0
        } else {
            self.clean_shutdowns as f64 / total as f64 * 100.0
        }
    }

    /// Check if currently running
    pub fn is_running(&self) -> bool {
        self.current_session_start.is_some()
    }

    /// Generate summary
    pub fn summary(&self, now: u64) -> UptimeSummary {
        UptimeSummary {
            days_installed: self.days_since_install(now),
            total_hours: self.total_uptime_hours(),
            sessions: self.session_count,
            uptime_pct: self.uptime_percentage(now),
            clean_rate: self.clean_shutdown_rate(),
            is_running: self.is_running(),
        }
    }
}

/// Uptime summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeSummary {
    /// Days since install
    pub days_installed: u64,
    /// Total uptime hours
    pub total_hours: f64,
    /// Session count
    pub sessions: u64,
    /// Uptime percentage
    pub uptime_pct: f64,
    /// Clean shutdown rate
    pub clean_rate: f64,
    /// Currently running
    pub is_running: bool,
}

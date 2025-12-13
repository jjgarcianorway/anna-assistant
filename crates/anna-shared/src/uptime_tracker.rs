//! Uptime Tracking (v0.0.492).
//!
//! Tracks Anna's uptime and availability.
//! Provides installation date tracking and session statistics.

use serde::{Deserialize, Serialize};

/// Uptime record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeRecord {
    /// Session start timestamp
    pub start_time: u64,
    /// Session end timestamp (None if still running)
    pub end_time: Option<u64>,
    /// Duration in seconds
    pub duration_secs: u64,
    /// Was this a clean shutdown
    pub clean_shutdown: bool,
}

impl UptimeRecord {
    /// Create new session record
    pub fn start(timestamp: u64) -> Self {
        Self {
            start_time: timestamp,
            end_time: None,
            duration_secs: 0,
            clean_shutdown: false,
        }
    }

    /// End the session
    pub fn end(&mut self, timestamp: u64, clean: bool) {
        self.end_time = Some(timestamp);
        self.duration_secs = timestamp.saturating_sub(self.start_time);
        self.clean_shutdown = clean;
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Get duration in hours
    pub fn duration_hours(&self) -> f64 {
        self.duration_secs as f64 / 3600.0
    }
}

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

impl UptimeTracker {
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

/// Format duration in seconds to human readable
pub fn format_duration_secs(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let s = secs % 60;
        format!("{}m {}s", mins, s)
    } else if secs < 86400 {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    } else {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Format uptime stats for display
pub fn format_uptime(tracker: &UptimeTracker, now: u64) -> String {
    let mut output = String::new();

    output.push_str("Uptime Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    output.push_str(&format!(
        "Installed: {} days ago\n",
        tracker.days_since_install(now)
    ));
    output.push_str(&format!(
        "Status: {}\n\n",
        if tracker.is_running() { "Running" } else { "Stopped" }
    ));

    if let Some(duration) = tracker.current_session_duration(now) {
        output.push_str(&format!(
            "Current Session: {}\n\n",
            format_duration_secs(duration)
        ));
    }

    output.push_str(&format!(
        "Total Uptime:    {}\n",
        format_duration_secs(tracker.total_uptime_secs)
    ));
    output.push_str(&format!(
        "Sessions:        {}\n",
        tracker.session_count
    ));
    output.push_str(&format!(
        "Avg Session:     {}\n",
        format_duration_secs(tracker.avg_session_duration() as u64)
    ));
    output.push_str(&format!(
        "Uptime Rate:     {:.1}%\n",
        tracker.uptime_percentage(now)
    ));
    output.push_str(&format!(
        "Clean Shutdowns: {:.1}%\n",
        tracker.clean_shutdown_rate()
    ));

    if tracker.longest_session_secs > 0 {
        output.push_str(&format!(
            "\nLongest Session: {}\n",
            format_duration_secs(tracker.longest_session_secs)
        ));
    }

    output
}

/// Format compact uptime info
pub fn format_uptime_compact(tracker: &UptimeTracker, now: u64) -> String {
    let status = if tracker.is_running() { "up" } else { "down" };

    format!(
        "{} for {}, {} sessions, {:.0}% uptime",
        status,
        format_duration_secs(tracker.current_session_duration(now).unwrap_or(0)),
        tracker.session_count,
        tracker.uptime_percentage(now)
    )
}

/// Format uptime as one-liner
pub fn format_uptime_oneline(tracker: &UptimeTracker, now: u64) -> String {
    if tracker.is_running() {
        format!(
            "Up {} | {} total",
            format_duration_secs(tracker.current_session_duration(now).unwrap_or(0)),
            format_duration_secs(tracker.total_uptime_secs)
        )
    } else {
        format!("Down | {} total uptime", format_duration_secs(tracker.total_uptime_secs))
    }
}

/// Generate fun fact about uptime
pub fn uptime_fun_fact(tracker: &UptimeTracker, now: u64) -> Option<String> {
    if tracker.session_count < 2 {
        return None;
    }

    let days = tracker.days_since_install(now);
    let facts = vec![
        format!(
            "Anna has been installed for {} days - that's {} weeks!",
            days,
            days / 7
        ),
        format!(
            "Total uptime is {} - Anna is dedicated!",
            format_duration_secs(tracker.total_uptime_secs)
        ),
        format!(
            "{:.0}% clean shutdowns - {}!",
            tracker.clean_shutdown_rate(),
            if tracker.clean_shutdown_rate() > 90.0 {
                "very reliable"
            } else {
                "room for improvement"
            }
        ),
        format!(
            "Average session lasts {} - {}!",
            format_duration_secs(tracker.avg_session_duration() as u64),
            if tracker.avg_session_duration() > 3600.0 {
                "marathon sessions"
            } else {
                "quick check-ins"
            }
        ),
    ];

    let index = (tracker.session_count as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about uptime
pub fn is_uptime_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "uptime",
        "how long",
        "running for",
        "installed",
        "since install",
        "session",
        "availability",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime_record_start() {
        let record = UptimeRecord::start(1000);
        assert!(record.is_active());
        assert_eq!(record.start_time, 1000);
    }

    #[test]
    fn test_uptime_record_end() {
        let mut record = UptimeRecord::start(1000);
        record.end(2000, true);

        assert!(!record.is_active());
        assert_eq!(record.duration_secs, 1000);
        assert!(record.clean_shutdown);
    }

    #[test]
    fn test_tracker_new() {
        let tracker = UptimeTracker::new(1000);
        assert_eq!(tracker.installed_at, 1000);
        assert!(!tracker.is_running());
    }

    #[test]
    fn test_start_session() {
        let mut tracker = UptimeTracker::new(1000);
        tracker.start_session(2000);

        assert!(tracker.is_running());
        assert_eq!(tracker.session_count, 1);
    }

    #[test]
    fn test_end_session() {
        let mut tracker = UptimeTracker::new(1000);
        tracker.start_session(2000);
        tracker.end_session(3000, true);

        assert!(!tracker.is_running());
        assert_eq!(tracker.total_uptime_secs, 1000);
        assert_eq!(tracker.clean_shutdowns, 1);
    }

    #[test]
    fn test_crash_tracking() {
        let mut tracker = UptimeTracker::new(1000);
        tracker.start_session(2000);
        tracker.end_session(3000, false);

        assert_eq!(tracker.crashes, 1);
        assert_eq!(tracker.clean_shutdown_rate(), 0.0);
    }

    #[test]
    fn test_days_since_install() {
        let tracker = UptimeTracker::new(0);
        // 2 days later
        assert_eq!(tracker.days_since_install(86400 * 2), 2);
    }

    #[test]
    fn test_uptime_percentage() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);
        tracker.end_session(50, true);

        // 50 secs uptime / 100 secs total = 50%
        assert_eq!(tracker.uptime_percentage(100), 50.0);
    }

    #[test]
    fn test_avg_session_duration() {
        let mut tracker = UptimeTracker::new(0);

        tracker.start_session(0);
        tracker.end_session(100, true);

        tracker.start_session(100);
        tracker.end_session(300, true);

        // (100 + 200) / 2 = 150
        assert_eq!(tracker.avg_session_duration(), 150.0);
    }

    #[test]
    fn test_longest_shortest() {
        let mut tracker = UptimeTracker::new(0);

        tracker.start_session(0);
        tracker.end_session(100, true);

        tracker.start_session(100);
        tracker.end_session(400, true);

        assert_eq!(tracker.longest_session_secs, 300);
        assert_eq!(tracker.shortest_session_secs, 100);
    }

    #[test]
    fn test_recent_sessions_limit() {
        let mut tracker = UptimeTracker::new(0);

        for i in 0..15 {
            tracker.start_session(i * 100);
            tracker.end_session(i * 100 + 50, true);
        }

        assert_eq!(tracker.recent_sessions.len(), 10);
    }

    #[test]
    fn test_format_duration_secs() {
        assert_eq!(format_duration_secs(30), "30s");
        assert_eq!(format_duration_secs(90), "1m 30s");
        assert_eq!(format_duration_secs(3700), "1h 1m");
        assert_eq!(format_duration_secs(90000), "1d 1h");
    }

    #[test]
    fn test_summary() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);
        tracker.end_session(3600, true);

        let summary = tracker.summary(7200);
        assert_eq!(summary.sessions, 1);
        assert!(summary.total_hours > 0.0);
    }

    #[test]
    fn test_format_compact() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);

        let output = format_uptime_compact(&tracker, 100);
        assert!(output.contains("up"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = UptimeTracker::new(0);
        tracker.start_session(0);
        tracker.end_session(100, true);
        tracker.start_session(100);
        tracker.end_session(200, true);

        let fact = uptime_fun_fact(&tracker, 86400 * 30);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_uptime_query() {
        assert!(is_uptime_query("show uptime"));
        assert!(is_uptime_query("how long running"));
        assert!(is_uptime_query("when installed"));

        assert!(!is_uptime_query("install vim"));
    }
}

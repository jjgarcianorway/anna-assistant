//! Idle Time Detector - Phase 85
//!
//! Detects when machine is idle for background research tasks.
//! VISION.md: "Investigate when machine is idle"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Idle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum IdleState {
    #[default]
    Active,
    Idle,
    DeepIdle,
    Suspended,
    Unknown,
}

impl IdleState {
    pub fn name(&self) -> &'static str {
        match self {
            IdleState::Active => "Active",
            IdleState::Idle => "Idle",
            IdleState::DeepIdle => "Deep Idle",
            IdleState::Suspended => "Suspended",
            IdleState::Unknown => "Unknown",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            IdleState::Active => "*",
            IdleState::Idle => "~",
            IdleState::DeepIdle => ".",
            IdleState::Suspended => "z",
            IdleState::Unknown => "?",
        }
    }

    pub fn allows_background_work(&self) -> bool {
        matches!(self, IdleState::Idle | IdleState::DeepIdle)
    }
}

/// System activity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityLevel {
    High,
    Medium,
    Low,
    Minimal,
}

impl ActivityLevel {
    pub fn name(&self) -> &'static str {
        match self {
            ActivityLevel::High => "High",
            ActivityLevel::Medium => "Medium",
            ActivityLevel::Low => "Low",
            ActivityLevel::Minimal => "Minimal",
        }
    }
}

/// Idle time configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleConfig {
    /// Seconds of inactivity before considered idle
    pub idle_threshold_secs: u64,
    /// Seconds before deep idle
    pub deep_idle_threshold_secs: u64,
    /// CPU usage threshold for idle (percent)
    pub cpu_idle_threshold: f32,
    /// Enable background work during idle
    pub enable_background_work: bool,
    /// Quiet hours (start, end in 24h format)
    pub quiet_hours: Option<(u8, u8)>,
}

impl Default for IdleConfig {
    fn default() -> Self {
        Self {
            idle_threshold_secs: 300,       // 5 minutes
            deep_idle_threshold_secs: 900,  // 15 minutes
            cpu_idle_threshold: 10.0,       // 10% CPU
            enable_background_work: true,
            quiet_hours: None,
        }
    }
}

/// An idle period record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlePeriod {
    /// Start timestamp
    pub start: u64,
    /// End timestamp (None if ongoing)
    pub end: Option<u64>,
    /// Peak idle state reached
    pub peak_state: IdleState,
    /// Background tasks completed during this period
    pub tasks_completed: u32,
}

impl IdlePeriod {
    /// Duration in seconds
    pub fn duration_secs(&self) -> u64 {
        match self.end {
            Some(end) => end.saturating_sub(self.start),
            None => 0,
        }
    }
}

/// Idle time tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdleTimeTracker {
    /// Configuration
    pub config: IdleConfig,
    /// Current state
    pub current_state: IdleState,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Idle period history
    pub periods: Vec<IdlePeriod>,
    /// Total idle time (seconds)
    pub total_idle_secs: u64,
    /// Background tasks completed
    pub tasks_completed: u32,
    /// Count by state
    pub by_state: HashMap<String, u64>,
}

impl IdleTimeTracker {
    pub fn new() -> Self {
        Self {
            config: IdleConfig::default(),
            current_state: IdleState::Active,
            ..Self::default()
        }
    }

    /// Record activity (resets idle timer)
    pub fn record_activity(&mut self, timestamp: u64) {
        if self.current_state != IdleState::Active {
            // End current idle period
            self.end_idle_period(timestamp);
        }
        self.last_activity = timestamp;
        self.current_state = IdleState::Active;
    }

    /// Check and update idle state
    pub fn check_idle(&mut self, current_time: u64) -> IdleState {
        let inactive_secs = current_time.saturating_sub(self.last_activity);

        let new_state = if inactive_secs >= self.config.deep_idle_threshold_secs {
            IdleState::DeepIdle
        } else if inactive_secs >= self.config.idle_threshold_secs {
            IdleState::Idle
        } else {
            IdleState::Active
        };

        // State transition
        if new_state != self.current_state {
            if self.current_state == IdleState::Active && new_state.allows_background_work() {
                // Starting idle period
                self.start_idle_period(current_time, new_state);
            } else if !self.current_state.allows_background_work() && new_state.allows_background_work() {
                // Transitioning to idle
                self.start_idle_period(current_time, new_state);
            }

            *self.by_state.entry(new_state.name().to_string()).or_insert(0) += 1;
            self.current_state = new_state;
        }

        // Update peak state if deeper
        if let Some(period) = self.periods.last_mut() {
            if period.end.is_none() && new_state == IdleState::DeepIdle {
                period.peak_state = IdleState::DeepIdle;
            }
        }

        new_state
    }

    fn start_idle_period(&mut self, timestamp: u64, state: IdleState) {
        self.periods.push(IdlePeriod {
            start: timestamp,
            end: None,
            peak_state: state,
            tasks_completed: 0,
        });
    }

    fn end_idle_period(&mut self, timestamp: u64) {
        if let Some(period) = self.periods.last_mut() {
            if period.end.is_none() {
                period.end = Some(timestamp);
                self.total_idle_secs += period.duration_secs();
            }
        }
    }

    /// Record background task completion
    pub fn record_task_completed(&mut self) {
        self.tasks_completed += 1;
        if let Some(period) = self.periods.last_mut() {
            if period.end.is_none() {
                period.tasks_completed += 1;
            }
        }
    }

    /// Check if background work is allowed
    pub fn can_do_background_work(&self) -> bool {
        self.config.enable_background_work && self.current_state.allows_background_work()
    }

    /// Check if in quiet hours
    pub fn is_quiet_hours(&self, hour: u8) -> bool {
        match self.config.quiet_hours {
            Some((start, end)) => {
                if start <= end {
                    hour >= start && hour < end
                } else {
                    hour >= start || hour < end
                }
            }
            None => false,
        }
    }

    /// Get current idle duration (seconds)
    pub fn current_idle_duration(&self, current_time: u64) -> u64 {
        if self.current_state.allows_background_work() {
            current_time.saturating_sub(self.last_activity)
        } else {
            0
        }
    }

    /// Get recent idle periods
    pub fn recent_periods(&self, limit: usize) -> Vec<&IdlePeriod> {
        self.periods.iter().rev().take(limit).collect()
    }

    /// Get completed idle periods
    pub fn completed_periods(&self) -> Vec<&IdlePeriod> {
        self.periods.iter().filter(|p| p.end.is_some()).collect()
    }

    /// Average idle period duration
    pub fn avg_idle_duration(&self) -> f64 {
        let completed: Vec<_> = self.completed_periods();
        if completed.is_empty() {
            return 0.0;
        }
        let total: u64 = completed.iter().map(|p| p.duration_secs()).sum();
        total as f64 / completed.len() as f64
    }

    /// Longest idle period
    pub fn longest_idle(&self) -> Option<u64> {
        self.completed_periods()
            .iter()
            .map(|p| p.duration_secs())
            .max()
    }

    /// Total period count
    pub fn period_count(&self) -> usize {
        self.periods.len()
    }
}

/// Format idle tracker for display
pub fn format_idle_tracker(tracker: &IdleTimeTracker) -> String {
    let mut lines = vec!["=== Idle Time Tracker ===".to_string()];
    lines.push(String::new());

    // Current state
    lines.push(format!("Current state: {} [{}]",
        tracker.current_state.name(),
        tracker.current_state.symbol()
    ));

    lines.push(format!("Background work: {}",
        if tracker.can_do_background_work() { "allowed" } else { "not allowed" }
    ));

    // Stats
    lines.push(String::new());
    lines.push(format!("Total idle time: {} min", tracker.total_idle_secs / 60));
    lines.push(format!("Idle periods: {}", tracker.period_count()));
    lines.push(format!("Avg idle duration: {:.1} min", tracker.avg_idle_duration() / 60.0));
    lines.push(format!("Tasks completed: {}", tracker.tasks_completed));

    // Longest
    if let Some(longest) = tracker.longest_idle() {
        lines.push(format!("Longest idle: {} min", longest / 60));
    }

    // Config
    lines.push(String::new());
    lines.push(format!("Idle threshold: {} sec", tracker.config.idle_threshold_secs));
    lines.push(format!("Deep idle: {} sec", tracker.config.deep_idle_threshold_secs));

    lines.join("\n")
}

/// Format idle tracker compact
pub fn format_idle_tracker_compact(tracker: &IdleTimeTracker) -> String {
    format!(
        "Idle: {} | {} min total | {} tasks",
        tracker.current_state.name(),
        tracker.total_idle_secs / 60,
        tracker.tasks_completed
    )
}

/// Format idle tracker one-line
pub fn format_idle_tracker_oneline(tracker: &IdleTimeTracker) -> String {
    format!(
        "{} ({} periods)",
        tracker.current_state.name(),
        tracker.period_count()
    )
}

/// Check if query is about idle time
pub fn is_idle_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "idle time",
        "when idle",
        "machine idle",
        "system idle",
        "background work",
        "background task",
        "idle period",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about idle time
pub fn idle_fun_fact(tracker: &IdleTimeTracker) -> String {
    if tracker.periods.is_empty() {
        return "No idle periods recorded yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has tracked {} idle periods.",
            tracker.period_count()
        ),
        format!(
            "Total idle time: {} minutes.",
            tracker.total_idle_secs / 60
        ),
        format!(
            "{} background tasks completed during idle.",
            tracker.tasks_completed
        ),
        format!(
            "Average idle period: {:.1} minutes.",
            tracker.avg_idle_duration() / 60.0
        ),
        {
            if let Some(longest) = tracker.longest_idle() {
                format!("Longest idle period: {} minutes.", longest / 60)
            } else {
                "No completed idle periods yet.".to_string()
            }
        },
    ];

    facts[tracker.period_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_state() {
        assert_eq!(IdleState::Active.name(), "Active");
        assert_eq!(IdleState::Idle.symbol(), "~");
        assert!(IdleState::Idle.allows_background_work());
        assert!(!IdleState::Active.allows_background_work());
    }

    #[test]
    fn test_activity_level() {
        assert_eq!(ActivityLevel::High.name(), "High");
        assert_eq!(ActivityLevel::Minimal.name(), "Minimal");
    }

    #[test]
    fn test_idle_config_default() {
        let config = IdleConfig::default();
        assert_eq!(config.idle_threshold_secs, 300);
        assert_eq!(config.deep_idle_threshold_secs, 900);
    }

    #[test]
    fn test_idle_tracker_new() {
        let tracker = IdleTimeTracker::new();
        assert_eq!(tracker.current_state, IdleState::Active);
        assert_eq!(tracker.tasks_completed, 0);
    }

    #[test]
    fn test_record_activity() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);
        assert_eq!(tracker.last_activity, 1000);
        assert_eq!(tracker.current_state, IdleState::Active);
    }

    #[test]
    fn test_check_idle_transition() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);

        // Not enough time passed
        let state = tracker.check_idle(1100);
        assert_eq!(state, IdleState::Active);

        // Idle threshold passed (300 sec)
        let state = tracker.check_idle(1400);
        assert_eq!(state, IdleState::Idle);
        assert_eq!(tracker.period_count(), 1);

        // Deep idle threshold passed (900 sec)
        let state = tracker.check_idle(2000);
        assert_eq!(state, IdleState::DeepIdle);
    }

    #[test]
    fn test_can_do_background_work() {
        let mut tracker = IdleTimeTracker::new();
        assert!(!tracker.can_do_background_work());

        tracker.current_state = IdleState::Idle;
        assert!(tracker.can_do_background_work());

        tracker.config.enable_background_work = false;
        assert!(!tracker.can_do_background_work());
    }

    #[test]
    fn test_quiet_hours() {
        let mut tracker = IdleTimeTracker::new();
        tracker.config.quiet_hours = Some((22, 6));

        assert!(tracker.is_quiet_hours(23));
        assert!(tracker.is_quiet_hours(2));
        assert!(!tracker.is_quiet_hours(12));
    }

    #[test]
    fn test_record_task_completed() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);
        tracker.check_idle(1400); // Go idle

        tracker.record_task_completed();
        assert_eq!(tracker.tasks_completed, 1);
        assert_eq!(tracker.periods.last().unwrap().tasks_completed, 1);
    }

    #[test]
    fn test_format_idle_tracker() {
        let tracker = IdleTimeTracker::new();
        let output = format_idle_tracker(&tracker);
        assert!(output.contains("Idle Time Tracker"));
        assert!(output.contains("Active"));
    }

    #[test]
    fn test_is_idle_query() {
        assert!(is_idle_query("when is the machine idle?"));
        assert!(is_idle_query("show idle time"));
        assert!(is_idle_query("background work status"));
        assert!(!is_idle_query("what is the weather?"));
    }

    #[test]
    fn test_idle_fun_fact() {
        let mut tracker = IdleTimeTracker::new();
        tracker.record_activity(1000);
        tracker.check_idle(1400);

        let fact = idle_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }
}

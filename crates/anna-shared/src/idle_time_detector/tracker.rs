//! Idle time tracker implementation

use super::types::{IdleConfig, IdlePeriod, IdleState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

//! Types for idle time detection

use serde::{Deserialize, Serialize};

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

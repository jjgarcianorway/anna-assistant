//! Boot Time Tracking Types
//!
//! Data structures for tracking boot times and analyzing trends.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Boot time record for a single boot event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootRecord {
    /// Timestamp when boot completed (Unix timestamp)
    pub timestamp: u64,
    /// Total boot time in seconds
    pub boot_time_secs: f64,
    /// Breakdown by service (service name -> time in ms)
    pub service_times: HashMap<String, u64>,
    /// Kernel boot time in seconds
    pub kernel_time_secs: f64,
    /// Userspace boot time in seconds
    pub userspace_time_secs: f64,
    /// Any notable services that slowed boot
    pub slow_services: Vec<SlowService>,
}

/// A service that contributed significantly to boot time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowService {
    pub name: String,
    pub time_ms: u64,
    pub reason: Option<String>,
    pub is_necessary: bool,
}

/// Trend direction for boot time changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootTrend {
    Faster,
    Slower,
    Stable,
}

impl BootTrend {
    pub fn symbol(&self) -> &'static str {
        match self {
            BootTrend::Faster => "v",
            BootTrend::Slower => "^",
            BootTrend::Stable => "-",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BootTrend::Faster => "getting faster",
            BootTrend::Slower => "getting slower",
            BootTrend::Stable => "stable",
        }
    }
}

/// Boot time statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootTimeTracker {
    /// All recorded boot times
    pub records: Vec<BootRecord>,
    /// Fastest boot time ever recorded (seconds)
    pub fastest_boot_secs: Option<f64>,
    /// Slowest boot time ever recorded (seconds)
    pub slowest_boot_secs: Option<f64>,
    /// Services that consistently slow boot
    pub problem_services: HashMap<String, u32>,
}

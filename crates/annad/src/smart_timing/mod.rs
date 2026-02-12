//! Smart Timing - Know WHEN to do things based on system usage patterns.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc, Timelike, Datelike};

const USAGE_PATTERNS_FILE: &str = "/var/lib/anna/usage_patterns.json";

/// System usage patterns learned over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    pub hourly_activity: [f32; 24], // Average activity level for each hour (0.0-1.0)
    pub daily_activity: [f32; 7],   // Average activity for each day of week (0.0-1.0)
    pub sample_count: u32,
}

impl Default for UsagePatterns {
    fn default() -> Self {
        Self {
            hourly_activity: [0.5; 24], // Start with assumption of moderate activity
            daily_activity: [0.5; 7],
            sample_count: 0,
        }
    }
}

impl UsagePatterns {
    /// Load usage patterns from disk
    pub fn load() -> Self {
        let path = PathBuf::from(USAGE_PATTERNS_FILE);
        if !path.exists() {
            return Self::default();
        }

        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save usage patterns to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = PathBuf::from(USAGE_PATTERNS_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Record current system activity
    pub fn record_activity(&mut self, activity_level: f32) {
        let now = chrono::Local::now();
        let hour = now.hour() as usize;
        let day = now.weekday().num_days_from_monday() as usize;

        // Update with exponential moving average
        let alpha = 0.1;
        self.hourly_activity[hour] = self.hourly_activity[hour] * (1.0 - alpha) + activity_level * alpha;
        self.daily_activity[day] = self.daily_activity[day] * (1.0 - alpha) + activity_level * alpha;

        self.sample_count += 1;
    }

    /// Get predicted activity level at a specific time
    pub fn predict_activity(&self, time: &DateTime<Utc>) -> f32 {
        let hour = time.hour() as usize;
        let day = time.weekday().num_days_from_monday() as usize;

        // Combine hourly and daily patterns
        (self.hourly_activity[hour] + self.daily_activity[day]) / 2.0
    }

    /// Find the best time window for maintenance operations
    pub fn find_maintenance_window(&self, duration_hours: u32) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        let mut best_time = now;
        let mut lowest_activity = 1.0;

        // Look ahead 7 days
        for day_offset in 0..7 {
            for hour in 0..24 {
                let test_time = now + chrono::Duration::hours((day_offset * 24 + hour) as i64);

                // Calculate average activity for the duration
                let mut avg_activity = 0.0;
                for h in 0..duration_hours {
                    let check_time = test_time + chrono::Duration::hours(h as i64);
                    avg_activity += self.predict_activity(&check_time);
                }
                avg_activity /= duration_hours as f32;

                if avg_activity < lowest_activity {
                    lowest_activity = avg_activity;
                    best_time = test_time;
                }
            }
        }

        Some(best_time)
    }

    /// Check if now is a good time for a disruptive operation
    pub fn is_good_time_for(&self, operation: OperationType) -> bool {
        let now = Utc::now();
        let current_activity = self.predict_activity(&now);

        match operation {
            OperationType::ServiceRestart => current_activity < 0.4,
            OperationType::SystemUpdate => current_activity < 0.3,
            OperationType::HeavyMaintenance => current_activity < 0.2,
            OperationType::MinorCleanup => current_activity < 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OperationType {
    ServiceRestart,
    SystemUpdate,
    HeavyMaintenance,
    MinorCleanup,
}

/// Measure current system activity level
pub fn measure_current_activity() -> f32 {
    let mut activity = 0.0;

    // Factor 1: Load average (0.0 to 1.0, capped at CPU count)
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load_1min) = load.split_whitespace().next() {
            if let Ok(load_val) = load_1min.parse::<f32>() {
                let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
                let cpu_count = cpuinfo.lines().filter(|l| l.starts_with("processor")).count().max(1) as f32;
                activity += (load_val / cpu_count).min(1.0) * 0.4; // 40% weight
            }
        }
    }

    // Factor 2: Memory usage (0.0 to 1.0)
    if let Ok(output) = std::process::Command::new("free").output() {
        let mem_info = String::from_utf8_lossy(&output.stdout);
        for line in mem_info.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>()) {
                        activity += (used / total) * 0.3; // 30% weight
                    }
                }
            }
        }
    }

    // Factor 3: Number of active processes
    if let Ok(entries) = std::fs::read_dir("/proc") {
        let process_count = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().chars().all(|c| c.is_ascii_digit()))
            .count();

        // Normalize: 100 processes = 0.5 activity
        activity += (process_count as f32 / 200.0).min(1.0) * 0.3; // 30% weight
    }

    activity.min(1.0)
}

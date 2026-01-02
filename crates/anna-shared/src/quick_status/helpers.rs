//! Helper functions for creating common status items.

use super::types::{HealthLevel, StatusItem};

/// Create status from memory usage percentage
pub fn memory_status(used_percent: f32) -> StatusItem {
    let health = if used_percent > 90.0 {
        HealthLevel::Critical
    } else if used_percent > 75.0 {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if used_percent > 90.0 {
        "Memory critically low"
    } else if used_percent > 75.0 {
        "Memory usage high"
    } else {
        "Memory OK"
    };

    StatusItem::new("Memory", health, message).with_value(&format!("{:.0}%", used_percent))
}

/// Create status from disk usage percentage
pub fn disk_status(used_percent: f32, mount_point: &str) -> StatusItem {
    let health = if used_percent > 95.0 {
        HealthLevel::Critical
    } else if used_percent > 85.0 {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if used_percent > 95.0 {
        format!("{} almost full", mount_point)
    } else if used_percent > 85.0 {
        format!("{} getting full", mount_point)
    } else {
        format!("{} OK", mount_point)
    };

    StatusItem::new("Disk", health, &message).with_value(&format!("{:.0}%", used_percent))
}

/// Create status from CPU load
pub fn cpu_status(load_1min: f32, core_count: u32) -> StatusItem {
    let load_per_core = load_1min / core_count as f32;

    let health = if load_per_core > 2.0 {
        HealthLevel::Critical
    } else if load_per_core > 1.0 {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if load_per_core > 2.0 {
        "CPU overloaded"
    } else if load_per_core > 1.0 {
        "CPU busy"
    } else {
        "CPU OK"
    };

    StatusItem::new("CPU", health, message).with_value(&format!("{:.1}", load_1min))
}

/// Create status from service state
pub fn service_status(name: &str, running: bool, failed: bool) -> StatusItem {
    let health = if failed {
        HealthLevel::Critical
    } else if !running {
        HealthLevel::Warning
    } else {
        HealthLevel::Good
    };

    let message = if failed {
        format!("{} failed", name)
    } else if !running {
        format!("{} stopped", name)
    } else {
        format!("{} running", name)
    };

    StatusItem::new("Service", health, &message)
}

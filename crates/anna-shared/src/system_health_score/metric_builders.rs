//! Metric builder functions for different system components

use super::types::{HealthCategory, HealthMetric};

/// Create CPU health metric from usage percentage
pub fn cpu_health(usage_percent: f64) -> HealthMetric {
    let score = if usage_percent < 50.0 {
        100
    } else if usage_percent < 70.0 {
        85
    } else if usage_percent < 85.0 {
        70
    } else if usage_percent < 95.0 {
        55
    } else {
        30
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Cpu,
        score,
        format!("{:.1}% used", usage_percent),
    );

    if score < 70 {
        metric = metric.with_recommendation("High CPU usage. Check running processes.");
    }

    metric
}

/// Create memory health metric from usage percentage
pub fn memory_health(usage_percent: f64) -> HealthMetric {
    let score = if usage_percent < 60.0 {
        100
    } else if usage_percent < 75.0 {
        85
    } else if usage_percent < 85.0 {
        70
    } else if usage_percent < 95.0 {
        50
    } else {
        25
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Memory,
        score,
        format!("{:.1}% used", usage_percent),
    );

    if score < 70 {
        metric = metric.with_recommendation("High memory usage. Consider closing applications.");
    }

    metric
}

/// Create disk health metric from usage percentage
pub fn disk_health(usage_percent: f64) -> HealthMetric {
    let score = if usage_percent < 70.0 {
        100
    } else if usage_percent < 80.0 {
        85
    } else if usage_percent < 90.0 {
        65
    } else if usage_percent < 95.0 {
        40
    } else {
        20
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Disk,
        score,
        format!("{:.1}% used", usage_percent),
    );

    if score < 70 {
        metric = metric.with_recommendation("Low disk space. Clean up or expand storage.");
    }

    metric
}

/// Create services health metric
pub fn services_health(failed_count: u32, total_count: u32) -> HealthMetric {
    let score = if failed_count == 0 {
        100
    } else if failed_count == 1 {
        80
    } else if failed_count <= 3 {
        60
    } else {
        40
    };

    let mut metric = HealthMetric::new(
        HealthCategory::Services,
        score,
        format!("{}/{} running", total_count - failed_count, total_count),
    );

    if failed_count > 0 {
        metric = metric.with_recommendation(format!(
            "{} service{} failed. Check systemctl status.",
            failed_count,
            if failed_count == 1 { "" } else { "s" }
        ));
    }

    metric
}

/// Create network health metric
pub fn network_health(connected: bool, latency_ms: Option<u32>) -> HealthMetric {
    let score = if !connected {
        0
    } else if let Some(latency) = latency_ms {
        if latency < 50 {
            100
        } else if latency < 100 {
            90
        } else if latency < 200 {
            75
        } else {
            60
        }
    } else {
        85 // Connected but no latency data
    };

    let value = if !connected {
        "Disconnected".to_string()
    } else if let Some(latency) = latency_ms {
        format!("{}ms latency", latency)
    } else {
        "Connected".to_string()
    };

    let mut metric = HealthMetric::new(HealthCategory::Network, score, value);

    if !connected {
        metric = metric.with_recommendation("No network connection. Check connectivity.");
    }

    metric
}

/// Create daemon health metric
pub fn daemon_health(running: bool, uptime_hours: Option<u64>) -> HealthMetric {
    let score = if !running {
        0
    } else if let Some(hours) = uptime_hours {
        if hours > 24 {
            100
        } else if hours > 1 {
            90
        } else {
            80
        }
    } else {
        90
    };

    let value = if !running {
        "Not running".to_string()
    } else if let Some(hours) = uptime_hours {
        format!("{}h uptime", hours)
    } else {
        "Running".to_string()
    };

    let mut metric = HealthMetric::new(HealthCategory::Daemon, score, value);

    if !running {
        metric = metric.with_recommendation("Anna daemon not running. Start with systemctl.");
    }

    metric
}

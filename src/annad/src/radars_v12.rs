// Anna v0.12.0 - Radar Scoring System
// Three radars: System Health, Usage Habit, Network Posture

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Radar score (0-10 scale)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarScore {
    pub category: String,
    pub score: Option<f64>, // None if metric unavailable
    pub max: f64,
    pub description: String,
}

/// Complete radar classification result
#[derive(Debug, Serialize, Deserialize)]
pub struct UserClassification {
    pub uid: u32,
    pub username: String,
    pub radars: HashMap<String, RadarResult>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RadarResult {
    pub name: String,
    pub categories: HashMap<String, RadarScore>,
    pub overall_score: f64,
}

/// System Health Radar
/// Categories: cpu_load, memory_pressure, disk_headroom, thermal_ok
pub fn score_system_health(
    load_avg_1m: f64,
    num_cores: u32,
    mem_free_pct: f64,
    root_free_pct: f64,
    cpu_temp_c: Option<f64>,
) -> RadarResult {
    let mut categories = HashMap::new();

    // CPU Load: 10 at < 0.5 per core, linear to 0 at >= 2.0 per core
    let load_per_core = load_avg_1m / num_cores as f64;
    let cpu_score = if load_per_core < 0.5 {
        10.0
    } else if load_per_core >= 2.0 {
        0.0
    } else {
        10.0 - (load_per_core - 0.5) / 1.5 * 10.0
    };

    categories.insert(
        "cpu_load".to_string(),
        RadarScore {
            category: "cpu_load".to_string(),
            score: Some(cpu_score),
            max: 10.0,
            description: format!(
                "Load {:.2} per core ({}  cores)",
                load_per_core, num_cores
            ),
        },
    );

    // Memory Pressure: 10 at >= 40% free, linear to 0 at <= 5% free
    let mem_score = if mem_free_pct >= 40.0 {
        10.0
    } else if mem_free_pct <= 5.0 {
        0.0
    } else {
        (mem_free_pct - 5.0) / 35.0 * 10.0
    };

    categories.insert(
        "memory_pressure".to_string(),
        RadarScore {
            category: "memory_pressure".to_string(),
            score: Some(mem_score),
            max: 10.0,
            description: format!("{:.1}% memory free", mem_free_pct),
        },
    );

    // Disk Headroom: 10 at >= 30% free, linear to 0 at <= 5% free
    let disk_score = if root_free_pct >= 30.0 {
        10.0
    } else if root_free_pct <= 5.0 {
        0.0
    } else {
        (root_free_pct - 5.0) / 25.0 * 10.0
    };

    categories.insert(
        "disk_headroom".to_string(),
        RadarScore {
            category: "disk_headroom".to_string(),
            score: Some(disk_score),
            max: 10.0,
            description: format!("{:.1}% disk free on /", root_free_pct),
        },
    );

    // Thermal: 10 at <= 70C, linear to 0 at >= 90C
    let thermal_score = if let Some(temp) = cpu_temp_c {
        if temp <= 70.0 {
            10.0
        } else if temp >= 90.0 {
            0.0
        } else {
            10.0 - (temp - 70.0) / 20.0 * 10.0
        }
    } else {
        5.0 // Unknown = neutral
    };

    categories.insert(
        "thermal_ok".to_string(),
        RadarScore {
            category: "thermal_ok".to_string(),
            score: cpu_temp_c.map(|_| thermal_score),
            max: 10.0,
            description: if let Some(temp) = cpu_temp_c {
                format!("CPU temp {:.0}°C", temp)
            } else {
                "Temp sensor unavailable".to_string()
            },
        },
    );

    let overall_score = categories
        .values()
        .filter_map(|s| s.score)
        .sum::<f64>()
        / categories.values().filter(|s| s.score.is_some()).count() as f64;

    RadarResult {
        name: "System Health".to_string(),
        categories,
        overall_score,
    }
}

/// Usage Habit Radar (per user)
/// Categories: interactive_time, cpu_bursty, work_window_regular
pub fn score_usage_habit(
    interactive_hours_24h: f64,
    cpu_variance: Option<f64>, // Variance of per-minute CPU share
    login_time_stddev_hours: Option<f64>, // Std dev of login times over 7d
) -> RadarResult {
    let mut categories = HashMap::new();

    // Interactive Time: 10 at >= 2h, linear to 0 at 0h
    let interactive_score = if interactive_hours_24h >= 2.0 {
        10.0
    } else {
        interactive_hours_24h / 2.0 * 10.0
    };

    categories.insert(
        "interactive_time".to_string(),
        RadarScore {
            category: "interactive_time".to_string(),
            score: Some(interactive_score),
            max: 10.0,
            description: format!("{:.1}h interactive in last 24h", interactive_hours_24h),
        },
    );

    // CPU Bursty: Inverse of variance (smoother = higher score)
    // Variance 0.0 = 10, variance 1.0 = 0
    let bursty_score = cpu_variance.map(|var| {
        if var <= 0.0 {
            10.0
        } else if var >= 1.0 {
            0.0
        } else {
            10.0 - var * 10.0
        }
    });

    categories.insert(
        "cpu_bursty".to_string(),
        RadarScore {
            category: "cpu_bursty".to_string(),
            score: bursty_score,
            max: 10.0,
            description: if let Some(var) = cpu_variance {
                format!("CPU variance {:.3}", var)
            } else {
                "Insufficient data".to_string()
            },
        },
    );

    // Work Window Regular: Lower stddev = higher score
    // Stddev 0h = 10, stddev >= 4h = 0
    let regular_score = login_time_stddev_hours.map(|stddev| {
        if stddev <= 0.0 {
            10.0
        } else if stddev >= 4.0 {
            0.0
        } else {
            10.0 - stddev / 4.0 * 10.0
        }
    });

    categories.insert(
        "work_window_regular".to_string(),
        RadarScore {
            category: "work_window_regular".to_string(),
            score: regular_score,
            max: 10.0,
            description: if let Some(stddev) = login_time_stddev_hours {
                format!("Login time stddev {:.1}h", stddev)
            } else {
                "Insufficient data".to_string()
            },
        },
    );

    let overall_score = categories
        .values()
        .filter_map(|s| s.score)
        .sum::<f64>()
        / categories.values().filter(|s| s.score.is_some()).count().max(1) as f64;

    RadarResult {
        name: "Usage Habit".to_string(),
        categories,
        overall_score,
    }
}

/// Network Posture Radar
/// Categories: latency, loss, dns_reliability
pub fn score_network_posture(
    ping_latency_ms: Option<f64>,
    ping_loss_pct: Option<f64>,
    dns_success: bool,
) -> RadarResult {
    let mut categories = HashMap::new();

    // Latency: 10 at <= 20ms, linear to 0 at >= 250ms
    let latency_score = ping_latency_ms.map(|lat| {
        if lat <= 20.0 {
            10.0
        } else if lat >= 250.0 {
            0.0
        } else {
            10.0 - (lat - 20.0) / 230.0 * 10.0
        }
    });

    categories.insert(
        "latency".to_string(),
        RadarScore {
            category: "latency".to_string(),
            score: latency_score,
            max: 10.0,
            description: if let Some(lat) = ping_latency_ms {
                format!("{:.1}ms ping latency", lat)
            } else {
                "Ping unavailable".to_string()
            },
        },
    );

    // Loss: 10 at 0%, linear to 0 at >= 10%
    let loss_score = ping_loss_pct.map(|loss| {
        if loss <= 0.0 {
            10.0
        } else if loss >= 10.0 {
            0.0
        } else {
            10.0 - loss
        }
    });

    categories.insert(
        "loss".to_string(),
        RadarScore {
            category: "loss".to_string(),
            score: loss_score,
            max: 10.0,
            description: if let Some(loss) = ping_loss_pct {
                format!("{:.1}% packet loss", loss)
            } else {
                "Loss check unavailable".to_string()
            },
        },
    );

    // DNS Reliability: Binary score
    let dns_score = if dns_success { 10.0 } else { 0.0 };

    categories.insert(
        "dns_reliability".to_string(),
        RadarScore {
            category: "dns_reliability".to_string(),
            score: Some(dns_score),
            max: 10.0,
            description: format!("DNS: {}", if dns_success { "OK" } else { "Failed" }),
        },
    );

    let overall_score = categories
        .values()
        .filter_map(|s| s.score)
        .sum::<f64>()
        / categories.values().filter(|s| s.score.is_some()).count().max(1) as f64;

    RadarResult {
        name: "Network Posture".to_string(),
        categories,
        overall_score,
    }
}

/// Compute full user classification
pub fn classify_user(
    uid: u32,
    username: String,
    // System health inputs
    load_avg_1m: f64,
    num_cores: u32,
    mem_free_pct: f64,
    root_free_pct: f64,
    cpu_temp_c: Option<f64>,
    // Usage habit inputs
    interactive_hours_24h: f64,
    cpu_variance: Option<f64>,
    login_time_stddev_hours: Option<f64>,
    // Network posture inputs
    ping_latency_ms: Option<f64>,
    ping_loss_pct: Option<f64>,
    dns_success: bool,
) -> UserClassification {
    let mut radars = HashMap::new();

    radars.insert(
        "system_health".to_string(),
        score_system_health(load_avg_1m, num_cores, mem_free_pct, root_free_pct, cpu_temp_c),
    );

    radars.insert(
        "usage_habit".to_string(),
        score_usage_habit(interactive_hours_24h, cpu_variance, login_time_stddev_hours),
    );

    radars.insert(
        "network_posture".to_string(),
        score_network_posture(ping_latency_ms, ping_loss_pct, dns_success),
    );

    UserClassification {
        uid,
        username,
        radars,
        timestamp: chrono::Utc::now().timestamp(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_health_ideal() {
        let result = score_system_health(0.4, 8, 50.0, 40.0, Some(65.0));
        assert_eq!(result.categories.len(), 4);
        assert!(result.overall_score > 9.0);
    }

    #[test]
    fn test_system_health_degraded() {
        let result = score_system_health(16.0, 8, 10.0, 8.0, Some(85.0));
        assert!(result.overall_score < 5.0);
    }

    #[test]
    fn test_usage_habit_active_user() {
        let result = score_usage_habit(3.0, Some(0.1), Some(0.5));
        assert!(result.overall_score > 8.0);
    }

    #[test]
    fn test_network_posture_good() {
        let result = score_network_posture(Some(15.0), Some(0.0), true);
        assert!(result.overall_score > 9.0);
    }

    #[test]
    fn test_network_posture_poor() {
        let result = score_network_posture(Some(300.0), Some(15.0), false);
        assert!(result.overall_score < 2.0);
    }

    #[test]
    fn test_missing_metrics() {
        let result = score_usage_habit(1.5, None, None);
        // Should still return a result with available data
        assert!(result.overall_score > 0.0);
        assert_eq!(result.categories["cpu_bursty"].score, None);
    }
}

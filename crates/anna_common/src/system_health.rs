use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// System health monitoring (load averages, daemon crashes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub load_averages: LoadAverages,
    pub daemon_crashes: DaemonCrashes,
    pub system_uptime: SystemUptime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverages {
    pub one_minute: f64,
    pub five_minutes: f64,
    pub fifteen_minutes: f64,
    pub cpu_cores: usize,
    pub load_per_core_1min: f64,
    pub load_per_core_5min: f64,
    pub load_per_core_15min: f64,
    pub status_1min: LoadStatus,
    pub status_5min: LoadStatus,
    pub status_15min: LoadStatus,
    pub overall_status: LoadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadStatus {
    Low,      // < 0.7 per core
    Moderate, // 0.7-1.0 per core
    High,     // 1.0-2.0 per core
    Critical, // > 2.0 per core
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonCrashes {
    pub total_crashes_24h: u32,
    pub total_crashes_7d: u32,
    pub crashed_services: Vec<CrashedService>,
    pub recent_crashes: Vec<CrashEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashedService {
    pub service_name: String,
    pub crash_count_24h: u32,
    pub crash_count_7d: u32,
    pub last_crash_time: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashEvent {
    pub timestamp: DateTime<Utc>,
    pub service_name: String,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUptime {
    pub uptime_seconds: u64,
    pub uptime_days: f64,
    pub boot_time: Option<DateTime<Utc>>,
}

impl SystemHealth {
    /// Detect system health metrics
    pub fn detect() -> Self {
        let load_averages = detect_load_averages();
        let daemon_crashes = detect_daemon_crashes();
        let system_uptime = detect_system_uptime();

        Self {
            load_averages,
            daemon_crashes,
            system_uptime,
        }
    }
}

fn detect_load_averages() -> LoadAverages {
    // Read load averages from /proc/loadavg
    let (one_minute, five_minutes, fifteen_minutes) =
        if let Ok(content) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 3 {
                let one = parts[0].parse::<f64>().unwrap_or(0.0);
                let five = parts[1].parse::<f64>().unwrap_or(0.0);
                let fifteen = parts[2].parse::<f64>().unwrap_or(0.0);
                (one, five, fifteen)
            } else {
                (0.0, 0.0, 0.0)
            }
        } else {
            (0.0, 0.0, 0.0)
        };

    // Get CPU core count
    let cpu_cores = num_cpus::get();

    // Calculate load per core
    let load_per_core_1min = if cpu_cores > 0 {
        one_minute / cpu_cores as f64
    } else {
        one_minute
    };

    let load_per_core_5min = if cpu_cores > 0 {
        five_minutes / cpu_cores as f64
    } else {
        five_minutes
    };

    let load_per_core_15min = if cpu_cores > 0 {
        fifteen_minutes / cpu_cores as f64
    } else {
        fifteen_minutes
    };

    // Categorize load status
    let status_1min = categorize_load(load_per_core_1min);
    let status_5min = categorize_load(load_per_core_5min);
    let status_15min = categorize_load(load_per_core_15min);

    // Overall status is the worst of the three
    let overall_status = [&status_1min, &status_5min, &status_15min]
        .iter()
        .max()
        .cloned()
        .cloned()
        .unwrap_or(LoadStatus::Low);

    LoadAverages {
        one_minute,
        five_minutes,
        fifteen_minutes,
        cpu_cores,
        load_per_core_1min,
        load_per_core_5min,
        load_per_core_15min,
        status_1min,
        status_5min,
        status_15min,
        overall_status,
    }
}

fn categorize_load(load_per_core: f64) -> LoadStatus {
    if load_per_core < 0.7 {
        LoadStatus::Low
    } else if load_per_core < 1.0 {
        LoadStatus::Moderate
    } else if load_per_core < 2.0 {
        LoadStatus::High
    } else {
        LoadStatus::Critical
    }
}

fn detect_daemon_crashes() -> DaemonCrashes {
    let mut crashed_services: Vec<CrashedService> = Vec::new();
    let mut recent_crashes: Vec<CrashEvent> = Vec::new();

    // Query journalctl for service failures in the last 7 days
    // Look for systemd service failures
    let output_7d = Command::new("journalctl")
        .args(&[
            "--since",
            "7 days ago",
            "--unit=*.service",
            "--grep=Failed with result",
            "--no-pager",
            "--output=json",
        ])
        .output();

    let output_24h = Command::new("journalctl")
        .args(&[
            "--since",
            "24 hours ago",
            "--unit=*.service",
            "--grep=Failed with result",
            "--no-pager",
            "--output=json",
        ])
        .output();

    // Parse crash events from journalctl
    let crashes_7d = parse_crash_events(&output_7d);
    let crashes_24h = parse_crash_events(&output_24h);

    let total_crashes_7d = crashes_7d.len() as u32;
    let total_crashes_24h = crashes_24h.len() as u32;

    // Group by service name
    let mut service_crashes: std::collections::HashMap<String, Vec<CrashEvent>> =
        std::collections::HashMap::new();

    for crash in &crashes_7d {
        service_crashes
            .entry(crash.service_name.clone())
            .or_insert_with(Vec::new)
            .push(crash.clone());
    }

    // Build CrashedService structs
    for (service_name, crashes) in &service_crashes {
        let crash_count_7d = crashes.len() as u32;
        let crash_count_24h = crashes
            .iter()
            .filter(|c| {
                let now = Utc::now();
                let duration = now.signed_duration_since(c.timestamp);
                duration.num_hours() <= 24
            })
            .count() as u32;

        let last_crash = crashes.iter().max_by_key(|c| c.timestamp).cloned();

        crashed_services.push(CrashedService {
            service_name: service_name.clone(),
            crash_count_24h,
            crash_count_7d,
            last_crash_time: last_crash.as_ref().map(|c| c.timestamp),
            exit_code: last_crash.as_ref().and_then(|c| c.exit_code),
            signal: last_crash.and_then(|c| c.signal),
        });
    }

    // Sort by crash count (most crashes first)
    crashed_services.sort_by(|a, b| b.crash_count_24h.cmp(&a.crash_count_24h));

    // Keep only the 10 most recent crashes for the recent_crashes list
    let mut all_crashes = crashes_7d.clone();
    all_crashes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    recent_crashes = all_crashes.into_iter().take(10).collect();

    DaemonCrashes {
        total_crashes_24h,
        total_crashes_7d,
        crashed_services,
        recent_crashes,
    }
}

fn parse_crash_events(output: &Result<std::process::Output, std::io::Error>) -> Vec<CrashEvent> {
    let mut events = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                // Parse JSON log entry
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    // Extract timestamp
                    let timestamp = if let Some(timestamp_str) =
                        json.get("__REALTIME_TIMESTAMP").and_then(|v| v.as_str())
                    {
                        // Timestamp is in microseconds since epoch
                        if let Ok(micros) = timestamp_str.parse::<i64>() {
                            let secs = micros / 1_000_000;
                            let nsecs = ((micros % 1_000_000) * 1000) as u32;
                            DateTime::from_timestamp(secs, nsecs).unwrap_or_else(|| Utc::now())
                        } else {
                            Utc::now()
                        }
                    } else {
                        Utc::now()
                    };

                    // Extract service name
                    let service_name = json
                        .get("UNIT")
                        .or_else(|| json.get("_SYSTEMD_UNIT"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .trim_end_matches(".service")
                        .to_string();

                    // Extract message
                    let message = json
                        .get("MESSAGE")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Try to extract exit code and signal from message
                    let (exit_code, signal) = parse_exit_info(&message);

                    events.push(CrashEvent {
                        timestamp,
                        service_name,
                        exit_code,
                        signal,
                        message,
                    });
                }
            }
        }
    }

    events
}

fn parse_exit_info(message: &str) -> (Option<i32>, Option<String>) {
    let mut exit_code = None;
    let mut signal = None;

    // Look for "code=exited, status=N"
    if let Some(start) = message.find("status=") {
        let rest = &message[start + 7..];
        if let Some(end) = rest.find(|c: char| !c.is_numeric() && c != '-') {
            if let Ok(code) = rest[..end].parse::<i32>() {
                exit_code = Some(code);
            }
        } else if let Ok(code) = rest.parse::<i32>() {
            exit_code = Some(code);
        }
    }

    // Look for "signal=SIG..."
    if let Some(start) = message.find("signal=") {
        let rest = &message[start + 7..];
        if let Some(end) = rest.find(|c: char| c.is_whitespace() || c == ')' || c == ',') {
            signal = Some(rest[..end].to_string());
        } else {
            signal = Some(rest.to_string());
        }
    }

    (exit_code, signal)
}

fn detect_system_uptime() -> SystemUptime {
    // Read uptime from /proc/uptime
    let uptime_seconds = if let Ok(content) = fs::read_to_string("/proc/uptime") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if !parts.is_empty() {
            parts[0].parse::<f64>().unwrap_or(0.0) as u64
        } else {
            0
        }
    } else {
        0
    };

    let uptime_days = uptime_seconds as f64 / 86400.0;

    // Calculate boot time
    let boot_time = if uptime_seconds > 0 {
        let now = Utc::now();
        let boot_timestamp = now.timestamp() - uptime_seconds as i64;
        DateTime::from_timestamp(boot_timestamp, 0)
    } else {
        None
    };

    SystemUptime {
        uptime_seconds,
        uptime_days,
        boot_time,
    }
}

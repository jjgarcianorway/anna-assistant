use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// CPU throttling and power state detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuThrottling {
    pub throttling_events: ThrottlingEvents,
    pub power_states: PowerStates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottlingEvents {
    pub total_core_throttles: u64,
    pub total_package_throttles: u64,
    pub per_cpu_throttles: Vec<CpuThrottleInfo>,
    pub has_throttling: bool,
    pub thermal_events_24h: Vec<ThermalEvent>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuThrottleInfo {
    pub cpu_id: usize,
    pub core_throttle_count: u64,
    pub package_throttle_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalEvent {
    pub timestamp: DateTime<Utc>,
    pub cpu_id: Option<usize>,
    pub temperature: Option<f64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStates {
    pub c_states_available: Vec<String>,
    pub c_states_enabled: bool,
    pub deepest_c_state: Option<String>,
    pub per_cpu_c_states: Vec<CpuCStateInfo>,
    pub total_cpus: usize,
    pub power_management_enabled: bool,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCStateInfo {
    pub cpu_id: usize,
    pub current_state: Option<String>,
    pub available_states: Vec<String>,
    pub state_latencies: Vec<(String, u64)>, // (state_name, latency_us)
}

impl CpuThrottling {
    /// Detect CPU throttling events and power states
    pub fn detect() -> Self {
        let throttling_events = detect_throttling_events();
        let power_states = detect_power_states();

        Self {
            throttling_events,
            power_states,
        }
    }
}

fn detect_throttling_events() -> ThrottlingEvents {
    let mut per_cpu_throttles = Vec::new();
    let mut total_core_throttles = 0u64;
    let mut total_package_throttles = 0u64;

    // Detect number of CPUs
    let cpu_count = num_cpus::get();

    for cpu_id in 0..cpu_count {
        let throttle_path = format!("/sys/devices/system/cpu/cpu{}/thermal_throttle", cpu_id);

        if let Ok(core_throttle) =
            fs::read_to_string(format!("{}/core_throttle_count", throttle_path))
        {
            let core_count = core_throttle.trim().parse::<u64>().unwrap_or(0);
            let package_count =
                fs::read_to_string(format!("{}/package_throttle_count", throttle_path))
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .unwrap_or(0);

            total_core_throttles += core_count;
            total_package_throttles += package_count;

            per_cpu_throttles.push(CpuThrottleInfo {
                cpu_id,
                core_throttle_count: core_count,
                package_throttle_count: package_count,
            });
        }
    }

    let has_throttling = total_core_throttles > 0 || total_package_throttles > 0;

    // Detect recent thermal events from journalctl
    let thermal_events_24h = detect_thermal_events();

    // Generate recommendations
    let mut recommendations = Vec::new();

    if has_throttling {
        recommendations.push(format!(
            "CPU throttling detected: {} core throttles, {} package throttles",
            total_core_throttles, total_package_throttles
        ));

        if total_core_throttles > 1000 {
            recommendations.push(
                "High throttling count detected - check CPU cooling and reduce workload"
                    .to_string(),
            );
        }

        if !thermal_events_24h.is_empty() {
            recommendations.push(format!(
                "{} thermal events in last 24h - consider improving cooling",
                thermal_events_24h.len()
            ));
        }
    } else {
        recommendations
            .push("No CPU throttling detected - thermal performance is good".to_string());
    }

    ThrottlingEvents {
        total_core_throttles,
        total_package_throttles,
        per_cpu_throttles,
        has_throttling,
        thermal_events_24h,
        recommendations,
    }
}

fn detect_thermal_events() -> Vec<ThermalEvent> {
    let mut events = Vec::new();

    // Query journalctl for thermal events in the last 24 hours
    let output = Command::new("journalctl")
        .args([
            "--since",
            "24 hours ago",
            "--grep=thermal|temperature|throttle",
            "--no-pager",
            "--output=json",
        ])
        .output();

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
                            DateTime::from_timestamp(secs, nsecs).unwrap_or_else(Utc::now)
                        } else {
                            Utc::now()
                        }
                    } else {
                        Utc::now()
                    };

                    // Extract message
                    let message = json
                        .get("MESSAGE")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Try to extract CPU ID and temperature from message
                    let cpu_id = extract_cpu_id(&message);
                    let temperature = extract_temperature(&message);

                    events.push(ThermalEvent {
                        timestamp,
                        cpu_id,
                        temperature,
                        message,
                    });
                }
            }
        }
    }

    // Sort by timestamp (most recent first)
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Keep only the 20 most recent events
    events.truncate(20);

    events
}

fn extract_cpu_id(message: &str) -> Option<usize> {
    // Try to extract CPU ID from patterns like "CPU 0", "cpu0", "Core 0", etc.
    if let Some(start) = message.to_lowercase().find("cpu") {
        let rest = &message[start + 3..];
        if let Some(first_char) = rest.chars().next() {
            if first_char.is_numeric() {
                let num_str: String = rest.chars().take_while(|c| c.is_numeric()).collect();
                return num_str.parse::<usize>().ok();
            }
        }
    }
    None
}

fn extract_temperature(message: &str) -> Option<f64> {
    // Try to extract temperature from patterns like "85.0°C", "85 C", "temp=85", etc.
    // Look for numbers followed by °C, C, or preceded by "temp="
    if let Some(start) = message.find("°C") {
        let before = &message[..start];
        if let Some(num_start) = before.rfind(|c: char| !c.is_numeric() && c != '.') {
            let num_str = &before[num_start + 1..];
            return num_str.parse::<f64>().ok();
        }
    }
    None
}

fn detect_power_states() -> PowerStates {
    let cpu_count = num_cpus::get();
    let mut per_cpu_c_states = Vec::new();
    let mut all_c_states: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut deepest_c_state: Option<String> = None;

    for cpu_id in 0..cpu_count {
        let cpuidle_path = format!("/sys/devices/system/cpu/cpu{}/cpuidle", cpu_id);

        let mut available_states = Vec::new();
        let mut state_latencies = Vec::new();
        let mut current_state = None;

        // Read available C-states
        if let Ok(entries) = fs::read_dir(&cpuidle_path) {
            for entry in entries.flatten() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    if dir_name.starts_with("state") {
                        let state_path = entry.path();

                        // Read state name
                        if let Ok(name) = fs::read_to_string(state_path.join("name")) {
                            let state_name = name.trim().to_string();
                            available_states.push(state_name.clone());
                            all_c_states.insert(state_name.clone());

                            // Track deepest C-state (highest number)
                            if deepest_c_state.is_none()
                                || state_name > deepest_c_state.clone().unwrap()
                            {
                                deepest_c_state = Some(state_name.clone());
                            }

                            // Read latency
                            if let Ok(latency_str) = fs::read_to_string(state_path.join("latency"))
                            {
                                if let Ok(latency) = latency_str.trim().parse::<u64>() {
                                    state_latencies.push((state_name, latency));
                                }
                            }
                        }

                        // Check if this state is currently in use (disabled=0 means enabled)
                        if let Ok(disabled) = fs::read_to_string(state_path.join("disable")) {
                            if disabled.trim() == "0" {
                                if let Ok(name) = fs::read_to_string(state_path.join("name")) {
                                    current_state = Some(name.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        per_cpu_c_states.push(CpuCStateInfo {
            cpu_id,
            current_state,
            available_states,
            state_latencies,
        });
    }

    let c_states_available: Vec<String> = all_c_states.into_iter().collect();
    let c_states_enabled = !c_states_available.is_empty();
    let power_management_enabled = c_states_enabled;

    // Generate recommendations
    let mut recommendations = Vec::new();

    if c_states_enabled {
        recommendations.push(format!(
            "CPU power management enabled with {} C-states available",
            c_states_available.len()
        ));

        if let Some(ref deepest) = deepest_c_state {
            recommendations.push(format!(
                "Deepest C-state: {} (better power savings)",
                deepest
            ));
        }
    } else {
        recommendations.push(
            "CPU power management not detected - C-states may not be available on this system"
                .to_string(),
        );
    }

    PowerStates {
        c_states_available,
        c_states_enabled,
        deepest_c_state,
        per_cpu_c_states,
        total_cpus: cpu_count,
        power_management_enabled,
        recommendations,
    }
}

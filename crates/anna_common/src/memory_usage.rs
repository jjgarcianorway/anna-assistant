//! Memory usage detection
//!
//! Detects memory configuration and usage:
//! - RAM usage (total, available, buffers, cache)
//! - Swap configuration and usage
//! - OOM (Out of Memory) events
//! - Memory pressure

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// Memory usage and configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageInfo {
    /// Total RAM in GB
    pub total_ram_gb: f32,
    /// Available RAM in GB
    pub available_ram_gb: f32,
    /// Used RAM in GB
    pub used_ram_gb: f32,
    /// RAM usage percentage
    pub ram_usage_percent: f32,
    /// Buffers in GB
    pub buffers_gb: f32,
    /// Cached in GB
    pub cached_gb: f32,
    /// Swap configuration
    pub swap: SwapInfo,
    /// OOM events detected
    pub oom_events: Vec<OOMEvent>,
    /// Memory pressure (if available)
    pub memory_pressure: Option<MemoryPressure>,
}

/// Swap configuration and usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    /// Total swap in GB
    pub total_gb: f32,
    /// Used swap in GB
    pub used_gb: f32,
    /// Swap usage percentage
    pub usage_percent: f32,
    /// Swap type (partition, file, zram, none)
    pub swap_type: SwapType,
    /// Swap devices/files
    pub devices: Vec<SwapDevice>,
}

/// Type of swap
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SwapType {
    /// Swap partition
    Partition,
    /// Swap file
    File,
    /// Zram (compressed RAM swap)
    Zram,
    /// Multiple types
    Mixed,
    /// No swap configured
    None,
}

/// Individual swap device or file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapDevice {
    /// Device path or filename
    pub name: String,
    /// Type (partition, file, zram)
    pub swap_type: String,
    /// Size in GB
    pub size_gb: f32,
    /// Used in GB
    pub used_gb: f32,
    /// Priority
    pub priority: i32,
}

/// OOM (Out of Memory) event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OOMEvent {
    /// Timestamp (if parseable)
    pub timestamp: Option<String>,
    /// Process that was killed
    pub killed_process: String,
    /// PID of killed process
    pub pid: Option<u32>,
    /// OOM score
    pub oom_score: Option<i32>,
}

/// Memory pressure information (PSI - Pressure Stall Information)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPressure {
    /// Some pressure (avg10)
    pub some_avg10: f32,
    /// Some pressure (avg60)
    pub some_avg60: f32,
    /// Some pressure (avg300)
    pub some_avg300: f32,
    /// Full pressure (avg10)
    pub full_avg10: f32,
    /// Full pressure (avg60)
    pub full_avg60: f32,
    /// Full pressure (avg300)
    pub full_avg300: f32,
}

impl MemoryUsageInfo {
    /// Detect memory usage and configuration
    pub fn detect() -> Self {
        let (total_ram_gb, available_ram_gb, used_ram_gb, buffers_gb, cached_gb) =
            parse_meminfo();

        let ram_usage_percent = if total_ram_gb > 0.0 {
            (used_ram_gb / total_ram_gb * 100.0).min(100.0)
        } else {
            0.0
        };

        let swap = detect_swap_info();
        let oom_events = detect_oom_events();
        let memory_pressure = detect_memory_pressure();

        Self {
            total_ram_gb,
            available_ram_gb,
            used_ram_gb,
            ram_usage_percent,
            buffers_gb,
            cached_gb,
            swap,
            oom_events,
            memory_pressure,
        }
    }
}

/// Parse /proc/meminfo for memory statistics
fn parse_meminfo() -> (f32, f32, f32, f32, f32) {
    let content = match fs::read_to_string("/proc/meminfo") {
        Ok(c) => c,
        Err(_) => return (0.0, 0.0, 0.0, 0.0, 0.0),
    };

    let mut total_kb = 0u64;
    let mut available_kb = 0u64;
    let mut buffers_kb = 0u64;
    let mut cached_kb = 0u64;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].trim_end_matches(':');
        if let Ok(value) = parts[1].parse::<u64>() {
            match key {
                "MemTotal" => total_kb = value,
                "MemAvailable" => available_kb = value,
                "Buffers" => buffers_kb = value,
                "Cached" => cached_kb = value,
                _ => {}
            }
        }
    }

    let total_gb = total_kb as f32 / 1024.0 / 1024.0;
    let available_gb = available_kb as f32 / 1024.0 / 1024.0;
    let buffers_gb = buffers_kb as f32 / 1024.0 / 1024.0;
    let cached_gb = cached_kb as f32 / 1024.0 / 1024.0;
    let used_gb = total_gb - available_gb;

    (total_gb, available_gb, used_gb, buffers_gb, cached_gb)
}

/// Detect swap configuration and usage
fn detect_swap_info() -> SwapInfo {
    let devices = parse_swaps();

    let total_gb: f32 = devices.iter().map(|d| d.size_gb).sum();
    let used_gb: f32 = devices.iter().map(|d| d.used_gb).sum();

    let usage_percent = if total_gb > 0.0 {
        (used_gb / total_gb * 100.0).min(100.0)
    } else {
        0.0
    };

    let swap_type = determine_swap_type(&devices);

    SwapInfo {
        total_gb,
        used_gb,
        usage_percent,
        swap_type,
        devices,
    }
}

/// Parse /proc/swaps for swap devices
fn parse_swaps() -> Vec<SwapDevice> {
    let content = match fs::read_to_string("/proc/swaps") {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut devices = Vec::new();

    for line in content.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let name = parts[0].to_string();
        let swap_type = parts[1].to_string();
        let size_kb = parts[2].parse::<u64>().unwrap_or(0);
        let used_kb = parts[3].parse::<u64>().unwrap_or(0);
        let priority = parts[4].parse::<i32>().unwrap_or(0);

        devices.push(SwapDevice {
            name,
            swap_type,
            size_gb: size_kb as f32 / 1024.0 / 1024.0,
            used_gb: used_kb as f32 / 1024.0 / 1024.0,
            priority,
        });
    }

    devices
}

/// Determine the type of swap configuration
fn determine_swap_type(devices: &[SwapDevice]) -> SwapType {
    if devices.is_empty() {
        return SwapType::None;
    }

    let has_partition = devices.iter().any(|d| d.swap_type == "partition");
    let has_file = devices.iter().any(|d| d.swap_type == "file");
    let has_zram = devices
        .iter()
        .any(|d| d.name.contains("zram") || d.swap_type == "zram");

    match (has_partition, has_file, has_zram) {
        (true, false, false) => SwapType::Partition,
        (false, true, false) => SwapType::File,
        (false, false, true) => SwapType::Zram,
        _ => SwapType::Mixed,
    }
}

/// Detect OOM events from kernel logs
fn detect_oom_events() -> Vec<OOMEvent> {
    let output = match Command::new("journalctl")
        .args(["-k", "--no-pager", "--since", "24 hours ago"])
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    let mut events = Vec::new();

    for line in output.lines() {
        if line.contains("Out of memory") || line.contains("oom-kill") || line.contains("Killed process") {
            // Try to parse OOM event
            if let Some(event) = parse_oom_line(line) {
                events.push(event);
            }
        }
    }

    // Limit to last 10 events
    events.truncate(10);
    events
}

/// Parse an OOM event line from journalctl
fn parse_oom_line(line: &str) -> Option<OOMEvent> {
    // Try to extract timestamp (first part before hostname)
    let timestamp = line
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");

    // Look for "Killed process XXXX (name)"
    let killed_process = if let Some(start) = line.find("Killed process") {
        let rest = &line[start..];
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 4 {
            let pid = parts[2].parse::<u32>().ok();
            let name = parts[3].trim_matches(|c| c == '(' || c == ')').to_string();

            // Try to extract OOM score
            let oom_score = if let Some(score_pos) = line.find("oom_score_adj") {
                let score_str = &line[score_pos..];
                score_str
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<i32>().ok())
            } else {
                None
            };

            return Some(OOMEvent {
                timestamp: Some(timestamp),
                killed_process: name,
                pid,
                oom_score,
            });
        }
        "Unknown".to_string()
    } else {
        "Unknown".to_string()
    };

    Some(OOMEvent {
        timestamp: Some(timestamp),
        killed_process,
        pid: None,
        oom_score: None,
    })
}

/// Detect memory pressure using PSI (Pressure Stall Information)
fn detect_memory_pressure() -> Option<MemoryPressure> {
    let content = fs::read_to_string("/proc/pressure/memory").ok()?;

    let mut some_avg10 = 0.0;
    let mut some_avg60 = 0.0;
    let mut some_avg300 = 0.0;
    let mut full_avg10 = 0.0;
    let mut full_avg60 = 0.0;
    let mut full_avg300 = 0.0;

    for line in content.lines() {
        if line.starts_with("some") {
            // Parse: some avg10=0.00 avg60=0.00 avg300=0.00 total=123456
            for part in line.split_whitespace() {
                if let Some(val) = part.strip_prefix("avg10=") {
                    some_avg10 = val.parse().unwrap_or(0.0);
                } else if let Some(val) = part.strip_prefix("avg60=") {
                    some_avg60 = val.parse().unwrap_or(0.0);
                } else if let Some(val) = part.strip_prefix("avg300=") {
                    some_avg300 = val.parse().unwrap_or(0.0);
                }
            }
        } else if line.starts_with("full") {
            for part in line.split_whitespace() {
                if let Some(val) = part.strip_prefix("avg10=") {
                    full_avg10 = val.parse().unwrap_or(0.0);
                } else if let Some(val) = part.strip_prefix("avg60=") {
                    full_avg60 = val.parse().unwrap_or(0.0);
                } else if let Some(val) = part.strip_prefix("avg300=") {
                    full_avg300 = val.parse().unwrap_or(0.0);
                }
            }
        }
    }

    Some(MemoryPressure {
        some_avg10,
        some_avg60,
        some_avg300,
        full_avg10,
        full_avg60,
        full_avg300,
    })
}

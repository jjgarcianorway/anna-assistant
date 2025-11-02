// Anna v0.12.0 - Lightweight Telemetry Collectors
// Optional-aware collectors: sensors, net, disk, top

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

/// Sensors data (CPU, memory, temps)
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorsData {
    pub cpu: CpuData,
    pub mem: MemData,
    pub power: Option<PowerData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuData {
    pub load_avg: [f64; 3],
    pub cores: Vec<CoreData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreData {
    pub core: u32,
    pub util_pct: f64,
    pub temp_c: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemData {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerData {
    pub percent: u64,
    pub status: String,
    pub power_now_w: Option<f64>,
}

/// Network data
#[derive(Debug, Serialize, Deserialize)]
pub struct NetData {
    pub interfaces: Vec<InterfaceData>,
    pub default_route: Option<String>,
    pub dns_latency_ms: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceData {
    pub iface: String,
    pub link_state: String,
    pub rx_kbps: f64,
    pub tx_kbps: f64,
    pub ipv4_redacted: Option<String>,
    pub ipv6_redacted: Option<String>,
}

/// Disk data
#[derive(Debug, Serialize, Deserialize)]
pub struct DiskData {
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount: String,
    pub device: String,
    pub pct: f64,
    pub used_gb: f64,
    pub total_gb: f64,
    pub inode_pct: Option<f64>,
    pub smart_status: Option<String>,
}

/// Top processes
#[derive(Debug, Serialize, Deserialize)]
pub struct TopData {
    pub by_cpu: Vec<ProcessInfo>,
    pub by_mem: Vec<ProcessInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: f64,
    pub mem_mb: f64,
}

/// Collect sensors data (CPU, memory, temps, battery)
pub fn collect_sensors() -> Result<SensorsData> {
    let cpu = collect_cpu()?;
    let mem = collect_memory()?;
    let power = collect_power().ok();

    Ok(SensorsData { cpu, mem, power })
}

fn collect_cpu() -> Result<CpuData> {
    // Load average
    let loadavg = fs::read_to_string("/proc/loadavg")?;
    let parts: Vec<&str> = loadavg.split_whitespace().collect();
    let load_avg = [
        parts[0].parse().unwrap_or(0.0),
        parts[1].parse().unwrap_or(0.0),
        parts[2].parse().unwrap_or(0.0),
    ];

    // Core temps (try lm_sensors first, fallback to /sys/class/thermal)
    let temps = collect_temps();

    // Core utilization (simplified: read from /proc/stat)
    let mut cores = Vec::new();
    let stat = fs::read_to_string("/proc/stat")?;
    for (i, line) in stat.lines().enumerate() {
        if line.starts_with("cpu") && !line.starts_with("cpu ") {
            // Extract core number
            let core_num = line[3..]
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(i as u32);

            cores.push(CoreData {
                core: core_num,
                util_pct: 0.0, // TODO: calculate from idle time delta
                temp_c: temps.get(i).copied(),
            });
        }
    }

    Ok(CpuData { load_avg, cores })
}

fn collect_temps() -> Vec<f64> {
    // Try sensors command
    if let Ok(output) = Command::new("sensors").arg("-A").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return parse_sensors_output(&stdout);
        }
    }

    // Fallback: /sys/class/thermal
    let mut temps = Vec::new();
    for i in 0..16 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(millidegrees) = content.trim().parse::<i32>() {
                temps.push(millidegrees as f64 / 1000.0);
            }
        } else {
            break;
        }
    }
    temps
}

fn parse_sensors_output(output: &str) -> Vec<f64> {
    let mut temps = Vec::new();
    for line in output.lines() {
        if line.contains("°C") {
            if let Some(temp_str) = line
                .split_whitespace()
                .find(|s| s.starts_with('+') || s.starts_with('-'))
            {
                let temp_str = temp_str.trim_start_matches('+').trim_end_matches("°C");
                if let Ok(temp) = temp_str.parse::<f64>() {
                    temps.push(temp);
                }
            }
        }
    }
    temps
}

fn collect_memory() -> Result<MemData> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    let mut data = HashMap::new();

    for line in meminfo.lines() {
        if let Some((key, val)) = line.split_once(':') {
            let val_kb = val
                .trim()
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            data.insert(key.to_string(), val_kb);
        }
    }

    let total_kb = data.get("MemTotal").copied().unwrap_or(0);
    let available_kb = data.get("MemAvailable").copied().unwrap_or(0);
    let swap_total_kb = data.get("SwapTotal").copied().unwrap_or(0);
    let swap_free_kb = data.get("SwapFree").copied().unwrap_or(0);

    Ok(MemData {
        total_mb: total_kb / 1024,
        used_mb: (total_kb - available_kb) / 1024,
        free_mb: available_kb / 1024,
        swap_total_mb: swap_total_kb / 1024,
        swap_used_mb: (swap_total_kb - swap_free_kb) / 1024,
    })
}

fn collect_power() -> Result<PowerData> {
    // Try common battery paths
    for bat in &["BAT0", "BAT1"] {
        let base = format!("/sys/class/power_supply/{}", bat);
        if let Ok(capacity) = fs::read_to_string(format!("{}/capacity", base)) {
            let percent = capacity.trim().parse().unwrap_or(0);
            let status = fs::read_to_string(format!("{}/status", base))
                .unwrap_or_else(|_| "Unknown".to_string())
                .trim()
                .to_string();

            let power_now_w = fs::read_to_string(format!("{}/power_now", base))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(|uw| uw as f64 / 1_000_000.0);

            return Ok(PowerData {
                percent,
                status,
                power_now_w,
            });
        }
    }

    anyhow::bail!("No battery found")
}

/// Collect network data
pub fn collect_net() -> Result<NetData> {
    let interfaces = collect_interfaces()?;
    let default_route = find_default_route();
    let dns_latency_ms = check_dns_latency();

    Ok(NetData {
        interfaces,
        default_route,
        dns_latency_ms,
    })
}

fn collect_interfaces() -> Result<Vec<InterfaceData>> {
    let mut ifaces = Vec::new();

    // Read /sys/class/net for interfaces
    let net_dir = fs::read_dir("/sys/class/net")?;
    for entry in net_dir {
        let entry = entry?;
        let iface_name = entry.file_name().to_string_lossy().to_string();

        if iface_name == "lo" {
            continue; // Skip loopback
        }

        let base = format!("/sys/class/net/{}", iface_name);

        let link_state = fs::read_to_string(format!("{}/operstate", base))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        // Read RX/TX bytes (simplified, no rate calc)
        let rx_bytes = fs::read_to_string(format!("{}/statistics/rx_bytes", base))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let tx_bytes = fs::read_to_string(format!("{}/statistics/tx_bytes", base))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        ifaces.push(InterfaceData {
            iface: iface_name,
            link_state,
            rx_kbps: rx_bytes as f64 / 1024.0, // TODO: calculate rate
            tx_kbps: tx_bytes as f64 / 1024.0,
            ipv4_redacted: None, // TODO: redact IP
            ipv6_redacted: None,
        });
    }

    Ok(ifaces)
}

fn find_default_route() -> Option<String> {
    let output = Command::new("ip")
        .args(&["route", "show", "default"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().next().map(|s| s.to_string())
}

fn check_dns_latency() -> Option<f64> {
    // Simple ping to 8.8.8.8
    let output = Command::new("ping")
        .args(&["-c", "1", "-W", "1", "8.8.8.8"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("time=") {
                if let Some(time_str) = line.split("time=").nth(1) {
                    if let Some(ms_str) = time_str.split_whitespace().next() {
                        return ms_str.parse().ok();
                    }
                }
            }
        }
    }
    None
}

/// Collect disk data
pub fn collect_disk() -> Result<DiskData> {
    let mut disks = Vec::new();

    let output = Command::new("df")
        .args(&["-BG", "--output=source,target,size,used,pcent"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let device = parts[0].to_string();
        let mount = parts[1].to_string();
        let total_str = parts[2].trim_end_matches('G');
        let used_str = parts[3].trim_end_matches('G');
        let pct_str = parts[4].trim_end_matches('%');

        let total_gb = total_str.parse().unwrap_or(0.0);
        let used_gb = used_str.parse().unwrap_or(0.0);
        let pct = pct_str.parse().unwrap_or(0.0);

        disks.push(DiskInfo {
            mount,
            device,
            pct,
            used_gb,
            total_gb,
            inode_pct: None,    // TODO: add inode check
            smart_status: None, // TODO: add smartctl check
        });
    }

    Ok(DiskData { disks })
}

/// Collect top processes
pub fn collect_top(limit: usize) -> Result<TopData> {
    let output = Command::new("ps")
        .args(&["-eo", "pid,comm,%cpu,%mem", "--sort=-%cpu"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut by_cpu = Vec::new();
    for line in stdout.lines().skip(1).take(limit) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        by_cpu.push(ProcessInfo {
            pid: parts[0].parse().unwrap_or(0),
            name: parts[1].to_string(),
            cpu_pct: parts[2].parse().unwrap_or(0.0),
            mem_mb: parts[3].parse::<f64>().unwrap_or(0.0) * 10.0, // Rough estimate
        });
    }

    // Sort by memory
    let output = Command::new("ps")
        .args(&["-eo", "pid,comm,%cpu,%mem", "--sort=-%mem"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut by_mem = Vec::new();
    for line in stdout.lines().skip(1).take(limit) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        by_mem.push(ProcessInfo {
            pid: parts[0].parse().unwrap_or(0),
            name: parts[1].to_string(),
            cpu_pct: parts[2].parse().unwrap_or(0.0),
            mem_mb: parts[3].parse::<f64>().unwrap_or(0.0) * 10.0,
        });
    }

    Ok(TopData { by_cpu, by_mem })
}

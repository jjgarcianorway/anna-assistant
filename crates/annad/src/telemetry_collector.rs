//! Telemetry collector for system metrics (v0.0.280).
//!
//! Collects system state periodically for trend analysis.

use anna_shared::system_telemetry::{
    NetworkStatus, ServiceStatus, TelemetrySample, TelemetryStore,
};
use chrono::Utc;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

/// Collection interval in seconds
const COLLECTION_INTERVAL_SECS: u64 = 300; // 5 minutes

/// Start the telemetry collector background task
pub fn start_collector(store: Arc<RwLock<TelemetryStore>>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(COLLECTION_INTERVAL_SECS));
        info!(
            "Telemetry collector started (interval: {}s)",
            COLLECTION_INTERVAL_SECS
        );

        loop {
            ticker.tick().await;

            match collect_sample().await {
                Ok(sample) => {
                    let mut store = store.write().await;
                    store.add_sample(sample);
                    store.analyze();

                    // Save periodically
                    if let Err(e) = store.save() {
                        warn!("Failed to save telemetry: {}", e);
                    } else {
                        debug!("Telemetry sample collected and saved");
                    }
                }
                Err(e) => {
                    warn!("Failed to collect telemetry sample: {}", e);
                }
            }
        }
    });
}

/// Collect a single telemetry sample
async fn collect_sample() -> anyhow::Result<TelemetrySample> {
    let (cpu, memory, disk, load, procs, uptime) = tokio::join!(
        collect_cpu_usage(),
        collect_memory(),
        collect_disk(),
        collect_load_average(),
        collect_process_count(),
        collect_uptime(),
    );

    let services = collect_services().await;
    let network = collect_network().await;

    Ok(TelemetrySample {
        timestamp: Utc::now(),
        cpu_usage_percent: cpu.ok(),
        memory_used_bytes: memory.as_ref().ok().map(|(used, _)| *used),
        memory_total_bytes: memory.ok().map(|(_, total)| total),
        disk_used_bytes: disk.as_ref().ok().map(|(used, _)| *used),
        disk_total_bytes: disk.ok().map(|(_, total)| total),
        load_average_1m: load.as_ref().ok().map(|(l1, _, _)| *l1),
        load_average_5m: load.as_ref().ok().map(|(_, l5, _)| *l5),
        load_average_15m: load.ok().map(|(_, _, l15)| l15),
        process_count: procs.ok(),
        uptime_secs: uptime.ok(),
        services,
        network,
    })
}

/// Collect CPU usage from /proc/stat
async fn collect_cpu_usage() -> anyhow::Result<f32> {
    // Read /proc/stat twice with a small delay to calculate CPU usage
    let stat1 = fs::read_to_string("/proc/stat")?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let stat2 = fs::read_to_string("/proc/stat")?;

    let parse_cpu_line = |line: &str| -> Option<(u64, u64)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return None;
        }
        let user: u64 = parts[1].parse().ok()?;
        let nice: u64 = parts[2].parse().ok()?;
        let system: u64 = parts[3].parse().ok()?;
        let idle: u64 = parts[4].parse().ok()?;
        let total = user + nice + system + idle;
        let busy = user + nice + system;
        Some((busy, total))
    };

    let cpu1 = stat1
        .lines()
        .find(|l| l.starts_with("cpu "))
        .and_then(parse_cpu_line);
    let cpu2 = stat2
        .lines()
        .find(|l| l.starts_with("cpu "))
        .and_then(parse_cpu_line);

    match (cpu1, cpu2) {
        (Some((busy1, total1)), Some((busy2, total2))) => {
            let busy_diff = busy2.saturating_sub(busy1) as f32;
            let total_diff = total2.saturating_sub(total1) as f32;
            if total_diff > 0.0 {
                Ok((busy_diff / total_diff) * 100.0)
            } else {
                Ok(0.0)
            }
        }
        _ => Err(anyhow::anyhow!("Could not parse CPU stats")),
    }
}

/// Collect memory usage from /proc/meminfo
async fn collect_memory() -> anyhow::Result<(u64, u64)> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;

    let mut total: Option<u64> = None;
    let mut available: Option<u64> = None;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_meminfo_value(line);
        } else if line.starts_with("MemAvailable:") {
            available = parse_meminfo_value(line);
        }
    }

    match (total, available) {
        (Some(t), Some(a)) => Ok((t.saturating_sub(a), t)),
        _ => Err(anyhow::anyhow!("Could not parse memory info")),
    }
}

fn parse_meminfo_value(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        // Value is in kB, convert to bytes
        parts[1].parse::<u64>().ok().map(|v| v * 1024)
    } else {
        None
    }
}

/// Collect disk usage for root filesystem
async fn collect_disk() -> anyhow::Result<(u64, u64)> {
    let output = Command::new("df").args(["-B1", "/"]).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let total: u64 = parts[1].parse().unwrap_or(0);
            let used: u64 = parts[2].parse().unwrap_or(0);
            return Ok((used, total));
        }
    }

    Err(anyhow::anyhow!("Could not parse disk usage"))
}

/// Collect load averages from /proc/loadavg
async fn collect_load_average() -> anyhow::Result<(f32, f32, f32)> {
    let loadavg = fs::read_to_string("/proc/loadavg")?;
    let parts: Vec<&str> = loadavg.split_whitespace().collect();

    if parts.len() >= 3 {
        let l1: f32 = parts[0].parse().unwrap_or(0.0);
        let l5: f32 = parts[1].parse().unwrap_or(0.0);
        let l15: f32 = parts[2].parse().unwrap_or(0.0);
        Ok((l1, l5, l15))
    } else {
        Err(anyhow::anyhow!("Could not parse load average"))
    }
}

/// Collect process count
async fn collect_process_count() -> anyhow::Result<u32> {
    let entries = fs::read_dir("/proc")?;
    let count = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false)
        })
        .count();
    Ok(count as u32)
}

/// Collect system uptime from /proc/uptime
async fn collect_uptime() -> anyhow::Result<u64> {
    let uptime = fs::read_to_string("/proc/uptime")?;
    let parts: Vec<&str> = uptime.split_whitespace().collect();
    if let Some(secs_str) = parts.first() {
        let secs: f64 = secs_str.parse().unwrap_or(0.0);
        Ok(secs as u64)
    } else {
        Err(anyhow::anyhow!("Could not parse uptime"))
    }
}

/// Collect status of key services
async fn collect_services() -> Vec<ServiceStatus> {
    let services = ["sshd", "nginx", "docker", "postgresql", "mysql", "redis"];
    let mut results = Vec::new();

    for service in services {
        if let Ok(status) = check_service_status(service).await {
            results.push(status);
        }
    }

    results
}

async fn check_service_status(service: &str) -> anyhow::Result<ServiceStatus> {
    let output = Command::new("systemctl")
        .args(["is-active", service])
        .output()?;

    let running = output.status.success();

    let enabled_output = Command::new("systemctl")
        .args(["is-enabled", service])
        .output()?;

    let enabled = enabled_output.status.success();

    Ok(ServiceStatus {
        name: service.to_string(),
        running,
        enabled,
        memory_bytes: None,
        cpu_percent: None,
    })
}

/// Collect network interface statistics
async fn collect_network() -> Vec<NetworkStatus> {
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.filter_map(|e| e.ok()) {
            let iface = entry.file_name().to_string_lossy().to_string();

            // Skip loopback
            if iface == "lo" {
                continue;
            }

            if let Ok(status) = collect_interface_stats(&iface).await {
                results.push(status);
            }
        }
    }

    results
}

async fn collect_interface_stats(iface: &str) -> anyhow::Result<NetworkStatus> {
    let base = format!("/sys/class/net/{}/statistics", iface);

    let rx_bytes = fs::read_to_string(format!("{}/rx_bytes", base))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let tx_bytes = fs::read_to_string(format!("{}/tx_bytes", base))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let rx_errors = fs::read_to_string(format!("{}/rx_errors", base))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let tx_errors = fs::read_to_string(format!("{}/tx_errors", base))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let operstate = fs::read_to_string(format!("/sys/class/net/{}/operstate", iface))
        .ok()
        .map(|s| s.trim() == "up")
        .unwrap_or(false);

    Ok(NetworkStatus {
        interface: iface.to_string(),
        up: operstate,
        rx_bytes,
        tx_bytes,
        rx_errors,
        tx_errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collect_memory() {
        // This test may fail on non-Linux systems
        if cfg!(target_os = "linux") {
            let result = collect_memory().await;
            assert!(result.is_ok());
            let (used, total) = result.unwrap();
            assert!(total > 0);
            assert!(used <= total);
        }
    }

    #[tokio::test]
    async fn test_collect_load_average() {
        if cfg!(target_os = "linux") {
            let result = collect_load_average().await;
            assert!(result.is_ok());
        }
    }
}

//! Process Statistics Collection
//!
//! This module provides utilities for collecting real-time process statistics
//! for the Historian system, including CPU and memory usage tracking.

use anna_common::historian::{ProcessCpuInfo, ProcessMemoryInfo};
use sysinfo::System;

/// Collect top N CPU-consuming processes
pub fn get_top_cpu_processes(limit: usize) -> Vec<ProcessCpuInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Wait a bit for CPU usage to be calculated
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_all();

    let mut process_cpu: Vec<(String, f64, usize)> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let name = process.name().to_string();
            let cpu_usage = process.cpu_usage() as f64;
            let occurrences = 1; // Single sample
            (name, cpu_usage, occurrences)
        })
        .filter(|(_, cpu, _)| *cpu > 0.1) // Filter out idle processes
        .collect();

    // Sort by CPU usage descending
    process_cpu.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    process_cpu
        .into_iter()
        .take(limit)
        .map(|(name, cpu, _occ)| ProcessCpuInfo {
            name,
            cpu_percent: cpu,
            cumulative_time_ms: 0, // Not tracked in single sample
        })
        .collect()
}

/// Collect top N memory-consuming processes
pub fn get_top_memory_processes(limit: usize) -> Vec<ProcessMemoryInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut process_mem: Vec<(String, i64, usize)> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let name = process.name().to_string();
            let mem_mb = (process.memory() / 1024 / 1024) as i64;
            let occurrences = 1; // Single sample
            (name, mem_mb, occurrences)
        })
        .filter(|(_, mem, _)| *mem > 10) // Filter out tiny processes
        .collect();

    // Sort by memory usage descending
    process_mem.sort_by(|a, b| b.1.cmp(&a.1));

    process_mem
        .into_iter()
        .take(limit)
        .map(|(name, mem, _occ)| ProcessMemoryInfo { name, rss_mb: mem })
        .collect()
}

/// Calculate overall CPU utilization percentage
pub fn get_cpu_utilization() -> (f64, f64) {
    let mut sys = System::new_all();
    sys.refresh_cpu();

    // Wait for CPU measurements
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu();

    let cpus = sys.cpus();
    if cpus.is_empty() {
        return (0.0, 0.0);
    }

    let total: f64 = cpus.iter().map(|cpu| cpu.cpu_usage() as f64).sum();
    let avg = total / cpus.len() as f64;
    let peak = cpus
        .iter()
        .map(|cpu| cpu.cpu_usage() as f64)
        .fold(0.0, f64::max);

    (avg, peak)
}

/// Get boot time from systemd-analyze
pub fn get_boot_duration_ms() -> Option<i64> {
    let output = std::process::Command::new("systemd-analyze")
        .arg("time")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Parse output like: "Startup finished in 2.345s (kernel) + 5.678s (userspace) = 8.023s"
    // We want the total time
    for line in output_str.lines() {
        if line.contains("Startup finished") {
            // Extract the total time (last number before 's')
            if let Some(total_part) = line.split('=').nth(1) {
                if let Some(time_str) = total_part.trim().split('s').next() {
                    if let Ok(seconds) = time_str.parse::<f64>() {
                        return Some((seconds * 1000.0) as i64);
                    }
                }
            }
        }
    }

    None
}

/// Get slowest systemd units from systemd-analyze blame
pub fn get_slowest_units(limit: usize) -> Vec<(String, i64)> {
    let output = std::process::Command::new("systemd-analyze")
        .arg("blame")
        .output()
        .ok();

    let output = match output {
        Some(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut units = Vec::new();

    for line in output_str.lines().take(limit) {
        // Format: "  5.234s some-service.service"
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let time_str = parts[0].trim_end_matches('s').trim_end_matches("ms");
            let unit_name = parts[1..].join(" ");

            // Parse time (could be in seconds or milliseconds)
            if let Ok(time) = time_str.parse::<f64>() {
                let time_ms = if parts[0].ends_with("ms") {
                    time as i64
                } else {
                    (time * 1000.0) as i64
                };
                units.push((unit_name, time_ms));
            }
        }
    }

    units
}

/// Detect CPU spikes by comparing current usage to recent average
/// Returns number of detected spikes in recent samples
pub fn detect_cpu_spikes(current_cpu: f64, history: &[f64], threshold_multiplier: f64) -> usize {
    if history.is_empty() {
        return 0;
    }

    let avg: f64 = history.iter().sum::<f64>() / history.len() as f64;
    let spike_threshold = avg * threshold_multiplier;

    history.iter().filter(|&&cpu| cpu > spike_threshold).count()
}

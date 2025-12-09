//! System snapshot collector loop (v0.0.266).
//!
//! Periodically collects system state (disk, memory, services, etc.)
//! and saves it for fast path queries.

use anna_shared::snapshot::{save_snapshot, SystemSnapshot};
use std::process::Command;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Snapshot collection interval in seconds
const SNAPSHOT_INTERVAL: u64 = 60;

/// Run the snapshot collection loop
pub async fn snapshot_loop() {
    // Collect initial snapshot immediately
    if let Err(e) = collect_and_save_snapshot() {
        warn!("Initial snapshot collection failed: {}", e);
    } else {
        info!("Initial system snapshot collected");
    }

    let mut interval = interval(Duration::from_secs(SNAPSHOT_INTERVAL));

    loop {
        interval.tick().await;

        if let Err(e) = collect_and_save_snapshot() {
            error!("Snapshot collection failed: {}", e);
        } else {
            debug!("System snapshot updated");
        }
    }
}

/// Collect system snapshot and save it
fn collect_and_save_snapshot() -> anyhow::Result<()> {
    let mut snapshot = SystemSnapshot::now();

    // Collect disk usage
    collect_disk_usage(&mut snapshot);

    // Collect memory usage
    collect_memory_usage(&mut snapshot);

    // Collect failed services
    collect_failed_services(&mut snapshot);

    // Collect boot time
    collect_boot_time(&mut snapshot);

    // Collect load averages
    collect_load_average(&mut snapshot);

    // Collect network status
    collect_network_status(&mut snapshot);

    // Save snapshot
    save_snapshot(&snapshot)?;

    Ok(())
}

/// Collect disk usage from df -h
fn collect_disk_usage(snapshot: &mut SystemSnapshot) {
    let output = Command::new("df")
        .args(["-h", "--output=pcent,target"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let percent_str = parts[0].trim_end_matches('%');
                    let mount = parts[1];

                    // Only track relevant mounts
                    if mount == "/" || mount == "/home" || mount == "/var" || mount == "/tmp"
                        || mount.starts_with("/mnt") || mount.starts_with("/media")
                    {
                        if let Ok(pct) = percent_str.parse::<u8>() {
                            snapshot.add_disk(mount, pct);
                        }
                    }
                }
            }
        }
    }
}

/// Collect memory usage from free -b
fn collect_memory_usage(snapshot: &mut SystemSnapshot) {
    let output = Command::new("free").args(["-b"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let (Ok(total), Ok(used)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                            snapshot.set_memory(total, used);
                            return;
                        }
                    }
                }
            }
        }
    }
}

/// Collect failed systemd services
fn collect_failed_services(snapshot: &mut SystemSnapshot) {
    let output = Command::new("systemctl")
        .args(["--failed", "--no-pager", "-q", "--plain"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.ends_with(".service") || part.ends_with(".socket") || part.ends_with(".mount") {
                        snapshot.add_failed_service(part);
                        break;
                    }
                }
            }
        }
    }
}

/// Collect boot time from uptime -s
fn collect_boot_time(snapshot: &mut SystemSnapshot) {
    let output = Command::new("uptime").args(["-s"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Parse "2024-12-05 10:15:32" format
            let parts: Vec<&str> = stdout.split(&['-', ' ', ':'][..]).collect();
            if parts.len() >= 6 {
                if let (Ok(y), Ok(mon), Ok(d), Ok(h), Ok(m), Ok(s)) = (
                    parts[0].parse::<i64>(),
                    parts[1].parse::<u32>(),
                    parts[2].parse::<u32>(),
                    parts[3].parse::<u32>(),
                    parts[4].parse::<u32>(),
                    parts[5].parse::<u32>(),
                ) {
                    // Approximate Unix timestamp calculation
                    let days_since_epoch = days_from_year(y) + days_from_month(mon, is_leap_year(y)) + d - 1;
                    let secs = (days_since_epoch as u64 * 86400)
                        + (h as u64 * 3600)
                        + (m as u64 * 60)
                        + s as u64;
                    snapshot.boot_time_secs = secs;
                }
            }
        }
    }
}

/// Collect load average from uptime
fn collect_load_average(snapshot: &mut SystemSnapshot) {
    let output = Command::new("uptime").output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(load_idx) = stdout.find("load average:") {
                let load_part = &stdout[load_idx + 13..];
                let loads: Vec<&str> = load_part.split(',').collect();
                if loads.len() >= 3 {
                    if let Ok(l1) = loads[0].trim().parse::<f32>() {
                        snapshot.load_1min = l1;
                    }
                    if let Ok(l5) = loads[1].trim().parse::<f32>() {
                        snapshot.load_5min = l5;
                    }
                    if let Ok(l15) = loads[2].trim().parse::<f32>() {
                        snapshot.load_15min = l15;
                    }
                }
            }
        }
    }
}

/// Collect network status from ip addr
fn collect_network_status(snapshot: &mut SystemSnapshot) {
    let output = Command::new("ip").args(["addr", "show"]).output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut has_carrier = false;

            for line in stdout.lines() {
                let line = line.trim();
                // Look for inet addresses (skip loopback)
                if line.starts_with("inet ") && !line.contains("127.0.0.1") {
                    if let Some(ip_with_mask) = line.split_whitespace().nth(1) {
                        let ip = ip_with_mask.split('/').next().unwrap_or(ip_with_mask);
                        if !snapshot.ip_addresses.contains(&ip.to_string()) {
                            snapshot.ip_addresses.push(ip.to_string());
                            has_carrier = true;
                        }
                    }
                }
                if line.contains("state UP") {
                    has_carrier = true;
                }
            }
            snapshot.network_connected = has_carrier;
        }
    }
}

// Helper functions for date calculation
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_year(year: i64) -> u32 {
    let mut days = 0u32;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    days
}

fn days_from_month(month: u32, leap: bool) -> u32 {
    const DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut d = 0;
    for m in 1..month {
        d += DAYS[(m - 1) as usize];
        if m == 2 && leap {
            d += 1;
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_and_save_snapshot() {
        let result = collect_and_save_snapshot();
        // Should succeed (even if some commands fail)
        assert!(result.is_ok());
    }
}

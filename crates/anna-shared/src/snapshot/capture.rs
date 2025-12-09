//! Snapshot capture from probe results (v0.0.259).
//!
//! v0.0.259: Added uptime and network capture.

use crate::rpc::ProbeResult;

use super::types::SystemSnapshot;

/// Capture snapshot from probe results
pub fn capture_snapshot(probes: &[ProbeResult]) -> SystemSnapshot {
    let mut snapshot = SystemSnapshot::now();

    for probe in probes {
        if probe.exit_code != 0 {
            continue; // Skip failed probes
        }

        // Parse df output for disk usage
        if probe.command.contains("df") {
            parse_df_into_snapshot(&probe.stdout, &mut snapshot);
        }

        // Parse free output for memory
        if probe.command.contains("free") {
            parse_free_into_snapshot(&probe.stdout, &mut snapshot);
        }

        // Parse systemctl --failed for failed services
        if probe.command.contains("--failed") {
            parse_failed_services_into_snapshot(&probe.stdout, &mut snapshot);
        }

        // v0.0.259: Parse uptime output for load average and boot time
        if probe.command.contains("uptime") && !probe.command.contains("-s") {
            parse_uptime_into_snapshot(&probe.stdout, &mut snapshot);
        }

        // v0.0.259: Parse uptime -s for boot time
        if probe.command.contains("uptime -s") {
            parse_boot_time_into_snapshot(&probe.stdout, &mut snapshot);
        }

        // v0.0.259: Parse ip addr for network info
        if probe.command.contains("ip addr") || probe.command.contains("ip a ") {
            parse_ip_addr_into_snapshot(&probe.stdout, &mut snapshot);
        }
    }

    snapshot
}

/// Parse df -h output into snapshot
fn parse_df_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // df output format: Filesystem Size Used Avail Use% Mounted
    for line in output.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let use_percent = parts[4].trim_end_matches('%');
            let mount = parts[5];

            // Only track relevant mounts
            if mount == "/"
                || mount == "/home"
                || mount == "/var"
                || mount == "/tmp"
                || mount.starts_with("/mnt")
                || mount.starts_with("/media")
            {
                if let Ok(pct) = use_percent.parse::<u8>() {
                    snapshot.add_disk(mount, pct);
                }
            }
        }
    }
}

/// Parse free -b output into snapshot
fn parse_free_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // Try to find "Mem:" line
    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                // Format: Mem: total used free...
                if let (Ok(total), Ok(used)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                    snapshot.set_memory(total, used);
                    return;
                }
            }
        }
    }
}

/// Parse systemctl --failed output into snapshot
fn parse_failed_services_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // systemctl --failed output has units that are in failed state
    // Format: [●] UNIT LOAD ACTIVE SUB DESCRIPTION
    // The bullet point (●) may appear before the unit name
    for line in output.lines() {
        let line = line.trim();
        // Skip header and summary lines
        if line.is_empty()
            || line.starts_with("UNIT")
            || line.starts_with("LOAD")
            || line.contains("loaded units listed")
            || line.contains("0 loaded units")
        {
            continue;
        }

        // Extract unit name - handle bullet point prefix (●)
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            // Skip the bullet point and empty strings
            if part == "●" || part.is_empty() {
                continue;
            }
            // Found the unit name
            if part.ends_with(".service") || part.ends_with(".socket") || part.ends_with(".timer") {
                snapshot.add_failed_service(part);
                break; // Only take the first matching unit per line
            }
        }
    }
}

/// v0.0.259: Parse uptime output for load average
fn parse_uptime_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // uptime output: " 14:32:01 up 3 days, 2:15, 1 user, load average: 0.52, 0.58, 0.59"
    if let Some(load_idx) = output.find("load average:") {
        let load_part = &output[load_idx + 13..]; // after "load average:"
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

/// v0.0.259: Parse uptime -s output for boot time
fn parse_boot_time_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    // uptime -s output: "2024-12-05 10:15:32"
    // We'll parse manually to avoid chrono dependency
    let line = output.trim();
    let parts: Vec<&str> = line.split(&['-', ' ', ':'][..]).collect();
    if parts.len() >= 6 {
        if let (Ok(y), Ok(mon), Ok(d), Ok(h), Ok(m), Ok(s)) = (
            parts[0].parse::<i64>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
            parts[3].parse::<u32>(),
            parts[4].parse::<u32>(),
            parts[5].parse::<u32>(),
        ) {
            // Simple Unix timestamp calculation (approximate, ignores leap seconds)
            let days_since_epoch = days_from_year(y) + days_from_month(mon, is_leap_year(y)) + d - 1;
            let secs = (days_since_epoch as u64 * 86400)
                + (h as u64 * 3600)
                + (m as u64 * 60)
                + s as u64;
            snapshot.boot_time_secs = secs;
        }
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_year(year: i64) -> u32 {
    // Days from 1970 to start of year
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

/// v0.0.259: Parse ip addr output for network info
fn parse_ip_addr_into_snapshot(output: &str, snapshot: &mut SystemSnapshot) {
    let mut has_carrier = false;
    for line in output.lines() {
        let line = line.trim();
        // Look for inet addresses (skip loopback)
        if line.starts_with("inet ") && !line.contains("127.0.0.1") {
            // Format: inet 192.168.1.5/24 brd 192.168.1.255 scope global
            if let Some(ip_with_mask) = line.split_whitespace().nth(1) {
                let ip = ip_with_mask.split('/').next().unwrap_or(ip_with_mask);
                if !snapshot.ip_addresses.contains(&ip.to_string()) {
                    snapshot.ip_addresses.push(ip.to_string());
                    has_carrier = true;
                }
            }
        }
        // Look for state UP
        if line.contains("state UP") {
            has_carrier = true;
        }
    }
    snapshot.network_connected = has_carrier;
}

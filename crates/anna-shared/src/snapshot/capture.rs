//! Snapshot capture from probe results (v0.0.219).

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

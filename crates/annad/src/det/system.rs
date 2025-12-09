//! System answer functions (v0.0.171).
//!
//! Handles package updates, swap, timezone, uptime, users, battery, load, boot,
//! hostname, OS info, network connectivity, filesystems, and USB devices.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer package updates query using checkupdates probe
pub fn answer_package_updates(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "checkupdates").or_else(|| find_probe(probes, "pacman"));
    let probe = probe?;

    let output = probe.stdout.trim();

    if output.is_empty() || probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "No package updates available. Your system is up to date.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let update_count = output.lines().count();
    let preview: Vec<&str> = output.lines().take(5).collect();
    let preview_str = preview.join("\n  ");

    let answer = if update_count == 1 {
        format!("1 package update available:\n  {}", preview_str)
    } else if update_count <= 5 {
        format!("{} package updates available:\n  {}", update_count, preview_str)
    } else {
        format!(
            "{} package updates available:\n  {}\n  ...and {} more",
            update_count, preview_str, update_count - 5
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: update_count,
        route_class: route_class.to_string(),
    })
}

/// Answer swap info query using free probe
pub fn answer_swap_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    if probe.exit_code != 0 {
        return None;
    }

    for line in probe.stdout.lines() {
        if line.starts_with("Swap:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let total = parts[1];
                let used = parts[2];
                let free = parts[3];
                let answer = format!("Swap: {} total, {} used, {} free", total, used, free);
                return Some(DeterministicResult {
                    answer,
                    grounded: true,
                    parsed_data_count: 1,
                    route_class: route_class.to_string(),
                });
            }
        }
    }

    Some(DeterministicResult {
        answer: "No swap space is configured on this system.".to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer timezone info query using timedatectl probe
pub fn answer_timezone_info(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "timedatectl")?;
    if probe.exit_code != 0 {
        return None;
    }

    let mut timezone = String::new();
    let mut local_time = String::new();
    let mut ntp_status = String::new();

    for line in probe.stdout.lines() {
        let line = line.trim();
        if line.starts_with("Time zone:") {
            timezone = line.strip_prefix("Time zone:").unwrap_or("").trim().to_string();
        } else if line.starts_with("Local time:") {
            local_time = line.strip_prefix("Local time:").unwrap_or("").trim().to_string();
        } else if line.starts_with("NTP service:") || line.starts_with("System clock synchronized:") {
            ntp_status = line.to_string();
        }
    }

    let mut answer = String::new();
    if !timezone.is_empty() {
        answer.push_str(&format!("Timezone: {}\n", timezone));
    }
    if !local_time.is_empty() {
        answer.push_str(&format!("Local time: {}\n", local_time));
    }
    if !ntp_status.is_empty() {
        answer.push_str(&ntp_status);
    }

    if answer.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: answer.trim().to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system uptime query using uptime probe
pub fn answer_system_uptime(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "uptime")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let answer = format!("System has been {}", output);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer logged in users query using who command
pub fn answer_logged_in_users(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "who")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No users currently logged in.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let sessions: Vec<&str> = output.lines().collect();
    let user_count = sessions.len();

    let unique_users: std::collections::HashSet<&str> = sessions
        .iter()
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    let answer = if unique_users.len() == 1 && user_count == 1 {
        format!("1 user logged in: {}", unique_users.iter().next().unwrap_or(&"unknown"))
    } else if unique_users.len() == 1 {
        format!("{} sessions for user: {}", user_count, unique_users.iter().next().unwrap_or(&"unknown"))
    } else {
        format!(
            "{} users logged in ({} sessions): {}",
            unique_users.len(),
            user_count,
            unique_users.into_iter().collect::<Vec<_>>().join(", ")
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: user_count,
        route_class: route_class.to_string(),
    })
}

/// Answer battery status query using upower or /sys
pub fn answer_battery_status(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "battery")?;
    let output = probe.stdout.trim();

    if output.is_empty() || probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "No battery detected. This may be a desktop system.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    if output.contains("percentage:") {
        let mut percentage = String::new();
        let mut state = String::new();
        let mut time_to_empty = String::new();
        let mut time_to_full = String::new();

        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("percentage:") {
                percentage = line.strip_prefix("percentage:").unwrap_or("").trim().to_string();
            } else if line.starts_with("state:") {
                state = line.strip_prefix("state:").unwrap_or("").trim().to_string();
            } else if line.starts_with("time to empty:") {
                time_to_empty = line.strip_prefix("time to empty:").unwrap_or("").trim().to_string();
            } else if line.starts_with("time to full:") {
                time_to_full = line.strip_prefix("time to full:").unwrap_or("").trim().to_string();
            }
        }

        let mut answer = format!("Battery: {}", percentage);
        if !state.is_empty() {
            answer.push_str(&format!(" ({})", state));
        }
        if !time_to_empty.is_empty() {
            answer.push_str(&format!("\nTime remaining: {}", time_to_empty));
        }
        if !time_to_full.is_empty() {
            answer.push_str(&format!("\nTime to full: {}", time_to_full));
        }

        return Some(DeterministicResult {
            answer,
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    if let Ok(pct) = output.parse::<u32>() {
        let status = if pct > 80 { "Good" } else if pct > 20 { "OK" } else { "Low" };
        return Some(DeterministicResult {
            answer: format!("Battery: {}% ({})", pct, status),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("Battery info: {}", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer system load query using /proc/loadavg
pub fn answer_system_load(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "load_average")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() >= 3 {
        let answer = format!(
            "System load averages:\n  1 min:  {}\n  5 min:  {}\n  15 min: {}",
            parts[0], parts[1], parts[2]
        );
        return Some(DeterministicResult {
            answer,
            grounded: true,
            parsed_data_count: 3,
            route_class: route_class.to_string(),
        });
    }

    None
}

/// Answer last boot query using who -b
pub fn answer_last_boot(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "last_boot")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let boot_time = output
        .strip_prefix("system boot")
        .or_else(|| output.split("system boot").nth(1))
        .map(|s| s.trim())
        .unwrap_or(output);

    let answer = format!("System last booted: {}", boot_time);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer hostname query using hostname command
pub fn answer_hostname(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "hostname")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: format!("Hostname: {}", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer OS info query using /etc/os-release
pub fn answer_os_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "os_release")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let mut name = String::new();
    let mut version = String::new();
    let mut pretty_name = String::new();

    for line in output.lines() {
        if line.starts_with("PRETTY_NAME=") {
            pretty_name = line.strip_prefix("PRETTY_NAME=").unwrap_or("").trim_matches('"').to_string();
        } else if line.starts_with("NAME=") {
            name = line.strip_prefix("NAME=").unwrap_or("").trim_matches('"').to_string();
        } else if line.starts_with("VERSION=") {
            version = line.strip_prefix("VERSION=").unwrap_or("").trim_matches('"').to_string();
        }
    }

    let answer = if !pretty_name.is_empty() {
        format!("OS: {}", pretty_name)
    } else if !name.is_empty() && !version.is_empty() {
        format!("OS: {} {}", name, version)
    } else if !name.is_empty() {
        format!("OS: {}", name)
    } else {
        return None;
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer network connectivity query using ping
pub fn answer_network_connectivity(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ping_check")?;

    let answer = if probe.exit_code == 0 {
        let output = probe.stdout.trim();
        let latency = output
            .lines()
            .find(|line| line.contains("time="))
            .and_then(|line| line.split("time=").nth(1).and_then(|s| s.split_whitespace().next()));

        if let Some(lat) = latency {
            format!("Online - ping to 8.8.8.8: {} ms", lat)
        } else {
            "Online - network connectivity confirmed".to_string()
        }
    } else {
        "Offline - cannot reach 8.8.8.8 (Google DNS)".to_string()
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer mounted filesystems query using findmnt
pub fn answer_mounted_filesystems(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "findmnt")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No mounted filesystems found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let lines: Vec<&str> = output.lines().collect();
    let mount_count = lines.len().saturating_sub(1);

    Some(DeterministicResult {
        answer: format!("Mounted filesystems ({}):\n{}", mount_count, output),
        grounded: true,
        parsed_data_count: mount_count,
        route_class: route_class.to_string(),
    })
}

/// Answer USB devices query using lsusb
pub fn answer_usb_devices(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "lsusb")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No USB devices detected.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let device_count = output.lines().count();

    let devices: Vec<String> = output
        .lines()
        .filter_map(|line| {
            line.split(": ").nth(1).map(|s| {
                if let Some(pos) = s.find(' ') {
                    s[pos + 1..].trim().to_string()
                } else {
                    s.to_string()
                }
            })
        })
        .collect();

    let answer = if device_count <= 10 {
        format!("USB devices ({}):\n  {}", device_count, devices.join("\n  "))
    } else {
        let preview: Vec<&str> = devices.iter().take(8).map(|s| s.as_str()).collect();
        format!(
            "USB devices ({}):\n  {}\n  ...and {} more",
            device_count,
            preview.join("\n  "),
            device_count - 8
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

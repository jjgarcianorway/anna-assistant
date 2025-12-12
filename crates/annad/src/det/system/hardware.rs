//! Hardware-related answer functions (v0.0.187).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

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
                percentage = line
                    .strip_prefix("percentage:")
                    .unwrap_or("")
                    .trim()
                    .to_string();
            } else if line.starts_with("state:") {
                state = line.strip_prefix("state:").unwrap_or("").trim().to_string();
            } else if line.starts_with("time to empty:") {
                time_to_empty = line
                    .strip_prefix("time to empty:")
                    .unwrap_or("")
                    .trim()
                    .to_string();
            } else if line.starts_with("time to full:") {
                time_to_full = line
                    .strip_prefix("time to full:")
                    .unwrap_or("")
                    .trim()
                    .to_string();
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
        let status = if pct > 80 {
            "Good"
        } else if pct > 20 {
            "OK"
        } else {
            "Low"
        };
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
        format!(
            "USB devices ({}):\n  {}",
            device_count,
            devices.join("\n  ")
        )
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

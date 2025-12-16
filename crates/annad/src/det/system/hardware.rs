//! Hardware-related answer functions (v0.0.805).

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

/// v0.0.802: Answer webcam/camera status query
pub fn answer_webcam_status(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "webcam_devices")?;
    let output = probe.stdout.trim();

    // Check for "NO_WEBCAM_FOUND" marker
    if output.contains("NO_WEBCAM_FOUND") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "**No webcam detected**. Check if camera is connected or drivers are loaded."
                .to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    // Parse lsusb output for camera devices
    let mut cameras: Vec<String> = Vec::new();

    for line in output.lines() {
        let line_lower = line.to_lowercase();
        // lsusb format: Bus 001 Device 002: ID 0c45:636b Microdia Integrated Webcam
        if line_lower.contains("webcam")
            || line_lower.contains("camera")
            || line_lower.contains("video")
            || line_lower.contains("cam")
        {
            // Extract the device name part after the ID
            if let Some(pos) = line.find(": ID ") {
                let after_id = &line[pos + 5..];
                if let Some(name_pos) = after_id.find(' ') {
                    cameras.push(after_id[name_pos + 1..].trim().to_string());
                } else {
                    cameras.push(after_id.to_string());
                }
            } else {
                cameras.push(line.to_string());
            }
        }
    }

    // Also check for /dev/video* entries
    for line in output.lines() {
        if line.starts_with("/dev/video") {
            cameras.push(format!("Video device: {}", line.trim()));
        }
    }

    if cameras.is_empty() {
        return Some(DeterministicResult {
            answer: "**No webcam detected**. The USB scan found no camera devices.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let answer = if cameras.len() == 1 {
        format!("**Webcam detected:** {}", cameras[0])
    } else {
        format!(
            "**Webcams detected ({}):**\n  {}",
            cameras.len(),
            cameras.join("\n  ")
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: cameras.len(),
        route_class: route_class.to_string(),
    })
}

/// v0.0.805: Answer screen/display resolution query
pub fn answer_screen_resolution(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "xrandr")?;
    let output = probe.stdout.trim();

    if output.contains("DISPLAY_INFO_NOT_AVAILABLE") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "**Display info not available**. No display server detected (Xorg/Wayland)."
                .to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let mut monitors: Vec<String> = Vec::new();

    // Parse xrandr output
    // Format: DP-1 connected primary 2560x1440+0+0 (normal left inverted right x axis y axis) 597mm x 336mm
    for line in output.lines() {
        if line.contains(" connected") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let name = parts[0];
                let mut resolution = "unknown";
                let mut is_primary = false;

                for part in parts.iter() {
                    if *part == "primary" {
                        is_primary = true;
                    }
                    // Resolution pattern: 2560x1440+0+0 or 1920x1080+2560+0
                    if part.contains('x') && part.contains('+') {
                        resolution = part.split('+').next().unwrap_or(part);
                    }
                }

                let primary_str = if is_primary { " (primary)" } else { "" };
                monitors.push(format!("**{}**{}: {}", name, primary_str, resolution));
            }
        }
    }

    // Also parse wlr-randr/hyprctl output if it's Wayland
    if monitors.is_empty() {
        for line in output.lines() {
            // hyprctl monitors format: Monitor DP-1 (ID 0):
            if line.contains("Monitor ") && line.contains("(ID") {
                let name = line
                    .split("Monitor ")
                    .nth(1)
                    .and_then(|s| s.split(" (ID").next())
                    .unwrap_or("unknown");
                monitors.push(format!("**{}**", name));
            }
            // Resolution line from hyprctl: 2560x1440@143.97800 at 0x0
            if line.trim().contains('@') && line.contains('x') && !line.contains("Monitor") {
                let trimmed = line.trim();
                if let Some(pos) = trimmed.find('@') {
                    let resolution = &trimmed[..pos];
                    if !monitors.is_empty() {
                        let last = monitors.pop().unwrap();
                        monitors.push(format!("{}: {}", last, resolution));
                    }
                }
            }
        }
    }

    if monitors.is_empty() {
        // Fallback: show raw output
        return Some(DeterministicResult {
            answer: format!("**Display info:**\n```\n{}\n```", output),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let answer = if monitors.len() == 1 {
        format!("**Display:** {}", monitors[0])
    } else {
        format!(
            "**Displays ({}):**\n  {}",
            monitors.len(),
            monitors.join("\n  ")
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: monitors.len(),
        route_class: route_class.to_string(),
    })
}

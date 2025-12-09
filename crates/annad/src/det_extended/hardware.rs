//! Hardware answer functions (v0.0.175).
//!
//! Battery, CPU frequency, memory slots, sensors, GPU.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer battery status query using upower or /sys
pub fn answer_battery_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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

/// Answer CPU frequency query
pub fn answer_cpu_frequency(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "cpu_frequency")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "CPU frequency information not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let freq = if let Some(mhz_line) = output.lines().find(|l| l.contains("MHz")) {
        if let Some(value) = mhz_line.split(':').nth(1) {
            let mhz: f64 = value.trim().parse().unwrap_or(0.0);
            if mhz > 1000.0 {
                format!("{:.2} GHz", mhz / 1000.0)
            } else {
                format!("{:.0} MHz", mhz)
            }
        } else {
            output.to_string()
        }
    } else {
        output.to_string()
    };

    Some(DeterministicResult {
        answer: format!("CPU frequency: {}", freq),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer memory slots query
pub fn answer_memory_slots(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "memory_slots")?;

    let output = probe.stdout.trim();
    if output.contains("Requires root") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Memory slot information requires root access (sudo dmidecode).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("Memory slots:\n{}", output),
        grounded: true,
        parsed_data_count: output.lines().count(),
        route_class: route_class.to_string(),
    })
}

/// Answer sensors temperature query
pub fn answer_sensors_temp(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "sensors_temp")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "lm-sensors is not installed. Run `sudo sensors-detect` to set up hardware monitoring.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("Hardware sensors:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer GPU memory query
pub fn answer_gpu_memory(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "gpu_memory")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "nvidia-smi not available. This requires NVIDIA drivers to be installed.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("GPU memory usage:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer PCI devices query
pub fn answer_pci_devices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "pci_devices")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No PCI devices found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let device_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("PCI devices ({}):\n```\n{}\n```", device_count, output),
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

/// Answer USB devices query using lsusb
pub fn answer_usb_devices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
            device_count, preview.join("\n  "), device_count - 8
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: device_count,
        route_class: route_class.to_string(),
    })
}

/// Answer CPU governor query using cpufreq
pub fn answer_cpu_governor(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "cpu_governor")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.contains("not available") || output.is_empty() {
        ("CPU frequency scaling not available on this system.".to_string(), 0)
    } else {
        let governors: Vec<&str> = output.lines().collect();
        let summary: Vec<String> = governors
            .iter()
            .map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!("{} cores: {}", parts[0], parts[1])
                } else {
                    line.to_string()
                }
            })
            .collect();
        (format!("CPU scaling governors:\n  {}", summary.join("\n  ")), governors.len())
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

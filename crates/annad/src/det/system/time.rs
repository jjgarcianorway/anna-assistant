//! Time-related answer functions (v0.0.187).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

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
            timezone = line
                .strip_prefix("Time zone:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if line.starts_with("Local time:") {
            local_time = line
                .strip_prefix("Local time:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if line.starts_with("NTP service:") || line.starts_with("System clock synchronized:")
        {
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

/// v0.0.799: Answer boot blame query - "why is my boot slow?"
/// Uses boot_time and boot_blame probes
pub fn answer_boot_blame(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    // Get boot time from systemd-analyze (command prefix, not probe name)
    // boot_time probe -> "systemd-analyze"
    // boot_blame probe -> "systemd-analyze blame"
    let boot_time_probe = find_probe(probes, "systemd-analyze")
        .filter(|p| !p.command.contains("blame")); // Exclude blame variant
    let boot_blame_probe = find_probe(probes, "systemd-analyze blame");

    let mut answer = String::new();

    // Parse boot time (e.g., "Startup finished in 2.5s (kernel) + 5.3s (userspace) = 7.8s")
    if let Some(probe) = boot_time_probe {
        if probe.exit_code == 0 {
            let output = probe.stdout.trim();
            if !output.is_empty() {
                // Extract total time and components
                if output.contains("Startup finished") {
                    answer.push_str(&format!("**Boot Time Analysis**\n{}\n\n", output));
                }
            }
        }
    }

    // Parse boot blame (slowest services)
    if let Some(probe) = boot_blame_probe {
        if probe.exit_code == 0 {
            let output = probe.stdout.trim();
            if !output.is_empty() {
                answer.push_str("**Slowest Services (systemd-analyze blame)**\n");
                // Parse each line: "XXXms service-name.service"
                let mut count = 0;
                for line in output.lines() {
                    let line = line.trim();
                    if line.is_empty() || count >= 10 {
                        continue;
                    }
                    // Format: "1.234s service.service" or "123ms service.service"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let time = parts[0];
                        let service = parts[1];
                        answer.push_str(&format!("- {} {}\n", time, service));
                        count += 1;
                    }
                }
            }
        }
    }

    if answer.is_empty() {
        return None;
    }

    Some(DeterministicResult {
        answer: answer.trim().to_string(),
        grounded: true,
        parsed_data_count: 2,
        route_class: route_class.to_string(),
    })
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

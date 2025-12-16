//! System info answer functions (v0.0.801).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

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
            pretty_name = line
                .strip_prefix("PRETTY_NAME=")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        } else if line.starts_with("NAME=") {
            name = line
                .strip_prefix("NAME=")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
        } else if line.starts_with("VERSION=") {
            version = line
                .strip_prefix("VERSION=")
                .unwrap_or("")
                .trim_matches('"')
                .to_string();
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

/// v0.0.801: Answer device type query (laptop vs desktop) using hostnamectl
pub fn answer_device_type(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "hostnamectl")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.to_lowercase();

    // Look for "Chassis:" line in hostnamectl output
    let chassis = output
        .lines()
        .find(|line| line.trim().starts_with("chassis:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim().to_string());

    let answer = match chassis.as_deref() {
        Some("laptop") | Some("notebook") | Some("convertible") | Some("tablet") => {
            "This is a **laptop** (portable device with battery).".to_string()
        }
        Some("desktop") | Some("tower") | Some("mini-tower") | Some("all-in-one") => {
            "This is a **desktop** computer (stationary workstation).".to_string()
        }
        Some("server") => "This is a **server** (rack-mounted or standalone server).".to_string(),
        Some("handset") => "This is a **handheld** device.".to_string(),
        Some("vm") | Some("container") => {
            "This is a **virtual machine** or container (virtualized environment).".to_string()
        }
        Some(other) => format!("Device type: **{}**", other),
        None => {
            // Fallback: check for battery as indicator of laptop
            let has_battery = std::path::Path::new("/sys/class/power_supply/BAT0").exists()
                || std::path::Path::new("/sys/class/power_supply/BAT1").exists();
            if has_battery {
                "This appears to be a **laptop** (battery detected).".to_string()
            } else {
                "Unable to determine device type from system information.".to_string()
            }
        }
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

//! Host-related answer functions (v0.0.212).

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

/// Answer system architecture query using uname -m
pub fn answer_system_architecture(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "arch")?;
    if probe.exit_code != 0 {
        return None;
    }

    let arch = probe.stdout.trim();
    if arch.is_empty() {
        return None;
    }

    let desc = match arch {
        "x86_64" => "64-bit x86 (AMD64/Intel64)",
        "i686" | "i386" => "32-bit x86",
        "aarch64" => "64-bit ARM",
        "armv7l" => "32-bit ARM (ARMv7)",
        "riscv64" => "64-bit RISC-V",
        _ => arch,
    };

    Some(DeterministicResult {
        answer: format!("Architecture: {} ({})", arch, desc),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

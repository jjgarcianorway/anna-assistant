//! Services and user answer functions (v0.0.171).
//!
//! Handles listening ports, running services, current user, architecture, and environment.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer listening ports query using ss
/// v0.0.793: Fixed to search for "ss" command instead of probe ID "listening_ports"
pub fn answer_listening_ports(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    // v0.0.793: The probe result command field stores "ss -tulpn", not "listening_ports"
    let probe = find_probe(probes, "ss")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No listening ports found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let lines: Vec<&str> = output.lines().collect();
    let port_count = lines.len().saturating_sub(1);

    Some(DeterministicResult {
        answer: format!("Listening ports ({}):\n{}", port_count, output),
        grounded: true,
        parsed_data_count: port_count,
        route_class: route_class.to_string(),
    })
}

/// Answer running services query using systemctl
pub fn answer_running_services(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "running_services")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No running services found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let service_count = output.lines().count();
    let services: Vec<&str> = output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    let answer = if service_count <= 15 {
        format!(
            "Running services ({}):\n  {}",
            service_count,
            services.join("\n  ")
        )
    } else {
        let preview: Vec<&str> = services.iter().take(12).copied().collect();
        format!(
            "Running services ({}):\n  {}\n  ...and {} more",
            service_count,
            preview.join("\n  "),
            service_count - 12
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: service_count,
        route_class: route_class.to_string(),
    })
}

/// Answer current user query using id
pub fn answer_current_user(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "current_user")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return None;
    }

    let mut username = String::new();
    let mut uid = String::new();
    let mut groups = Vec::new();

    for part in output.split_whitespace() {
        if part.starts_with("uid=") {
            if let Some(name) = part.split('(').nth(1) {
                username = name.trim_end_matches(')').to_string();
            }
            if let Some(id) = part.strip_prefix("uid=") {
                uid = id.split('(').next().unwrap_or("").to_string();
            }
        } else if part.starts_with("groups=") {
            let grp = part.strip_prefix("groups=").unwrap_or("");
            for g in grp.split(',') {
                if let Some(name) = g.split('(').nth(1) {
                    groups.push(name.trim_end_matches(')').to_string());
                }
            }
        }
    }

    Some(DeterministicResult {
        answer: format!(
            "User: {} (uid={})\nGroups: {}",
            username,
            uid,
            groups.join(", ")
        ),
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

/// Answer environment variables query
pub fn answer_environment_vars(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "env_vars")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No environment variables found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let var_count = output.lines().count();
    let important_vars = [
        "PATH",
        "HOME",
        "USER",
        "SHELL",
        "TERM",
        "DISPLAY",
        "XDG_SESSION_TYPE",
    ];
    let mut key_vars = Vec::new();
    let mut other_count = 0;

    for line in output.lines() {
        let key = line.split('=').next().unwrap_or("");
        if important_vars.contains(&key) {
            key_vars.push(line);
        } else {
            other_count += 1;
        }
    }

    let answer = if !key_vars.is_empty() {
        format!(
            "Environment variables ({}):\n  {}\n  ...and {} others",
            var_count,
            key_vars.join("\n  "),
            other_count
        )
    } else {
        format!("Environment variables ({}):\n{}", var_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: var_count,
        route_class: route_class.to_string(),
    })
}

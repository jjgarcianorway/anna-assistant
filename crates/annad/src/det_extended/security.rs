//! Security answer functions (v0.0.175).
//!
//! Firewall, SELinux, AppArmor, logins, sudoers, SSH, iptables.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer firewall status query
pub fn answer_firewall_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "firewall_status")?;

    let output = probe.stdout.trim();
    if output.contains("No firewall detected") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No active firewall detected (iptables, nftables, or ufw).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let fw_type = if output.contains("Chain") {
        "iptables"
    } else if output.contains("table") && output.contains("chain") {
        "nftables"
    } else if output.contains("Status:") {
        "ufw"
    } else {
        "Unknown"
    };

    Some(DeterministicResult {
        answer: format!("Firewall ({}): Active\n```\n{}\n```", fw_type, output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer SSH connections query
pub fn answer_ssh_connections(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ssh_connections")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No active SSH connections.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let conn_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("SSH connections ({}):\n```\n{}\n```", conn_count, output),
        grounded: true,
        parsed_data_count: conn_count,
        route_class: route_class.to_string(),
    })
}

/// Answer last logins query
pub fn answer_last_logins(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "last_logins")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Login history not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let login_count = output.lines().filter(|l| !l.is_empty() && !l.starts_with("wtmp")).count();
    Some(DeterministicResult {
        answer: format!("Recent logins ({}):\n```\n{}\n```", login_count, output),
        grounded: true,
        parsed_data_count: login_count,
        route_class: route_class.to_string(),
    })
}

/// Answer failed logins query
pub fn answer_failed_logins(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "failed_logins")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No failed login attempts found (or data not available).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let failure_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Failed login attempts ({}):\n```\n{}\n```", failure_count, output),
        grounded: true,
        parsed_data_count: failure_count,
        route_class: route_class.to_string(),
    })
}

/// Answer sudoers info query
pub fn answer_sudoers_info(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "sudoers_info")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() || output.contains("password is required") {
        return Some(DeterministicResult {
            answer: "Sudo access information not available (may require password).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("Sudo access:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer SELinux status query
pub fn answer_selinux_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "selinux_status")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "SELinux is not installed on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("SELinux status:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer AppArmor status query
pub fn answer_apparmor_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "apparmor_status")?;

    let output = probe.stdout.trim();
    if output.contains("not installed") || output.is_empty() || output == "N" {
        return Some(DeterministicResult {
            answer: "AppArmor is not installed or not enabled on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    if output == "Y" {
        return Some(DeterministicResult {
            answer: "AppArmor is **enabled** on this system.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("AppArmor status:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer iptables rules query
pub fn answer_iptables_rules(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "iptables_rules")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.contains("requires root") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "iptables rules not available (may require root privileges).".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("iptables rules:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer sysctl settings query
pub fn answer_sysctl_settings(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "sysctl_settings")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No sysctl output available.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (format!("Sysctl settings ({} lines shown):\n```\n{}\n```", count, output), count)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

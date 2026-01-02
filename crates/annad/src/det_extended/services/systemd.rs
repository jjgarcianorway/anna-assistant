//! Systemd-related answer functions (v0.0.175).
//!
//! Handles systemd units, timers, sockets, targets, paths, slices, scopes, and journal.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

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

/// Answer systemd units query
pub fn answer_systemd_units(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_units")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd units found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let unit_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Systemd units ({}):\n```\n{}\n```", unit_count, output),
        grounded: true,
        parsed_data_count: unit_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd timers query
pub fn answer_systemd_timers(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_timers")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd timers found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let timer_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Systemd timers ({}):\n```\n{}\n```", timer_count, output),
        grounded: true,
        parsed_data_count: timer_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd journal query
pub fn answer_systemd_journal(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_journal")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Systemd journal not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let line_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!(
            "Recent system logs ({} entries):\n```\n{}\n```",
            line_count, output
        ),
        grounded: true,
        parsed_data_count: line_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd targets query
pub fn answer_systemd_targets(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_targets")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd targets found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let target_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!(
            "Active systemd targets ({}):\n```\n{}\n```",
            target_count, output
        ),
        grounded: true,
        parsed_data_count: target_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd sockets query
pub fn answer_systemd_sockets(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_sockets")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd sockets found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let socket_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Systemd sockets ({}):\n```\n{}\n```", socket_count, output),
        grounded: true,
        parsed_data_count: socket_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd slices query
pub fn answer_systemd_slices(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_slices")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Cgroup slice information not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("Systemd cgroup slices:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd paths query
pub fn answer_systemd_paths(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_paths")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No systemd path units found.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let path_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!("Systemd path units ({}):\n```\n{}\n```", path_count, output),
        grounded: true,
        parsed_data_count: path_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemctl mask query
pub fn answer_systemctl_mask(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemctl_mask")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No masked systemd units found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (
            format!("Masked units ({}):\n```\n{}\n```", count, output),
            count,
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd scopes query
pub fn answer_systemd_scopes(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_scopes")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No systemd scope units found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (
            format!("Systemd scopes ({}):\n```\n{}\n```", count, output),
            count,
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

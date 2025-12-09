//! Services answer functions (v0.0.175).
//!
//! Systemd units, timers, sockets, scopes, paths, docker, crontabs.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer running services query using systemctl
pub fn answer_running_services(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
        format!("Running services ({}):\n  {}", service_count, services.join("\n  "))
    } else {
        let preview: Vec<&str> = services.iter().take(12).copied().collect();
        format!(
            "Running services ({}):\n  {}\n  ...and {} more",
            service_count, preview.join("\n  "), service_count - 12
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
pub fn answer_systemd_units(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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

/// Answer crontabs query
pub fn answer_crontabs(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "crontabs")?;

    let output = probe.stdout.trim();
    if output.contains("No crontab") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No crontab entries for current user.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let job_count = output.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).count();
    Some(DeterministicResult {
        answer: format!("Crontab ({} jobs):\n```\n{}\n```", job_count, output),
        grounded: true,
        parsed_data_count: job_count,
        route_class: route_class.to_string(),
    })
}

/// Answer Docker containers query
pub fn answer_docker_containers(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "docker_containers")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Docker is not installed or not running.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let container_count = output.lines().count().saturating_sub(1);
    let answer = if container_count == 0 {
        "No running containers.".to_string()
    } else {
        format!("Docker containers ({}):\n```\n{}\n```", container_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: container_count,
        route_class: route_class.to_string(),
    })
}

/// Answer Docker images query
pub fn answer_docker_images(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "docker_images")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "Docker is not installed or not running.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let image_count = output.lines().count().saturating_sub(1);
    let answer = if image_count == 0 {
        "No Docker images found.".to_string()
    } else {
        format!("Docker images ({}):\n```\n{}\n```", image_count, output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: image_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd timers query
pub fn answer_systemd_timers(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
pub fn answer_systemd_journal(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
        answer: format!("Recent system logs ({} entries):\n```\n{}\n```", line_count, output),
        grounded: true,
        parsed_data_count: line_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd targets query
pub fn answer_systemd_targets(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
        answer: format!("Active systemd targets ({}):\n```\n{}\n```", target_count, output),
        grounded: true,
        parsed_data_count: target_count,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd sockets query
pub fn answer_systemd_sockets(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
pub fn answer_systemd_slices(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
pub fn answer_systemd_paths(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
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
pub fn answer_systemctl_mask(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemctl_mask")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No masked systemd units found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (format!("Masked units ({}):\n```\n{}\n```", count, output), count)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer systemd scopes query
pub fn answer_systemd_scopes(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "systemd_scopes")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() {
        ("No systemd scope units found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (format!("Systemd scopes ({}):\n```\n{}\n```", count, output), count)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

/// Answer NTP status query
pub fn answer_ntp_status(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ntp_status")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "NTP/time synchronization status not available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    Some(DeterministicResult {
        answer: format!("Time synchronization status:\n```\n{}\n```", output),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer loginctl sessions query
pub fn answer_loginctl_sessions(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "loginctl_sessions")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() || output.contains("not available") {
        ("No login sessions found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (format!("Login sessions ({}):\n```\n{}\n```", count, output), count)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: parsed,
        route_class: route_class.to_string(),
    })
}

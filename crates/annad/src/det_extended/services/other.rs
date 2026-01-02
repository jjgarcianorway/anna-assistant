//! Other service management answer functions (v0.0.175).
//!
//! Handles crontabs, NTP status, and loginctl sessions.

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

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

    let job_count = output
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .count();
    Some(DeterministicResult {
        answer: format!("Crontab ({} jobs):\n```\n{}\n```", job_count, output),
        grounded: true,
        parsed_data_count: job_count,
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
pub fn answer_loginctl_sessions(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "loginctl_sessions")?;
    let output = probe.stdout.trim();

    let (answer, parsed) = if output.is_empty() || output.contains("not available") {
        ("No login sessions found.".to_string(), 0)
    } else {
        let count = output.lines().count();
        (
            format!("Login sessions ({}):\n```\n{}\n```", count, output),
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

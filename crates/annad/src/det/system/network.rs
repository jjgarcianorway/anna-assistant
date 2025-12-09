//! Network-related answer functions (v0.0.187).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer network connectivity query using ping
pub fn answer_network_connectivity(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "ping_check")?;

    let answer = if probe.exit_code == 0 {
        let output = probe.stdout.trim();
        let latency = output
            .lines()
            .find(|line| line.contains("time="))
            .and_then(|line| line.split("time=").nth(1).and_then(|s| s.split_whitespace().next()));

        if let Some(lat) = latency {
            format!("Online - ping to 8.8.8.8: {} ms", lat)
        } else {
            "Online - network connectivity confirmed".to_string()
        }
    } else {
        "Offline - cannot reach 8.8.8.8 (Google DNS)".to_string()
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer mounted filesystems query using findmnt
pub fn answer_mounted_filesystems(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "findmnt")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No mounted filesystems found.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let lines: Vec<&str> = output.lines().collect();
    let mount_count = lines.len().saturating_sub(1);

    Some(DeterministicResult {
        answer: format!("Mounted filesystems ({}):\n{}", mount_count, output),
        grounded: true,
        parsed_data_count: mount_count,
        route_class: route_class.to_string(),
    })
}

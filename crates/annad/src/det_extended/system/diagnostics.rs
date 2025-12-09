//! Diagnostic-related answer functions (v0.0.212).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer virtualization info query
pub fn answer_virtualization_info(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "virtualization_info")?;

    let output = probe.stdout.trim();
    let answer = if output == "none" || output.is_empty() {
        "Running on bare metal (no virtualization detected).".to_string()
    } else {
        format!("Virtualization: **{}**", output)
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer coredump list query
pub fn answer_coredump_list(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "coredump_list")?;

    let output = probe.stdout.trim();
    if output.contains("not available") || output.contains("No coredumps") || output.is_empty() {
        return Some(DeterministicResult {
            answer: "No coredumps found on this system.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let dump_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!(
            "Coredumps ({} found):\n```\n{}\n```",
            dump_count, output
        ),
        grounded: true,
        parsed_data_count: dump_count,
        route_class: route_class.to_string(),
    })
}

/// Answer tmp files query
pub fn answer_tmp_files(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "tmp_files")?;

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "/tmp directory is empty.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let file_count = output.lines().count().saturating_sub(1);
    Some(DeterministicResult {
        answer: format!("Files in /tmp ({}):\n```\n{}\n```", file_count, output),
        grounded: true,
        parsed_data_count: file_count,
        route_class: route_class.to_string(),
    })
}

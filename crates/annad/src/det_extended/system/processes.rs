//! Process-related answer functions (v0.0.212).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer process tree query
pub fn answer_process_tree(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "pstree")?;
    if probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "pstree not available (install psmisc package)".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No process tree available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let line_count = output.lines().count();
    Some(DeterministicResult {
        answer: format!(
            "Process tree ({} lines):\n```\n{}\n```",
            line_count, output
        ),
        grounded: true,
        parsed_data_count: line_count,
        route_class: route_class.to_string(),
    })
}

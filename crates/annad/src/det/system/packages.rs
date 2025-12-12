//! Package update answer functions (v0.0.187).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer package updates query using checkupdates probe
pub fn answer_package_updates(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "checkupdates").or_else(|| find_probe(probes, "pacman"));
    let probe = probe?;

    let output = probe.stdout.trim();

    if output.is_empty() || probe.exit_code != 0 {
        return Some(DeterministicResult {
            answer: "No package updates available. Your system is up to date.".to_string(),
            grounded: true,
            parsed_data_count: 1,
            route_class: route_class.to_string(),
        });
    }

    let update_count = output.lines().count();
    let preview: Vec<&str> = output.lines().take(5).collect();
    let preview_str = preview.join("\n  ");

    let answer = if update_count == 1 {
        format!("1 package update available:\n  {}", preview_str)
    } else if update_count <= 5 {
        format!(
            "{} package updates available:\n  {}",
            update_count, preview_str
        )
    } else {
        format!(
            "{} package updates available:\n  {}\n  ...and {} more",
            update_count,
            preview_str,
            update_count - 5
        )
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: update_count,
        route_class: route_class.to_string(),
    })
}

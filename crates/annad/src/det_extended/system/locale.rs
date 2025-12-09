//! Locale-related answer functions (v0.0.212).

use anna_shared::rpc::ProbeResult;

use crate::deterministic::DeterministicResult;
use crate::parsers::find_probe;

/// Answer system locale query
pub fn answer_system_locale(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "locale")?;
    if probe.exit_code != 0 {
        return None;
    }

    let output = probe.stdout.trim();
    if output.is_empty() {
        return Some(DeterministicResult {
            answer: "No locale settings available.".to_string(),
            grounded: true,
            parsed_data_count: 0,
            route_class: route_class.to_string(),
        });
    }

    let mut lang = None;
    let mut lc_all = None;

    for line in output.lines() {
        if let Some(val) = line.strip_prefix("LANG=") {
            lang = Some(val.trim_matches('"'));
        }
        if let Some(val) = line.strip_prefix("LC_ALL=") {
            lc_all = Some(val.trim_matches('"'));
        }
    }

    let primary = lc_all.unwrap_or_else(|| lang.unwrap_or("not set"));
    Some(DeterministicResult {
        answer: format!("System locale: {}\n\nFull output:\n{}", primary, output),
        grounded: true,
        parsed_data_count: output.lines().count(),
        route_class: route_class.to_string(),
    })
}

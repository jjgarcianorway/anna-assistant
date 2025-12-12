//! Memory and disk answer handlers (v0.0.176).

use anna_shared::rpc::ProbeResult;

use super::DeterministicResult;
use crate::parsers::find_probe;

/// Answer memory usage query using parsed free output
pub fn answer_memory_usage(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    let answer = crate::answers::answer_from_free_probe(probe)?;
    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer memory free query using free probe
pub fn answer_memory_free(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "free")?;
    let answer = crate::answers::answer_from_free_probe_available(probe)?;
    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer disk usage query using parsed df output
pub fn answer_disk_usage(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "df")?;
    let answer = crate::answers::answer_from_df_probe(probe)?;
    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer service status query using parsed systemctl output
pub fn answer_service_status(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    // Try is-active probe first
    if let Some(probe) = find_probe(probes, "systemctl is-active") {
        let service_name = probe
            .command
            .strip_prefix("systemctl is-active ")
            .unwrap_or("service");
        if let Some(answer) = crate::answers::answer_from_is_active_probe(probe, service_name) {
            return Some(DeterministicResult {
                answer,
                grounded: true,
                parsed_data_count: 1,
                route_class: route_class.to_string(),
            });
        }
    }
    // Try failed units probe
    if let Some(probe) = find_probe(probes, "systemctl --failed") {
        if let Some(answer) = crate::answers::answer_from_failed_units_probe(probe) {
            return Some(DeterministicResult {
                answer,
                grounded: true,
                parsed_data_count: 1,
                route_class: route_class.to_string(),
            });
        }
    }
    None
}

/// Answer system health summary using health brief (v0.0.32: relevant-only, not full report)
pub fn answer_system_health_summary(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    use crate::health_brief_builder::build_health_brief;

    // Build health brief from probes (only shows warnings/errors)
    let brief = build_health_brief(probes);

    // Always return an answer - even if healthy
    let answer = brief.format_answer();
    let count = if brief.all_healthy {
        1
    } else {
        brief.items.len()
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: count,
        route_class: route_class.to_string(),
    })
}

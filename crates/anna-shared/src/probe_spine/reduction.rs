//! Probe reduction logic (v0.0.193).
//!
//! Minimal Probe Policy (v0.45.3)

use super::types::{ProbeId, Urgency};

/// Reduce probes to minimal set based on route and urgency.
/// Default: max 3 probes. System health: max 4. ConfigureEditor: 10. Never run both errors and warnings.
pub fn reduce_probes(planned: Vec<ProbeId>, route_class: &str, urgency: Urgency) -> Vec<ProbeId> {
    let max_probes = match (route_class, urgency) {
        // v0.0.60: ConfigureEditor needs to probe all editors (10 probes)
        // This is cheap (command -v is fast) and necessary for grounded selection
        ("configure_editor", _) | ("ConfigureEditor", _) => 10,
        // System health queries get 4 probes
        ("system_health_summary", _) | ("system_triage", _) => 4,
        // Quick urgency = 2 probes max
        (_, Urgency::Quick) => 2,
        // Detailed urgency = 5 probes max
        (_, Urgency::Detailed) => 5,
        // Default: 3 probes
        _ => 3,
    };

    // Filter out redundant probes
    let mut reduced: Vec<ProbeId> = Vec::new();
    let mut has_journal_errors = false;
    let mut has_journal_warnings = false;

    for probe in planned {
        // Track journal probes
        if matches!(probe, ProbeId::JournalErrors) {
            has_journal_errors = true;
        }
        if matches!(probe, ProbeId::JournalWarnings) {
            // Skip warnings if we already have errors (unless detailed)
            if has_journal_errors && urgency != Urgency::Detailed {
                continue;
            }
            has_journal_warnings = true;
        }

        reduced.push(probe);

        if reduced.len() >= max_probes {
            break;
        }
    }

    // If we only have warnings but no errors, that's fine for explicit "warnings" queries
    // But for general queries, prefer errors over warnings
    if has_journal_warnings && !has_journal_errors && route_class != "journal_warnings" {
        // Swap warnings for errors if possible
        for p in &mut reduced {
            if matches!(p, ProbeId::JournalWarnings) {
                *p = ProbeId::JournalErrors;
                break;
            }
        }
    }

    reduced
}

/// Check if a query explicitly asks for warnings
pub fn query_wants_warnings(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("warning") && !lower.contains("error")
}

/// Check if a query explicitly asks for errors
pub fn query_wants_errors(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("error") && !lower.contains("warning")
}

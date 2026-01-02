//! Utility functions for recipe learning.

use super::observation::TicketObservation;

/// Find probes common across all observations
pub fn find_common_probes(observations: &[&TicketObservation]) -> Vec<String> {
    if observations.is_empty() {
        return vec![];
    }

    let first_probes: std::collections::HashSet<_> =
        observations[0].probes_used.iter().cloned().collect();

    let mut common: Vec<String> = first_probes
        .into_iter()
        .filter(|p| observations.iter().all(|o| o.probes_used.contains(p)))
        .collect();

    common.sort();
    common
}

/// Get path to candidates storage file
pub fn candidates_path() -> std::path::PathBuf {
    let base = std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| "/var/lib/anna".to_string());
    std::path::PathBuf::from(base).join("learning_candidates.json")
}

/// Get current timestamp in seconds since UNIX epoch
pub fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

//! Probe filtering utilities
//!
//! Extracted from probe_registry.rs for modularization.

use super::constants::PROBE_IDS;
use super::dynamic::probe_id_to_command_dynamic;

/// Filter probe IDs to only valid ones
/// v0.0.797: Now uses probe_id_to_command_dynamic for dynamic probe support
pub fn filter_valid_probes(probes: Vec<String>) -> Vec<String> {
    probes
        .into_iter()
        .filter(|p| PROBE_IDS.contains(&p.as_str()) || probe_id_to_command_dynamic(p).is_some())
        .collect()
}

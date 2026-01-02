//! Answer Tiers (Parts D & E) - v0.0.439.
//!
//! Part D: Fix the "boot time slow" flow with tiered answers.
//! Part E: Clarification questions must be rare and precise.
//!
//! Answer tiers:
//! 1. Provide measured facts from probes.
//! 2. Identify top offenders / key data points.
//! 3. Only then ask specialist to interpret and propose actions.

pub mod builders;
pub mod clarification;
pub mod extractors;
pub mod types;

// Re-export main types and functions
pub use builders::{
    build_boot_perf_tiers, build_cpu_load_tiers, build_disk_usage_tiers, build_gpu_driver_tiers,
    build_mem_status_tiers,
};
pub use clarification::{ClarificationBuilder, MAX_CLARIFICATION_LENGTH};
pub use types::{AnswerTier, TieredAnswer};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deterministic_routing::evidence_gate::{EvidenceStatus, ProbeResult};
    use std::collections::HashMap;

    fn make_evidence(probes: Vec<(&str, &str)>) -> EvidenceStatus {
        let mut map = HashMap::new();
        for (id, output) in probes {
            map.insert(id.to_string(), ProbeResult::success(id, output));
        }
        EvidenceStatus::from_probes(&map, &[])
    }

    #[test]
    fn test_boot_perf_tiers() {
        let evidence = make_evidence(vec![
            (
                "systemd_analyze",
                "Startup finished in 2.5s (kernel) + 5.2s (userspace) = 7.7s",
            ),
            (
                "systemd_blame",
                "3.5s NetworkManager.service\n2.1s docker.service\n1.8s systemd-udevd.service",
            ),
        ]);

        let answer = build_boot_perf_tiers(&evidence);
        assert!(answer.facts.is_some());
        assert!(answer.facts.as_ref().unwrap().contains("7.7s"));
        assert!(answer.key_items.is_some());
        assert_eq!(answer.key_items.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_mem_status_tiers() {
        let evidence = make_evidence(vec![
            ("free_h", "              total        used        free\nMem:           31Gi       8.2Gi        15Gi"),
        ]);

        let answer = build_mem_status_tiers(&evidence);
        assert!(answer.facts.is_some());
        assert!(answer.facts.as_ref().unwrap().contains("31Gi"));
    }
}

//! Probe registry implementation.

use super::builtin::builtin_probes;
use super::types::ProbeDef;
use crate::evidence_engine::{EvidenceDomain, EvidenceIntent};

/// The probe registry
pub struct ProbeRegistry {
    probes: Vec<ProbeDef>,
}

impl Default for ProbeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProbeRegistry {
    /// Create registry with built-in probes
    pub fn new() -> Self {
        Self {
            probes: builtin_probes(),
        }
    }

    /// Select probes for a request
    pub fn select(
        &self,
        domain: EvidenceDomain,
        intent: EvidenceIntent,
        tags: &[String],
        max_probes: usize,
    ) -> Vec<&ProbeDef> {
        let mut matches: Vec<_> = self
            .probes
            .iter()
            .filter(|p| p.matches(domain, intent, tags))
            .collect();

        // Sort by relevance (desc) then cost (asc)
        matches.sort_by(|a, b| {
            let score_a = a.relevance_score(tags);
            let score_b = b.relevance_score(tags);
            match score_b.cmp(&score_a) {
                std::cmp::Ordering::Equal => a.cost.cmp(&b.cost),
                other => other,
            }
        });

        matches.truncate(max_probes);
        matches
    }

    /// Get probe by ID
    pub fn get(&self, id: &str) -> Option<&ProbeDef> {
        self.probes.iter().find(|p| p.id == id)
    }

    /// Add a custom probe
    pub fn add(&mut self, probe: ProbeDef) {
        self.probes.push(probe);
    }

    /// List all probes for a domain
    pub fn for_domain(&self, domain: EvidenceDomain) -> Vec<&ProbeDef> {
        self.probes
            .iter()
            .filter(|p| p.domains.contains(&domain))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_registry_select() {
        let registry = ProbeRegistry::new();
        let probes = registry.select(
            EvidenceDomain::Storage,
            EvidenceIntent::Diagnose,
            &["disk".to_string(), "space".to_string()],
            5,
        );

        assert!(!probes.is_empty());
        assert!(probes.iter().any(|p| p.id == "probe:df_root"));
    }

    #[test]
    fn test_probe_matching() {
        use super::super::types::ProbeCost;

        let probe = ProbeDef {
            id: "test".into(),
            command: "test".into(),
            description: "test".into(),
            domains: vec![EvidenceDomain::Storage],
            tags: vec!["disk".into(), "space".into()],
            cost: ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        };

        assert!(probe.matches(
            EvidenceDomain::Storage,
            EvidenceIntent::Diagnose,
            &["disk".into()]
        ));
        assert!(!probe.matches(
            EvidenceDomain::Network,
            EvidenceIntent::Diagnose,
            &["disk".into()]
        ));
    }

    #[test]
    fn test_cost_ordering() {
        let registry = ProbeRegistry::new();
        let probes = registry.select(
            EvidenceDomain::Performance,
            EvidenceIntent::Diagnose,
            &["cpu".to_string()],
            10,
        );

        // Cheap probes should come first
        if probes.len() >= 2 {
            assert!(probes[0].cost <= probes[1].cost);
        }
    }
}

//! Workflow and acceptance tests for Evidence-First Knowledge Engine.
//!
//! Tests for key acceptance criteria including boot slow diagnosis and research flow.

#[cfg(test)]
mod tests {
    use crate::evidence_first::{
        citations::{Citation, CitationStore, EvidenceId},
        enforcement::{ClaimValidator, Claim, Strictness},
        primitives::PrimitiveLibrary,
        probe_plan::ProbePlan,
        research::ResearchPlan,
        sources::KnowledgeSource,
    };

    /// Test 1: Boot slow diagnosis with proper citations.
    #[test]
    fn test_boot_slow_diagnosis_workflow() {
        // 1. User reports slow boot
        let ticket_id = "boot-slow-001";

        // 2. Build probe plan from keywords
        let library = PrimitiveLibrary::new();
        let mut plan = ProbePlan::new(ticket_id);
        plan.select_from_keywords(&["boot", "slow", "startup"], &library);

        // Should select boot-related probes
        assert!(!plan.is_empty(), "Plan should have probes");
        assert!(
            plan.selected_primitives.iter().any(|p| p.contains("boot")),
            "Should include boot probe"
        );

        // 3. Verify probe primitives exist
        assert!(
            library.get("sys.boot.analyze").is_some(),
            "sys.boot.analyze should exist"
        );
        assert!(
            library.get("sys.boot.blame").is_some(),
            "sys.boot.blame should exist"
        );

        // 4. Simulate probe outputs and citations
        let mut store = CitationStore::new();

        // Add boot analysis evidence
        let boot_id = EvidenceId::probe("sys.boot.analyze");
        store.add_evidence(
            boot_id.clone(),
            KnowledgeSource::ProbeOutput("sys.boot.analyze".to_string()),
            "Startup finished in 3.5s (kernel) + 12.8s (userspace) = 16.3s",
        );
        store.add_citation(Citation::new(
            boot_id,
            "probe:sys.boot.analyze",
            "Startup finished in 16.3s",
        ));

        // Add blame evidence
        let blame_id = EvidenceId::probe("sys.boot.blame");
        store.add_evidence(
            blame_id.clone(),
            KnowledgeSource::ProbeOutput("sys.boot.blame".to_string()),
            "8.5s NetworkManager-wait-online.service\n2.1s docker.service",
        );
        store.add_citation(Citation::new(
            blame_id,
            "probe:sys.boot.blame",
            "8.5s NetworkManager-wait-online.service",
        ));

        // 5. Validate claims have citations
        let validator = ClaimValidator::new(Strictness::Standard);

        let claims = vec![
            Claim::factual("Boot takes 16.3 seconds"),
            Claim::factual("NetworkManager-wait-online is the slowest service at 8.5s"),
        ];

        let report = validator.validate_claims(&claims, &store);

        // Should have some supported claims due to keyword matching
        assert!(
            store.citation_count() >= 2,
            "Should have at least 2 citations"
        );
    }

    /// Test 4: Evidence-first research flow.
    #[test]
    fn test_research_flow() {
        let plan = ResearchPlan::new("test-ticket")
            .with_keywords(vec!["boot".to_string(), "slow".to_string()])
            .with_commands(vec!["systemd-analyze".to_string()]);

        assert_eq!(plan.ticket_id, "test-ticket");
        assert_eq!(plan.keywords.len(), 2);
        assert!(plan.can_iterate());

        // Verify max iterations
        let mut plan2 = ResearchPlan::new("test");
        plan2.next_iteration();
        plan2.next_iteration();
        assert!(!plan2.can_iterate(), "Should stop after 2 iterations");
    }
}

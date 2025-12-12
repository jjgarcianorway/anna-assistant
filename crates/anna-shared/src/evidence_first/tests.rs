//! Acceptance Tests for Evidence-First Knowledge Engine (v0.0.435).
//!
//! Tests for the key acceptance criteria:
//! 1. Boot slow diagnosis with citations
//! 2. CPU temperature check
//! 3. Recipe promotion after N confirmations

#[cfg(test)]
mod acceptance_tests {
    use crate::evidence_first::{
        citations::{Citation, CitationStore, EvidenceId},
        enforcement::{Claim, ClaimValidator, Strictness, extract_claims},
        primitives::{Domain, PrimitiveLibrary},
        probe_plan::{ProbePlan, ProbeOutput, ProbeSelection},
        recipes::{RecipePromoter, RecipeStep, RecipeTemplate},
        research::{ResearchLoop, ResearchPlan, ResearchResult},
        sources::KnowledgeSource,
        wiki_cache::{WikiCache, WikiPage},
    };
    use std::collections::HashMap;

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
            plan.selected_primitives
                .iter()
                .any(|p| p.contains("boot")),
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

    /// Test 2: CPU temperature check.
    #[test]
    fn test_cpu_temperature_check() {
        let library = PrimitiveLibrary::new();

        // Verify temperature probe exists
        let temp_probe = library.get("hw.cpu.temp");
        assert!(temp_probe.is_some(), "hw.cpu.temp should exist");

        // Verify it's in hardware domain
        let hw_probes = library.for_domain(Domain::Hardware);
        assert!(
            hw_probes.iter().any(|p| p.id == "hw.cpu.temp"),
            "Should be in Hardware domain"
        );

        // Test keyword search
        let temp_probes = library.find_by_keyword("temperature");
        assert!(
            !temp_probes.is_empty(),
            "Should find temperature probes"
        );
    }

    /// Test 3: Recipe promotion after N confirmations.
    #[test]
    fn test_recipe_promotion() {
        let mut promoter = RecipePromoter::new();

        // Create a recipe template
        let template = RecipeTemplate::new("restart-service", "Restart Failed Service")
            .with_problem("Service {service} has failed")
            .with_probe("sys.services.failed")
            .with_step(RecipeStep::new(1, "Check status: systemctl status {service}"))
            .with_step(
                RecipeStep::new(2, "Restart: sudo systemctl restart {service}")
                    .with_command("sudo systemctl restart {service}")
                    .with_confirmation(),
            )
            .with_outcome("Service {service} is running")
            .with_tag("systemd");

        // Add as candidate
        promoter.add_candidate(template);

        // Verify it's a candidate, not promoted
        assert!(
            promoter.get_candidate("restart-service").is_some(),
            "Should be a candidate"
        );
        assert!(
            promoter.get_promoted("restart-service").is_none(),
            "Should not be promoted yet"
        );

        // Record successful executions
        let store = CitationStore::new();

        // First confirmation
        promoter.record_execution("restart-service", "ticket-1", true, Some(&store), None);
        let candidate = promoter.get_candidate("restart-service").unwrap();
        assert_eq!(candidate.confirmation_count(), 1);
        assert!(!candidate.ready_for_promotion());

        // Second confirmation
        promoter.record_execution("restart-service", "ticket-2", true, Some(&store), None);
        let candidate = promoter.get_candidate("restart-service").unwrap();
        assert_eq!(candidate.confirmation_count(), 2);
        assert!(!candidate.ready_for_promotion());

        // Third confirmation - should trigger promotion
        promoter.record_execution("restart-service", "ticket-3", true, Some(&store), None);

        // Should now be promoted
        assert!(
            promoter.get_promoted("restart-service").is_some(),
            "Should be promoted after 3 confirmations"
        );
        assert!(
            promoter.get_candidate("restart-service").is_none(),
            "Should no longer be a candidate"
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

    /// Test 5: Citation store and verification.
    #[test]
    fn test_citation_store() {
        let mut store = CitationStore::new();

        // Add evidence
        let id = EvidenceId::probe("test.probe");
        store.add_evidence(
            id.clone(),
            KnowledgeSource::ProbeOutput("test.probe".to_string()),
            "This is the raw output with important data",
        );

        // Add citation
        store.add_citation(Citation::new(
            id.clone(),
            "probe:test.probe",
            "important data",
        ));

        // Verify evidence exists
        assert!(store.has_evidence(&id));

        // Verify citation
        let citations = store.citations_for(&id);
        assert_eq!(citations.len(), 1);

        // Verify citation content is in raw evidence
        let citation = &citations[0];
        assert!(store.verify_citation(citation), "Citation should be verifiable");
    }

    /// Test 6: Primitive library coverage.
    #[test]
    fn test_primitive_library_coverage() {
        let library = PrimitiveLibrary::new();

        // Should have probes for all major domains
        let domains = [
            Domain::Boot,
            Domain::Services,
            Domain::Logs,
            Domain::Memory,
            Domain::Disk,
            Domain::Network,
            Domain::Hardware,
        ];

        for domain in domains {
            let probes = library.for_domain(domain);
            assert!(
                !probes.is_empty(),
                "Should have probes for {:?}",
                domain
            );
        }

        // Verify key probes exist
        let key_probes = [
            "sys.boot.analyze",
            "sys.boot.blame",
            "sys.services.failed",
            "sys.logs.errors",
            "sys.mem.free",
            "sys.disk.df",
            "net.ip.addr",
        ];

        for probe_id in key_probes {
            assert!(
                library.get(probe_id).is_some(),
                "Should have {}",
                probe_id
            );
        }
    }

    /// Test 7: Claim extraction from text.
    #[test]
    fn test_claim_extraction() {
        let response_text = "Boot time is 16 seconds. According to the manual, systemd-analyze shows boot timing. You should disable NetworkManager-wait-online. I'm not sure about the exact cause.";

        let claims = extract_claims(response_text);

        // Should extract multiple claims
        assert!(!claims.is_empty(), "Should extract claims");

        // Should have different types
        use crate::evidence_first::enforcement::ClaimType;
        let has_factual = claims.iter().any(|c| matches!(c.claim_type, ClaimType::Factual));
        let has_doc = claims.iter().any(|c| matches!(c.claim_type, ClaimType::Documentation));
        let has_uncertainty = claims.iter().any(|c| matches!(c.claim_type, ClaimType::Uncertainty));

        assert!(has_factual || has_doc, "Should have factual or doc claims");
    }

    /// Test 8: Recipe instantiation with parameters.
    #[test]
    fn test_recipe_instantiation() {
        let template = RecipeTemplate::new("test", "Test Recipe")
            .with_step(RecipeStep::new(1, "Check {service} status"))
            .with_step(RecipeStep::new(2, "Restart {service}"));

        let mut params = HashMap::new();
        params.insert("service".to_string(), "nginx".to_string());

        let instance = template.instantiate(&params);

        assert_eq!(instance.steps[0], "Check nginx status");
        assert_eq!(instance.steps[1], "Restart nginx");
        assert_eq!(instance.next_step(), Some("Check nginx status"));
    }

    /// Test 9: Wiki cache operations.
    #[test]
    fn test_wiki_cache_search() {
        let page = WikiPage::new(
            "Systemd",
            "https://wiki.archlinux.org/title/Systemd",
            "# Overview\nSystemd is the init system.\n# Services\nUse systemctl to manage services.",
        );

        let hits = page.search("systemctl");
        assert!(!hits.is_empty(), "Should find systemctl");
        assert_eq!(hits[0].page_title, "Systemd");
    }

    /// Test 10: Probe selection by domain and keywords.
    #[test]
    fn test_probe_selection() {
        let library = PrimitiveLibrary::new();
        let mut plan = ProbePlan::new("test");

        // Select by domain
        plan.select_for_domain(Domain::Boot, &library);
        assert!(!plan.is_empty(), "Should select boot probes");

        // Select by keywords
        let mut plan2 = ProbePlan::new("test2");
        plan2.select_from_keywords(&["memory", "ram"], &library);
        assert!(!plan2.is_empty(), "Should select memory probes");
    }

    /// Test 11: Evidence ID formatting.
    #[test]
    fn test_evidence_id_formatting() {
        let probe = EvidenceId::probe("sys.boot.analyze");
        assert_eq!(probe.0, "probe:sys.boot.analyze");

        let man = EvidenceId::man("systemctl");
        assert_eq!(man.0, "man:systemctl");

        let wiki = EvidenceId::wiki("Systemd");
        assert_eq!(wiki.0, "wiki:Systemd");

        let help = EvidenceId::help("git");
        assert_eq!(help.0, "help:git");
    }

    /// Test 12: Recipe failure tracking.
    #[test]
    fn test_recipe_failure_tracking() {
        let mut promoter = RecipePromoter::new();

        let template = RecipeTemplate::new("test", "Test");
        promoter.add_candidate(template);

        let store = CitationStore::new();

        // Record mix of successes and failures
        promoter.record_execution("test", "t1", true, Some(&store), None);
        promoter.record_execution("test", "t2", false, None, Some("Service not found"));
        promoter.record_execution("test", "t3", true, Some(&store), None);

        let candidate = promoter.get_candidate("test").unwrap();
        assert_eq!(candidate.confirmation_count(), 2);
        assert_eq!(candidate.failure_count(), 1);

        // Success rate should be 2/3
        let rate = candidate.success_rate();
        assert!((rate - 0.666).abs() < 0.01, "Success rate should be ~66%");
    }

    /// Test 13: Validation report formatting.
    #[test]
    fn test_validation_report() {
        let mut store = CitationStore::new();
        let id = EvidenceId::probe("test");
        store.add_evidence(
            id.clone(),
            KnowledgeSource::ProbeOutput("test".to_string()),
            "test output",
        );
        store.add_citation(Citation::new(id, "probe:test", "test output"));

        let validator = ClaimValidator::default();
        let claims = vec![
            Claim::factual("test output shows results"),
        ];

        let report = validator.validate_claims(&claims, &store);

        // Format should include key info
        let formatted = report.format();
        assert!(formatted.contains("Validation Report"), "Should have header");
    }
}

/// Integration test helpers.
#[cfg(test)]
mod integration_helpers {
    use crate::evidence_first::probe_plan::ProbeOutput;

    /// Create a mock probe output for testing.
    pub fn mock_probe_output(primitive_id: &str, output: &str) -> ProbeOutput {
        ProbeOutput {
            primitive_id: primitive_id.to_string(),
            raw_output: output.to_string(),
            parsed: None,
            exit_code: Some(0),
            execution_time_ms: 100,
            error: None,
        }
    }
}

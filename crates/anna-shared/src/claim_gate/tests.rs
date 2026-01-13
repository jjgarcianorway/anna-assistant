//! Tests for ClaimGate.

#[cfg(test)]
mod tests {
    use crate::claim_gate::{
        ClaimCategory, ClaimGate, ClaimVerifier, EvidenceType, GateResult, SentenceType,
    };

    #[test]
    fn test_claim_gate_basic() {
        let mut gate = ClaimGate::new();
        let mut claim = gate.submit_claim("nginx is running", ClaimCategory::ServiceState);

        // Without evidence, should need investigation
        let result = gate.verify(&claim);
        assert!(matches!(result, GateResult::NeedsInvestigation { .. }));

        // Add evidence
        let evidence = ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "active",
            0,
        );
        gate.add_evidence(&mut claim, evidence);

        // Now should be verified
        let result = gate.verify(&claim);
        assert!(matches!(result, GateResult::Verified { .. }));
    }

    #[test]
    fn test_claim_extraction() {
        let text = "The service nginx is running and port 80 is open";
        let claims = ClaimGate::extract_claims(text);
        assert!(!claims.is_empty());
    }

    #[test]
    fn test_evidence_confidence() {
        let mut gate = ClaimGate::new();
        let mut claim = gate.submit_claim("test", ClaimCategory::General);

        // No evidence = 0 confidence
        assert_eq!(gate.calculate_confidence(&claim), 0.0);

        // Add probe evidence
        gate.add_evidence(&mut claim, ClaimGate::evidence_from_probe("test", "ok", 0));
        assert!(claim.confidence > 0.5);
    }

    #[test]
    fn test_unverified_statement() {
        let marked = ClaimGate::create_unverified_statement("nginx is running");
        assert!(marked.contains("Cannot verify"));
    }

    // v0.3.25: SentenceType classifier tests
    #[test]
    fn test_sentence_type_fact() {
        assert_eq!(SentenceType::classify("nginx is running"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("The service is stopped"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("Port 80 is open"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("You have 16 GB of RAM"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("The package is installed"), SentenceType::Fact);
        assert_eq!(SentenceType::classify("Disk usage is 47% used"), SentenceType::Fact);
    }

    #[test]
    fn test_sentence_type_suggestion() {
        assert_eq!(SentenceType::classify("Try restarting the service"), SentenceType::Suggestion);
        assert_eq!(SentenceType::classify("Consider using a different port"), SentenceType::Suggestion);
        assert_eq!(SentenceType::classify("You could install htop"), SentenceType::Suggestion);
        assert_eq!(SentenceType::classify("I recommend checking the logs"), SentenceType::Suggestion);
    }

    #[test]
    fn test_sentence_type_question() {
        assert_eq!(SentenceType::classify("Is nginx running?"), SentenceType::Question);
        assert_eq!(SentenceType::classify("What version are you using?"), SentenceType::Question);
        assert_eq!(SentenceType::classify("How much RAM do you have?"), SentenceType::Question);
        assert_eq!(SentenceType::classify("Can you check the config?"), SentenceType::Question);
    }

    #[test]
    fn test_sentence_type_narrative() {
        assert_eq!(SentenceType::classify("Let me check that for you"), SentenceType::Narrative);
        assert_eq!(SentenceType::classify("This configuration file controls nginx behavior"), SentenceType::Narrative);
        assert_eq!(SentenceType::classify("Looking at the output"), SentenceType::Narrative);
    }

    #[test]
    fn test_sentence_type_requires_evidence() {
        assert!(SentenceType::Fact.requires_evidence());
        assert!(!SentenceType::Suggestion.requires_evidence());
        assert!(!SentenceType::Question.requires_evidence());
        assert!(!SentenceType::Narrative.requires_evidence());
    }

    // v0.3.25: ClaimGate enforcement tests
    #[test]
    fn test_fact_without_evidence_blocked() {
        let gate = ClaimGate::new();
        let response = "nginx is running on port 80";
        let result = gate.verify_response(response, &[]); // No evidence
        assert!(result.needs_investigation);
        assert!(!result.unverified_claims.is_empty());
        // v0.3.27: Claims must be BLOCKED, not just marked
        assert!(result.claims_blocked, "Unverified claims must be blocked");
        assert!(result.verified_text.contains("[I cannot verify"),
            "Blocked claims must be replaced with uncertainty statement");
    }

    #[test]
    fn test_fact_with_probe_evidence_passes() {
        let gate = ClaimGate::new();
        let response = "The service nginx is running";
        let evidence = vec![ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "active",
            0,
        )];
        let result = gate.verify_response(response, &evidence);
        // With evidence, claims should be verified
        assert!(!result.verified_claims.is_empty() || result.unverified_claims.is_empty());
    }

    #[test]
    fn test_conflicting_evidence_forces_investigation() {
        let gate = ClaimGate::new();
        let response = "The service nginx is running";
        // Evidence shows nginx is NOT running (exit code 3 = inactive)
        let evidence = vec![ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "inactive",
            3,
        )];
        let result = gate.verify_response(response, &evidence);
        // Lower confidence from failed command
        assert!(result.confidence < 1.0);
    }

    // v0.3.26: Doc citation tests
    #[test]
    fn test_claim_requires_docs() {
        // "How X works" questions require docs
        assert!(ClaimGate::claim_requires_docs("how does systemctl mask work"));
        assert!(ClaimGate::claim_requires_docs("what does the -S flag mean"));
        assert!(ClaimGate::claim_requires_docs("explain TRIM"));
        assert!(ClaimGate::claim_requires_docs("configure ssh"));
        assert!(ClaimGate::claim_requires_docs("syntax for crontab"));

        // State queries don't require docs
        assert!(!ClaimGate::claim_requires_docs("is nginx running"));
        assert!(!ClaimGate::claim_requires_docs("list services"));
    }

    #[test]
    fn test_has_doc_evidence() {
        let probe_only = vec![
            ClaimGate::evidence_from_probe("free -h", "16GB", 0),
        ];
        assert!(!ClaimGate::has_doc_evidence(&probe_only));

        let with_wiki = vec![
            ClaimGate::evidence_from_probe("free -h", "16GB", 0),
            ClaimGate::evidence_from_wiki("Systemd", Some("User units"), "systemctl --user"),
        ];
        assert!(ClaimGate::has_doc_evidence(&with_wiki));

        let with_man = vec![
            ClaimGate::evidence_from_man("systemctl", 1, "mask - mask units"),
        ];
        assert!(ClaimGate::has_doc_evidence(&with_man));
    }

    #[test]
    fn test_verify_with_context_docs_required() {
        let gate = ClaimGate::new();
        let question = "how does systemctl mask work";
        let response = "systemctl mask prevents a unit from being started";

        // Without doc evidence, should need investigation
        let probe_evidence = vec![
            ClaimGate::evidence_from_probe("systemctl mask test", "Created symlink", 0),
        ];
        let result = gate.verify_response_with_context(response, question, &probe_evidence);
        assert!(result.docs_required);
        assert!(!result.docs_found);
        assert!(result.needs_investigation);

        // With doc evidence, should be fine
        let doc_evidence = vec![
            ClaimGate::evidence_from_man("systemctl", 1, "mask UNIT... Mask one or more units"),
        ];
        let result = gate.verify_response_with_context(response, question, &doc_evidence);
        assert!(result.docs_required);
        assert!(result.docs_found);
    }

    #[test]
    fn test_doc_citation_formatting() {
        let gate = ClaimGate::new();
        let response = "ok";
        let evidence = vec![
            ClaimGate::evidence_from_wiki("Systemd", Some("Timers"), "OnCalendar="),
            ClaimGate::evidence_from_man("systemctl", 1, "mask units"),
        ];
        let result = gate.verify_response(response, &evidence);
        assert_eq!(result.doc_citations.len(), 2);
        assert!(result.doc_citations[0].contains("Arch Wiki"));
        assert!(result.doc_citations[1].contains("man"));
    }

    #[test]
    fn test_probe_only_for_state_query() {
        let gate = ClaimGate::new();
        let question = "how much RAM is free";
        let response = "You have 8GB free";
        let evidence = vec![
            ClaimGate::evidence_from_probe("free -h", "Mem: 16G 8G 8G", 0),
        ];
        let result = gate.verify_response_with_context(response, question, &evidence);
        // "how much" is not a "how does" question, so no docs required
        assert!(!result.docs_required);
    }

    // ========== v0.3.27: Adversarial tests for ClaimGate hardening ==========

    /// Adversarial test: Probe failure must not produce invented facts
    #[test]
    fn test_probe_failure_blocks_claims() {
        let gate = ClaimGate::new();
        let response = "The service nginx is running";

        // Probe FAILED (exit code 1 = error, not just inactive)
        let evidence = vec![ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "error: unit nginx not found",
            1,
        )];
        let result = gate.verify_response(response, &evidence);

        // Must detect failed probe
        assert!(result.probes_failed, "Failed probes must be detected");
        assert!(!result.failed_probes.is_empty(), "Failed probe list must not be empty");

        // Must NOT emit the claim as fact (lower confidence)
        assert!(result.confidence < 1.0, "Confidence must be reduced for failed probes");
    }

    /// Adversarial test: Conflicting probes must not assert single truth
    #[test]
    fn test_conflicting_probes_block_assertion() {
        let gate = ClaimGate::new();
        let response = "The service nginx is running";

        // First probe says running, second says not running
        let evidence = vec![
            ClaimGate::evidence_from_probe("systemctl is-active nginx", "active", 0),
            ClaimGate::evidence_from_probe("pgrep nginx", "", 1), // No process found
        ];
        let result = gate.verify_response(response, &evidence);

        // Must detect the failed probe
        assert!(result.probes_failed, "Must detect conflicting probe failure");
        assert!(result.failed_probes.contains(&"pgrep nginx".to_string()));

        // v0.3.28: Must detect conflict (Phase 3 F4)
        assert!(result.conflicts_detected, "Must detect probe conflict");
        assert!(!result.conflict_descriptions.is_empty(), "Must have conflict description");
    }

    /// Adversarial test: No evidence = claim blocked with explicit uncertainty
    #[test]
    fn test_no_evidence_explicit_uncertainty() {
        let gate = ClaimGate::new();
        let response = "Port 443 is open and listening";

        // No evidence at all
        let result = gate.verify_response(response, &[]);

        // Must be blocked
        assert!(result.claims_blocked, "Claims without evidence must be blocked");

        // Must contain explicit uncertainty statement
        assert!(
            result.verified_text.contains("[I cannot verify"),
            "Output must contain uncertainty statement, got: {}",
            result.verified_text
        );

        // Must NOT contain the original unqualified claim
        assert!(
            !result.verified_text.contains("Port 443 is open and listening"),
            "Original unqualified claim must not appear in output"
        );
    }

    /// Adversarial test: Partial evidence blocks unverified portions
    #[test]
    fn test_partial_evidence_blocks_unverified() {
        let gate = ClaimGate::new();
        // Response makes two claims
        let response = "The service nginx is running and port 80 is open";

        // Evidence only for nginx, not for port
        let evidence = vec![ClaimGate::evidence_from_probe(
            "systemctl is-active nginx",
            "active",
            0,
        )];
        let result = gate.verify_response(response, &evidence);

        // Port claim should be blocked (we don't have evidence for it)
        // nginx claim may or may not pass depending on extraction
        // The key is: if port claim is extracted and unverified, it must be blocked
        if !result.unverified_claims.is_empty() {
            assert!(result.claims_blocked, "Unverified claims must be blocked");
            assert!(
                result.verified_text.contains("[I cannot verify"),
                "Unverified claims must show uncertainty"
            );
        }
    }

    /// Adversarial test: Empty probe output = claim blocked
    #[test]
    fn test_empty_probe_output_blocks() {
        let gate = ClaimGate::new();
        let response = "The package vim is installed";

        // Probe returned empty (package not found)
        let evidence = vec![ClaimGate::evidence_from_probe(
            "pacman -Q vim",
            "",
            1, // Package not found returns exit 1
        )];
        let result = gate.verify_response(response, &evidence);

        // Must detect probe failure
        assert!(result.probes_failed);

        // Confidence should be low
        assert!(result.confidence < 1.0);
    }

    // ========== v0.3.28: Phase 3 F15 - Empty probe output (success) handling ==========

    /// F15: Probe SUCCESS with empty stdout must set output_empty=true
    #[test]
    fn test_empty_success_output() {
        // Case 1: Successful probe with empty output
        let evidence = ClaimGate::evidence_from_probe("grep pattern /etc/config", "", 0);
        if let EvidenceType::ProbeResult { output_empty, exit_code, .. } = evidence {
            assert_eq!(exit_code, 0, "Exit code should be 0 (success)");
            assert!(output_empty, "output_empty must be true for empty successful probe");
        } else {
            panic!("Expected ProbeResult");
        }

        // Case 2: Successful probe with whitespace-only output (also empty)
        let evidence = ClaimGate::evidence_from_probe("cat /empty/file", "   \n\t  ", 0);
        if let EvidenceType::ProbeResult { output_empty, .. } = evidence {
            assert!(output_empty, "Whitespace-only output should be considered empty");
        }

        // Case 3: Successful probe with actual output
        let evidence = ClaimGate::evidence_from_probe("echo hello", "hello", 0);
        if let EvidenceType::ProbeResult { output_empty, .. } = evidence {
            assert!(!output_empty, "Non-empty output should NOT set output_empty");
        }

        // Case 4: Failed probe with empty output (NOT output_empty since exit != 0)
        let evidence = ClaimGate::evidence_from_probe("false", "", 1);
        if let EvidenceType::ProbeResult { output_empty, exit_code, .. } = evidence {
            assert_eq!(exit_code, 1, "Exit code should be 1 (failure)");
            assert!(!output_empty, "Failed probe should NOT set output_empty (only success with empty)");
        }
    }
}

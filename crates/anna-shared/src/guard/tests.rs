//! Tests for guard module (v0.0.194).

#[cfg(test)]
mod tests {
    use crate::claims::extract_claims;
    use crate::grounding::ParsedEvidence;
    use crate::guard::{run_guard, VerifyResult};
    use crate::parsers::{DiskUsage, MemoryInfo, ServiceState, ServiceStatus};

    // === A) Contradiction always flags, regardless of evidence_required ===

    #[test]
    fn golden_contradiction_flags_even_without_evidence_required() {
        // "memory uses 4GB" but evidence shows 3GB
        let claims = extract_claims("memory uses 4294967296B"); // 4GB in bytes
        let evidence = ParsedEvidence {
            memory: Some(MemoryInfo {
                total_bytes: 16_000_000_000,
                used_bytes: 3_221_225_472, // 3GB - different!
                free_bytes: 1_000_000_000,
                shared_bytes: 0,
                buff_cache_bytes: 0,
                available_bytes: 0,
                swap_total_bytes: None,
                swap_used_bytes: None,
                swap_free_bytes: None,
            }),
            ..Default::default()
        };

        // evidence_required = false, but contradiction still flags
        let report = run_guard(&claims, &evidence, false);
        assert_eq!(report.contradictions, 1);
        assert_eq!(report.unverifiable_specifics, 0);
        assert!(report.invention_detected);

        // evidence_required = true, contradiction also flags
        let report = run_guard(&claims, &evidence, true);
        assert!(report.invention_detected);
    }

    #[test]
    fn golden_contradiction_formats_deterministic() {
        let claims = extract_claims("memory uses 4294967296B");
        let evidence = ParsedEvidence {
            memory: Some(MemoryInfo {
                total_bytes: 16_000_000_000,
                used_bytes: 3_221_225_472,
                free_bytes: 1_000_000_000,
                shared_bytes: 0,
                buff_cache_bytes: 0,
                available_bytes: 0,
                swap_total_bytes: None,
                swap_used_bytes: None,
                swap_free_bytes: None,
            }),
            ..Default::default()
        };

        let report = run_guard(&claims, &evidence, false);
        assert_eq!(report.details.len(), 1);
        if let VerifyResult::Contradiction { claimed, evidence } = &report.details[0].result {
            assert_eq!(claimed, "4294967296B");
            assert_eq!(evidence, "3221225472B");
        } else {
            panic!("Expected contradiction");
        }
    }

    // === B) Unverifiable only flags when evidence_required=true ===

    #[test]
    fn golden_unverifiable_no_flag_without_evidence_required() {
        // Claim about firefox, but no process-level evidence
        let claims = extract_claims("firefox uses 4294967296B");
        let evidence = ParsedEvidence::default(); // No evidence

        let report = run_guard(&claims, &evidence, false);
        assert_eq!(report.contradictions, 0);
        assert_eq!(report.unverifiable_specifics, 1);
        assert!(!report.invention_detected); // NOT flagged
    }

    #[test]
    fn golden_unverifiable_flags_with_evidence_required() {
        let claims = extract_claims("firefox uses 4294967296B");
        let evidence = ParsedEvidence::default();

        let report = run_guard(&claims, &evidence, true);
        assert_eq!(report.contradictions, 0);
        assert_eq!(report.unverifiable_specifics, 1);
        assert!(report.invention_detected); // IS flagged
    }

    // === C) Verified claim does not flag ===

    #[test]
    fn golden_verified_no_flag() {
        let claims = extract_claims("memory uses 8804682957B");
        let evidence = ParsedEvidence {
            memory: Some(MemoryInfo {
                total_bytes: 16_000_000_000,
                used_bytes: 8_804_682_957, // Exact match
                free_bytes: 1_000_000_000,
                shared_bytes: 0,
                buff_cache_bytes: 0,
                available_bytes: 0,
                swap_total_bytes: None,
                swap_used_bytes: None,
                swap_free_bytes: None,
            }),
            ..Default::default()
        };

        let report = run_guard(&claims, &evidence, true);
        assert_eq!(report.contradictions, 0);
        assert_eq!(report.unverifiable_specifics, 0);
        assert!(!report.invention_detected);
        assert!(matches!(report.details[0].result, VerifyResult::Verified));
    }

    // === D) Mixed claims with ordering stability ===

    #[test]
    fn golden_mixed_claims_ordering_stable() {
        // Three claims: unverifiable firefox, contradiction on disk, verified nginx
        let answer = "nginx is running and / is 90% full and firefox uses 1073741824B";
        let claims = extract_claims(answer);
        assert_eq!(claims.len(), 3);

        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            }],
            disks: vec![DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 100_000_000_000,
                used_bytes: 85_000_000_000,
                available_bytes: 15_000_000_000,
                percent_used: 85, // Claim says 90%, evidence is 85% - contradiction
            }],
            ..Default::default()
        };

        let report = run_guard(&claims, &evidence, false);

        // Counts
        assert_eq!(report.total_specific_claims, 3);
        assert_eq!(report.contradictions, 1); // disk percent
        assert_eq!(report.unverifiable_specifics, 1); // firefox

        // Ordering matches extraction order: numeric → percent → status
        assert_eq!(report.details.len(), 3);

        // First: firefox (numeric) - unverifiable
        assert!(matches!(
            report.details[0].result,
            VerifyResult::Unverifiable
        ));

        // Second: disk (percent) - contradiction
        assert!(report.details[1].result.is_contradiction());
        if let VerifyResult::Contradiction { claimed, evidence } = &report.details[1].result {
            assert_eq!(claimed, "90%");
            assert_eq!(evidence, "85%");
        }

        // Third: nginx (status) - verified
        assert!(matches!(report.details[2].result, VerifyResult::Verified));

        // Invention detected due to contradiction
        assert!(report.invention_detected);
    }

    // === E) Service normalization impacts contradictions ===

    #[test]
    fn golden_service_normalization_contradiction() {
        // Claim: nginx is running, Evidence: nginx.service is failed
        let claims = extract_claims("nginx is running");
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Failed,
                description: None,
            }],
            ..Default::default()
        };

        let report = run_guard(&claims, &evidence, false);
        assert_eq!(report.contradictions, 1);
        assert!(report.invention_detected);

        if let VerifyResult::Contradiction { claimed, evidence } = &report.details[0].result {
            assert_eq!(claimed, "running");
            assert_eq!(evidence, "failed");
        } else {
            panic!("Expected contradiction");
        }
    }

    #[test]
    fn golden_service_normalization_verified() {
        // Claim: nginx is running, Evidence: nginx.service is running
        let claims = extract_claims("nginx is running");
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            }],
            ..Default::default()
        };

        let report = run_guard(&claims, &evidence, false);
        assert_eq!(report.contradictions, 0);
        assert!(!report.invention_detected);
        assert!(matches!(report.details[0].result, VerifyResult::Verified));
    }

    // === Additional edge cases ===

    #[test]
    fn golden_no_claims_no_invention() {
        let claims = extract_claims("Everything looks fine.");
        let evidence = ParsedEvidence::default();

        let report = run_guard(&claims, &evidence, true);
        assert_eq!(report.total_specific_claims, 0);
        assert_eq!(report.contradictions, 0);
        assert_eq!(report.unverifiable_specifics, 0);
        assert!(!report.invention_detected);
    }

    #[test]
    fn golden_disk_percent_verified() {
        let claims = extract_claims("root is 85% full");
        let evidence = ParsedEvidence {
            disks: vec![DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 100_000_000_000,
                used_bytes: 85_000_000_000,
                available_bytes: 15_000_000_000,
                percent_used: 85,
            }],
            ..Default::default()
        };

        let report = run_guard(&claims, &evidence, true);
        assert_eq!(report.contradictions, 0);
        assert!(!report.invention_detected);
    }

    #[test]
    fn golden_determinism_same_input() {
        let answer = "nginx is running and root is 75% full";
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            }],
            disks: vec![DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 100_000_000_000,
                used_bytes: 75_000_000_000,
                available_bytes: 25_000_000_000,
                percent_used: 75,
            }],
            ..Default::default()
        };

        // Run twice
        let claims1 = extract_claims(answer);
        let claims2 = extract_claims(answer);
        let report1 = run_guard(&claims1, &evidence, true);
        let report2 = run_guard(&claims2, &evidence, true);

        assert_eq!(report1, report2);
    }
}

//! Tests for grounding module (v0.0.195).

#[cfg(test)]
mod tests {
    use crate::claims::extract_claims;
    use crate::grounding::{compute_grounding, is_answer_grounded, ParsedEvidence};
    use crate::parsers::{DiskUsage, MemoryInfo, ServiceState, ServiceStatus};

    // === Verified numeric claim ===

    #[test]
    fn golden_verified_numeric_memory() {
        let claims = extract_claims("memory uses 8804682957B");
        let evidence = ParsedEvidence {
            memory: Some(MemoryInfo {
                total_bytes: 16_106_127_360,
                used_bytes: 8_804_682_957,
                free_bytes: 1_610_612_736,
                shared_bytes: 536_870_912,
                buff_cache_bytes: 6_227_702_579,
                available_bytes: 6_979_321_856,
                swap_total_bytes: None,
                swap_used_bytes: None,
                swap_free_bytes: None,
            }),
            ..Default::default()
        };

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 1);
        assert_eq!(report.verified_claims, 1);
        assert_eq!(report.grounding_ratio, 1.0);
        assert!(is_answer_grounded(&report));
    }

    // === Contradicted numeric claim ===

    #[test]
    fn golden_contradicted_numeric() {
        let claims = extract_claims("memory uses 4294967296B"); // 4GB
        let evidence = ParsedEvidence {
            memory: Some(MemoryInfo {
                total_bytes: 16_106_127_360,
                used_bytes: 3_221_225_472, // 3GB - different!
                free_bytes: 1_610_612_736,
                shared_bytes: 0,
                buff_cache_bytes: 0,
                available_bytes: 0,
                swap_total_bytes: None,
                swap_used_bytes: None,
                swap_free_bytes: None,
            }),
            ..Default::default()
        };

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 1);
        assert_eq!(report.verified_claims, 0);
        assert_eq!(report.grounding_ratio, 0.0);
        assert!(!is_answer_grounded(&report));
    }

    // === Verified disk percent with alias ===

    #[test]
    fn golden_verified_disk_percent_alias() {
        let claims = extract_claims("root is 85% full");
        let evidence = ParsedEvidence {
            disks: vec![DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 107_374_182_400,
                used_bytes: 91_268_055_040,
                available_bytes: 16_106_127_360,
                percent_used: 85,
            }],
            ..Default::default()
        };

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 1);
        assert_eq!(report.verified_claims, 1);
        assert_eq!(report.grounding_ratio, 1.0);
        assert!(is_answer_grounded(&report));
    }

    // === Unverifiable disk percent (no mount key) ===

    #[test]
    fn golden_unverifiable_disk_no_mount() {
        // "disk is 85% full" has no mount key → no claims extracted
        let claims = extract_claims("disk is 85% full");
        let evidence = ParsedEvidence {
            disks: vec![DiskUsage {
                filesystem: "/dev/sda1".to_string(),
                mount: "/".to_string(),
                size_bytes: 107_374_182_400,
                used_bytes: 91_268_055_040,
                available_bytes: 16_106_127_360,
                percent_used: 85,
            }],
            ..Default::default()
        };

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 0);
        assert_eq!(report.grounding_ratio, 0.0);
        assert!(!is_answer_grounded(&report));
    }

    // === No claims rule ===

    #[test]
    fn golden_no_claims_not_grounded() {
        let claims = extract_claims("Everything looks fine on your system.");
        let evidence = ParsedEvidence::default();

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 0);
        assert_eq!(report.grounding_ratio, 0.0);
        assert!(!is_answer_grounded(&report));
    }

    // === Service status verification ===

    #[test]
    fn golden_verified_service_running() {
        let claims = extract_claims("nginx is running");
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            }],
            ..Default::default()
        };

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 1);
        assert_eq!(report.verified_claims, 1);
        assert_eq!(report.grounding_ratio, 1.0);
        assert!(is_answer_grounded(&report));
    }

    #[test]
    fn golden_contradicted_service_status() {
        let claims = extract_claims("nginx is running");
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Failed, // Actually failed!
                description: None,
            }],
            ..Default::default()
        };

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 1);
        assert_eq!(report.verified_claims, 0);
        assert_eq!(report.grounding_ratio, 0.0);
        assert!(!is_answer_grounded(&report));
    }

    // === Determinism test ===

    #[test]
    fn golden_deterministic_same_input() {
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

        // Run twice, must be identical
        let claims1 = extract_claims(answer);
        let claims2 = extract_claims(answer);
        let report1 = compute_grounding(&claims1, &evidence);
        let report2 = compute_grounding(&claims2, &evidence);

        assert_eq!(report1, report2);
        assert_eq!(report1.total_claims, 2);
        assert_eq!(report1.verified_claims, 2);
        assert_eq!(report1.grounding_ratio, 1.0);
    }

    // === Partial verification ===

    #[test]
    fn golden_partial_verification() {
        // Two claims: one verifiable, one not
        let claims = extract_claims("nginx is running and postgresql is failed");
        let evidence = ParsedEvidence {
            services: vec![ServiceStatus {
                name: "nginx.service".to_string(),
                state: ServiceState::Running,
                description: None,
            }],
            // postgresql not in evidence
            ..Default::default()
        };

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 2);
        assert_eq!(report.verified_claims, 1);
        assert_eq!(report.grounding_ratio, 0.5);
        assert!(is_answer_grounded(&report)); // 0.5 >= 0.5 threshold
    }

    #[test]
    fn golden_below_threshold() {
        // Two claims: neither verifiable
        let claims = extract_claims("nginx is running and postgresql is failed");
        let evidence = ParsedEvidence::default(); // No evidence at all

        let report = compute_grounding(&claims, &evidence);
        assert_eq!(report.total_claims, 2);
        assert_eq!(report.verified_claims, 0);
        assert_eq!(report.grounding_ratio, 0.0);
        assert!(!is_answer_grounded(&report));
    }
}

//! GUARD: Evidence-aware contradiction and invention detection.
//!
//! Reuses the same `Claim` extraction as ANCHOR but outputs contradiction/invention
//! signals instead of grounding ratios.
//!
//! # Invention Detection Rules
//!
//! - **Contradiction**: always flags `invention_detected = true`
//! - **Unverifiable + evidence_required**: flags `invention_detected = true`
//! - **Unverifiable + !evidence_required**: does NOT flag invention (but is counted)
//! - **Verified**: never flags invention

use crate::claims::{Claim, NumericClaim, PercentClaim, StatusClaim};
use crate::grounding::ParsedEvidence;
use crate::parsers::ServiceState;
use serde::{Deserialize, Serialize};

/// Result of verifying a single claim against evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VerifyResult {
    /// Claim matches evidence exactly
    Verified,
    /// Claim contradicts evidence
    Contradiction {
        /// What the claim stated (deterministic format)
        claimed: String,
        /// What the evidence shows (deterministic format)
        evidence: String,
    },
    /// No evidence available to verify this claim
    Unverifiable,
}

impl VerifyResult {
    /// Check if this is a contradiction
    pub fn is_contradiction(&self) -> bool {
        matches!(self, Self::Contradiction { .. })
    }

    /// Check if this is unverifiable
    pub fn is_unverifiable(&self) -> bool {
        matches!(self, Self::Unverifiable)
    }
}

/// GUARD verification report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardReport {
    /// Total specific claims checked
    pub total_specific_claims: u32,
    /// Number of claims that contradict evidence
    pub contradictions: u32,
    /// Number of specific claims that couldn't be verified
    pub unverifiable_specifics: u32,
    /// Whether invention was detected (triggers CHAOS ceiling)
    pub invention_detected: bool,
    /// Per-claim verification details (same order as input claims)
    pub details: Vec<GuardItem>,
}

/// Single claim verification result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardItem {
    /// The claim that was checked
    pub claim: Claim,
    /// Verification result
    pub result: VerifyResult,
}

/// Run GUARD verification on extracted claims.
///
/// # Arguments
/// - `claims`: Claims extracted from the answer (same as ANCHOR uses)
/// - `evidence`: Parsed probe data (same as ANCHOR uses)
/// - `evidence_required`: Whether the query type requires evidence
///
/// # Invention Detection Rules
/// - Any contradiction → invention_detected = true
/// - Any unverifiable + evidence_required → invention_detected = true
pub fn run_guard(
    claims: &[Claim],
    evidence: &ParsedEvidence,
    evidence_required: bool,
) -> GuardReport {
    let mut details = Vec::with_capacity(claims.len());
    let mut contradictions = 0u32;
    let mut unverifiable_specifics = 0u32;

    for claim in claims {
        let result = verify_claim(claim, evidence);

        if result.is_contradiction() {
            contradictions += 1;
        } else if result.is_unverifiable() {
            unverifiable_specifics += 1;
        }

        details.push(GuardItem {
            claim: claim.clone(),
            result,
        });
    }

    // Invention detection rules:
    // 1. Contradictions always flag invention
    // 2. Unverifiable specifics only flag when evidence_required
    let invention_detected = contradictions > 0
        || (unverifiable_specifics > 0 && evidence_required);

    GuardReport {
        total_specific_claims: claims.len() as u32,
        contradictions,
        unverifiable_specifics,
        invention_detected,
        details,
    }
}

/// Verify a single claim against evidence.
fn verify_claim(claim: &Claim, evidence: &ParsedEvidence) -> VerifyResult {
    match claim {
        Claim::Numeric(c) => verify_numeric(c, evidence),
        Claim::Percent(c) => verify_percent(c, evidence),
        Claim::Status(c) => verify_status(c, evidence),
    }
}

/// Verify a numeric claim against memory evidence.
fn verify_numeric(claim: &NumericClaim, evidence: &ParsedEvidence) -> VerifyResult {
    if let Some(mem) = &evidence.memory {
        // Check if subject matches memory keywords
        let subject_lower = claim.subject.to_lowercase();
        if matches!(
            subject_lower.as_str(),
            "memory" | "ram" | "mem" | "total" | "used" | "free" | "available"
        ) {
            // Map subject to appropriate memory field
            let evidence_bytes = if subject_lower.contains("total") {
                Some(mem.total_bytes)
            } else if subject_lower.contains("free") {
                Some(mem.free_bytes)
            } else if subject_lower.contains("available") {
                Some(mem.available_bytes)
            } else {
                // Default to used_bytes for generic "memory" claims
                Some(mem.used_bytes)
            };

            if let Some(actual) = evidence_bytes {
                if claim.bytes == actual {
                    return VerifyResult::Verified;
                } else {
                    return VerifyResult::Contradiction {
                        claimed: format!("{}B", claim.bytes),
                        evidence: format!("{}B", actual),
                    };
                }
            }
        }

        // For process names (firefox, chrome, etc.), we don't have per-process
        // memory data yet, so these are unverifiable
    }

    VerifyResult::Unverifiable
}

/// Verify a percent claim against disk evidence.
fn verify_percent(claim: &PercentClaim, evidence: &ParsedEvidence) -> VerifyResult {
    for disk in &evidence.disks {
        if disk.mount == claim.mount {
            if disk.percent_used == claim.percent {
                return VerifyResult::Verified;
            } else {
                return VerifyResult::Contradiction {
                    claimed: format!("{}%", claim.percent),
                    evidence: format!("{}%", disk.percent_used),
                };
            }
        }
    }

    VerifyResult::Unverifiable
}

/// Verify a status claim against service evidence.
fn verify_status(claim: &StatusClaim, evidence: &ParsedEvidence) -> VerifyResult {
    for svc in &evidence.services {
        if svc.name == claim.service {
            if svc.state == claim.state {
                return VerifyResult::Verified;
            } else {
                return VerifyResult::Contradiction {
                    claimed: format_state(&claim.state),
                    evidence: format_state(&svc.state),
                };
            }
        }
    }

    VerifyResult::Unverifiable
}

/// Format ServiceState as canonical lowercase string.
fn format_state(state: &ServiceState) -> String {
    match state {
        ServiceState::Running => "running".to_string(),
        ServiceState::Active => "active".to_string(),
        ServiceState::Failed => "failed".to_string(),
        ServiceState::Inactive => "inactive".to_string(),
        ServiceState::Activating => "activating".to_string(),
        ServiceState::Deactivating => "deactivating".to_string(),
        ServiceState::Reloading => "reloading".to_string(),
        ServiceState::Unknown => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::extract_claims;
    use crate::parsers::{DiskUsage, MemoryInfo, ServiceStatus};

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
        // Extraction order is by type: numeric → percent → status
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
        assert!(matches!(report.details[0].result, VerifyResult::Unverifiable));

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

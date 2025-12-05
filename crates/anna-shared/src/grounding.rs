//! Claim verification against typed evidence.
//!
//! Computes grounding ratio by verifying extracted claims against
//! ParsedProbeData from STRUCT-lite parsers. No fuzzy matching.
//!
//! # Verification Rules
//!
//! - **Numeric claims**: exact byte equality (no tolerance)
//! - **Percent claims**: exact u8 equality with DiskUsage.percent_used
//! - **Status claims**: exact enum equality with ServiceStatus.state
//! - **Missing evidence**: claim is unverifiable (not counted as verified)

use crate::claims::{Claim, NumericClaim, PercentClaim, StatusClaim};
use crate::parsers::{DiskUsage, MemoryInfo, ParsedProbeData, ServiceStatus};
use serde::{Deserialize, Serialize};

/// Grounding verification report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundingReport {
    /// Total number of auditable claims extracted
    pub total_claims: u32,
    /// Number of claims verified against evidence
    pub verified_claims: u32,
    /// Grounding ratio: verified / total (0.0 when total == 0)
    pub grounding_ratio: f32,
    /// Individual claim verification results
    pub details: Vec<ClaimVerification>,
}

/// Result of verifying a single claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimVerification {
    /// The claim that was checked
    pub claim: Claim,
    /// Whether the claim was verified
    pub verified: bool,
    /// Reason for verification result
    pub reason: VerificationReason,
}

/// Why a claim was or wasn't verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationReason {
    /// Claim matches evidence exactly
    ExactMatch,
    /// Evidence value differs from claim
    Mismatch { expected: String, actual: String },
    /// No evidence available for this claim key
    NoEvidence,
}

/// Collected evidence from parsed probe outputs.
#[derive(Debug, Clone, Default)]
pub struct ParsedEvidence {
    /// Memory info from free -h
    pub memory: Option<MemoryInfo>,
    /// Disk usage entries from df -h
    pub disks: Vec<DiskUsage>,
    /// Service statuses from systemctl
    pub services: Vec<ServiceStatus>,
}

impl ParsedEvidence {
    /// Build evidence from a list of parsed probe data.
    pub fn from_probes(probes: &[ParsedProbeData]) -> Self {
        let mut evidence = Self::default();

        for probe in probes {
            match probe {
                ParsedProbeData::Memory(m) => {
                    evidence.memory = Some(m.clone());
                }
                ParsedProbeData::Disk(d) => {
                    evidence.disks.extend(d.iter().cloned());
                }
                ParsedProbeData::Services(s) => {
                    evidence.services.extend(s.iter().cloned());
                }
                ParsedProbeData::Service(s) => {
                    evidence.services.push(s.clone());
                }
                ParsedProbeData::BlockDevices(_)
                | ParsedProbeData::Cpu(_)
                | ParsedProbeData::JournalErrors(_)
                | ParsedProbeData::JournalWarnings(_)
                | ParsedProbeData::BootTime(_)
                | ParsedProbeData::Error(_)
                | ParsedProbeData::Unsupported => {}
            }
        }

        evidence
    }
}

/// Compute grounding report for claims against evidence.
///
/// Returns a report with:
/// - total_claims: number of auditable claims
/// - verified_claims: number verified against evidence
/// - grounding_ratio: verified / total (0.0 when total == 0)
pub fn compute_grounding(claims: &[Claim], evidence: &ParsedEvidence) -> GroundingReport {
    let mut details = Vec::with_capacity(claims.len());
    let mut verified_count = 0u32;

    for claim in claims {
        let verification = verify_claim(claim, evidence);
        if verification.verified {
            verified_count += 1;
        }
        details.push(verification);
    }

    let total = claims.len() as u32;
    let ratio = if total == 0 {
        0.0
    } else {
        verified_count as f32 / total as f32
    };

    GroundingReport {
        total_claims: total,
        verified_claims: verified_count,
        grounding_ratio: ratio,
        details,
    }
}

/// Verify a single claim against evidence.
fn verify_claim(claim: &Claim, evidence: &ParsedEvidence) -> ClaimVerification {
    match claim {
        Claim::Numeric(c) => verify_numeric(c, evidence),
        Claim::Percent(c) => verify_percent(c, evidence),
        Claim::Status(c) => verify_status(c, evidence),
    }
}

/// Verify a numeric claim.
/// Checks if claimed bytes match any known memory value.
fn verify_numeric(claim: &NumericClaim, evidence: &ParsedEvidence) -> ClaimVerification {
    // For now, we check against memory info fields
    // Future: could check process-specific memory from ps aux parser
    if let Some(mem) = &evidence.memory {
        // Check common memory fields
        let memory_values = [
            ("total", mem.total_bytes),
            ("used", mem.used_bytes),
            ("free", mem.free_bytes),
            ("available", mem.available_bytes),
            ("shared", mem.shared_bytes),
            ("buff_cache", mem.buff_cache_bytes),
        ];

        // If subject matches a memory keyword, check those values
        let subject_lower = claim.subject.to_lowercase();
        if matches!(
            subject_lower.as_str(),
            "memory" | "ram" | "mem" | "total" | "used" | "free" | "available"
        ) {
            for (name, value) in &memory_values {
                if subject_lower.contains(name) || *name == "used" && subject_lower == "memory" {
                    if claim.bytes == *value {
                        return ClaimVerification {
                            claim: Claim::Numeric(claim.clone()),
                            verified: true,
                            reason: VerificationReason::ExactMatch,
                        };
                    } else {
                        return ClaimVerification {
                            claim: Claim::Numeric(claim.clone()),
                            verified: false,
                            reason: VerificationReason::Mismatch {
                                expected: claim.bytes.to_string(),
                                actual: value.to_string(),
                            },
                        };
                    }
                }
            }
        }

        // For process names (firefox, chrome, etc.), we'd need ps aux parser
        // For now, these are unverifiable without that data
    }

    ClaimVerification {
        claim: Claim::Numeric(claim.clone()),
        verified: false,
        reason: VerificationReason::NoEvidence,
    }
}

/// Verify a percent claim against disk evidence.
fn verify_percent(claim: &PercentClaim, evidence: &ParsedEvidence) -> ClaimVerification {
    // Find disk entry by mount path
    for disk in &evidence.disks {
        if disk.mount == claim.mount {
            if disk.percent_used == claim.percent {
                return ClaimVerification {
                    claim: Claim::Percent(claim.clone()),
                    verified: true,
                    reason: VerificationReason::ExactMatch,
                };
            } else {
                return ClaimVerification {
                    claim: Claim::Percent(claim.clone()),
                    verified: false,
                    reason: VerificationReason::Mismatch {
                        expected: claim.percent.to_string(),
                        actual: disk.percent_used.to_string(),
                    },
                };
            }
        }
    }

    ClaimVerification {
        claim: Claim::Percent(claim.clone()),
        verified: false,
        reason: VerificationReason::NoEvidence,
    }
}

/// Verify a status claim against service evidence.
fn verify_status(claim: &StatusClaim, evidence: &ParsedEvidence) -> ClaimVerification {
    for svc in &evidence.services {
        if svc.name == claim.service {
            if svc.state == claim.state {
                return ClaimVerification {
                    claim: Claim::Status(claim.clone()),
                    verified: true,
                    reason: VerificationReason::ExactMatch,
                };
            } else {
                return ClaimVerification {
                    claim: Claim::Status(claim.clone()),
                    verified: false,
                    reason: VerificationReason::Mismatch {
                        expected: claim.state.to_string(),
                        actual: svc.state.to_string(),
                    },
                };
            }
        }
    }

    ClaimVerification {
        claim: Claim::Status(claim.clone()),
        verified: false,
        reason: VerificationReason::NoEvidence,
    }
}

/// Derive the answer_grounded boolean from grounding report.
///
/// Rule: answer_grounded = (grounding_ratio >= 0.5) && (total_claims > 0)
/// This prevents gaming by making no-claims answers = NOT grounded.
pub fn is_answer_grounded(report: &GroundingReport) -> bool {
    report.total_claims > 0 && report.grounding_ratio >= 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::extract_claims;
    use crate::parsers::ServiceState;

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

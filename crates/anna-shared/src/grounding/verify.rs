//! Claim verification logic (v0.0.195).

use crate::claims::{Claim, NumericClaim, PercentClaim, StatusClaim};

use super::types::{ClaimVerification, GroundingReport, ParsedEvidence, VerificationReason};

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

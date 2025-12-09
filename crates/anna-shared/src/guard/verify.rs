//! Guard verification logic (v0.0.194).

use crate::claims::{Claim, NumericClaim, PercentClaim, StatusClaim};
use crate::grounding::ParsedEvidence;
use crate::parsers::ServiceState;

use super::types::{GuardItem, GuardReport, VerifyResult};

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
    let invention_detected =
        contradictions > 0 || (unverifiable_specifics > 0 && evidence_required);

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

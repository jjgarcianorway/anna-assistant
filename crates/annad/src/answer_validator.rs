//! Answer validation with LLM-based self-healing (v0.0.376).
//!
//! Every answer goes through validation before reaching the user:
//! 1. Extract claims from answer
//! 2. Verify claims against evidence
//! 3. If validation fails, regenerate with explicit constraints
//! 4. Retry until score >= threshold or max attempts
//!
//! v0.0.376: Domain-specific validation thresholds
//! - Security: 90 (high stakes, must be accurate)
//! - System: 80 (standard reliability)
//! - Network: 75 (often partial visibility)
//! - Storage: 80 (standard reliability)
//! - Packages: 75 (version info can vary)
//!
//! This implements the principle: "Any answer gathered must always be run
//! against the specialists to know if it's the right answer or not."

use anna_shared::claims::extract_claims;
use anna_shared::grounding::{compute_grounding, ParsedEvidence};
use anna_shared::guard::{run_guard, VerifyResult};
use anna_shared::reliability::{compute_reliability, ReliabilityInput};
use anna_shared::rpc::SpecialistDomain;
use tracing::{debug, info, warn};

use crate::ollama;

/// Maximum self-healing attempts before giving up
const MAX_HEAL_ATTEMPTS: u8 = 3;

/// Base minimum acceptable reliability score (used when domain unknown)
const BASE_ACCEPTABLE_SCORE: u8 = 80;

/// v0.0.376: Get domain-specific validation threshold
/// v0.0.405: Expanded for all domains
fn domain_threshold(domain: Option<SpecialistDomain>) -> u8 {
    match domain {
        Some(SpecialistDomain::Security) => 90, // Security: high stakes
        Some(SpecialistDomain::System) => 80,   // System: standard
        Some(SpecialistDomain::Storage) => 80,  // Storage: standard
        Some(SpecialistDomain::Network) => 75,  // Network: often partial
        Some(SpecialistDomain::Packages) => 75, // Packages: versions vary
        Some(SpecialistDomain::Boot) => 80,     // Boot: standard
        Some(SpecialistDomain::Services) => 80, // Services: standard
        Some(SpecialistDomain::Audio) => 75,    // Audio: hardware varies
        Some(SpecialistDomain::Display) => 75,  // Display: hardware varies
        Some(SpecialistDomain::Desktop) => 75,  // Desktop: DE-specific
        None => BASE_ACCEPTABLE_SCORE,          // Fallback to base
    }
}

/// Result of answer validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// The (possibly revised) answer
    pub answer: String,
    /// Final reliability score
    pub score: u8,
    /// Whether the answer passed validation
    pub passed: bool,
    /// Number of heal attempts made
    pub heal_attempts: u8,
    /// Issues found during validation
    pub issues: Vec<ValidationIssue>,
    /// Detailed validation path for debugging
    pub validation_path: Vec<String>,
}

/// Types of validation issues
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationIssue {
    /// Claims not grounded in evidence
    UngroundedClaims { count: usize },
    /// Invented facts detected
    InventionDetected { claim: String },
    /// Missing required evidence
    MissingEvidence { kind: String },
    /// Answer too vague
    TooVague,
    /// Low confidence from translator
    LowConfidence { confidence: f32 },
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UngroundedClaims { count } => write!(f, "{} ungrounded claims", count),
            Self::InventionDetected { claim } => write!(f, "invented: {}", claim),
            Self::MissingEvidence { kind } => write!(f, "missing {} evidence", kind),
            Self::TooVague => write!(f, "answer too vague"),
            Self::LowConfidence { confidence } => {
                write!(f, "low confidence: {:.0}%", confidence * 100.0)
            }
        }
    }
}

/// Validate an answer against evidence and attempt self-healing if needed.
///
/// This is the core validation function that every answer should go through.
/// v0.0.376: Added optional domain for domain-specific validation thresholds.
pub async fn validate_and_heal(
    answer: &str,
    query: &str,
    evidence: &ParsedEvidence,
    reliability_input: &ReliabilityInput,
    model: &str,
    timeout_secs: u64,
) -> ValidationResult {
    // Use base threshold when domain not provided
    validate_and_heal_with_domain(
        answer,
        query,
        evidence,
        reliability_input,
        model,
        timeout_secs,
        None,
    )
    .await
}

/// v0.0.376: Validate with domain-specific thresholds
pub async fn validate_and_heal_with_domain(
    answer: &str,
    query: &str,
    evidence: &ParsedEvidence,
    reliability_input: &ReliabilityInput,
    model: &str,
    timeout_secs: u64,
    domain: Option<SpecialistDomain>,
) -> ValidationResult {
    let threshold = domain_threshold(domain);
    let mut current_answer = answer.to_string();
    let mut heal_attempts = 0;
    let mut validation_path = Vec::new();

    info!("Validation threshold for {:?}: {}", domain, threshold);

    loop {
        // Step 1: Validate current answer
        let (score, issues) = validate_answer(&current_answer, evidence, reliability_input);

        validation_path.push(format!(
            "attempt {}: score={}, issues={}, threshold={}",
            heal_attempts,
            score,
            issues.len(),
            threshold
        ));

        // Step 2: Check if we pass (v0.0.376: use domain-specific threshold)
        if score >= threshold && issues.is_empty() {
            info!(
                "Answer validated: score={} (threshold={})",
                score, threshold
            );
            return ValidationResult {
                answer: current_answer,
                score,
                passed: true,
                heal_attempts,
                issues,
                validation_path,
            };
        }

        // Step 3: Check if we've exhausted attempts
        if heal_attempts >= MAX_HEAL_ATTEMPTS {
            warn!(
                "Validation failed after {} attempts: score={} (threshold={})",
                heal_attempts, score, threshold
            );
            return ValidationResult {
                answer: current_answer,
                score,
                passed: false,
                heal_attempts,
                issues,
                validation_path,
            };
        }

        // Step 4: Attempt self-healing
        heal_attempts += 1;
        info!(
            "Self-healing attempt {}/{}: {} issues to fix",
            heal_attempts,
            MAX_HEAL_ATTEMPTS,
            issues.len()
        );

        match heal_answer(
            &current_answer,
            query,
            evidence,
            &issues,
            model,
            timeout_secs,
        )
        .await
        {
            Ok(healed) => {
                validation_path.push(format!("healed: {} chars", healed.len()));
                current_answer = healed;
            }
            Err(e) => {
                warn!("Self-healing failed: {}", e);
                validation_path.push(format!("heal error: {}", e));
                // Return with current answer if healing fails
                return ValidationResult {
                    answer: current_answer,
                    score,
                    passed: false,
                    heal_attempts,
                    issues,
                    validation_path,
                };
            }
        }
    }
}

/// Validate an answer and return score + issues
fn validate_answer(
    answer: &str,
    evidence: &ParsedEvidence,
    reliability_input: &ReliabilityInput,
) -> (u8, Vec<ValidationIssue>) {
    let mut issues = Vec::new();

    // Extract claims from answer
    let claims = extract_claims(answer);
    debug!("Extracted {} claims from answer", claims.len());

    // Compute grounding against evidence
    let grounding = compute_grounding(&claims, evidence);
    if grounding.verified_claims < grounding.total_claims {
        let ungrounded = (grounding.total_claims - grounding.verified_claims) as usize;
        if ungrounded > 0 {
            issues.push(ValidationIssue::UngroundedClaims { count: ungrounded });
        }
    }

    // Run invention guard
    let guard = run_guard(&claims, evidence, reliability_input.evidence_required);
    if guard.invention_detected {
        // Extract unverifiable claims from details
        for item in &guard.details {
            if matches!(
                item.result,
                VerifyResult::Unverifiable | VerifyResult::Contradiction { .. }
            ) {
                issues.push(ValidationIssue::InventionDetected {
                    claim: format!("{:?}", item.claim),
                });
            }
        }
    }

    // Check for missing evidence
    // Note: evidence_kinds are EvidenceKind, not String
    // Skip this check if no evidence kinds specified
    // TODO: Convert reliability_input.evidence_kinds to EvidenceKind

    // Check confidence
    if reliability_input.translator_confidence < 0.7 {
        issues.push(ValidationIssue::LowConfidence {
            confidence: reliability_input.translator_confidence,
        });
    }

    // Compute final score
    let output = compute_reliability(reliability_input);

    (output.score, issues)
}

/// Attempt to heal an answer by regenerating with constraints
async fn heal_answer(
    original_answer: &str,
    query: &str,
    evidence: &ParsedEvidence,
    issues: &[ValidationIssue],
    model: &str,
    timeout_secs: u64,
) -> anyhow::Result<String> {
    // Build correction prompt based on issues
    let correction_prompt = build_correction_prompt(original_answer, query, evidence, issues);

    debug!("Sending correction prompt to LLM");
    let response = ollama::chat_with_timeout(model, &correction_prompt, timeout_secs).await?;

    // Extract just the answer part (remove any thinking)
    let cleaned = clean_llm_response(&response);

    Ok(cleaned)
}

/// Build a prompt that instructs the LLM to fix specific issues
fn build_correction_prompt(
    original_answer: &str,
    query: &str,
    evidence: &ParsedEvidence,
    issues: &[ValidationIssue],
) -> String {
    let mut constraints: Vec<String> = Vec::new();

    for issue in issues {
        match issue {
            ValidationIssue::UngroundedClaims { .. } => {
                constraints.push(
                    "- Only make claims that are directly supported by the evidence below"
                        .to_string(),
                );
            }
            ValidationIssue::InventionDetected { ref claim } => {
                constraints.push(format!("- Do NOT claim: {}", claim));
            }
            ValidationIssue::MissingEvidence { ref kind } => {
                constraints.push(format!("- Include {} information from the evidence", kind));
            }
            ValidationIssue::TooVague => {
                constraints
                    .push("- Be specific with numbers and values from the evidence".to_string());
            }
            ValidationIssue::LowConfidence { .. } => {
                constraints.push("- Focus on answering exactly what was asked".to_string());
            }
        }
    }

    let evidence_text = evidence.summary();

    format!(
        r#"The user asked: "{}"

Your previous answer had issues. Please write a corrected answer.

EVIDENCE (use ONLY this data):
{}

CONSTRAINTS (you MUST follow these):
{}

Previous answer (has errors):
{}

Write a corrected answer that:
1. Only uses facts from the evidence
2. Directly answers the question
3. Is concise and specific

Corrected answer:"#,
        query,
        evidence_text,
        constraints.join("\n"),
        original_answer
    )
}

/// Clean LLM response (remove thinking markers, etc.)
fn clean_llm_response(response: &str) -> String {
    let mut result = response.to_string();

    // Remove <think>...</think> blocks
    while let (Some(start), Some(end)) = (result.find("<think>"), result.find("</think>")) {
        if end > start {
            result = format!("{}{}", &result[..start], &result[end + 8..]);
        } else {
            break;
        }
    }

    // Remove /no_think and similar markers
    result = result.replace("/no_think", "");
    result = result.replace("<|endofthink|>", "");

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_issue_display() {
        let issue = ValidationIssue::UngroundedClaims { count: 3 };
        assert_eq!(issue.to_string(), "3 ungrounded claims");

        let issue = ValidationIssue::InventionDetected {
            claim: "nginx is running".to_string(),
        };
        assert!(issue.to_string().contains("invented"));
    }

    #[test]
    fn test_clean_llm_response() {
        let response = "<think>Let me think...</think>The answer is 42.";
        let cleaned = clean_llm_response(response);
        assert_eq!(cleaned, "The answer is 42.");
    }

    #[test]
    fn test_domain_threshold() {
        // Security has highest threshold
        assert_eq!(domain_threshold(Some(SpecialistDomain::Security)), 90);
        // System/Storage are standard
        assert_eq!(domain_threshold(Some(SpecialistDomain::System)), 80);
        assert_eq!(domain_threshold(Some(SpecialistDomain::Storage)), 80);
        // Network/Packages allow more flexibility
        assert_eq!(domain_threshold(Some(SpecialistDomain::Network)), 75);
        assert_eq!(domain_threshold(Some(SpecialistDomain::Packages)), 75);
        // None falls back to base
        assert_eq!(domain_threshold(None), BASE_ACCEPTABLE_SCORE);
    }
}

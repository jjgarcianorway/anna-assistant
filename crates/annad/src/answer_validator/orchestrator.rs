//! Orchestration of validation and self-healing loops.
//!
//! Every answer goes through validation before reaching the user:
//! 1. Extract claims from answer
//! 2. Verify claims against evidence
//! 3. If validation fails, regenerate with explicit constraints
//! 4. Retry until score >= threshold or max attempts
//!
//! This implements the principle: "Any answer gathered must always be run
//! against the specialists to know if it's the right answer or not."

use anna_shared::grounding::ParsedEvidence;
use anna_shared::reliability::ReliabilityInput;
use anna_shared::rpc::SpecialistDomain;
use tracing::{info, warn};

use super::healing::heal_answer;
use super::thresholds::domain_threshold;
use super::types::{ValidationResult, MAX_HEAL_ATTEMPTS};
use super::validation::validate_answer;

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

//! Core validation logic for specialist responses (v0.0.428).
//!
//! Enforces:
//! - No invented numbers or facts
//! - No generic how-tos when user asked for current state
//! - Explicit unknowns (no vague language)
//! - All claims must trace to evidence

use super::{
    ResponseStatus, StrictResponse, MAX_KEY_FACTS, MAX_RECOMMENDATIONS, MAX_SUMMARY_LENGTH,
};
use super::validation_types::{ValidationError, ValidationResult};
use super::validation_checks::{
    check_forbidden_patterns, check_invented_numbers, check_generic_howto, check_vague_language,
};

/// Validate a specialist response
pub fn validate_response(response: &StrictResponse) -> ValidationResult {
    let mut errors = vec![];
    let mut warnings = vec![];

    // 1. Check for forbidden patterns (nonsense data)
    check_forbidden_patterns(&response.summary, &mut errors);
    for fact in &response.details.key_facts {
        check_forbidden_patterns(fact, &mut errors);
    }

    // 2. Check for invented numbers (numbers that don't appear in evidence)
    check_invented_numbers(response, &mut errors, &mut warnings);

    // 3. Check for generic how-tos when intent suggests state query
    check_generic_howto(response, &mut errors);

    // 4. Check for vague language in success responses
    if response.status == ResponseStatus::Success {
        check_vague_language(&response.summary, &mut errors);
        for fact in &response.details.key_facts {
            check_vague_language(fact, &mut errors);
        }
    }

    // 5. Check evidence requirements
    if response.status == ResponseStatus::Success {
        if response.evidence.probes_used.is_empty()
            && response.evidence.arch_wiki_pages.is_empty()
            && response.evidence.man_pages.is_empty()
        {
            if response.confidence > 0.7 {
                errors.push(ValidationError::MissingEvidence);
            } else {
                warnings.push("No evidence for moderate-confidence success".to_string());
            }
        }
    }

    // 6. Check confidence range
    if response.confidence < 0.0 || response.confidence > 1.0 {
        errors.push(ValidationError::InvalidConfidence(response.confidence));
    }

    // 7. Check summary length
    if response.summary.len() > MAX_SUMMARY_LENGTH {
        errors.push(ValidationError::SummaryTooLong(response.summary.len()));
    }

    // 8. Check empty summary for success
    if response.status == ResponseStatus::Success && response.summary.trim().is_empty() {
        errors.push(ValidationError::EmptySummary);
    }

    // 9. Check key facts count
    if response.details.key_facts.len() > MAX_KEY_FACTS {
        warnings.push(format!(
            "Too many key facts ({}), consider consolidating",
            response.details.key_facts.len()
        ));
    }

    // 10. Check recommendations count
    if response.details.recommendations.len() > MAX_RECOMMENDATIONS {
        warnings.push(format!(
            "Too many recommendations ({})",
            response.details.recommendations.len()
        ));
    }

    // Compute adjusted status and confidence
    let (adjusted_status, adjusted_confidence) = compute_adjustments(response, &errors);

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        adjusted_status,
        adjusted_confidence,
    }
}

/// Compute adjusted status and confidence based on validation errors
pub fn compute_adjustments(
    response: &StrictResponse,
    errors: &[ValidationError],
) -> (ResponseStatus, f32) {
    if errors.is_empty() {
        return (response.status, response.confidence);
    }

    // Serious errors that should downgrade to failure
    let serious_errors = errors.iter().any(|e| {
        matches!(
            e,
            ValidationError::InventedData(_)
                | ValidationError::ForbiddenPattern(_)
                | ValidationError::EmptySummary
        )
    });

    if serious_errors {
        return (ResponseStatus::Failure, 0.0);
    }

    // Moderate errors that should downgrade success to partial
    let moderate_errors = errors.iter().any(|e| {
        matches!(
            e,
            ValidationError::GenericHowTo
                | ValidationError::MissingEvidence
                | ValidationError::VagueLanguage(_)
        )
    });

    if moderate_errors && response.status == ResponseStatus::Success {
        return (
            ResponseStatus::Partial,
            (response.confidence * 0.5).min(0.5),
        );
    }

    // Other errors: lower confidence
    let penalty = errors.len() as f32 * 0.1;
    let adjusted_confidence = (response.confidence - penalty).max(0.1);

    (response.status, adjusted_confidence)
}

/// Check if a response is useful enough to show to user
pub fn is_useful_response(response: &StrictResponse) -> bool {
    let validation = validate_response(response);

    // Failures are honest and useful (they tell user we couldn't help)
    if response.status == ResponseStatus::Failure {
        return true;
    }

    // Success/partial must pass validation
    if !validation.valid {
        // Check if adjusted response is still useful
        return validation.adjusted_status != ResponseStatus::Failure
            && validation.adjusted_confidence > 0.0;
    }

    // Has some content
    !response.summary.trim().is_empty()
}

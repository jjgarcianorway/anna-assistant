//! Response validator with no-bullshit policy (v0.0.428).
//!
//! Enforces:
//! - No invented numbers or facts
//! - No generic how-tos when user asked for current state
//! - Explicit unknowns (no vague language)
//! - All claims must trace to evidence

use super::{
    ResponseStatus, StrictResponse, MAX_KEY_FACTS, MAX_RECOMMENDATIONS, MAX_SUMMARY_LENGTH,
};

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the response is valid
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings (response still usable)
    pub warnings: Vec<String>,
    /// Adjusted status (may be downgraded)
    pub adjusted_status: ResponseStatus,
    /// Adjusted confidence (may be lowered)
    pub adjusted_confidence: f32,
}

/// Types of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Response contains invented/nonsense data
    InventedData(String),
    /// Generic how-to when user asked for state
    GenericHowTo,
    /// Missing required evidence for success status
    MissingEvidence,
    /// Summary doesn't match evidence
    SummaryEvidenceMismatch(String),
    /// Forbidden pattern detected
    ForbiddenPattern(String),
    /// Confidence out of range
    InvalidConfidence(f32),
    /// Summary too long
    SummaryTooLong(usize),
    /// Too many key facts
    TooManyKeyFacts(usize),
    /// Empty summary for success status
    EmptySummary,
    /// Vague language in success response
    VagueLanguage(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InventedData(s) => write!(f, "Invented data detected: {}", s),
            Self::GenericHowTo => write!(f, "Generic how-to when user asked for current state"),
            Self::MissingEvidence => write!(f, "Success status but no evidence provided"),
            Self::SummaryEvidenceMismatch(s) => write!(f, "Summary doesn't match evidence: {}", s),
            Self::ForbiddenPattern(s) => write!(f, "Forbidden pattern: {}", s),
            Self::InvalidConfidence(c) => write!(f, "Invalid confidence: {}", c),
            Self::SummaryTooLong(len) => write!(f, "Summary too long: {} chars", len),
            Self::TooManyKeyFacts(n) => write!(f, "Too many key facts: {}", n),
            Self::EmptySummary => write!(f, "Empty summary for success status"),
            Self::VagueLanguage(s) => write!(f, "Vague language in success response: {}", s),
        }
    }
}

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

/// Check for forbidden patterns that indicate nonsense output
fn check_forbidden_patterns(text: &str, errors: &mut Vec<ValidationError>) {
    let lower = text.to_lowercase();

    // Patterns that indicate parse bugs or hallucinations
    let forbidden = [
        "unknown is installed",
        "unknown is not installed",
        "**unknown**",
        "2 is installed",
        "1 is installed",
        "n/a is installed",
        "null is installed",
        "undefined is installed",
        "true is installed",
        "false is installed",
    ];

    for pattern in &forbidden {
        if lower.contains(pattern) {
            errors.push(ValidationError::ForbiddenPattern(pattern.to_string()));
        }
    }

    // Patterns that indicate copied placeholder text
    let placeholders = [
        "lorem ipsum",
        "todo:",
        "fixme:",
        "placeholder",
        "example.com",
        "your_",
        "my_example",
    ];

    for pattern in &placeholders {
        if lower.contains(pattern) {
            errors.push(ValidationError::ForbiddenPattern(format!(
                "placeholder: {}",
                pattern
            )));
        }
    }
}

/// Check for invented numbers not backed by evidence
fn check_invented_numbers(
    response: &StrictResponse,
    errors: &mut Vec<ValidationError>,
    warnings: &mut Vec<String>,
) {
    // Extract numbers from summary and key_facts
    let summary_numbers = extract_numbers(&response.summary);
    let fact_numbers: Vec<String> = response
        .details
        .key_facts
        .iter()
        .flat_map(|f| extract_numbers(f))
        .collect();

    // Extract numbers from evidence summaries
    let evidence_numbers: Vec<String> = response
        .evidence
        .probes_used
        .iter()
        .flat_map(|p| extract_numbers(&p.summary))
        .collect();

    // Check if significant numbers in claims appear in evidence
    for num in summary_numbers.iter().chain(fact_numbers.iter()) {
        // Skip very common numbers (0, 1, 100, etc.)
        if is_common_number(num) {
            continue;
        }

        // Check if this number appears anywhere in evidence
        let found_in_evidence = evidence_numbers.iter().any(|e| e == num)
            || response
                .evidence
                .probes_used
                .iter()
                .any(|p| p.summary.contains(num));

        if !found_in_evidence && response.status == ResponseStatus::Success {
            // This could be invented - add warning
            warnings.push(format!("Number '{}' not found in evidence", num));
        }
    }
}

/// Extract numbers from text
fn extract_numbers(text: &str) -> Vec<String> {
    let mut numbers = vec![];
    let mut current = String::new();
    let mut in_number = false;

    for c in text.chars() {
        if c.is_ascii_digit() || (c == '.' && in_number) || (c == '%' && in_number) {
            current.push(c);
            in_number = true;
        } else {
            if in_number && !current.is_empty() {
                numbers.push(current.clone());
                current.clear();
            }
            in_number = false;
        }
    }
    if !current.is_empty() {
        numbers.push(current);
    }

    numbers
}

/// Check if a number is too common to flag
fn is_common_number(num: &str) -> bool {
    let common = ["0", "1", "2", "3", "4", "5", "10", "100", "100%", "0%"];
    common.contains(&num)
}

/// Check for generic how-to responses when user asked for current state
fn check_generic_howto(response: &StrictResponse, errors: &mut Vec<ValidationError>) {
    // Intent patterns that indicate "check current state" questions
    let state_intents = [
        "check_",
        "is_",
        "are_",
        "do_i_have",
        "show_",
        "list_",
        "get_",
        "current_",
    ];

    let is_state_query = state_intents.iter().any(|p| response.intent.contains(p));

    if !is_state_query {
        return; // Not a state query, how-tos are fine
    }

    // Patterns that indicate generic tutorial content
    let howto_patterns = [
        "step 1:",
        "step 2:",
        "step 3:",
        "first, you",
        "to troubleshoot",
        "to debug",
        "you can try",
        "here's how to",
        "follow these steps",
        "common solutions include",
        "typical approaches",
        "generally, you would",
    ];

    let summary_lower = response.summary.to_lowercase();
    let diagnosis_lower = response
        .details
        .diagnosis
        .as_ref()
        .map(|d| d.to_lowercase())
        .unwrap_or_default();

    let combined = format!("{} {}", summary_lower, diagnosis_lower);

    for pattern in &howto_patterns {
        if combined.contains(pattern) {
            // This looks like a generic how-to, not a direct state answer
            errors.push(ValidationError::GenericHowTo);
            return;
        }
    }
}

/// Check for vague language that shouldn't appear in success responses
fn check_vague_language(text: &str, errors: &mut Vec<ValidationError>) {
    let lower = text.to_lowercase();

    let vague_patterns = [
        "might be",
        "could be",
        "possibly",
        "perhaps",
        "not sure",
        "i don't know",
        "i cannot determine",
        "may help",
        "should work",
        "typically",
        "usually",
        "probably",
        "it seems",
        "appears to",
        "i think",
        "i believe",
    ];

    for pattern in &vague_patterns {
        if lower.contains(pattern) {
            errors.push(ValidationError::VagueLanguage(pattern.to_string()));
            return; // One is enough
        }
    }
}

/// Compute adjusted status and confidence based on validation errors
fn compute_adjustments(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_protocol::{ProbeEvidence, ResponseMeta};

    fn make_meta() -> ResponseMeta {
        ResponseMeta {
            handled_by: "Test".to_string(),
            ticket_id: "TEST-001".to_string(),
            version: 1,
        }
    }

    #[test]
    fn test_valid_response() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "No failed systemd services detected.",
            vec!["0 failed units".to_string()],
            vec![ProbeEvidence {
                id: "systemctl_failed".to_string(),
                summary: "0 failed units".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_forbidden_pattern() {
        let mut response = StrictResponse::success(
            "packages",
            "check_installed",
            "unknown is installed on your system",
            vec![],
            vec![],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::ForbiddenPattern(_))));
        assert_eq!(result.adjusted_status, ResponseStatus::Failure);
    }

    #[test]
    fn test_generic_howto_blocked() {
        let response = StrictResponse::success(
            "services.systemd",
            "check_failed_services",
            "Step 1: Run systemctl status. Step 2: Check the logs.",
            vec![],
            vec![],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::GenericHowTo)));
        // Should be downgraded
        assert!(
            result.adjusted_status != ResponseStatus::Success || result.adjusted_confidence < 0.8
        );
    }

    #[test]
    fn test_vague_language_blocked() {
        let response = StrictResponse::success(
            "system",
            "check_memory",
            "Your system might be running low on memory.",
            vec![],
            vec![ProbeEvidence {
                id: "free".to_string(),
                summary: "2GB available".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        );

        let result = validate_response(&response);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::VagueLanguage(_))));
    }

    #[test]
    fn test_missing_evidence_for_high_confidence() {
        let response = StrictResponse::success(
            "packages",
            "check_installed",
            "vim is installed",
            vec!["vim version 9.0".to_string()],
            vec![], // No evidence!
            make_meta(),
        )
        .with_confidence(0.95);

        let result = validate_response(&response);
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingEvidence)));
    }

    #[test]
    fn test_number_extraction() {
        let nums = extract_numbers("Root is at 97% used, 30 GiB free");
        assert!(nums.contains(&"97%".to_string()));
        assert!(nums.contains(&"30".to_string()));
    }

    #[test]
    fn test_useful_response() {
        let good = StrictResponse::success(
            "system",
            "check_ram",
            "You have 16GB RAM available.",
            vec!["16GB available".to_string()],
            vec![ProbeEvidence {
                id: "free".to_string(),
                summary: "16GB available".to_string(),
                raw_reference: None,
            }],
            make_meta(),
        );
        assert!(is_useful_response(&good));

        let bad = StrictResponse::success(
            "system",
            "check_ram",
            "unknown is installed",
            vec![],
            vec![],
            make_meta(),
        );
        assert!(!is_useful_response(&bad));
    }
}

//! Streaming Answer Validation (v0.0.889)
//!
//! Validates LLM answers as they stream, detecting:
//! - Hallucinations (claims not supported by command output)
//! - Uncertainty markers (vague language)
//! - Contradictions with command output
//! - Generic responses not specific to the system

mod checks;
mod contradiction;
mod streaming;

pub use checks::{
    check_hallucination, check_too_generic, check_uncertainty, extract_grounding_values,
    is_common_word,
};
pub use contradiction::{
    check_arithmetic_error, check_boolean_contradiction, check_contradiction,
    check_existence_contradiction, check_numeric_contradiction, check_presence_contradiction,
    check_status_contradiction, normalize_to_gb,
};
pub use streaming::{StreamingValidator, ValidationResult};

use anna_shared::rpc::{ValidationIssueType, ValidationWarning};
use regex::Regex;
use std::sync::LazyLock;

/// Pre-compiled regexes for validation
pub(crate) static RE_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d+\.?\d*)\s*(GB|MB|KB|TB|GiB|MiB|cores?|threads?|%)\b").unwrap()
});
pub(crate) static RE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([a-z][\w-]*\.service|[a-z][\w-]{3,})\b").unwrap());
pub(crate) static RE_MEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(gb|mb|tb|gib|mib|tib|gi|mi|ti|g|m|t)\b").unwrap()
});
pub(crate) static RE_CONTEXT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:is|=|:)\s*(\w+)").unwrap());

/// Confidence penalties for each issue type
const PENALTY_HALLUCINATION: f32 = 0.25;
const PENALTY_CONTRADICTION: f32 = 0.30;
const PENALTY_UNCERTAINTY: f32 = 0.10;
const PENALTY_TOO_GENERIC: f32 = 0.15;
/// Threshold below which self-correction is recommended
const CORRECTION_THRESHOLD: f32 = 0.5;

/// Validate a complete answer against command output
pub fn validate_complete_answer(answer: &str, command_output: &str) -> ValidationResult {
    validate_and_learn(answer, command_output, None)
}

/// Validate and optionally record contradictions for learning
pub fn validate_and_learn(
    answer: &str,
    command_output: &str,
    source_cmd: Option<&str>,
) -> ValidationResult {
    use anna_shared::session::ContradictionStore;

    let mut warnings = Vec::new();
    let mut confidence = 1.0f32;
    let mut correction_hints = Vec::new();
    let mut detected_contradictions: Vec<(String, String, String)> = Vec::new();

    if let Some(warning) = check_uncertainty(answer) {
        confidence -= PENALTY_UNCERTAINTY;
        warnings.push(warning);
    }

    let grounding_values = extract_grounding_values(command_output);
    if let Some(warning) = check_hallucination(answer, command_output, &grounding_values) {
        confidence -= PENALTY_HALLUCINATION;
        correction_hints.push(format!("Verify: {}", warning.message));

        if let Some(claim) = extract_claim_from_warning(&warning.message) {
            detected_contradictions.push((
                "numeric_claim".to_string(),
                claim,
                "see output".to_string(),
            ));
        }
        warnings.push(warning);
    }

    if let Some(warning) = check_too_generic(answer) {
        confidence -= PENALTY_TOO_GENERIC;
        correction_hints.push("Be more specific with system data".to_string());
        warnings.push(warning);
    }

    if let Some(warning) = check_contradiction(answer, command_output) {
        confidence -= PENALTY_CONTRADICTION;
        correction_hints.push(format!("Fix: {}", warning.message));

        if let Some((wrong, correct)) = extract_contradiction_details(&warning.message) {
            detected_contradictions.push(("status".to_string(), wrong, correct));
        }
        warnings.push(warning);
    }

    if let Some(warning) = check_existence_contradiction(answer, command_output) {
        confidence -= PENALTY_CONTRADICTION;
        correction_hints.push(format!("Entity issue: {}", warning.message));
        detected_contradictions.push((
            "existence".to_string(),
            "exists".to_string(),
            "does not exist".to_string(),
        ));
        warnings.push(warning);
    }

    if let Some(warning) = check_arithmetic_error(answer, command_output) {
        confidence -= PENALTY_HALLUCINATION;
        correction_hints.push(format!("Math error: {}", warning.message));
        if let Some(claim) = extract_claim_from_warning(&warning.message) {
            detected_contradictions.push(("arithmetic".to_string(), claim, "incorrect math".to_string()));
        }
        warnings.push(warning);
    }

    // Record contradictions for future prevention
    if !detected_contradictions.is_empty() {
        let cmd = source_cmd.unwrap_or("unknown");
        let mut store = ContradictionStore::load();
        for (claim_type, wrong, correct) in &detected_contradictions {
            store.record(claim_type, wrong, correct, cmd);
        }
        let _ = store.save();

        // Penalize experiences that suggested this command
        if let Ok(mut memory) = anna_shared::memory::Memory::load() {
            let mut penalized = false;
            for exp in memory.experiences.iter_mut() {
                if exp.successful_commands.iter().any(|c| {
                    c == cmd
                        || cmd
                            .starts_with(c.split_whitespace().next().unwrap_or(""))
                }) {
                    exp.usefulness_score = exp.usefulness_score.saturating_sub(1).max(1);
                    penalized = true;
                }
            }
            if penalized {
                let _ = memory.save();
                tracing::debug!("Penalized experiences for contradiction from command: {}", cmd);
            }
        }
    }

    confidence = confidence.max(0.0);

    ValidationResult {
        warnings,
        confidence,
        needs_correction: confidence < CORRECTION_THRESHOLD,
        correction_hint: if correction_hints.is_empty() {
            None
        } else {
            Some(correction_hints.join("; "))
        },
    }
}

/// Extract a claim value from warning message
fn extract_claim_from_warning(message: &str) -> Option<String> {
    if let Some(start) = message.find('\'') {
        if let Some(end) = message[start + 1..].find('\'') {
            return Some(message[start + 1..start + 1 + end].to_string());
        }
    }
    for word in message.split_whitespace() {
        if word.chars().any(|c| c.is_numeric()) && word.len() < 20 {
            return Some(word.to_string());
        }
    }
    None
}

/// Extract wrong/correct values from contradiction message
fn extract_contradiction_details(message: &str) -> Option<(String, String)> {
    if message.contains("says") && message.contains("but") {
        let parts: Vec<&str> = message.split("but").collect();
        if parts.len() == 2 {
            let wrong = extract_claim_from_warning(parts[0]);
            let correct = extract_claim_from_warning(parts[1]);
            if let (Some(w), Some(c)) = (wrong, correct) {
                return Some((w, c));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uncertainty_detection() {
        let warning = check_uncertainty("I think the disk might be full");
        assert!(warning.is_some());
        assert!(matches!(
            warning.unwrap().issue_type,
            ValidationIssueType::Uncertainty
        ));
    }

    #[test]
    fn test_grounding_extraction() {
        let output = "Mem: 16Gi total\n/dev/sda1 500G\nlinux 6.8.1-arch1-1";
        let values = extract_grounding_values(output);
        assert!(values.iter().any(|v| v.contains("16")));
        assert!(values.iter().any(|v| v.contains("6.8.1")));
    }

    #[test]
    fn test_no_false_positives() {
        let text = "Your system has 16GB of RAM and is running kernel 6.8.1";
        let output = "Mem: 16Gi total\nlinux 6.8.1-arch1-1";
        let warning = check_hallucination(text, output, &extract_grounding_values(output));
        assert!(warning.is_none());
    }

    #[test]
    fn test_status_contradiction() {
        let answer = "The service is running normally";
        let output = "nginx.service - A high performance web server\n   Active: inactive (dead)";
        let warning = check_contradiction(answer, output);
        assert!(warning.is_some());
        assert!(matches!(
            warning.unwrap().issue_type,
            ValidationIssueType::Contradiction
        ));
    }

    #[test]
    fn test_no_status_contradiction() {
        let answer = "The service is running";
        let output = "nginx.service\n   Active: active (running)";
        let warning = check_contradiction(answer, output);
        assert!(warning.is_none());
    }

    #[test]
    fn test_numeric_contradiction() {
        let answer = "You have 32GB of RAM";
        let output = "Mem:           15Gi";
        let warning = check_numeric_contradiction(answer, output);
        assert!(warning.is_some());
    }

    #[test]
    fn test_presence_contradiction() {
        let answer = "Firefox is installed on your system";
        let output = "error: package 'firefox' was not found";
        let warning = check_presence_contradiction(answer, output);
        assert!(warning.is_some());
    }
}

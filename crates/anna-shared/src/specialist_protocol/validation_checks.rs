//! Validation check functions for response content (v0.0.428).
//!
//! Contains pattern matching, number validation, and language checks.

use super::{ResponseStatus, StrictResponse};
use super::validation_types::ValidationError;

/// Check for forbidden patterns that indicate nonsense output
pub fn check_forbidden_patterns(text: &str, errors: &mut Vec<ValidationError>) {
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
pub fn check_invented_numbers(
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
pub fn extract_numbers(text: &str) -> Vec<String> {
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
pub fn is_common_number(num: &str) -> bool {
    let common = ["0", "1", "2", "3", "4", "5", "10", "100", "100%", "0%"];
    common.contains(&num)
}

/// Check for generic how-to responses when user asked for current state
pub fn check_generic_howto(response: &StrictResponse, errors: &mut Vec<ValidationError>) {
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
pub fn check_vague_language(text: &str, errors: &mut Vec<ValidationError>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_extraction() {
        let nums = extract_numbers("Root is at 97% used, 30 GiB free");
        assert!(nums.contains(&"97%".to_string()));
        assert!(nums.contains(&"30".to_string()));
    }
}

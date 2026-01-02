//! Guardrail violation types and checking functions.

use super::intent::IntentType;
use super::response_type::ResponseType;
use crate::specialist_protocol::schema::StrictResponse;

/// Types of guardrail violations
#[derive(Debug, Clone)]
pub enum GuardrailViolation {
    /// Response type doesn't match intent (e.g., how-to for state query)
    IntentMismatch {
        expected: IntentType,
        got: ResponseType,
    },
    /// Response contains invented facts not in evidence
    InventedFacts(Vec<String>),
    /// Response is too vague for the intent type
    TooVague,
    /// Response validation failed
    ValidationFailed(Vec<String>),
    /// Summary doesn't match evidence
    SummaryMismatch,
}

/// Check if response type matches intent type
pub fn check_intent_match(
    intent: IntentType,
    response: ResponseType,
) -> Option<GuardrailViolation> {
    let mismatch = match intent {
        IntentType::CheckState => {
            // State query should get state answer, not tutorial
            response == ResponseType::Tutorial
        }
        IntentType::HowTo => {
            // How-to query shouldn't just get state answer
            // (Tutorial or Explanation are fine)
            false // More permissive
        }
        _ => false,
    };

    if mismatch {
        Some(GuardrailViolation::IntentMismatch {
            expected: intent,
            got: response,
        })
    } else {
        None
    }
}

/// Check for facts not backed by evidence
pub fn check_invented_facts(
    response: &StrictResponse,
    probes: &std::collections::HashMap<String, String>,
) -> Option<GuardrailViolation> {
    // Skip if no probes to check against
    if probes.is_empty() {
        return None;
    }

    // Extract numbers from response
    let response_numbers: Vec<String> = extract_numbers(&response.summary)
        .into_iter()
        .chain(
            response
                .details
                .key_facts
                .iter()
                .flat_map(|f| extract_numbers(f)),
        )
        .filter(|n| !is_common_number(n))
        .collect();

    // Extract numbers from probe outputs
    let evidence_text: String = probes
        .values()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let evidence_numbers: Vec<String> = extract_numbers(&evidence_text);

    // Find numbers in response not in evidence
    let invented: Vec<String> = response_numbers
        .into_iter()
        .filter(|n| !evidence_numbers.contains(n) && !evidence_text.contains(n))
        .collect();

    if invented.is_empty() {
        None
    } else {
        Some(GuardrailViolation::InventedFacts(invented))
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

/// Check if state answer is too vague
pub fn is_vague_state_answer(response: &StrictResponse) -> bool {
    let lower = response.summary.to_lowercase();

    let vague_patterns = [
        "might",
        "could",
        "possibly",
        "perhaps",
        "typically",
        "usually",
        "generally",
        "you can try",
    ];

    vague_patterns.iter().any(|p| lower.contains(p))
}

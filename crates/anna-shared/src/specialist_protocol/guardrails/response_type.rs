//! Response type classification.

use crate::specialist_protocol::schema::{ResponseStatus, StrictResponse};

/// Response type classification (what kind of answer was given)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    /// Direct answer about current state
    StateAnswer,
    /// Tutorial/how-to guide
    Tutorial,
    /// Explanation
    Explanation,
    /// Diagnosis
    Diagnosis,
    /// Action confirmation
    ActionResult,
    /// Failure message
    Failure,
}

/// Classify what type of response was given
pub fn classify_response(response: &StrictResponse) -> ResponseType {
    let summary_lower = response.summary.to_lowercase();

    // Check for tutorial patterns
    let tutorial_patterns = [
        "step 1",
        "step 2",
        "first,",
        "to do this",
        "you can",
        "you should",
        "here's how",
        "follow these",
    ];

    if tutorial_patterns.iter().any(|p| summary_lower.contains(p)) {
        return ResponseType::Tutorial;
    }

    // Check for failure
    if response.status == ResponseStatus::Failure {
        return ResponseType::Failure;
    }

    // Check for state answer patterns
    let state_patterns = [
        "is installed",
        "is not installed",
        "is running",
        "is not running",
        "is enabled",
        "is disabled",
        "are no",
        "there are",
        "you have",
        "currently",
        "available",
        "active",
        "inactive",
    ];

    if state_patterns.iter().any(|p| summary_lower.contains(p)) {
        return ResponseType::StateAnswer;
    }

    // Check for diagnosis patterns
    let diagnosis_patterns = [
        "because",
        "the cause",
        "appears to be",
        "the issue is",
        "the problem is",
        "failed due to",
    ];

    if diagnosis_patterns.iter().any(|p| summary_lower.contains(p)) {
        return ResponseType::Diagnosis;
    }

    // Default to state answer if has facts
    if !response.details.key_facts.is_empty() {
        return ResponseType::StateAnswer;
    }

    ResponseType::Explanation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_protocol::schema::ResponseMeta;

    #[test]
    fn test_response_type_classification() {
        let state_response = StrictResponse::success(
            "system",
            "check",
            "nginx is running.",
            vec![],
            vec![],
            ResponseMeta::default(),
        );
        assert_eq!(
            classify_response(&state_response),
            ResponseType::StateAnswer
        );

        let tutorial_response = StrictResponse::success(
            "system",
            "howto",
            "Step 1: First, install the package.",
            vec![],
            vec![],
            ResponseMeta::default(),
        );
        assert_eq!(
            classify_response(&tutorial_response),
            ResponseType::Tutorial
        );
    }
}

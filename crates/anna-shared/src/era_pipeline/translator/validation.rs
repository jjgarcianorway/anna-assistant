//! Answer type validation logic.

use super::super::pipeline::AnswerType;
use super::types::{
    TranslatedAnswer, MAX_BOOLEAN_ANSWER, MAX_BRIEF_ANSWER, MAX_ENTITY_ANSWER, MAX_NUMERIC_ANSWER,
};

/// Validate answer matches expected type.
pub fn validate_answer_type(text: &str, expected: AnswerType) -> bool {
    match expected {
        AnswerType::Numeric => {
            // Should be primarily numeric
            let has_number = text.chars().any(|c| c.is_ascii_digit());
            let short_enough = text.len() <= MAX_NUMERIC_ANSWER;
            has_number && short_enough
        }
        AnswerType::Boolean => {
            let lower = text.to_lowercase();
            let starts_with_yesno = lower.starts_with("yes") || lower.starts_with("no");
            let short_enough = text.len() <= MAX_BOOLEAN_ANSWER;
            starts_with_yesno && short_enough
        }
        AnswerType::List => {
            // Lists should have commas, newlines, or bullet points
            text.contains(',') || text.contains('\n') || text.contains('•') || text.contains('-')
        }
        AnswerType::Entity => {
            // Entity should be concise
            text.len() <= MAX_ENTITY_ANSWER && !text.contains('\n')
        }
        AnswerType::Brief => text.len() <= MAX_BRIEF_ANSWER,
    }
}

/// Create a validated translated answer.
pub fn create_validated_answer(
    text: &str,
    answer_type: AnswerType,
    confidence: f64,
) -> TranslatedAnswer {
    let type_match = validate_answer_type(text, answer_type);
    TranslatedAnswer::new(text, answer_type, confidence, type_match)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_validation() {
        assert!(validate_answer_type("17.0 GiB", AnswerType::Numeric));
        assert!(!validate_answer_type("hello world", AnswerType::Numeric));

        assert!(validate_answer_type("Yes.", AnswerType::Boolean));
        assert!(validate_answer_type(
            "No, it is not enabled.",
            AnswerType::Boolean
        ));
        assert!(!validate_answer_type("Maybe", AnswerType::Boolean));
    }
}

//! Fallback classification using keyword analysis (no LLM).

use anna_shared::rpc::{DeepUnderstanding, IntentCategory, IntentClassification};

use super::CLARIFICATION_THRESHOLD;

/// Fallback understanding using keyword analysis
pub fn fallback_understanding(question: &str) -> DeepUnderstanding {
    let fallback = fallback_classification(question);

    DeepUnderstanding {
        interpreted_as: question.to_string(),
        required_info: vec![],
        missing_info: vec![],
        ambiguities: vec![],
        confidence: fallback.confidence,
        category: fallback.category,
        entities: fallback.entities,
        topic: fallback.topic,
        sub_questions: fallback.sub_questions,
        clarification_needed: fallback.clarification,
        needs_confirmation: fallback.confidence < CLARIFICATION_THRESHOLD,
        suggested_commands: vec![],
    }
}

/// Fallback classification using keywords (when LLM response is malformed)
pub fn fallback_classification(question: &str) -> IntentClassification {
    let q = question.to_lowercase();

    // Check for multi-question patterns first
    let has_and = q.contains(" and ") || q.contains(" also ");
    let has_multiple_questions = q.matches('?').count() > 1;
    if has_and && (q.contains("what") || q.contains("how")) || has_multiple_questions {
        return IntentClassification {
            category: IntentCategory::Multi,
            confidence: 0.4,
            sub_questions: None,
            clarification: None,
            entities: vec![],
            topic: None,
        };
    }

    // Check for unclear/vague patterns
    let vague_patterns = ["fix it", "help me", "the thing", "that stuff", "do it"];
    if vague_patterns.iter().any(|p| q.contains(p)) || q.split_whitespace().count() <= 2 {
        return IntentClassification {
            category: IntentCategory::Unclear,
            confidence: 0.5,
            sub_questions: None,
            clarification: Some("Could you please be more specific about what you're asking?".into()),
            entities: vec![],
            topic: None,
        };
    }

    // Check for troubleshooting patterns
    let troubleshoot_patterns = [
        "not working",
        "doesn't work",
        "error",
        "fail",
        "broken",
        "why is",
        "why does",
        "why can't",
        "fix",
        "problem",
        "issue",
    ];
    if troubleshoot_patterns.iter().any(|p| q.contains(p)) {
        return IntentClassification {
            category: IntentCategory::Troubleshoot,
            confidence: 0.5,
            sub_questions: None,
            clarification: None,
            entities: vec![],
            topic: None,
        };
    }

    // Check for how-to patterns
    let howto_patterns = [
        "how do i",
        "how can i",
        "how to",
        "how should i",
        "install",
        "configure",
        "setup",
        "set up",
        "enable",
        "disable",
    ];
    if howto_patterns.iter().any(|p| q.contains(p)) {
        return IntentClassification {
            category: IntentCategory::HowTo,
            confidence: 0.5,
            sub_questions: None,
            clarification: None,
            entities: vec![],
            topic: None,
        };
    }

    // Default to factual
    IntentClassification {
        category: IntentCategory::Factual,
        confidence: 0.5,
        sub_questions: None,
        clarification: None,
        entities: vec![],
        topic: None,
    }
}

/// Fallback decomposition without LLM
pub fn fallback_decompose(question: &str) -> Vec<String> {
    let mut parts = Vec::new();

    // Split on common conjunctions
    let separators = [" and also ", " and ", " also ", "; "];

    let mut remaining = question.to_string();
    for sep in separators {
        let lower = remaining.to_lowercase();
        if let Some(idx) = lower.find(sep) {
            let first = remaining[..idx].trim().to_string();
            let second = remaining[idx + sep.len()..].trim().to_string();

            if !first.is_empty() && first.split_whitespace().count() >= 2 {
                parts.push(first);
            }
            remaining = second;
        }
    }

    if !remaining.is_empty() && remaining.split_whitespace().count() >= 2 {
        parts.push(remaining);
    }

    // If we couldn't split meaningfully, return original
    if parts.len() < 2 {
        return vec![question.to_string()];
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_factual() {
        let result = fallback_classification("what is my kernel version?");
        assert_eq!(result.category, IntentCategory::Factual);
    }

    #[test]
    fn test_fallback_howto() {
        let result = fallback_classification("how do I install neovim?");
        assert_eq!(result.category, IntentCategory::HowTo);
    }

    #[test]
    fn test_fallback_troubleshoot() {
        let result = fallback_classification("wifi is not working");
        assert_eq!(result.category, IntentCategory::Troubleshoot);
    }

    #[test]
    fn test_fallback_unclear() {
        let result = fallback_classification("fix it");
        assert_eq!(result.category, IntentCategory::Unclear);
    }

    #[test]
    fn test_fallback_decompose() {
        let parts = fallback_decompose("show disk usage and also memory usage");
        assert_eq!(parts.len(), 2);
    }
}

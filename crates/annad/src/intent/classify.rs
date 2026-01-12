//! LLM-based intent classification functions.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory, IntentClassification};
use anyhow::Result;
use tracing::{debug, info};

use super::detect::{has_specific_symptom, is_clear_error_report, is_investigation_question, is_known_system_context, is_semantically_destructive};
use super::fallback::fallback_decompose;
use super::parse::{extract_json_array_from_response, parse_quick_response, parse_understanding_response};
use super::CLARIFICATION_THRESHOLD;
use crate::ollama;

pub use super::detect::detect_off_topic;

/// Timeout for quick classification
const QUICK_TIMEOUT_SECS: u64 = 8;

/// Timeout for deep understanding
const DEEP_TIMEOUT_SECS: u64 = 20;

/// v0.0.895: Quick classification - lightweight prompt for simple queries
pub async fn quick_classify(model: &str, question: &str) -> Result<DeepUnderstanding> {
    let prompt = format!(
        r#"Classify this Linux question. Reply ONLY with JSON, no other text.

Question: "{}"

JSON format: {{"interpreted_as":"brief paraphrase","confidence":0.9,"category":"FACTUAL","entities":["item1"],"topic":"packages"}}

Categories: FACTUAL (status/info queries), HOWTO (instructions), TROUBLESHOOT (fix problems), UNCLEAR (vague/ambiguous)
Topics: network, audio, storage, boot, packages, services, security, performance, display, null"#,
        question
    );

    debug!("Quick classify prompt: {} chars", prompt.len());
    let response = ollama::chat_with_timeout(model, &prompt, QUICK_TIMEOUT_SECS).await?;
    debug!("Quick classify response: {}", response.trim());

    parse_quick_response(&response, question)
}

/// Deep understanding with full chain-of-thought (for complex cases only)
pub async fn deep_understand(
    model: &str,
    question: &str,
    session_context: Option<&str>,
) -> Result<DeepUnderstanding> {
    info!("Using deep understanding for complex question");

    let context_section = session_context
        .filter(|c| !c.is_empty())
        .map(|c| format!("\nPrevious context:\n{}", c))
        .unwrap_or_default();

    let prompt = format!(
        r#"Analyze this user request carefully. Output JSON only.

Request: "{question}"
{context}

Consider:
1. What are they asking? (paraphrase)
2. Is anything critical missing? (which service? which file?)
3. Could this mean multiple things?
4. Confidence 0.0-1.0 (0.9+=clear, 0.7-0.9=mostly clear, <0.7=unclear)

JSON: {{"interpreted_as":"...","missing_info":["item1"],"ambiguities":[],"confidence":0.85,"category":"FACTUAL/HOWTO/TROUBLESHOOT/UNCLEAR","entities":["entity1"],"topic":"packages/services/network/etc","clarification_needed":"question if unclear"}}"#,
        question = question,
        context = context_section
    );

    debug!("Deep understanding prompt: {} chars", prompt.len());
    let response = ollama::chat_with_timeout(model, &prompt, DEEP_TIMEOUT_SECS).await?;
    debug!("Deep understanding response: {}", response.trim());

    parse_understanding_response(&response, question)
}

/// Legacy classify_intent for backward compatibility
pub async fn classify_intent(
    model: &str,
    question: &str,
    session_context: Option<&str>,
) -> Result<IntentClassification> {
    let understanding = super::understand_request(model, question, session_context).await?;

    Ok(IntentClassification {
        category: understanding.category,
        confidence: understanding.confidence,
        sub_questions: understanding.sub_questions,
        clarification: understanding.clarification_needed,
        entities: understanding.entities,
        topic: understanding.topic,
    })
}

/// v0.0.898: Decompose a MULTI question into sub-questions
pub async fn decompose_multi_question(model: &str, question: &str) -> Result<Vec<String>> {
    let prompt = format!(
        r#"Break this into separate questions. Output JSON array only.

Question: "{}"

Rules:
1. Each sub-question should be independently answerable
2. Preserve the original intent of each part
3. Keep it simple - don't add questions that weren't asked

JSON: ["question 1", "question 2", ...]"#,
        question
    );

    let response = ollama::chat_with_timeout(model, &prompt, QUICK_TIMEOUT_SECS).await?;

    let json_str = extract_json_array_from_response(&response);
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(&json_str) {
        if !arr.is_empty() {
            return Ok(arr);
        }
    }

    Ok(fallback_decompose(question))
}

/// Determine if Anna should ask for confirmation before proceeding
/// v0.1.2: Simplified - only confirm for destructive actions or really unclear questions
pub fn should_ask_confirmation(
    confidence: f32,
    missing_info: &[String],
    ambiguities: &[String],
    category: &IntentCategory,
    question: &str,
) -> bool {
    let q_lower = question.to_lowercase();

    // ONLY ask confirmation for destructive actions - always
    if is_semantically_destructive(&q_lower) {
        info!("Potentially destructive action detected, will confirm");
        return true;
    }

    // Don't ask for confirmation for any clear question type
    // Error reports, investigation questions - just answer
    if is_clear_error_report(&q_lower) || is_investigation_question(&q_lower) {
        return false;
    }

    // FACTUAL/HOWTO/TROUBLESHOOT with any reasonable confidence - just answer
    // The LLM can figure it out
    if matches!(
        category,
        IntentCategory::Factual | IntentCategory::HowTo | IntentCategory::Troubleshoot
    ) {
        return false;
    }

    // UNCLEAR category with VERY low confidence - ask
    // But only if confidence is really bad (< 0.3)
    if matches!(category, IntentCategory::Unclear) && confidence < 0.3 {
        info!(
            "UNCLEAR with very low confidence ({:.0}%), asking for clarification",
            confidence * 100.0
        );
        return true;
    }

    // Default: don't ask, just try to answer
    // It's better to give a wrong answer than to annoy users with questions
    let _ = (missing_info, ambiguities); // Mark as intentionally unused
    false
}

/// Format intent result for display
pub fn format_intent_result(intent: &IntentClassification) -> String {
    let category_str = match intent.category {
        IntentCategory::Factual => "FACTUAL",
        IntentCategory::HowTo => "HOWTO",
        IntentCategory::Troubleshoot => "TROUBLESHOOT",
        IntentCategory::Multi => "MULTI",
        IntentCategory::Unclear => "UNCLEAR",
    };

    let mut result = format!("{} ({:.0}%)", category_str, intent.confidence * 100.0);

    if let Some(ref topic) = intent.topic {
        result.push_str(&format!(" [{}]", topic));
    }

    if !intent.entities.is_empty() {
        result.push_str(&format!(" entities: {}", intent.entities.join(", ")));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::detect::{detect_off_topic, is_known_system_context};

    #[test]
    fn test_needs_confirmation_very_low_confidence() {
        // v0.2.3: Only UNCLEAR category with very low confidence triggers confirmation
        let result =
            should_ask_confirmation(0.2, &[], &[], &IntentCategory::Unclear, "do something");
        assert!(result);
    }

    #[test]
    fn test_no_confirmation_howto_even_low_confidence() {
        // v0.2.3: HowTo never asks for confirmation (LLM handles it)
        let missing = vec!["which service".to_string()];
        let result = should_ask_confirmation(
            0.4,
            &missing,
            &[],
            &IntentCategory::HowTo,
            "enable the service",
        );
        assert!(!result); // HowTo doesn't ask confirmation
    }

    #[test]
    fn test_no_confirmation_missing_info_high_confidence() {
        let missing = vec!["which service".to_string()];
        let result = should_ask_confirmation(
            0.9,
            &missing,
            &[],
            &IntentCategory::HowTo,
            "enable the service",
        );
        assert!(!result);
    }

    #[test]
    fn test_factual_no_confirmation() {
        let result =
            should_ask_confirmation(0.6, &[], &[], &IntentCategory::Factual, "what is X?");
        assert!(!result);
    }

    #[test]
    fn test_known_system_context() {
        assert!(is_known_system_context("operating system"));
        assert!(is_known_system_context("package manager"));
        assert!(is_known_system_context("init system"));
        assert!(!is_known_system_context("specific file location"));
    }

    #[test]
    fn test_detect_off_topic() {
        assert!(detect_off_topic("what is the meaning of life?").is_some());
        assert!(detect_off_topic("how do I install neovim?").is_none());
    }
}

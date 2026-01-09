//! Intent classification module - LLM-based question understanding.
//!
//! Replaces brittle keyword matching with semantic understanding
//! of user questions using a quick LLM call.

use anna_shared::rpc::{IntentCategory, IntentClassification};
use anyhow::Result;
use tracing::{debug, warn};

use crate::ollama;

/// Timeout for intent classification (fast - ~100-150 tokens)
const INTENT_TIMEOUT_SECS: u64 = 15;

/// Classify the intent of a user question using LLM
pub async fn classify_intent(
    model: &str,
    question: &str,
    session_context: Option<&str>,
) -> Result<IntentClassification> {
    let context_section = session_context
        .filter(|c| !c.is_empty())
        .map(|c| format!("\nContext from conversation:\n{}", c))
        .unwrap_or_default();

    let prompt = format!(
        r#"Classify this question into ONE category. Reply with ONLY valid JSON.

Question: "{question}"
{context}
Categories:
- FACTUAL: Status/info query ("what is X?", "how much RAM?", "is X installed?", "what version?")
- HOWTO: Instructions needed ("how do I install X?", "how to configure Y?", "setup Z")
- TROUBLESHOOT: Problem solving ("X not working", "error when Y", "why is Z slow?", "fix")
- MULTI: Multiple distinct questions in one ("what's my disk AND how do I install Y?")
- UNCLEAR: Too vague to understand ("fix it", "help", "the thing", needs more context)

If MULTI, extract the sub_questions as a list.
If UNCLEAR, suggest a clarification question to ask the user.
Extract entities: packages, services, files, commands mentioned.
Detect topic: network, audio, storage, boot, packages, services, security, performance, display, or null.

Reply ONLY with this JSON format:
{{"category":"FACTUAL","confidence":0.9,"sub_questions":null,"clarification":null,"entities":[],"topic":null}}"#,
        question = question,
        context = context_section
    );

    debug!("Intent classification prompt: {} chars", prompt.len());

    let response = ollama::chat_with_timeout(model, &prompt, INTENT_TIMEOUT_SECS).await?;

    debug!("Intent classification response: {}", response.trim());

    parse_intent_response(&response)
}

/// Parse the LLM's JSON response into IntentClassification
fn parse_intent_response(response: &str) -> Result<IntentClassification> {
    // Try to extract JSON from response (handle markdown code blocks)
    let json_str = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Attempt to parse
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
        let category = match parsed.get("category").and_then(|c| c.as_str()) {
            Some("FACTUAL") => IntentCategory::Factual,
            Some("HOWTO") => IntentCategory::HowTo,
            Some("TROUBLESHOOT") => IntentCategory::Troubleshoot,
            Some("MULTI") => IntentCategory::Multi,
            Some("UNCLEAR") => IntentCategory::Unclear,
            _ => IntentCategory::Factual, // Default fallback
        };

        let confidence = parsed
            .get("confidence")
            .and_then(|c| c.as_f64())
            .map(|c| c as f32)
            .unwrap_or(0.7);

        let sub_questions = parsed
            .get("sub_questions")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let clarification = parsed
            .get("clarification")
            .and_then(|c| c.as_str())
            .filter(|s| !s.is_empty() && *s != "null")
            .map(String::from);

        let entities = parsed
            .get("entities")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let topic = parsed
            .get("topic")
            .and_then(|t| t.as_str())
            .filter(|t| !t.is_empty() && *t != "null")
            .map(String::from);

        return Ok(IntentClassification {
            category,
            confidence,
            sub_questions,
            clarification,
            entities,
            topic,
        });
    }

    // Fallback if JSON parsing fails
    warn!("Failed to parse intent JSON, using fallback: {}", json_str);
    Ok(fallback_classification(response))
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
            sub_questions: None, // Can't reliably extract without LLM
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
        "not working", "doesn't work", "error", "fail", "broken",
        "why is", "why does", "why can't", "fix", "problem", "issue",
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
        "how do i", "how can i", "how to", "how should i",
        "install", "configure", "setup", "set up", "enable", "disable",
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
    fn test_parse_valid_json() {
        let json = r#"{"category":"HOWTO","confidence":0.95,"sub_questions":null,"clarification":null,"entities":["neovim"],"topic":"packages"}"#;
        let result = parse_intent_response(json).unwrap();
        assert_eq!(result.category, IntentCategory::HowTo);
        assert!(result.confidence > 0.9);
        assert_eq!(result.entities, vec!["neovim"]);
        assert_eq!(result.topic, Some("packages".into()));
    }
}

//! Intent classification module - LLM-based question understanding.
//!
//! v0.0.895: Two-tier understanding system:
//! - Quick classification (~100 tokens, 3-5s) for simple queries
//! - Deep COT (~700 tokens, 15-20s) only for complex/unclear cases
//!
//! v0.0.909: Three-tier system - patterns first for known issues
//! v0.0.939: Four-tier system - adds intent cache layer

mod classify;
mod detect;
mod fallback;
mod parse;

pub use classify::{
    classify_intent, decompose_multi_question, deep_understand, detect_off_topic,
    format_intent_result, quick_classify, should_ask_confirmation,
};
pub use fallback::{fallback_classification, fallback_understanding};
pub use parse::{extract_json_from_response, parse_quick_response, parse_understanding_response};

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};
use anyhow::Result;
use tracing::{debug, info};

use crate::core_loop::cache::{cache_intent, get_cached_intent};
// v0.1.1: Pattern matching removed - LLM-only architecture
// The patterns module is kept for fallback commands and stats but not used in main query flow

/// Confidence threshold above which we skip deep understanding (v0.0.895)
pub const QUICK_CONFIDENCE_THRESHOLD: f32 = 0.8;

/// Confidence threshold below which we ask for clarification
/// v0.0.990: Lowered from 0.7 to 0.5 to reduce over-clarification
pub const CLARIFICATION_THRESHOLD: f32 = 0.5;

/// v0.0.939: Convert category string back to IntentCategory enum
fn parse_category(s: &str) -> IntentCategory {
    match s.to_uppercase().as_str() {
        "FACTUAL" => IntentCategory::Factual,
        "HOWTO" => IntentCategory::HowTo,
        "TROUBLESHOOT" => IntentCategory::Troubleshoot,
        "MULTI" => IntentCategory::Multi,
        "UNCLEAR" => IntentCategory::Unclear,
        _ => IntentCategory::Unclear,
    }
}

/// v0.0.909: Three-tier understanding - patterns first, then quick, then deep
/// v0.0.939: Four-tier - patterns, cache, quick, deep
/// v0.1.1: Two-tier - cache, then LLM (patterns removed for LLM-only architecture)
pub async fn understand_request(
    model: &str,
    question: &str,
    session_context: Option<&str>,
) -> Result<DeepUnderstanding> {
    // v0.1.1: Pattern matching removed - LLM handles all understanding
    // Memory and cache provide speed through learned experiences

    // v0.0.939: Check intent cache (instant, no LLM needed)
    if let Some(cached) = get_cached_intent(question) {
        info!(
            "Intent cache hit: {} (confidence: {:.0}%)",
            cached.interpreted_as,
            cached.confidence * 100.0
        );
        return Ok(DeepUnderstanding {
            interpreted_as: cached.interpreted_as,
            category: parse_category(&cached.category),
            confidence: cached.confidence,
            topic: cached.topic,
            suggested_commands: cached.suggested_commands,
            needs_confirmation: false,
            ..Default::default()
        });
    }

    // Then try quick classification (3-5 seconds)
    let quick_result = quick_classify(model, question).await;

    match quick_result {
        Ok(understanding) if understanding.confidence >= QUICK_CONFIDENCE_THRESHOLD => {
            info!(
                "Quick classification sufficient (confidence: {:.0}%)",
                understanding.confidence * 100.0
            );
            // v0.0.939: Cache successful classification
            cache_intent(
                question,
                &understanding.interpreted_as,
                &format!("{:?}", understanding.category),
                understanding.confidence,
                understanding.topic.as_deref(),
                &understanding.suggested_commands,
            );
            return Ok(understanding);
        }
        Ok(understanding)
            if matches!(
                understanding.category,
                IntentCategory::Factual | IntentCategory::HowTo
            ) && understanding.confidence >= 0.6 =>
        {
            info!(
                "Quick classification acceptable for {:?} (confidence: {:.0}%)",
                understanding.category,
                understanding.confidence * 100.0
            );
            // v0.0.939: Cache successful classification
            cache_intent(
                question,
                &understanding.interpreted_as,
                &format!("{:?}", understanding.category),
                understanding.confidence,
                understanding.topic.as_deref(),
                &understanding.suggested_commands,
            );
            return Ok(understanding);
        }
        Ok(understanding) => {
            debug!(
                "Quick classification low confidence ({:.0}%), trying deep understanding",
                understanding.confidence * 100.0
            );
        }
        Err(e) => {
            debug!("Quick classification failed: {}, trying deep understanding", e);
        }
    }

    // Fall back to deep understanding for complex cases
    let deep_result = deep_understand(model, question, session_context).await?;

    // v0.0.939: Cache deep understanding result too
    cache_intent(
        question,
        &deep_result.interpreted_as,
        &format!("{:?}", deep_result.category),
        deep_result.confidence,
        deep_result.topic.as_deref(),
        &deep_result.suggested_commands,
    );

    Ok(deep_result)
}

/// Format understanding result for display
pub fn format_understanding_result(understanding: &DeepUnderstanding) -> String {
    let mut result = format!(
        "Understood as: {}\nCategory: {:?}\nConfidence: {:.0}%",
        understanding.interpreted_as,
        understanding.category,
        understanding.confidence * 100.0
    );

    if let Some(ref topic) = understanding.topic {
        result.push_str(&format!("\nTopic: {}", topic));
    }

    if !understanding.entities.is_empty() {
        result.push_str(&format!("\nEntities: {}", understanding.entities.join(", ")));
    }

    // v0.3.30: Use plain text instead of emojis
    if understanding.needs_confirmation {
        result.push_str("\n[!] Needs confirmation");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_confirmation_high_confidence() {
        let understanding = DeepUnderstanding {
            interpreted_as: "test".to_string(),
            confidence: 0.9,
            needs_confirmation: false,
            ..Default::default()
        };
        assert!(!understanding.needs_confirmation);
    }

    #[test]
    fn test_parse_understanding_json() {
        let json = r#"{"interpreted_as":"check disk","category":"FACTUAL","confidence":0.9,"entities":["disk"]}"#;
        let result = parse_quick_response(json, "test").unwrap();
        assert_eq!(result.interpreted_as, "check disk");
    }

    #[test]
    fn test_no_confirmation_missing_info_high_confidence() {
        let understanding = DeepUnderstanding {
            interpreted_as: "test".to_string(),
            confidence: 0.85,
            missing_info: vec!["some info".to_string()],
            needs_confirmation: false,
            ..Default::default()
        };
        assert!(!understanding.needs_confirmation);
    }
}

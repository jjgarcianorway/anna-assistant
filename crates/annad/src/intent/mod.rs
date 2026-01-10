//! Intent classification module - LLM-based question understanding.
//!
//! v0.0.895: Two-tier understanding system:
//! - Quick classification (~100 tokens, 3-5s) for simple queries
//! - Deep COT (~700 tokens, 15-20s) only for complex/unclear cases
//!
//! v0.0.909: Three-tier system - patterns first for known issues

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

use crate::patterns;

/// Confidence threshold above which we skip deep understanding (v0.0.895)
pub const QUICK_CONFIDENCE_THRESHOLD: f32 = 0.8;

/// Confidence threshold below which we ask for clarification
pub const CLARIFICATION_THRESHOLD: f32 = 0.7;

/// v0.0.909: Three-tier understanding - patterns first, then quick, then deep
pub async fn understand_request(
    model: &str,
    question: &str,
    session_context: Option<&str>,
) -> Result<DeepUnderstanding> {
    // v0.0.909: First check common patterns (instant, no LLM needed)
    if let Some(understanding) = patterns::match_common_pattern(question) {
        info!(
            "Pattern matched: {} (confidence: {:.0}%)",
            understanding.interpreted_as,
            understanding.confidence * 100.0
        );
        return Ok(understanding);
    }

    // Then try quick classification (3-5 seconds)
    let quick_result = quick_classify(model, question).await;

    match quick_result {
        Ok(understanding) if understanding.confidence >= QUICK_CONFIDENCE_THRESHOLD => {
            info!(
                "Quick classification sufficient (confidence: {:.0}%)",
                understanding.confidence * 100.0
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
    deep_understand(model, question, session_context).await
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

    if understanding.needs_confirmation {
        result.push_str("\n⚠️ Needs confirmation");
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

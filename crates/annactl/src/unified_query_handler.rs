//! Unified Query Handler - v7.0.0 Clean Brain Architecture
//!
//! ALL queries flow through the v7 brain pipeline:
//! PLAN (LLM) → EXECUTE (Rust) → INTERPRET (LLM)
//!
//! Strict data contracts, reliability scoring, and retry logic.
//! NO legacy handlers, NO hardcoded recipes, NO shortcut paths.

use anna_common::action_plan_v3::ActionPlan;
use anna_common::llm_client::LlmConfig;
use anna_common::telemetry::SystemTelemetry;
use anyhow::Result;

/// Unified query result - simplified for v7.0.0
#[derive(Debug)]
pub enum UnifiedQueryResult {
    /// Action plan from planner pipeline
    ActionPlan {
        action_plan: ActionPlan,
        raw_json: String,
    },
    /// Conversational answer from interpreter
    ConversationalAnswer {
        answer: String,
        confidence: AnswerConfidence,
        sources: Vec<String>,
    },
    /// Error response
    Error {
        message: String,
    },
}

/// Answer confidence level
#[derive(Debug, Clone, Copy)]
pub enum AnswerConfidence {
    High,   // Reliability >= 0.8
    Medium, // Reliability 0.4-0.8
    Low,    // Reliability < 0.4
}

/// v7.0.0: All queries go through the brain_v7 pipeline
pub async fn process_unified_query(
    user_text: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<UnifiedQueryResult> {
    // v7.0.0: Use the clean brain architecture
    use crate::query_handler_v7;

    // Process through the v7 pipeline
    match query_handler_v7::handle_query_v7(user_text, telemetry, Some(llm_config)).await {
        Ok(result) => {
            // Convert to unified result
            Ok(UnifiedQueryResult::ConversationalAnswer {
                answer: result,
                confidence: AnswerConfidence::High,
                sources: vec!["brain_v7 (PLAN→EXECUTE→INTERPRET)".to_string()],
            })
        }
        Err(e) => {
            Ok(UnifiedQueryResult::Error {
                message: format!("Pipeline error: {}", e),
            })
        }
    }
}

/// Check if a query should use the planner pipeline
/// v6.57.0: ALWAYS returns true - everything goes through the pipeline
pub fn should_use_planner(_query: &str) -> bool {
    true // v6.57.0: Single pipeline for everything
}

/// Normalize query text for intent detection
pub fn normalize_query_for_intent(query: &str) -> String {
    query.to_lowercase().trim().to_string()
}

/// Check if this is a sysadmin report query
/// v6.57.0: Simplified - just pattern matching, no special handling
pub fn is_sysadmin_report_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("full report")
        || lower.contains("system report")
        || lower.contains("status report")
        || lower.contains("diagnostic report")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_uses_planner() {
        // v6.57.0: EVERYTHING goes through the pipeline
        assert!(should_use_planner("how much ram do I have"));
        assert!(should_use_planner("install firefox"));
        assert!(should_use_planner("what is the weather"));
        assert!(should_use_planner("random garbage query"));
    }

    #[test]
    fn test_normalize_query() {
        assert_eq!(normalize_query_for_intent("  Hello WORLD  "), "hello world");
    }

    #[test]
    fn test_sysadmin_report_detection() {
        assert!(is_sysadmin_report_query("give me a full report"));
        assert!(is_sysadmin_report_query("show system report"));
        assert!(!is_sysadmin_report_query("how much ram"));
    }
}

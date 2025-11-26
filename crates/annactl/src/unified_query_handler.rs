//! Unified Query Handler - v6.57.0 Single Pipeline
//!
//! ALL queries flow through ONE path:
//! `planner_core` → `executor_core` → `interpreter_core` → `trace_renderer`
//!
//! NO legacy handlers, NO hardcoded recipes, NO shortcut paths.
//!
//! This is the scorched-earth cleanup version. If you need to add a feature,
//! it must integrate with the planner_query_handler pipeline.

use anna_common::action_plan_v3::ActionPlan;
use anna_common::llm_client::LlmConfig;
use anna_common::telemetry::SystemTelemetry;
use anyhow::Result;

/// Unified query result - simplified for v6.57.0
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
    High,   // From telemetry/system data validated by interpreter
    Medium, // From LLM with validation
    Low,    // Fallback answer
}

/// v6.57.0: All queries go through the single pipeline
///
/// This function exists for backwards compatibility. The actual
/// work is delegated to planner_query_handler.
pub async fn process_unified_query(
    user_text: &str,
    telemetry: &SystemTelemetry,
    llm_config: &LlmConfig,
) -> Result<UnifiedQueryResult> {
    // v6.57.0: Single pipeline - delegate to planner_query_handler
    use crate::planner_query_handler;

    // Process through the unified pipeline
    match planner_query_handler::handle_with_planner(user_text, telemetry, Some(llm_config)).await {
        Ok(result) => {
            // Convert planner result to unified result
            Ok(UnifiedQueryResult::ConversationalAnswer {
                answer: result,
                confidence: AnswerConfidence::High,
                sources: vec!["Unified Pipeline (planner→executor→interpreter)".to_string()],
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

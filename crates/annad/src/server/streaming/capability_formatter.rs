//! Unified Capability Response Formatter
//!
//! Phase 34: Single canonical formatting path for all capability responses.
//! All capability output (ReadOnly and Mutating) goes through this module.
//!
//! Contracts:
//! - ReadOnly: "Anna:" + concise explanation + evidence (capped at 3 lines)
//! - Mutating: "Anna:" + explanation + confirmation prompt
//! - Evidence: capped, stable order, no blank lines, no raw commands
//! - Debug mode: shows probes/steps, deterministic, clearly labeled

use anna_shared::action_plan::ActionPlan;
use anna_shared::capability::{CapabilityExecutionResult, ResponseOutcome};
use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;

use super::helpers::send_filtered_final_answer;

// =============================================================================
// Phase 34: Formatting Constants
// =============================================================================

/// Maximum evidence lines for ReadOnly capabilities in non-Debug mode.
pub const MAX_EVIDENCE_LINES: usize = 3;

/// Debug mode environment variable.
const DEBUG_ENV_VAR: &str = "ANNA_DEBUG";

// =============================================================================
// Phase 34: LLM Call Counter (Test Infrastructure)
// =============================================================================
// The actual counter is defined in crate::lib.rs and hooked into ollama/mod.rs.
// Re-export for convenience in tests.

#[cfg(test)]
pub use crate::{get_llm_call_count, reset_llm_call_counter};

// =============================================================================
// Phase 34: Environment Helpers
// =============================================================================

/// Check if debug mode is enabled.
fn is_debug_mode() -> bool {
    std::env::var(DEBUG_ENV_VAR).map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false)
}

// =============================================================================
// Phase 34: Evidence Formatting
// =============================================================================

/// Enforce evidence cap on output.
/// Limits output to max_lines non-empty lines, stable order, no blanks.
pub fn enforce_evidence_cap(explanation: &str, max_lines: usize) -> String {
    explanation
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format evidence for ReadOnly capability (non-Debug mode).
fn format_readonly_evidence(result: &CapabilityExecutionResult) -> String {
    if result.explanation.is_empty() {
        return "No information available.".to_string();
    }

    if is_debug_mode() {
        // Debug mode: show full output with labels
        format_debug_evidence(result)
    } else {
        // Non-debug: cap at MAX_EVIDENCE_LINES
        enforce_evidence_cap(&result.explanation, MAX_EVIDENCE_LINES)
    }
}

/// Format evidence for Debug mode (shows probes/steps, clearly labeled).
fn format_debug_evidence(result: &CapabilityExecutionResult) -> String {
    let mut output = String::new();

    // Show evidence artifacts if any
    if !result.evidence.is_empty() {
        output.push_str("[DEBUG] Evidence:\n");
        for artifact in &result.evidence {
            output.push_str(&format!("  - {}: {}\n", artifact.label, artifact.content));
        }
    }

    // Show steps if any
    if !result.steps.is_empty() {
        output.push_str("[DEBUG] Steps:\n");
        for (i, step) in result.steps.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, step.content));
        }
    }

    // Always include explanation
    if !result.explanation.is_empty() {
        if !output.is_empty() {
            output.push_str("[DEBUG] Explanation:\n");
        }
        output.push_str(&result.explanation);
    }

    if output.is_empty() {
        "No information available.".to_string()
    } else {
        output
    }
}

// =============================================================================
// Phase 34: Unified Response Formatting
// =============================================================================

/// Format a ReadOnly capability response.
/// Returns the formatted answer string.
pub fn format_readonly_response(result: &CapabilityExecutionResult) -> String {
    format_readonly_evidence(result)
}

/// Format a Mutating capability confirmation response.
/// Returns (explanation, needs_clarification, clarification_question).
pub fn format_mutating_confirmation(plan: &ActionPlan) -> (String, bool, Option<String>) {
    let mut output = String::new();

    // Detected section
    output.push_str("Detected:\n");
    output.push_str(&format!("  {}\n\n", plan.explanation));

    // Plan section
    output.push_str("Plan:\n");
    for (i, step) in plan.steps.iter().enumerate() {
        let sudo_marker = if step.needs_sudo { " [requires approval]" } else { "" };
        output.push_str(&format!("  Step {}: {}{}\n", i + 1, step.description, sudo_marker));
    }

    // Confirmation prompt
    output.push_str("\nAnna: Proceed? (yes/no)");

    (output, true, Some("Proceed? (yes/no)".to_string()))
}

/// Format debug output for a Mutating capability (shows plan structure).
pub fn format_mutating_debug(plan: &ActionPlan) -> String {
    let mut output = String::new();

    output.push_str("[DEBUG] ActionPlan:\n");
    output.push_str(&format!("  ID: {}\n", plan.id));
    output.push_str(&format!("  Summary: {}\n", plan.summary));
    output.push_str(&format!("  Changes needed: {}\n", plan.changes_needed));

    output.push_str("[DEBUG] Steps:\n");
    for (i, step) in plan.steps.iter().enumerate() {
        output.push_str(&format!("  {}. {} (sudo: {})\n", i + 1, step.description, step.needs_sudo));
    }

    if let Some(ref verification) = plan.verification {
        output.push_str("[DEBUG] Verification:\n");
        output.push_str(&format!("  {}\n", verification.description));
    }

    output.push_str(&format!("[DEBUG] Rollback possible: {}\n", plan.rollback.possible));

    output
}

// =============================================================================
// Phase 34: Unified Streaming Output
// =============================================================================

/// Send a capability response through the streaming writer.
/// This is the canonical path for all capability output.
pub async fn send_capability_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    outcome: &ResponseOutcome,
) -> Result<AskResult> {
    match outcome {
        ResponseOutcome::Resolved { explanation, .. } => {
            send_resolved_response(writer, explanation).await
        }
        ResponseOutcome::ConfirmationRequired { action_plan, .. } => {
            send_confirmation_response(writer, action_plan).await
        }
        ResponseOutcome::Abstained { explanation, hints, .. } => {
            send_abstained_response(writer, explanation, hints).await
        }
        ResponseOutcome::Failed { diagnostic, .. } => {
            send_failed_response(writer, diagnostic).await
        }
    }
}

/// Send a Resolved (ReadOnly success) response.
async fn send_resolved_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    explanation: &str,
) -> Result<AskResult> {
    // Apply evidence cap for non-Debug mode
    let formatted = if is_debug_mode() {
        explanation.to_string()
    } else {
        enforce_evidence_cap(explanation, MAX_EVIDENCE_LINES)
    };

    send_filtered_final_answer(writer, &formatted).await?;

    Ok(AskResult {
        answer: formatted,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(1.0),
    })
}

/// Send a ConfirmationRequired (Mutating) response.
async fn send_confirmation_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    plan: &ActionPlan,
) -> Result<AskResult> {
    let (mut content, needs_clarification, clarification_question) = format_mutating_confirmation(plan);

    // In debug mode, prepend debug info to content
    if is_debug_mode() {
        let debug_content = format_mutating_debug(plan);
        content = format!("{}\n\n{}", debug_content, content);
    }

    // Send confirmation request
    let step = DialogueStep {
        step_type: StepType::ConfirmationRequest,
        content: content.clone(),
    };
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    Ok(AskResult {
        answer: content,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification,
        clarification_question,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: None,
    })
}

/// Send an Abstained response.
async fn send_abstained_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    explanation: &str,
    hints: &[String],
) -> Result<AskResult> {
    let mut content = explanation.to_string();
    if !hints.is_empty() {
        content.push_str("\n\nTry: ");
        content.push_str(&hints.join(", "));
    }

    send_filtered_final_answer(writer, &content).await?;

    Ok(AskResult {
        answer: content,
        success: false,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: true,
        final_confidence: None,
    })
}

/// Send a Failed response.
async fn send_failed_response<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    diagnostic: &str,
) -> Result<AskResult> {
    send_filtered_final_answer(writer, diagnostic).await?;

    Ok(AskResult {
        answer: diagnostic.to_string(),
        success: false,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: None,
    })
}

// =============================================================================
// Phase 34: Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_cap_enforced() {
        let input = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let result = enforce_evidence_cap(input, 3);
        assert_eq!(result.lines().count(), 3);
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 3"));
        assert!(!result.contains("Line 4"));
    }

    #[test]
    fn test_evidence_cap_removes_blank_lines() {
        let input = "\n\nLine 1\n\nLine 2\n\nLine 3\n\n";
        let result = enforce_evidence_cap(input, 3);
        assert_eq!(result.lines().filter(|l| !l.is_empty()).count(), 3);
    }

    #[test]
    fn test_max_evidence_lines_is_three() {
        assert_eq!(MAX_EVIDENCE_LINES, 3);
    }

    #[test]
    fn test_llm_call_counter() {
        use crate::{reset_llm_call_counter, get_llm_call_count, record_llm_call};

        reset_llm_call_counter();
        assert_eq!(get_llm_call_count(), 0);

        record_llm_call();
        assert_eq!(get_llm_call_count(), 1);

        record_llm_call();
        assert_eq!(get_llm_call_count(), 2);

        reset_llm_call_counter();
        assert_eq!(get_llm_call_count(), 0);
    }

    #[test]
    fn test_mutating_confirmation_format() {
        use anna_shared::action_plan::{ActionPlan, ActionStep};

        let mut plan = ActionPlan::new("test", "Test Plan", "Testing");
        plan.add_step_full(ActionStep::new("Step one", "echo one", false));
        plan.add_step_full(ActionStep::new("Step two", "echo two", true));

        let (content, needs_clarification, question) = format_mutating_confirmation(&plan);

        assert!(content.contains("Detected:"));
        assert!(content.contains("Plan:"));
        assert!(content.contains("Step 1:"));
        assert!(content.contains("Step 2:"));
        assert!(content.contains("[requires approval]"));
        assert!(content.contains("Proceed? (yes/no)"));
        assert!(needs_clarification);
        assert_eq!(question, Some("Proceed? (yes/no)".to_string()));
    }

    #[test]
    fn test_readonly_evidence_format() {
        let mut result = CapabilityExecutionResult::empty();
        result.explanation = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string();

        let formatted = format_readonly_response(&result);

        // Should be capped at 3 lines (non-Debug mode)
        assert_eq!(formatted.lines().count(), 3);
    }
}

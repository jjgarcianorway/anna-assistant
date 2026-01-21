//! Confirmation handlers for pending actions (recipes, plans, autofixes).
//! Phase 25: Elevated confirmation, outcome recording, and safety telemetry.
//! Phase 32: Capability confirmation lifecycle.

use anna_shared::action_plan::{ActionPlan, PreflightResult, Reversibility, VerificationStatus};
use anna_shared::intent_class::IntentClass;
use anna_shared::outcome_ledger::{append_outcome, Outcome, OutcomeRecord};
use anna_shared::rpc::{DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::autofix::{execute_autofix, is_no_response, is_yes_response, AutoFix};
use crate::plan_executor::{execute_plan, is_plan_expired, set_pending_plan};
use crate::recipes;

use super::helpers::{extract_recipe_id, send_filtered_final_answer, set_pending_recipe};

// =============================================================================
// Phase 32: Output Contract Constants
// =============================================================================

/// Success message after execution.
const MSG_DONE: &str = "Anna: Done.";

/// Failure message after execution.
const MSG_FAILED: &str = "Anna: Unable to complete request. Changes were rolled back.";

/// Cancel message.
const MSG_CANCELLED: &str = "Anna: Cancelled.";

/// Expired message.
const MSG_EXPIRED: &str = "Anna: The pending action has expired. Please repeat your request.";

/// Invalid input prompt.
const MSG_INVALID_INPUT: &str = "Anna: Please type 'yes' to proceed or 'no' to cancel.";

/// Elevated invalid input prompt.
const MSG_INVALID_INPUT_ELEVATED: &str = "Anna: Please type 'yes I understand' to proceed or 'no' to cancel.";

/// Phase 25: Check if response is elevated yes ("yes I understand").
fn is_elevated_yes_response(response: &str) -> bool {
    let r = response.to_lowercase().trim().to_string();
    r == "yes i understand" || r == "yes, i understand" || r == "i understand, yes"
}

/// Handle pending recipe confirmation
pub async fn handle_pending_recipe(
    pending_recipe_id: String,
    question: &str,
    _session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    if is_yes_response(question) {
        info!("Executing recipe {} (user confirmed)", pending_recipe_id);
        let result = recipes::execute_confirmed_recipe(&pending_recipe_id);

        // Phase 15: Filter through ExposureGate
        send_filtered_final_answer(writer, &result.message).await?;

        let ask_result = anna_shared::rpc::AskResult {
            answer: result.message,
            success: result.success,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result: ask_result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    } else if is_no_response(question) {
        info!("Recipe {} cancelled by user", pending_recipe_id);
        let cancel_msg = "No problem, I won't make any changes.";
        send_filtered_final_answer(writer, cancel_msg).await?;

        let result = anna_shared::rpc::AskResult {
            answer: cancel_msg.to_string(),
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }
    // Not yes/no - fall through to normal processing
    Ok(())
}

/// Handle recipe match
pub async fn handle_recipe_match(
    recipe_result: recipes::RecipeResult,
    question: &str,
    session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    info!("Recipe matched for: {}", question);

    if recipe_result.needs_confirmation {
        let step = DialogueStep {
            step_type: StepType::ConfirmationRequest,
            content: recipe_result.message.clone(),
        };
        let response = StreamingResponse::Step { step };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
    } else {
        send_filtered_final_answer(writer, &recipe_result.message).await?;
    }

    if recipe_result.needs_confirmation {
        let recipe_id = extract_recipe_id(question);
        set_pending_recipe(session_id, &recipe_id);

        let result = anna_shared::rpc::AskResult {
            answer: recipe_result.message,
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: true,
            clarification_question: recipe_result.confirmation_prompt,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
    } else {
        let result = anna_shared::rpc::AskResult {
            answer: recipe_result.message,
            success: recipe_result.success,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
    }
    Ok(())
}

/// Handle template plan match
pub async fn handle_template_plan(
    plan: ActionPlan,
    session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    info!("Template plan matched");

    // Phase 25: Check reversibility for elevated confirmation
    let needs_elevated = plan.reversibility() == Reversibility::NonReversible;

    let confirmation_msg = if needs_elevated {
        // Elevated confirmation message for non-reversible actions
        format!(
            "{}\n\nWARNING: This action cannot be undone.\nReason: {}\n\nType 'yes I understand' to proceed, or 'no' to cancel.",
            plan.format_for_confirmation().trim_end_matches("\nProceed? (yes/no)"),
            plan.rollback.reason.as_deref().unwrap_or("Non-reversible operation")
        )
    } else {
        plan.format_for_confirmation()
    };

    let step = DialogueStep {
        step_type: StepType::ConfirmationRequest,
        content: confirmation_msg.clone(),
    };
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    set_pending_plan(session_id, plan);

    let clarification = if needs_elevated {
        "Type 'yes I understand' to proceed".to_string()
    } else {
        "Proceed? (yes/no)".to_string()
    };

    let result = anna_shared::rpc::AskResult {
        answer: confirmation_msg,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: true,
        clarification_question: Some(clarification),
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: None,
    };
    let done = StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// Handle pending autofix confirmation
pub async fn handle_pending_autofix(
    pending_fix: &'static AutoFix,
    question: &str,
    _session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    if is_yes_response(question) {
        info!("Executing autofix {} (user confirmed)", pending_fix.id);

        let fix_cmd = pending_fix.fix_cmd.to_string();

        let step = DialogueStep {
            step_type: StepType::UnderstandingCheck,
            content: format!("Running fix: {}", fix_cmd),
        };
        let response = StreamingResponse::Step { step };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;

        let result_msg = match execute_autofix(pending_fix) {
            Ok(msg) => msg,
            Err(e) => format!("Fix failed: {}", e),
        };

        let result = anna_shared::rpc::AskResult {
            answer: result_msg,
            success: true,
            iterations: 0,
            commands_executed: vec![fix_cmd],
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    } else if is_no_response(question) {
        info!("Autofix {} cancelled by user", pending_fix.id);

        let cancel_msg = "No problem, I won't make any changes.";
        send_filtered_final_answer(writer, cancel_msg).await?;

        let result = anna_shared::rpc::AskResult {
            answer: cancel_msg.to_string(),
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }
    // Not a yes/no - continue with normal processing
    Ok(())
}

/// Handle pending action plan confirmation
pub async fn handle_pending_plan(
    pending_plan: ActionPlan,
    question: &str,
    session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    let start_time = Instant::now();
    let request_id = pending_plan.id.clone();

    // Phase 25: Check if elevated confirmation is needed
    let needs_elevated = pending_plan.reversibility() == Reversibility::NonReversible;

    // Phase 25: Determine if response is accepted
    let accepted = if needs_elevated {
        is_elevated_yes_response(question)
    } else {
        is_yes_response(question)
    };

    if accepted {
        info!("Executing plan {} (user confirmed)", pending_plan.id);

        let exec_result = execute_plan(&pending_plan);

        // Phase 32: Use contract output messages
        let result_msg = if exec_result.success {
            MSG_DONE.to_string()
        } else {
            MSG_FAILED.to_string()
        };

        // Phase 25: Record outcome with extended telemetry
        let outcome = if exec_result.success {
            Outcome::Resolved
        } else {
            Outcome::Failed
        };
        let outcome_record = OutcomeRecord::new_action(
            &request_id,
            IntentClass::Mutating,
            outcome,
            false,
            start_time.elapsed().as_millis() as u64,
            pending_plan.preflight_result,
            exec_result.verification_status,
            needs_elevated,
        );
        let _ = append_outcome(&outcome_record);

        send_filtered_final_answer(writer, &result_msg).await?;

        let result = anna_shared::rpc::AskResult {
            answer: result_msg,
            success: exec_result.success,
            iterations: 0,
            commands_executed: vec![], // Phase 32: No commands in output
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    } else if is_no_response(question) {
        info!("Plan {} cancelled by user", pending_plan.id);

        // Phase 25: Record cancellation
        let outcome_record = OutcomeRecord::new_action(
            &request_id,
            IntentClass::Mutating,
            Outcome::Cancelled,
            false,
            start_time.elapsed().as_millis() as u64,
            pending_plan.preflight_result,
            VerificationStatus::Unknown, // Never executed
            needs_elevated,
        );
        let _ = append_outcome(&outcome_record);

        // Phase 32: Use contract output message
        send_filtered_final_answer(writer, MSG_CANCELLED).await?;

        let result = anna_shared::rpc::AskResult {
            answer: MSG_CANCELLED.to_string(),
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: None,
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // Invalid input - prompt again
    info!("Invalid response for plan {}: '{}'", pending_plan.id, question);

    // Phase 32: Use contract output messages
    let prompt_msg = if needs_elevated {
        MSG_INVALID_INPUT_ELEVATED
    } else {
        MSG_INVALID_INPUT
    };
    send_filtered_final_answer(writer, prompt_msg).await?;

    // Re-store the plan for retry
    set_pending_plan(session_id, pending_plan);

    let clarification = if needs_elevated {
        "Type 'yes I understand' to proceed".to_string()
    } else {
        "Proceed? (yes/no)".to_string()
    };

    let result = anna_shared::rpc::AskResult {
        answer: prompt_msg.to_string(),
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: true,
        clarification_question: Some(clarification),
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: None,
    };
    let done = StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// Handle expired plan - inform user and clear state
pub async fn handle_expired_plan(
    session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    info!("Plan expired for session {}", session_id);

    // Phase 25: Record expired outcome
    let outcome_record = OutcomeRecord::new_action(
        &uuid::Uuid::new_v4().to_string(),
        IntentClass::Mutating,
        Outcome::Expired,
        false,
        0, // Duration unknown for expired plans
        PreflightResult::Unknown, // Plan may have had preflight but we lost the reference
        VerificationStatus::Unknown,
        false,
    );
    let _ = append_outcome(&outcome_record);

    // Phase 32: Use contract output message
    send_filtered_final_answer(writer, MSG_EXPIRED).await?;

    let result = anna_shared::rpc::AskResult {
        answer: MSG_EXPIRED.to_string(),
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: None,
    };
    let done = StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// Phase 32: Handle capability confirmation flow.
/// Formats the ActionPlan for user confirmation and stores it as pending.
pub async fn handle_capability_confirmation(
    plan: ActionPlan,
    session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    info!("Phase 32: Capability confirmation for plan: {}", plan.id);

    // Phase 32: Format confirmation message following output contract
    // - Detected block (evidence from probes)
    // - Plan preview (step descriptions only, no commands)
    // - "Anna: Proceed? (yes/no)"
    let confirmation_msg = format_capability_confirmation(&plan);

    // Send as confirmation request step
    let step = DialogueStep {
        step_type: StepType::ConfirmationRequest,
        content: confirmation_msg.clone(),
    };
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    // Store pending plan for yes/no handling
    set_pending_plan(session_id, plan);

    // Return AskResult with needs_clarification
    let result = anna_shared::rpc::AskResult {
        answer: confirmation_msg,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: true,
        clarification_question: Some("Proceed? (yes/no)".to_string()),
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: None,
    };
    let done = StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// Format capability confirmation message per Phase 32 contract.
/// Output format:
/// ```text
/// Detected:
///   Display Manager: GDM
///   Session: wayland
///   ...
///
/// Plan:
///   Step 1: <description> [requires approval]
///   Step 2: <description> [requires approval]
///   ...
///
/// Anna: Proceed? (yes/no)
/// ```
fn format_capability_confirmation(plan: &ActionPlan) -> String {
    let mut output = String::new();

    // Detected block - use explanation as evidence summary
    if !plan.explanation.is_empty() {
        output.push_str("Detected:\n");
        // Split explanation into lines for formatting
        for line in plan.explanation.lines().take(6) {
            output.push_str(&format!("  {}\n", line));
        }
        output.push('\n');
    }

    // Plan preview - step descriptions only, no commands
    output.push_str("Plan:\n");
    for (i, step) in plan.steps.iter().enumerate() {
        let approval_marker = if step.needs_sudo {
            " [requires approval]"
        } else {
            ""
        };
        output.push_str(&format!("  Step {}: {}{}\n", i + 1, step.description, approval_marker));
    }
    output.push('\n');

    // Final prompt
    output.push_str("Anna: Proceed? (yes/no)");

    output
}

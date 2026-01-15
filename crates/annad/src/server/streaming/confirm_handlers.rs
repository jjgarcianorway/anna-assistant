//! Confirmation handlers for pending actions (recipes, plans, autofixes).

use anna_shared::action_plan::ActionPlan;
use anna_shared::rpc::{DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::autofix::{execute_autofix, is_no_response, is_yes_response, AutoFix};
use crate::plan_executor::{execute_plan, format_execution_result, is_plan_expired, set_pending_plan};
use crate::recipes;

use super::helpers::{extract_recipe_id, send_filtered_final_answer, set_pending_recipe};

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

    let confirmation_msg = plan.format_for_confirmation();
    let step = DialogueStep {
        step_type: StepType::ConfirmationRequest,
        content: confirmation_msg.clone(),
    };
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    set_pending_plan(session_id, plan);

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
    if is_yes_response(question) {
        info!("Executing plan {} (user confirmed)", pending_plan.id);

        let step = DialogueStep {
            step_type: StepType::UnderstandingCheck,
            content: format!("Executing: {}", pending_plan.summary),
        };
        let response = StreamingResponse::Step { step };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;

        let exec_result = execute_plan(&pending_plan);
        let result_msg = format_execution_result(&exec_result, &pending_plan);

        send_filtered_final_answer(writer, &result_msg).await?;

        let result = anna_shared::rpc::AskResult {
            answer: result_msg,
            success: exec_result.success,
            iterations: 0,
            commands_executed: pending_plan.steps.iter().map(|s| s.command.clone()).collect(),
            dialogue: vec![],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    } else if is_no_response(question) {
        info!("Plan {} cancelled by user", pending_plan.id);

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
        };
        let done = StreamingResponse::Done { result };
        let json = serde_json::to_string(&done)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // Invalid input - prompt again
    info!("Invalid response for plan {}: '{}'", pending_plan.id, question);
    let prompt_msg = "Please type 'yes' to proceed or 'no' to cancel.";
    send_filtered_final_answer(writer, prompt_msg).await?;

    // Re-store the plan for retry
    set_pending_plan(session_id, pending_plan);

    let result = anna_shared::rpc::AskResult {
        answer: prompt_msg.to_string(),
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: true,
        clarification_question: Some("Proceed? (yes/no)".to_string()),
        cached: false,
        citations: vec![],
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

    let expire_msg = "The pending action has expired. Please repeat your request.";
    send_filtered_final_answer(writer, expire_msg).await?;

    let result = anna_shared::rpc::AskResult {
        answer: expire_msg.to_string(),
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    };
    let done = StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

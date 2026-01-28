//! Confirmation handlers for pending actions (plans, autofixes).

use anna_shared::action_plan::{ActionPlan, PreflightResult, Reversibility, VerificationStatus};
use anna_shared::intent_class::IntentClass;
use anna_shared::outcome_ledger::{append_outcome, Outcome, OutcomeRecord};
use anna_shared::rpc::{DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::autofix::{execute_autofix, is_no_response, is_yes_response, AutoFix};
use crate::plan_executor::{execute_plan, set_pending_plan};

use super::helpers::send_filtered_final_answer;

const MSG_DONE: &str = "Anna: Done.";
const MSG_FAILED: &str = "Anna: Unable to complete request. Changes were rolled back.";
const MSG_CANCELLED: &str = "Anna: Cancelled.";
const MSG_EXPIRED: &str = "Anna: The pending action has expired. Please repeat your request.";
const MSG_INVALID_INPUT: &str = "Anna: Please type 'yes' to proceed or 'no' to cancel.";
const MSG_INVALID_INPUT_ELEVATED: &str = "Anna: Please type 'yes I understand' to proceed or 'no' to cancel.";

fn is_elevated_yes_response(response: &str) -> bool {
    let r = response.to_lowercase().trim().to_string();
    r == "yes i understand" || r == "yes, i understand" || r == "i understand, yes"
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
    let needs_elevated = pending_plan.reversibility() == Reversibility::NonReversible;

    let accepted = if needs_elevated {
        is_elevated_yes_response(question)
    } else {
        is_yes_response(question)
    };

    if accepted {
        info!("Executing plan {} (user confirmed)", pending_plan.id);
        let exec_result = execute_plan(&pending_plan);

        let result_msg = if exec_result.success {
            MSG_DONE.to_string()
        } else {
            MSG_FAILED.to_string()
        };

        let outcome = if exec_result.success { Outcome::Resolved } else { Outcome::Failed };
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
    } else if is_no_response(question) {
        info!("Plan {} cancelled by user", pending_plan.id);

        let outcome_record = OutcomeRecord::new_action(
            &request_id,
            IntentClass::Mutating,
            Outcome::Cancelled,
            false,
            start_time.elapsed().as_millis() as u64,
            pending_plan.preflight_result,
            VerificationStatus::Unknown,
            needs_elevated,
        );
        let _ = append_outcome(&outcome_record);

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
    let prompt_msg = if needs_elevated { MSG_INVALID_INPUT_ELEVATED } else { MSG_INVALID_INPUT };
    send_filtered_final_answer(writer, prompt_msg).await?;

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

/// Handle expired plan
pub async fn handle_expired_plan(
    session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    info!("Plan expired for session {}", session_id);

    let outcome_record = OutcomeRecord::new_action(
        &uuid::Uuid::new_v4().to_string(),
        IntentClass::Mutating,
        Outcome::Expired,
        false,
        0,
        PreflightResult::Unknown,
        VerificationStatus::Unknown,
        false,
    );
    let _ = append_outcome(&outcome_record);

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

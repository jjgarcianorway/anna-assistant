//! Streaming Ralph loop with real-time progress updates.
//! LLM-first: no bypass paths. Every question goes through the LLM.

use anna_shared::exposure::ExposureGate;
use anna_shared::policy::get_policy;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::debug;

use super::early_handlers::{
    handle_morning_briefing_request, handle_multi_agent_query,
    handle_natural_system_query, handle_pattern_match, handle_reminder_request,
};
use super::run_loop::run_full_loop_streaming;

/// Streaming version of the Ralph loop with real-time progress updates.
/// LLM-first: all questions go through the full investigation loop.
pub async fn ralph_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    session_id: &str,
    writer: &mut W,
) -> Result<AskResult> {
    let gate = ExposureGate::from_config();

    if gate.diagnostic_visible() {
        let policy = get_policy();
        let basis = policy.format_debug_basis();
        debug!("{}", basis);
        let step = DialogueStep {
            step_type: StepType::PolicyBasis,
            content: basis,
        };
        let _ = super::streaming_helpers::send_step(writer, step, &gate).await;
    }

    // Check for reminder requests first
    if let Some(result) = handle_reminder_request(question, writer, &gate).await? {
        return Ok(result);
    }

    // Check for morning briefing setup
    if let Some(result) = handle_morning_briefing_request(question, writer, &gate).await? {
        return Ok(result);
    }

    // v0.3.120: Handle natural language system queries (brief summary, not full report)
    if let Some(result) = handle_natural_system_query(question, writer, &gate).await? {
        return Ok(result);
    }

    // v0.3.123: Handle well-known error patterns with instant answers
    if let Some(result) = handle_pattern_match(question, writer, &gate).await? {
        return Ok(result);
    }

    // v0.3.156: DISABLED - hardcoded parsing against LLM-first philosophy
    // Package queries now go through Ralph loop for LLM investigation of pacman.log
    // if let Some(result) = handle_package_history_query(question, writer, &gate).await? {
    //     return Ok(result);
    // }

    // v0.3.121: Check for multi-domain questions that benefit from parallel investigation
    if let Some(result) = handle_multi_agent_query(question, writer, &gate).await? {
        return Ok(result);
    }

    // All other questions go through the full loop
    run_full_loop_streaming(model, question, session_id, writer, &gate).await
}

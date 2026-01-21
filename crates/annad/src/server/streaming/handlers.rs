//! Streaming request handlers.
//! v0.0.998: Added Hollywood IT teams experience
//! v0.3.49: Phase 16 - Action plan execution
//! v0.3.75: Phase 32 - Capability routing for mutating capabilities

use anna_shared::capability::{
    execute_display_scale_gdm, execute_power_inhibit_sleep, execute_thermal_status,
    execute_audio_stack_detect, format_outcome_to_string, format_response,
    route_request, CapabilityRoutingResult, ResponseOutcome,
    CapabilityMode, InhibitAction, InhibitTarget, CAPABILITY_REGISTRY,
};
use anna_shared::rpc::{DialogueStep, RpcRequest, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::autofix::{get_fix_history_summary, take_pending_autofix};
use crate::plan_executor::{has_pending_plan, is_plan_expired, take_pending_plan};
use crate::plan_generator;
use crate::recipes;
use crate::state::SharedState;

use crate::server::alerts::get_pending_alerts;
use super::confirm_handlers::{
    handle_expired_plan, handle_pending_autofix, handle_pending_plan, handle_pending_recipe,
    handle_recipe_match, handle_template_plan, handle_capability_confirmation,
};
use super::helpers::{is_fix_history_question, send_filtered_final_answer, take_pending_recipe};

/// Handle a streaming AskStreaming request
pub async fn handle_streaming_request(
    request: RpcRequest,
    state: SharedState,
    mut writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    let question = request
        .params
        .as_ref()
        .and_then(|p| p.get("question"))
        .and_then(|q| q.as_str())
        .unwrap_or("");

    // Extract session_id from params (client generates it)
    let session_id = request
        .params
        .as_ref()
        .and_then(|p| p.get("session_id"))
        .and_then(|s| s.as_str())
        .unwrap_or("default");

    // v0.2.8: Track response time for RPG stats
    let start_time = std::time::Instant::now();

    if question.is_empty() {
        let response = StreamingResponse::Error {
            message: "Missing 'question' parameter".to_string(),
        };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(());
    }

    // Phase 16: Check if this is a response to a pending action plan
    // First check if a plan exists but has expired
    if has_pending_plan(session_id) && is_plan_expired(session_id) {
        // Take and discard the expired plan, then notify user
        let _ = take_pending_plan(session_id);
        return handle_expired_plan(session_id, &mut writer).await;
    }
    if let Some(pending_plan) = take_pending_plan(session_id) {
        return handle_pending_plan(pending_plan, question, session_id, &mut writer).await;
    }

    // v0.0.994: Check if this is a response to a pending autofix
    if let Some(pending_fix) = take_pending_autofix(session_id) {
        return handle_pending_autofix(pending_fix, question, session_id, &mut writer).await;
    }

    // v0.0.997: Check if user is asking about fix history
    if is_fix_history_question(question) {
        return handle_fix_history_question(&mut writer).await;
    }

    // v0.0.998: Check if this is a response to a pending recipe
    if let Some(pending_recipe_id) = take_pending_recipe(session_id) {
        return handle_pending_recipe(pending_recipe_id, question, session_id, &mut writer).await;
    }

    // v0.0.998: Check if this matches a configuration recipe
    if let Some(recipe_result) = recipes::try_recipe(question) {
        return handle_recipe_match(recipe_result, question, session_id, &mut writer).await;
    }

    // Phase 32: Route through capability system for mutating capabilities
    if let Some(result) = try_capability_routing(question, session_id, &mut writer).await? {
        return result;
    }

    // Phase 16: Check if this matches an action plan template
    if let Some(plan) = plan_generator::generate_template_plan(question) {
        // NOOP short-circuit: if preflight determined no changes needed,
        // emit terminal response without entering confirmation flow.
        // This prevents "Proceed?" prompts when action set is empty.
        if !plan.changes_needed {
            let msg = format!(
                "No changes needed. {}",
                plan.skip_reason.as_deref().unwrap_or("Already configured.")
            );
            send_filtered_final_answer(&mut writer, &msg).await?;

            let result = anna_shared::rpc::AskResult {
                answer: msg,
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
        return handle_template_plan(plan, session_id, &mut writer).await;
    }

    // Check for pending critical system alerts and notify user
    if let Some(alerts) = get_pending_alerts() {
        for alert in alerts {
            let step = DialogueStep {
                step_type: StepType::SystemAlert,
                content: alert,
            };
            let response = StreamingResponse::Step { step };
            let json = serde_json::to_string(&response)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
        }
    }

    // Main question processing
    super::main_handler::handle_main_question(question, session_id, state, start_time, &mut writer).await
}

/// Handle fix history question
async fn handle_fix_history_question(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    info!("User asking about fix history");
    let summary = get_fix_history_summary();

    // Phase 15: Filter through ExposureGate
    send_filtered_final_answer(writer, &summary).await?;

    let result = anna_shared::rpc::AskResult {
        answer: summary,
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

/// Phase 32/33: Try capability routing for all capabilities.
/// Returns Some(Ok(())) if capability was handled, None if should fall through.
async fn try_capability_routing(
    question: &str,
    session_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<Option<Result<()>>> {
    let routing = route_request(question);

    // Only handle Supported routing
    let capability_id = match &routing {
        CapabilityRoutingResult::Supported { capability_id } => capability_id,
        CapabilityRoutingResult::Unsupported { .. } => return Ok(None),
    };

    // Look up capability to check mode
    let capability = match CAPABILITY_REGISTRY.get(capability_id) {
        Some(cap) => cap,
        None => return Ok(None), // Unknown capability, fall through
    };

    // Phase 33: Handle ReadOnly capabilities directly (no confirmation needed)
    if capability.mode == CapabilityMode::ReadOnly {
        info!("Phase 33: Routing ReadOnly capability: {}", capability_id);
        return Ok(Some(handle_readonly_capability(capability_id.as_str(), writer).await));
    }

    info!("Phase 32: Routing mutating capability: {}", capability_id);

    // Execute the capability handler
    let execution_result = dispatch_capability_handler(capability_id.as_str());
    let outcome = format_response(&routing, Some(execution_result));

    match outcome {
        ResponseOutcome::ConfirmationRequired { action_plan, .. } => {
            // Wire to existing confirmation flow
            return Ok(Some(handle_capability_confirmation(action_plan, session_id, writer).await));
        }
        ResponseOutcome::Resolved { explanation, .. } => {
            // No changes needed - send directly
            send_filtered_final_answer(writer, &explanation).await?;
            let result = anna_shared::rpc::AskResult {
                answer: explanation,
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
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(Some(Ok(())));
        }
        ResponseOutcome::Abstained { .. } => {
            let answer = format_outcome_to_string(&outcome);
            send_filtered_final_answer(writer, &answer).await?;
            let result = anna_shared::rpc::AskResult {
                answer,
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
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(Some(Ok(())));
        }
        ResponseOutcome::Failed { diagnostic, .. } => {
            send_filtered_final_answer(writer, &diagnostic).await?;
            let result = anna_shared::rpc::AskResult {
                answer: diagnostic,
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
            };
            let done = StreamingResponse::Done { result };
            let json = serde_json::to_string(&done)?;
            writer.write_all(format!("{}\n", json).as_bytes()).await?;
            return Ok(Some(Ok(())));
        }
    }
}

/// Phase 33: Maximum evidence lines for ReadOnly capabilities in non-Debug mode.
const MAX_READONLY_EVIDENCE_LINES: usize = 3;

/// Phase 33.2: Enforce evidence cap on ReadOnly capability output.
/// Limits output to max_lines non-empty lines.
fn enforce_evidence_cap(explanation: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = explanation
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(max_lines)
        .collect();
    lines.join("\n")
}

/// Phase 33: Handle ReadOnly capabilities directly (no confirmation needed).
async fn handle_readonly_capability(
    capability_id: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    let execution_result = dispatch_capability_handler(capability_id);

    // Format the explanation with evidence cap enforcement
    let explanation = if execution_result.explanation.is_empty() {
        "No information available.".to_string()
    } else {
        // Phase 33.2: Enforce evidence cap - max 3 lines in non-Debug mode
        enforce_evidence_cap(&execution_result.explanation, MAX_READONLY_EVIDENCE_LINES)
    };
    let explanation = explanation.as_str();

    // Send the answer
    send_filtered_final_answer(writer, explanation).await?;

    let result = anna_shared::rpc::AskResult {
        answer: explanation.to_string(),
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
    };
    let done = StreamingResponse::Done { result };
    let json = serde_json::to_string(&done)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}

/// Dispatch to capability handler by ID.
fn dispatch_capability_handler(capability_id: &str) -> anna_shared::capability::CapabilityExecutionResult {
    match capability_id {
        "display.scale.gdm" => execute_display_scale_gdm(),
        // Phase 33: Power inhibit - defaults to disabling lid sleep
        // TODO: Parse target/action from question for more specific handling
        "power.inhibit.sleep" => execute_power_inhibit_sleep(InhibitTarget::LidClose, InhibitAction::Ignore),
        // ReadOnly capabilities (shouldn't reach here via mutating flow)
        "system.thermal.status" => execute_thermal_status(),
        "audio.stack.detect" => execute_audio_stack_detect(),
        _ => anna_shared::capability::CapabilityExecutionResult::abstain(
            anna_shared::capability::AbstainReason::NoMatchingCapability,
            &format!("No handler for: {}", capability_id),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // PHASE 34: Hard proof that LLM is not called on capability routing
    // =========================================================================

    /// Phase 34: HARD PROOF that capability routing does NOT call the LLM.
    /// This test will FAIL if any LLM entrypoint is invoked during capability dispatch.
    /// The counter is incremented by ollama::chat_single_attempt and chat_streaming_validated.
    #[test]
    fn test_phase34_capability_routing_zero_llm_calls() {
        use crate::{reset_llm_call_counter, get_llm_call_count};
        use anna_shared::capability::{
            execute_thermal_status, execute_audio_stack_detect,
            execute_display_scale_gdm, execute_power_inhibit_sleep,
            InhibitTarget, InhibitAction, route_request,
        };

        // Reset counter before test
        reset_llm_call_counter();
        assert_eq!(get_llm_call_count(), 0, "Counter should start at 0");

        // Test 1: Route thermal status (ReadOnly)
        let routing = route_request("what's my cpu temperature");
        assert!(routing.is_supported(), "Should route to thermal_status");
        let _ = execute_thermal_status();
        assert_eq!(
            get_llm_call_count(), 0,
            "CRITICAL: execute_thermal_status called LLM (count={})",
            get_llm_call_count()
        );

        // Test 2: Route audio stack (ReadOnly)
        let routing = route_request("am I using pipewire");
        assert!(routing.is_supported(), "Should route to audio_stack");
        let _ = execute_audio_stack_detect();
        assert_eq!(
            get_llm_call_count(), 0,
            "CRITICAL: execute_audio_stack_detect called LLM (count={})",
            get_llm_call_count()
        );

        // Test 3: Route display scale (Mutating)
        let routing = route_request("scale gdm login");
        assert!(routing.is_supported(), "Should route to display_scale");
        let _ = execute_display_scale_gdm();
        assert_eq!(
            get_llm_call_count(), 0,
            "CRITICAL: execute_display_scale_gdm called LLM (count={})",
            get_llm_call_count()
        );

        // Test 4: Route power inhibit (Mutating)
        let routing = route_request("stop sleep when closing lid");
        assert!(routing.is_supported(), "Should route to power_inhibit");
        let _ = execute_power_inhibit_sleep(InhibitTarget::LidClose, InhibitAction::Ignore);
        assert_eq!(
            get_llm_call_count(), 0,
            "CRITICAL: execute_power_inhibit_sleep called LLM (count={})",
            get_llm_call_count()
        );

        // Test 5: Full dispatch path
        let _ = dispatch_capability_handler("system.thermal.status");
        let _ = dispatch_capability_handler("audio.stack.detect");
        let _ = dispatch_capability_handler("display.scale.gdm");
        let _ = dispatch_capability_handler("power.inhibit.sleep");

        // Final assertion: counter must still be 0
        assert_eq!(
            get_llm_call_count(), 0,
            "CRITICAL: LLM was called during capability routing. Count={}\n\
             This proves the capability path is NOT bypassing the LLM.",
            get_llm_call_count()
        );
    }

    /// Phase 34: Verify counter actually works by testing increment.
    #[test]
    fn test_phase34_llm_counter_infrastructure() {
        use crate::{reset_llm_call_counter, get_llm_call_count, record_llm_call};

        reset_llm_call_counter();
        assert_eq!(get_llm_call_count(), 0);

        // Simulate what the LLM entrypoint would do
        record_llm_call();
        assert_eq!(get_llm_call_count(), 1, "Counter should increment");

        record_llm_call();
        assert_eq!(get_llm_call_count(), 2, "Counter should increment again");

        reset_llm_call_counter();
        assert_eq!(get_llm_call_count(), 0, "Counter should reset");
    }

    // =========================================================================
    // PHASE 33.2: Evidence cap enforcement tests
    // =========================================================================

    /// Phase 33.2: Prove evidence cap limits output to MAX_READONLY_EVIDENCE_LINES.
    #[test]
    fn test_phase33_evidence_cap_limits_output() {
        // Test with more lines than allowed
        let long_output = "Line 1: Temperature data\n\
                          Line 2: More temperature data\n\
                          Line 3: CPU info\n\
                          Line 4: Fan status\n\
                          Line 5: Should be truncated";

        let capped = enforce_evidence_cap(long_output, MAX_READONLY_EVIDENCE_LINES);

        // Count non-empty lines
        let line_count = capped.lines().filter(|l| !l.trim().is_empty()).count();

        assert_eq!(
            line_count, MAX_READONLY_EVIDENCE_LINES,
            "Evidence cap must limit to {} lines. Got {} lines in: {}",
            MAX_READONLY_EVIDENCE_LINES, line_count, capped
        );

        // Verify it contains the first 3 lines, not the last 2
        assert!(capped.contains("Line 1"), "Should keep first line");
        assert!(capped.contains("Line 2"), "Should keep second line");
        assert!(capped.contains("Line 3"), "Should keep third line");
        assert!(!capped.contains("Line 4"), "Should NOT include line 4");
        assert!(!capped.contains("Line 5"), "Should NOT include line 5");
    }

    /// Phase 33.2: Evidence cap filters empty lines before counting.
    #[test]
    fn test_phase33_evidence_cap_filters_empty_lines() {
        // Input with empty lines interspersed
        let input_with_empty = "\n\nLine 1\n\n\nLine 2\n\nLine 3\n\nLine 4\n";

        let capped = enforce_evidence_cap(input_with_empty, MAX_READONLY_EVIDENCE_LINES);

        // Should have exactly 3 non-empty lines
        let line_count = capped.lines().filter(|l| !l.trim().is_empty()).count();
        assert_eq!(line_count, 3);

        // Line 4 should be excluded
        assert!(!capped.contains("Line 4"));
    }

    /// Phase 33.2: Evidence cap constant is 3 per contract.
    #[test]
    fn test_phase33_evidence_cap_constant() {
        assert_eq!(
            MAX_READONLY_EVIDENCE_LINES, 3,
            "Evidence cap must be 3 lines per Phase 33.2 contract"
        );
    }

    /// Phase 33.2: Evidence cap handles short input gracefully.
    #[test]
    fn test_phase33_evidence_cap_short_input() {
        let short_input = "Only one line";
        let capped = enforce_evidence_cap(short_input, MAX_READONLY_EVIDENCE_LINES);

        assert_eq!(capped, "Only one line");
    }
}

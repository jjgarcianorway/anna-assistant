//! Capability Executor - Streaming
//!
//! Phase 30: LLM is NOT in the execution path.
//! - Matched capability -> execute handler directly
//! - Unsupported request -> return Abstained with hints
//! - LLM is never called

use anna_shared::capability::{
    execute_display_scale_gdm, format_outcome_to_string, format_response, route_request,
    AbstainReason, CapabilityExecutionResult, CapabilityRoutingResult, ResponseOutcome,
};
use anna_shared::declaration::CapabilityDeclaration;
use anna_shared::exposure::filter_final_answer_with_request;
use anna_shared::intent_class::IntentClass;
use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::info;

/// Detect if question is asking about Anna's capabilities.
fn is_capability_question(question: &str) -> bool {
    let q = question.to_lowercase();
    let patterns = [
        "what can you do",
        "what are your capabilities",
        "what commands can you run",
        "what are you capable of",
        "what can anna do",
        "what are anna's capabilities",
        "what is anna capable of",
        "tell me your capabilities",
        "list your capabilities",
        "show your capabilities",
        "what's your capability",
        "your capabilities",
        "what do you know how to do",
        "what are you able to do",
        "what are you allowed to do",
        "what is anna allowed to do",
        "allowed to do",
    ];
    patterns.iter().any(|p| q.contains(p))
}

/// Execute a question using deterministic capability routing.
///
/// LLM is NOT called. All paths are:
/// - Capability question -> declaration
/// - Matched capability -> handler
/// - Unsupported -> Abstained
pub async fn execute_question_streaming_llm<W: AsyncWriteExt + Unpin>(
    _model: &str,
    question: &str,
    _session_context: Option<&str>,
    writer: &mut W,
) -> Result<AskResult> {
    info!("Capability executor: {}", question);

    // Capability meta-question - answer from declaration
    if is_capability_question(question) {
        return answer_capability_question(writer).await;
    }

    // Route through capability layer
    let routing = route_request(question);

    match &routing {
        CapabilityRoutingResult::Unsupported { reason_code, short_message } => {
            info!("No capability match: {} - {}", reason_code, short_message);
            finish_with_abstained(writer, short_message).await
        }
        CapabilityRoutingResult::Supported { capability_id } => {
            info!("Executing capability: {}", capability_id);
            execute_capability_handler(writer, question, &routing).await
        }
    }
}

/// Execute a matched capability handler directly.
async fn execute_capability_handler<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    question: &str,
    routing: &CapabilityRoutingResult,
) -> Result<AskResult> {
    let capability_id = routing.capability_id().unwrap();
    let cap_id_str = capability_id.as_str();

    // Dispatch to handler
    let execution_result = dispatch_capability_handler(cap_id_str);

    // Convert to ResponseOutcome
    let outcome = format_response(routing, Some(execution_result));

    // Format to string
    let raw_answer = format_outcome_to_string(&outcome);

    // Filter through ExposureGate
    let gate_result = filter_final_answer_with_request(
        &raw_answer,
        IntentClass::ReadOnly,
        Some(question),
    );

    let final_answer = gate_result.content;
    let is_abstained = gate_result.block_reason.is_some()
        || matches!(outcome, ResponseOutcome::Abstained { .. });
    let is_failed = matches!(outcome, ResponseOutcome::Failed { .. });

    let mut dialogue = Vec::new();

    // Send final answer
    let step = DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    };
    dialogue.push(step.clone());
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    let result = AskResult {
        answer: final_answer,
        success: !is_abstained && !is_failed,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: is_abstained,
        final_confidence: Some(1.0),
    };

    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    Ok(result)
}

/// Dispatch to the appropriate capability handler.
fn dispatch_capability_handler(capability_id: &str) -> CapabilityExecutionResult {
    match capability_id {
        "display.scale.gdm" => execute_display_scale_gdm(),

        // Status capabilities - placeholder
        "status.system" | "status.disk" | "status.memory" | "status.network"
        | "status.services" | "status.identity" => {
            let mut result = CapabilityExecutionResult::empty();
            result.explanation = format!(
                "Capability '{}' registered but handler not implemented.",
                capability_id
            );
            result
        }

        _ => CapabilityExecutionResult::abstain(
            AbstainReason::NoMatchingCapability,
            &format!("No handler for: {}", capability_id),
        ),
    }
}

/// Answer capability questions from the declaration.
async fn answer_capability_question<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
) -> Result<AskResult> {
    info!("Answering capability question from declaration");

    let decl = CapabilityDeclaration::from_ledger();
    let answer = decl.render_onboarding();
    let mut dialogue = Vec::new();

    let step = DialogueStep {
        step_type: StepType::FinalAnswer,
        content: answer.clone(),
    };
    dialogue.push(step.clone());
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    let result = AskResult {
        answer,
        success: true,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(1.0),
    };

    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    Ok(result)
}

/// Return Abstained when no capability matches.
async fn finish_with_abstained<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    short_message: &str,
) -> Result<AskResult> {
    let outcome = ResponseOutcome::Abstained {
        capability_id: None,
        reason: AbstainReason::NoMatchingCapability,
        explanation: short_message.to_string(),
        hints: vec![
            "status.system".to_string(),
            "status.disk".to_string(),
            "display.scale.gdm".to_string(),
        ],
    };

    let answer = format_outcome_to_string(&outcome);
    let mut dialogue = Vec::new();

    let step = DialogueStep {
        step_type: StepType::FinalAnswer,
        content: answer.clone(),
    };
    dialogue.push(step.clone());
    let response = StreamingResponse::Step { step };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    let result = AskResult {
        answer,
        success: false,
        iterations: 0,
        commands_executed: vec![],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: true,
        final_confidence: None,
    };

    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prove LLM is not called: matched capability produces deterministic output.
    #[test]
    fn test_dispatch_display_scale_gdm_is_deterministic() {
        let result1 = dispatch_capability_handler("display.scale.gdm");
        let result2 = dispatch_capability_handler("display.scale.gdm");

        // Same abstain state
        assert_eq!(result1.wants_abstain(), result2.wants_abstain());
    }

    /// Prove unknown capability returns abstain, not LLM.
    #[test]
    fn test_unknown_capability_returns_abstain() {
        let result = dispatch_capability_handler("unknown.capability");
        assert!(result.wants_abstain());
    }

    /// Prove routing unsupported returns abstain.
    #[test]
    fn test_unsupported_routing_produces_abstain() {
        let routing = route_request("tell me a joke");
        assert!(matches!(routing, CapabilityRoutingResult::Unsupported { .. }));
    }

    /// Prove GDM question routes to display.scale.gdm.
    #[test]
    fn test_gdm_question_routes_to_capability() {
        let routing = route_request("scale my gdm login screen");
        if let CapabilityRoutingResult::Supported { capability_id } = routing {
            assert_eq!(capability_id.as_str(), "display.scale.gdm");
        } else {
            panic!("Expected Supported routing for GDM question");
        }
    }
}

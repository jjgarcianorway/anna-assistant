//! Capability Executor - Non-streaming
//!
//! Phase 30: LLM is NOT in the execution path.
//! - Matched capability -> execute handler directly
//! - Unsupported request -> return Abstained with hints
//! - LLM is never called

pub mod commands;
pub mod evidence;
pub mod investigate;
pub mod prompts;
pub mod streaming;
pub mod types;

pub use prompts::compiler_prompt;
pub use streaming::execute_question_streaming_llm;
pub use types::{Finding, InvestigationState, NextStep, Understanding, VerificationResult};

use anna_shared::capability::{
    execute_display_scale_gdm, format_outcome_to_string, format_response, route_request,
    AbstainReason, CapabilityExecutionResult, CapabilityRoutingResult, ResponseOutcome,
};
use anna_shared::declaration::CapabilityDeclaration;
use anna_shared::exposure::filter_final_answer_with_request;
use anna_shared::intent_class::IntentClass;
use anna_shared::rpc::AskResult;
use anyhow::Result;
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
pub async fn execute_question_llm(_model: &str, question: &str) -> Result<AskResult> {
    info!("Capability executor: {}", question);

    // Capability meta-question - answer from declaration
    if is_capability_question(question) {
        info!("Answering capability question from declaration");
        let decl = CapabilityDeclaration::from_ledger();
        let answer = decl.render_onboarding();
        return Ok(AskResult {
            answer,
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
        });
    }

    // Route through capability layer
    let routing = route_request(question);

    match &routing {
        CapabilityRoutingResult::Unsupported { reason_code, short_message } => {
            info!("No capability match: {} - {}", reason_code, short_message);
            Ok(build_abstained_result(short_message))
        }
        CapabilityRoutingResult::Supported { capability_id } => {
            info!("Executing capability: {}", capability_id);
            Ok(execute_capability_handler(question, &routing))
        }
    }
}

/// Execute a matched capability handler directly.
fn execute_capability_handler(question: &str, routing: &CapabilityRoutingResult) -> AskResult {
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

    AskResult {
        answer: final_answer,
        success: !is_abstained && !is_failed,
        iterations: 0,
        commands_executed: vec![],
        dialogue: vec![],
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: is_abstained,
        final_confidence: Some(1.0),
    }
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

/// Build Abstained result when no capability matches.
fn build_abstained_result(short_message: &str) -> AskResult {
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

    AskResult {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_is_deterministic() {
        let r1 = dispatch_capability_handler("display.scale.gdm");
        let r2 = dispatch_capability_handler("display.scale.gdm");
        assert_eq!(r1.wants_abstain(), r2.wants_abstain());
    }

    #[test]
    fn test_unknown_returns_abstain() {
        let result = dispatch_capability_handler("unknown");
        assert!(result.wants_abstain());
    }
}

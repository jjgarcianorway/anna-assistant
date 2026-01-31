//! Helper functions for streaming responses.
//!
//! v0.3.46: All dialogue emission filtered through ExposureGate.
//! No specialist or Ralph loop may bypass this filtering.

use anna_shared::exposure::{DialogueClassification, ExposureGate, filter_final_answer_default};
use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
use anyhow::Result;

/// Map StepType to DialogueClassification for exposure filtering.
///
/// Classifications determine minimum exposure level required:
/// - Informational: Summary+ (status updates)
/// - Procedural: Dialogue+ (step-by-step actions)
/// - Diagnostic: Debug only (raw output, errors)
///
/// Phase 15: FinalAnswer is NO LONGER privileged. It MUST be sanitized.
/// This prevents LLM-generated manual commands from reaching the user.
pub fn classify_step(step_type: &StepType) -> Option<DialogueClassification> {
    match step_type {
        // Phase 15: FinalAnswer MUST be sanitized (no longer bypasses)
        // Uses Informational so it's always visible, but still filtered
        StepType::FinalAnswer => Some(DialogueClassification::Informational),

        // UserQuestion is user input - no filtering needed
        StepType::UserQuestion => None,
        // Clarification interactions - no filtering needed
        StepType::ClarificationQuestion => None,
        StepType::ClarificationResponse => None,
        // System alerts are pre-validated - no filtering needed
        StepType::SystemAlert => None,

        // Informational: status updates, completion notifications
        StepType::TicketCreated => Some(DialogueClassification::Informational),
        StepType::InvestigationComplete => Some(DialogueClassification::Informational),
        StepType::IntentResult => Some(DialogueClassification::Informational),
        StepType::WikiResults => Some(DialogueClassification::Informational),
        StepType::MissingInfo => Some(DialogueClassification::Informational),
        StepType::SubQuestionResult => Some(DialogueClassification::Informational),
        StepType::LlmError => Some(DialogueClassification::Informational),

        // Procedural: step-by-step actions
        StepType::TeamAssignment => Some(DialogueClassification::Procedural),
        StepType::TeamDispatch => Some(DialogueClassification::Procedural),
        StepType::SpecialistWorking => Some(DialogueClassification::Procedural),
        StepType::TeamDialogue => Some(DialogueClassification::Procedural),
        StepType::TeamEscalation => Some(DialogueClassification::Procedural),
        StepType::InvestigationStart => Some(DialogueClassification::Procedural),
        StepType::InvestigationHypothesis => Some(DialogueClassification::Procedural),
        StepType::InvestigationProbe => Some(DialogueClassification::Procedural),
        StepType::ExperimentStart => Some(DialogueClassification::Procedural),
        StepType::AnnaToLlm => Some(DialogueClassification::Procedural),
        StepType::CommandExec => Some(DialogueClassification::Procedural),
        StepType::IntentClassifying => Some(DialogueClassification::Procedural),
        StepType::WikiSearch => Some(DialogueClassification::Procedural),
        StepType::WikiCommands => Some(DialogueClassification::Procedural),
        StepType::SubQuestion => Some(DialogueClassification::Procedural),
        StepType::UnderstandingCheck => Some(DialogueClassification::Procedural),
        StepType::ConfirmationRequest => Some(DialogueClassification::Procedural),

        // Diagnostic: raw output, technical details, prompts
        StepType::InvestigationResult => Some(DialogueClassification::Diagnostic),
        StepType::ExperimentResult => Some(DialogueClassification::Diagnostic),
        StepType::CommandOutput => Some(DialogueClassification::Diagnostic),
        StepType::LlmCommands => Some(DialogueClassification::Diagnostic),
        StepType::ValidationPrompt => Some(DialogueClassification::Diagnostic),
        StepType::ValidationResponse => Some(DialogueClassification::Diagnostic),
        StepType::FinalPrompt => Some(DialogueClassification::Diagnostic),
        // Phase 24: Policy basis - Debug only
        StepType::PolicyBasis => Some(DialogueClassification::Diagnostic),

        // Phase 22: Heartbeat - always visible (Informational)
        StepType::Heartbeat => Some(DialogueClassification::Informational),
    }
}

/// Send a step over the streaming connection (unfiltered, for internal use).
async fn send_step_internal<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    step: DialogueStep,
) -> Result<()> {
    let resp = StreamingResponse::Step { step };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

/// Send a step over the streaming connection with exposure filtering.
/// Phase 15: FinalAnswer is ALWAYS filtered - uses fallback on policy violation.
pub async fn send_step<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    step: DialogueStep,
    gate: &ExposureGate,
) -> Result<()> {
    // Check if this step type requires filtering
    if let Some(classification) = classify_step(&step.step_type) {
        // Phase 15/22: FinalAnswer uses special filter with fallback
        // Uses default (ReadOnly) intent for backwards compatibility
        let result = if step.step_type == StepType::FinalAnswer {
            filter_final_answer_default(&step.content)
        } else {
            gate.filter(&step.content, classification)
        };

        if !result.emit {
            return Ok(()); // Blocked by exposure gate
        }
        // Send with sanitized/fallback content
        let filtered_step = DialogueStep {
            step_type: step.step_type,
            content: result.content,
        };
        send_step_internal(writer, filtered_step).await
    } else {
        // No filtering required (UserQuestion, ClarificationQuestion, etc.)
        send_step_internal(writer, step).await
    }
}

/// Create a step, push it to dialogue, and send it with exposure filtering.
pub async fn push_and_send<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    dialogue: &mut Vec<DialogueStep>,
    step_type: StepType,
    content: String,
    gate: &ExposureGate,
) -> Result<()> {
    let step = DialogueStep {
        step_type: step_type.clone(),
        content: content.clone(),
    };

    // Always record in dialogue (for replay/history)
    dialogue.push(step.clone());

    // Filter before sending to user
    send_step(writer, step, gate).await
}

/// Phase 22: Send a heartbeat step during long operations.
pub async fn send_heartbeat<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<()> {
    let step = DialogueStep {
        step_type: StepType::Heartbeat,
        content: ".".to_string(),
    };
    send_step(writer, step, gate).await
}

/// Phase 22: Heartbeat interval (2 seconds)
const HEARTBEAT_INTERVAL_SECS: u64 = 2;

/// Phase 22: Run an async operation with periodic heartbeats.
/// Sends a heartbeat every 2 seconds while waiting for the operation.
pub async fn with_heartbeat<W, F, T>(
    writer: &mut W,
    gate: &ExposureGate,
    operation: F,
) -> Result<T>
where
    W: tokio::io::AsyncWriteExt + Unpin,
    F: std::future::Future<Output = Result<T>>,
{
    use tokio::time::{interval, Duration};

    let mut heartbeat_interval = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
    // Skip the first tick (fires immediately)
    heartbeat_interval.tick().await;

    tokio::pin!(operation);

    loop {
        tokio::select! {
            result = &mut operation => {
                return result;
            }
            _ = heartbeat_interval.tick() => {
                // Send heartbeat, ignore errors (non-critical)
                let _ = send_heartbeat(writer, gate).await;
            }
        }
    }
}

/// Send the final Done response.
pub async fn send_done<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    result: &AskResult,
) -> Result<()> {
    let resp = StreamingResponse::Done {
        result: result.clone(),
    };
    let json = serde_json::to_string(&resp)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

/// Send all dialogue steps from a result with exposure filtering.
pub async fn send_dialogue_steps<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    dialogue: &[DialogueStep],
    gate: &ExposureGate,
) -> Result<()> {
    for step in dialogue {
        send_step(writer, step.clone(), gate).await?;
    }
    Ok(())
}

/// Build the final answer with evidence, teaching block, and optional confidence.
pub fn build_final_answer(
    answer: &str,
    evidence_line: &str,
    teaching_block: Option<String>,
) -> String {
    let mut final_answer = answer.to_string();
    if !evidence_line.is_empty() {
        final_answer = format!("{}\n\n{}", final_answer, evidence_line);
    }
    if let Some(teaching) = teaching_block {
        final_answer = format!("{}{}", final_answer, teaching);
    }
    final_answer
}

/// v0.3.113: Format confidence indicator for user display.
pub fn format_confidence_indicator(confidence: f32) -> String {
    let (level, indicator) = match confidence {
        c if c >= 0.9 => ("high", "[Confidence: HIGH]"),
        c if c >= 0.7 => ("good", "[Confidence: GOOD]"),
        c if c >= 0.5 => ("moderate", "[Confidence: MODERATE]"),
        _ => ("low", "[Confidence: LOW - verify before acting]"),
    };

    tracing::debug!("Answer confidence: {:.0}% ({})", confidence * 100.0, level);
    indicator.to_string()
}

/// Build the final answer with confidence indicator (v0.3.113).
pub fn build_final_answer_with_confidence(
    answer: &str,
    evidence_line: &str,
    teaching_block: Option<String>,
    confidence: Option<f32>,
) -> String {
    let mut final_answer = answer.to_string();

    // Add confidence indicator for non-trivial answers
    if let Some(conf) = confidence {
        // Only show for moderate or low confidence, or in debug mode
        if conf < 0.7 {
            let indicator = format_confidence_indicator(conf);
            final_answer = format!("{}\n\n{}", final_answer, indicator);
        }
    }

    if !evidence_line.is_empty() {
        final_answer = format!("{}\n\n{}", final_answer, evidence_line);
    }
    if let Some(teaching) = teaching_block {
        final_answer = format!("{}{}", final_answer, teaching);
    }
    final_answer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_step_final_answer_filtered() {
        // Phase 15: FinalAnswer is now filtered (no longer bypasses)
        let classification = classify_step(&StepType::FinalAnswer);
        assert!(classification.is_some());
        assert_eq!(classification.unwrap(), DialogueClassification::Informational);
    }

    #[test]
    fn test_classify_step_user_question_bypasses() {
        assert!(classify_step(&StepType::UserQuestion).is_none());
    }

    #[test]
    fn test_classify_step_informational() {
        assert_eq!(
            classify_step(&StepType::TicketCreated),
            Some(DialogueClassification::Informational)
        );
        assert_eq!(
            classify_step(&StepType::InvestigationComplete),
            Some(DialogueClassification::Informational)
        );
    }

    #[test]
    fn test_classify_step_procedural() {
        assert_eq!(
            classify_step(&StepType::TeamAssignment),
            Some(DialogueClassification::Procedural)
        );
        assert_eq!(
            classify_step(&StepType::InvestigationProbe),
            Some(DialogueClassification::Procedural)
        );
    }

    #[test]
    fn test_classify_step_diagnostic() {
        assert_eq!(
            classify_step(&StepType::InvestigationResult),
            Some(DialogueClassification::Diagnostic)
        );
        assert_eq!(
            classify_step(&StepType::CommandOutput),
            Some(DialogueClassification::Diagnostic)
        );
    }
}

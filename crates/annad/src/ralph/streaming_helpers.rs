//! Helper functions for streaming responses.
//!
//! v0.3.46: All dialogue emission filtered through ExposureGate.
//! No specialist or Ralph loop may bypass this filtering.

use anna_shared::exposure::{DialogueClassification, ExposureGate};
use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
use anyhow::Result;

/// Map StepType to DialogueClassification for exposure filtering.
///
/// Classifications determine minimum exposure level required:
/// - Informational: Summary+ (status updates)
/// - Procedural: Dialogue+ (step-by-step actions)
/// - Diagnostic: Debug only (raw output, errors)
pub fn classify_step(step_type: &StepType) -> Option<DialogueClassification> {
    match step_type {
        // Always shown - bypass filtering
        StepType::FinalAnswer => None,
        StepType::UserQuestion => None,
        StepType::ClarificationQuestion => None,
        StepType::ClarificationResponse => None,
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
pub async fn send_step<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    step: DialogueStep,
    gate: &ExposureGate,
) -> Result<()> {
    // Check if this step type requires filtering
    if let Some(classification) = classify_step(&step.step_type) {
        let result = gate.filter(&step.content, classification);
        if !result.emit {
            return Ok(()); // Blocked by exposure gate
        }
        // Send with sanitized content
        let filtered_step = DialogueStep {
            step_type: step.step_type,
            content: result.content,
        };
        send_step_internal(writer, filtered_step).await
    } else {
        // No filtering required (FinalAnswer, UserQuestion)
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

/// Build the final answer with evidence and teaching block.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_step_final_answer_bypasses() {
        assert!(classify_step(&StepType::FinalAnswer).is_none());
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

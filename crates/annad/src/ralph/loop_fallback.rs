//! Fallback handlers for the Ralph loop when max iterations are reached.
//! Tries universal handler, adaptive intelligence, and opportunity detection.

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{info, warn};

use super::criteria::IterationState;

/// Try fallback handlers when the main loop exhausts max iterations.
/// Returns Some(result) if a fallback succeeded, None if all fallbacks failed.
pub async fn try_fallback_handlers(
    model: &str,
    question: &str,
    iteration: u32,
    state: &mut IterationState,
    dialogue: &mut Vec<DialogueStep>,
) -> Option<Result<AskResult>> {
    if state.confidence >= 0.7 {
        return None;
    }

    info!("Low confidence, attempting universal handler");
    match crate::universal_handler::handle_universal_task(model, question).await {
        Ok(universal_result) => {
            info!("Universal handler succeeded");
            dialogue.push(DialogueStep {
                step_type: StepType::FinalAnswer,
                content: universal_result.clone(),
            });
            return Some(Ok(AskResult {
                answer: universal_result,
                success: true,
                iterations: iteration + 1,
                commands_executed: state.commands.clone(),
                dialogue: dialogue.clone(),
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(0.7),
            }));
        }
        Err(e) => {
            warn!("Universal handler failed: {}", e);

            // v0.3.164: ADAPTIVE INTELLIGENCE - Final fallback, Anna NEVER gives up
            info!("ACTIVATING ADAPTIVE INTELLIGENCE (multi-strategy approach)");
            match crate::adaptive_intelligence::solve_adaptively(model, question).await {
                Ok(adaptive_result) => {
                    info!("ADAPTIVE INTELLIGENCE SUCCEEDED");
                    dialogue.push(DialogueStep {
                        step_type: StepType::FinalAnswer,
                        content: adaptive_result.clone(),
                    });
                    return Some(Ok(AskResult {
                        answer: adaptive_result,
                        success: true,
                        iterations: iteration + 2,
                        commands_executed: state.commands.clone(),
                        dialogue: dialogue.clone(),
                        needs_clarification: false,
                        clarification_question: None,
                        cached: false,
                        citations: vec![],
                        abstained: false,
                        final_confidence: Some(0.8),
                    }));
                }
                Err(adaptive_error) => {
                    warn!("Adaptive intelligence exhausted all strategies: {}", adaptive_error);

                    // v0.3.165: OPPORTUNITY DETECTION - Propose future solutions
                    info!("DETECTING OPPORTUNITIES (propose what Anna CAN do)");
                    if let Some(opportunity) = crate::opportunity_detector::detect_opportunity(question).await {
                        info!("OPPORTUNITY DETECTED: {}", opportunity.missing);
                        let opportunity_msg = crate::opportunity_detector::format_opportunity(&opportunity);

                        dialogue.push(DialogueStep {
                            step_type: StepType::FinalAnswer,
                            content: opportunity_msg.clone(),
                        });

                        return Some(Ok(AskResult {
                            answer: opportunity_msg,
                            success: true,
                            iterations: iteration + 3,
                            commands_executed: state.commands.clone(),
                            dialogue: dialogue.clone(),
                            needs_clarification: false,
                            clarification_question: None,
                            cached: false,
                            citations: vec![],
                            abstained: false,
                            final_confidence: Some(opportunity.confidence),
                        }));
                    }

                    // NOW we truly give up (no opportunities detected)
                    warn!("No opportunities detected, truly unable to help");
                }
            }
        }
    }

    None
}

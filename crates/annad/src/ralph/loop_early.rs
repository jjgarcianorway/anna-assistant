//! Early-exit handlers for the non-streaming Ralph loop.
//! Checks pattern learning, teaching, failure memory, feasibility, temporal tasks,
//! smart file ops, and orchestration before the main iteration loop.

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{debug, info, warn};

use super::temporal;

/// Check for early-exit conditions before the main loop.
/// Returns Some(result) if the question was handled, None to continue to main loop.
pub async fn check_early_returns(model: &str, question: &str) -> Result<Option<AskResult>> {
    // v0.3.166: Record question for pattern learning
    crate::pattern_learning::record_question(question);

    // v0.3.166: Check for automation opportunities (recurring questions)
    if let Some(automation) = crate::pattern_learning::check_for_automation_opportunity(question) {
        info!("Automation opportunity detected for recurring question");
        let suggestion = crate::pattern_learning::format_automation_suggestion(&automation);

        return Ok(Some(AskResult {
            answer: suggestion,
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![
                DialogueStep {
                    step_type: StepType::UserQuestion,
                    content: question.to_string(),
                },
                DialogueStep {
                    step_type: StepType::FinalAnswer,
                    content: automation.message.clone(),
                },
            ],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: Some(0.9),
        }));
    }

    // v0.3.167: Teaching Mode - Detect and handle teaching requests
    if crate::teaching_mode::is_teaching_request(question) {
        info!("Teaching request detected");
        if let Ok(teaching_response) = crate::teaching_mode::handle_teaching_question(question).await {
            return Ok(Some(AskResult {
                answer: teaching_response.clone(),
                success: true,
                iterations: 1,
                commands_executed: vec![],
                dialogue: vec![
                    DialogueStep {
                        step_type: StepType::UserQuestion,
                        content: question.to_string(),
                    },
                    DialogueStep {
                        step_type: StepType::FinalAnswer,
                        content: teaching_response,
                    },
                ],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(0.85),
            }));
        }
    }

    // v0.3.167: Failure Memory - Check for known failures that can be auto-fixed
    if let Some(auto_fix_result) = crate::failure_memory::check_and_handle_known_failure(question).await {
        info!("Known failure detected, attempting auto-fix");
        return Ok(Some(AskResult {
            answer: auto_fix_result.clone(),
            success: true,
            iterations: 1,
            commands_executed: vec![], // Commands are in the response text
            dialogue: vec![
                DialogueStep {
                    step_type: StepType::UserQuestion,
                    content: question.to_string(),
                },
                DialogueStep {
                    step_type: StepType::FinalAnswer,
                    content: auto_fix_result,
                },
            ],
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: Some(0.90),
        }));
    }

    // v0.3.162: Step 0 - Feasibility analysis (detect truly impossible requests)
    let feasibility = crate::feasibility::analyze_feasibility(question);
    match feasibility {
        crate::feasibility::Feasibility::Impossible(reason) => {
            info!("Request deemed impossible: {}", reason);
            let answer = format!("I cannot do this: {}", reason);
            return Ok(Some(AskResult {
                answer,
                success: false,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![
                    DialogueStep {
                        step_type: StepType::UserQuestion,
                        content: question.to_string(),
                    },
                    DialogueStep {
                        step_type: StepType::FinalAnswer,
                        content: format!("I cannot do this: {}", reason),
                    },
                ],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: true,
                final_confidence: Some(1.0), // Confident it's impossible
            }));
        }
        crate::feasibility::Feasibility::RequiresExternal(reason) => {
            info!("Request requires external resources: {}", reason);
            // Continue but inform user
        }
        crate::feasibility::Feasibility::Challenging => {
            info!("Challenging request detected - will use universal handler if needed");
        }
        crate::feasibility::Feasibility::Possible => {
            debug!("Request feasible, proceeding normally");
        }
    }

    // v0.3.162: Step 0.5 - Temporal task detection (background monitoring)
    if crate::temporal_tasks::requires_background_monitoring(question) {
        if let Some(duration) = crate::temporal_tasks::detect_temporal_requirement(question) {
            info!("Temporal task detected: {} seconds", duration);
            return Ok(Some(temporal::handle_temporal_task(model, question, duration).await?));
        }
    }

    // v0.3.164: Step 0.6 - Smart file operations (handle complex file tasks efficiently)
    if crate::smart_file_ops::is_file_operation(question) {
        info!("File operation detected, using smart handler");
        match crate::smart_file_ops::execute_smart_file_operation(model, question).await {
            Ok(result) => {
                info!("Smart file operation succeeded");
                return Ok(Some(AskResult {
                    answer: result.clone(),
                    success: true,
                    iterations: 1,
                    commands_executed: vec![],
                    dialogue: vec![
                        DialogueStep {
                            step_type: StepType::UserQuestion,
                            content: question.to_string(),
                        },
                        DialogueStep {
                            step_type: StepType::FinalAnswer,
                            content: result,
                        },
                    ],
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                    abstained: false,
                    final_confidence: Some(0.85),
                }));
            }
            Err(e) => {
                debug!("Smart file ops failed ({}), falling back to normal flow", e);
                // Continue with normal flow
            }
        }
    }

    // v0.3.169: Orchestration - Determine if deep analysis modules should run
    let system_context = crate::llm_core::system_context();
    let orchestration_plan = crate::llm_orchestration::determine_relevant_modules(
        model,
        question,
        &system_context,
    )
    .await?;

    // Run orchestrated analysis if modules were selected
    if !orchestration_plan.modules.is_empty() {
        info!(
            "Orchestration: Running {} modules - {}",
            orchestration_plan.modules.len(),
            orchestration_plan.rationale
        );

        match crate::llm_orchestration::execute_modules(&orchestration_plan.modules, Some(question)).await {
            Ok(results) if !results.is_empty() => {
                info!("Orchestration: Found {} analysis results", results.len());

                let base_answer = format!(
                    "I've analyzed your question through multiple perspectives:\n\n{}",
                    orchestration_plan.rationale
                );

                let enriched_answer = crate::llm_orchestration::synthesize_results(
                    model,
                    question,
                    &results,
                    &base_answer,
                )
                .await?;

                return Ok(Some(AskResult {
                    answer: enriched_answer.clone(),
                    success: true,
                    iterations: 1,
                    commands_executed: vec![],
                    dialogue: vec![
                        DialogueStep {
                            step_type: StepType::UserQuestion,
                            content: question.to_string(),
                        },
                        DialogueStep {
                            step_type: StepType::FinalAnswer,
                            content: enriched_answer,
                        },
                    ],
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                    abstained: false,
                    final_confidence: Some(0.85),
                }));
            }
            Ok(_) => {
                debug!("Orchestration: Modules ran but found nothing interesting, continuing normal flow");
            }
            Err(e) => {
                warn!("Orchestration: Module execution failed: {}, continuing normal flow", e);
            }
        }
    }

    Ok(None)
}

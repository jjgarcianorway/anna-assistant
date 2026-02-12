//! Ralph-style autonomous iteration loop for answering questions.
//!
//! The Ralph Wiggum approach: iteration beats perfection.
//! Instead of complex branching, use a simple loop with clear completion criteria.
//!
//! Principles:
//! 1. Define "done" upfront - what does success look like?
//! 2. Iterate until done - trust the loop, not complexity
//! 3. Self-evaluate - LLM checks its own work before declaring done
//! 4. Learn from attempts - each iteration improves the next

mod commands;
pub mod confidence;
mod config_handler;
mod criteria;
pub mod evidence;
mod parallel;
mod recipe_learning;
mod streaming;
pub mod streaming_helpers;
mod suggestions;
mod verification;

pub use suggestions::{generate_suggestions, format_suggestions};
pub use parallel::{should_parallelize, run_parallel_investigation, synthesize_parallel_results};

// Re-export public API
pub use criteria::{determine_criteria, AnswerType, CompletionCriteria};
pub use streaming::ralph_loop_streaming;

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::core_loop::{execute_command, strip_ansi_codes};

use commands::{generate_answer, get_commands, get_next_action, self_evaluate, NextAction};
use criteria::IterationState;
use recipe_learning::learn_recipe_from_answer;
use verification::truncate;

/// Handle temporal tasks (background monitoring for X duration).
/// v0.3.162: Enables "capture network traffic for 20 minutes" type requests.
async fn handle_temporal_task(model: &str, question: &str, duration_secs: u64) -> Result<AskResult> {
    info!("Handling temporal task: {} for {}s", question, duration_secs);

    let mut dialogue = vec![
        DialogueStep {
            step_type: StepType::UserQuestion,
            content: question.to_string(),
        },
    ];

    // Use universal handler to figure out HOW to do the monitoring
    let monitoring_setup = crate::universal_handler::handle_universal_task(model, question).await?;

    dialogue.push(DialogueStep {
        step_type: StepType::InvestigationProbe,
        content: format!("Setting up {} minute monitoring...", duration_secs / 60),
    });

    // Extract the commands from universal handler output
    // Parse the execution plan to get start/stop commands
    let start_cmd = extract_monitoring_command(&monitoring_setup);

    // Start the temporal task
    let task = crate::temporal_tasks::start_temporal_task(
        question.to_string(),
        start_cmd.clone(),
        None, // Stop command if needed
        duration_secs,
    )
    .await?;

    let answer = format!(
        "Started monitoring task (ID: {}). Will run for {} minutes and report back.\n\nTo check progress: annactl \"check task {}\"",
        task.id,
        duration_secs / 60,
        task.id
    );

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: answer.clone(),
    });

    Ok(AskResult {
        answer,
        success: true,
        iterations: 1,
        commands_executed: vec![start_cmd],
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(0.8),
    })
}

/// Extract monitoring command from universal handler output.
fn extract_monitoring_command(output: &str) -> String {
    // Look for "Step 1:" or first command in output
    for line in output.lines() {
        if line.contains("Step 1:") || line.starts_with("1.") {
            // Extract command after colon or number
            let cmd = line
                .split_once(':')
                .map(|(_, cmd)| cmd.trim())
                .unwrap_or(line.trim());
            return cmd.to_string();
        }
    }
    // Fallback: use whole output as command (probably wrong but better than nothing)
    output.lines().next().unwrap_or("echo 'monitoring'").to_string()
}

/// Extract task type from question for strategy learning
fn extract_task_type(question: &str) -> String {
    let q = question.to_lowercase();

    if q.contains("install") || q.contains("package") || q.contains("pacman") || q.contains("yay") {
        "package_management".to_string()
    } else if q.contains("network") || q.contains("wifi") || q.contains("ethernet") || q.contains("ip") {
        "network_configuration".to_string()
    } else if q.contains("service") || q.contains("systemctl") || q.contains("daemon") {
        "service_management".to_string()
    } else if q.contains("disk") || q.contains("partition") || q.contains("mount") || q.contains("filesystem") {
        "disk_management".to_string()
    } else if q.contains("user") || q.contains("permission") || q.contains("sudo") || q.contains("group") {
        "user_management".to_string()
    } else if q.contains("config") || q.contains("configure") || q.contains("setting") {
        "system_configuration".to_string()
    } else if q.contains("error") || q.contains("fix") || q.contains("broken") || q.contains("fail") {
        "troubleshooting".to_string()
    } else if q.contains("monitor") || q.contains("status") || q.contains("check") {
        "monitoring".to_string()
    } else {
        "general_task".to_string()
    }
}

/// The Ralph loop: iterate until done (non-streaming version)
/// LLM-first: no bypass paths. Every question goes through the LLM.
/// v0.3.162: Universal capability system with feasibility checking and temporal tasks.
pub async fn ralph_loop(model: &str, question: &str) -> Result<AskResult> {
    // v0.3.162: Step 0 - Feasibility analysis (detect truly impossible requests)
    let feasibility = crate::feasibility::analyze_feasibility(question);
    match feasibility {
        crate::feasibility::Feasibility::Impossible(reason) => {
            info!("Request deemed impossible: {}", reason);
            let answer = format!("I cannot do this: {}", reason);
            return Ok(AskResult {
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
            });
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
            // Use universal handler to set up monitoring
            return handle_temporal_task(model, question, duration).await;
        }
    }

    let criteria = determine_criteria(question);
    info!(
        "Ralph loop: {:?}, confidence >= {:.0}%, max {} iterations",
        criteria.answer_type,
        criteria.min_confidence * 100.0,
        criteria.max_iterations
    );

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record the question
    dialogue.push(DialogueStep {
        step_type: StepType::UserQuestion,
        content: question.to_string(),
    });

    // v0.3.159: Add memory context on first iteration
    if iteration == 0 {
        let memory_context = crate::intelligence::get_memory_context(question);
        if !memory_context.is_empty() {
            debug!("Memory context available for this question");
            state.feedback = Some(memory_context);
        }

        // v0.3.160: Add strategic guidance from meta-learning
        let strategic_guidance = crate::meta_learning::get_strategic_guidance(question);
        if !strategic_guidance.is_empty() {
            info!("Strategic guidance available from past experience");
            state.feedback = Some(
                state
                    .feedback
                    .as_ref()
                    .map(|f| format!("{}\n{}", f, strategic_guidance))
                    .unwrap_or(strategic_guidance),
            );
        }
    }

    // v0.3.159: Add disk health predictions
    let disk_predictions = crate::intelligence::get_disk_predictions();
    if !disk_predictions.is_empty() && iteration == 0 {
        info!("Adding disk health predictions to context");
        state.feedback = Some(
            state
                .feedback
                .as_ref()
                .map(|f| format!("{}\n{}", f, disk_predictions))
                .unwrap_or(disk_predictions),
        );
    }

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        info!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        // Step 1: Determine next action (config vs commands)
        let next_action = get_next_action(model, question, &state).await?;

        // v0.3.161: Handle configuration requests with dedicated config handler
        if matches!(next_action, NextAction::Config) {
            info!("Config request detected, using config handler");
            return config_handler::handle_config_request_sync(model, question, &criteria).await;
        }

        // Extract commands from next action
        let commands = match next_action {
            NextAction::Commands(cmds) => cmds,
            NextAction::None | NextAction::Config => Vec::new(),
        };

        if commands.is_empty() && state.outputs.is_empty() {
            debug!("No commands needed, generating direct answer");
        } else if !commands.is_empty() {
            for cmd in &commands {
                dialogue.push(DialogueStep {
                    step_type: StepType::CommandExec,
                    content: cmd.clone(),
                });

                match execute_command(cmd) {
                    Ok(output) => {
                        let clean_output = strip_ansi_codes(&output);
                        state.commands.push(cmd.clone());
                        state.outputs.push(clean_output.clone());
                        dialogue.push(DialogueStep {
                            step_type: StepType::CommandOutput,
                            content: truncate(&clean_output, 500),
                        });
                    }
                    Err(e) => {
                        debug!("Command failed: {}: {}", cmd, e);
                        let failure_msg = format!("Command '{}' failed: {}", cmd, e);

                        // v0.3.159: Try root cause analysis for failures
                        let rca_result = crate::intelligence::analyze_failure(&failure_msg, &[]);
                        if let Some(root_cause) = rca_result {
                            debug!("Root cause analysis: {}", root_cause);
                            state.feedback = Some(format!("{}\n\nRoot Cause Analysis:\n{}", failure_msg, root_cause));
                        } else {
                            state.feedback = Some(failure_msg);
                        }
                    }
                }
            }
        }

        // Step 2: Generate answer
        // v0.3.131: Non-streaming doesn't have wiki research yet - pass None for now
        let answer = generate_answer(model, question, &state, &criteria, None).await?;
        state.answer = Some(answer.clone());

        // Step 3: Self-evaluate
        let eval = self_evaluate(model, question, &answer, &state, &criteria).await?;
        state.confidence = eval.confidence;

        debug!(
            "Self-evaluation: complete={}, confidence={:.0}%",
            eval.is_complete,
            eval.confidence * 100.0
        );

        // Step 4: Check completion criteria
        if eval.is_complete && eval.confidence >= criteria.min_confidence {
            info!(
                "Ralph done! Confidence {:.0}% >= {:.0}% threshold",
                eval.confidence * 100.0,
                criteria.min_confidence * 100.0
            );

            dialogue.push(DialogueStep {
                step_type: StepType::FinalAnswer,
                content: answer.clone(),
            });

            // Learn recipe from successful answer
            learn_recipe_from_answer(question, &state.commands, eval.confidence);

            // v0.3.159: Record successful interaction in memory
            crate::intelligence::record_success(
                question,
                &answer,
                &state.commands,
                eval.confidence,
            );

            // v0.3.160: Self-reflect and learn strategy
            let execution_log = format!("Commands: {}\nAnswer: {}", state.commands.join("; "), answer);
            if let Ok(reflection) = crate::meta_learning::reflect_on_task(
                model, question, &execution_log, true
            ).await {
                // Learn strategy from this successful task
                let task_type = extract_task_type(question);
                crate::meta_learning::learn_strategy(
                    task_type,
                    question.chars().take(100).collect(),
                    state.commands.clone(),
                    true,
                    reflection.insights,
                );
            }

            return Ok(AskResult {
                answer,
                success: true,
                iterations: iteration,
                commands_executed: state.commands,
                dialogue,
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(eval.confidence),
            });
        }

        // Not done yet - prepare feedback for next iteration
        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
        info!(
            "Not done yet: {:?}",
            state.not_done_reason.as_deref().unwrap_or("confidence too low")
        );
    }

    // Max iterations reached - try universal handler as last resort
    warn!(
        "Ralph max iterations reached (confidence: {:.0}%), trying universal handler fallback",
        state.confidence * 100.0
    );

    // v0.3.162: Universal handler fallback for complex/novel tasks
    if state.confidence < 0.7 {
        info!("Low confidence, attempting universal handler");
        match crate::universal_handler::handle_universal_task(model, question).await {
            Ok(universal_result) => {
                info!("Universal handler succeeded");
                dialogue.push(DialogueStep {
                    step_type: StepType::FinalAnswer,
                    content: universal_result.clone(),
                });
                return Ok(AskResult {
                    answer: universal_result,
                    success: true,
                    iterations: iteration + 1,
                    commands_executed: state.commands,
                    dialogue,
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                    abstained: false,
                    final_confidence: Some(0.7),
                });
            }
            Err(e) => {
                warn!("Universal handler also failed: {}", e);
                // Continue with best effort
            }
        }
    }

    let final_answer = state.answer.unwrap_or_else(|| {
        "I wasn't able to fully answer your question. Please try rephrasing or ask about something more specific.".to_string()
    });

    // v0.3.159: Record failure if confidence is very low
    if state.confidence < 0.5 {
        let reason = state
            .not_done_reason
            .as_deref()
            .unwrap_or("Max iterations reached with low confidence");
        crate::intelligence::record_failure(question, reason, &state.commands);
    }

    dialogue.push(DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    });

    // Phase 26: Determine if this is abstention vs failure
    let has_execution_error = state.feedback.as_ref()
        .map(|f| f.contains("failed") || f.contains("error"))
        .unwrap_or(false);
    let is_abstained = state.confidence < 0.5 && !has_execution_error;

    Ok(AskResult {
        answer: final_answer,
        success: state.confidence >= 0.5,
        iterations: iteration,
        commands_executed: state.commands,
        dialogue,
        needs_clarification: state.confidence < 0.3,
        clarification_question: state.not_done_reason,
        cached: false,
        citations: vec![],
        abstained: is_abstained,
        final_confidence: Some(state.confidence),
    })
}

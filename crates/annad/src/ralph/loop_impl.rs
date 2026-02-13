//! Non-streaming Ralph loop implementation.
//! The full investigate-evaluate-iterate cycle.

use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::core_loop::{execute_command, strip_ansi_codes};

use super::commands::{generate_answer, get_next_action, self_evaluate, NextAction};
use super::criteria::{determine_criteria, IterationState};
use super::loop_early::check_early_returns;
use super::loop_fallback::try_fallback_handlers;
use super::recipe_learning::learn_recipe_from_answer;
use super::temporal;
use super::verification::truncate;

/// The Ralph loop: iterate until done (non-streaming version)
/// LLM-first: no bypass paths. Every question goes through the LLM.
/// v0.3.162: Universal capability system with feasibility checking and temporal tasks.
/// v0.3.166: Pattern learning, failure memory, and automation suggestions.
pub async fn ralph_loop_impl(model: &str, question: &str) -> Result<AskResult> {
    // Check early-exit conditions (automation, teaching, feasibility, temporal, orchestration)
    if let Some(result) = check_early_returns(model, question).await? {
        return Ok(result);
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
        // v0.3.163: Add system identity context (real names, not generic)
        // v0.3.172: Always get fresh username (daemon may have started before user login)
        let identity = crate::system_identity::get_system_identity();
        let real_username = crate::user_context::get_real_user().unwrap_or_else(|_| identity.username.clone());
        let identity_context = format!(
            "SYSTEM IDENTITY:\nHostname: {}\nUser: {} (IMPORTANT: This is the CALLING USER, NOT the daemon user. If asked 'What user am I?', answer with THIS user, not the output of whoami which shows the daemon user 'root')\nDistro: {}\nPackage Manager: {}\nShell: {}\nNetwork Devices: {}\nCurrent WiFi: {}\nDesktop: {}",
            identity.hostname,
            real_username,
            identity.distro_name,
            identity.package_manager(),
            identity.shell,
            identity.network_devices.iter().map(|d| format!("{} ({})", d.name, d.device_type)).collect::<Vec<_>>().join(", "),
            identity.current_ssid.as_deref().unwrap_or("not connected"),
            identity.desktop_environment.as_deref().unwrap_or("none")
        );

        state.feedback = Some(identity_context);

        let memory_context = crate::intelligence::get_memory_context(question);
        if !memory_context.is_empty() {
            debug!("Memory context available for this question");
            state.feedback = Some(format!("{}\n\n{}", state.feedback.unwrap(), memory_context));
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
            return super::config_handler::handle_config_request_sync(model, question, &criteria).await;
        }

        // Extract commands from next action
        let commands = match next_action {
            NextAction::Commands(cmds) => cmds,
            NextAction::None | NextAction::Config | NextAction::ListCreated
            | NextAction::CreateAutomation | NextAction::SetWallpaper | NextAction::AuditSsh
            | NextAction::ManageUser | NextAction::BuildKernel
            | NextAction::GeneratePdf | NextAction::FullReport => Vec::new(),
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
                let task_type = temporal::extract_task_type(question);
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
    if let Some(fallback_result) = try_fallback_handlers(model, question, iteration, &mut state, &mut dialogue).await {
        return fallback_result;
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

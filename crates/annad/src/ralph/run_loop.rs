//! The main Ralph investigation loop with streaming progress.

use anna_shared::experiment::estimate_command_risk;
use anna_shared::exposure::ExposureGate;
use anna_shared::probe_ledger::ProbeLedger;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::core_loop::{execute_command, strip_ansi_codes};
use crate::department;
use crate::team_speak;
use super::commands::{generate_answer, get_next_action, NextAction, self_evaluate};
use super::criteria::{determine_criteria, IterationState};
use super::streaming_helpers::{push_and_send, send_done, with_heartbeat};
use super::verification::{truncate, verify_answer};
use super::finish::finish_streaming;
use super::config_flow::{handle_config_request_with_research, handle_user_management};
use super::system_probe::investigate_system_state;

/// Run the full Ralph loop with streaming progress.
pub async fn run_full_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    session_id: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<AskResult> {
    let criteria = determine_criteria(question);
    info!("Ralph streaming: {:?}, max {} iterations", criteria.answer_type, criteria.max_iterations);

    let mut state = IterationState::default();
    let mut dialogue = Vec::new();
    let mut iteration = 0;

    // Record and send user's question
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;

    // Research Arch Wiki for relevant documentation using RAG
    push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
        "Searching Arch Wiki for relevant documentation...".to_string(), gate).await?;

    let wiki_research = match anna_shared::wiki::search::search(
        "http://localhost:11434", // Ollama URL
        question,
        3, // top 3 results
        true, // use semantic search
    ).await {
        Ok(results) if !results.is_empty() => {
            push_and_send(writer, &mut dialogue, StepType::WikiResults,
                format!("Found {} relevant wiki articles", results.len()), gate).await?;

            // Format wiki results for LLM context
            let mut formatted = vec!["ARCH WIKI RESEARCH:".to_string(), "".to_string()];
            for (i, result) in results.iter().enumerate() {
                formatted.push(format!("Article {}: {} (relevance: {:.0}%)",
                    i + 1, result.article.title, result.score * 100.0));
                formatted.push("".to_string());

                // Use relevant section if available, otherwise first 2000 chars
                let content = if let Some(ref section) = result.relevant_section {
                    section.chars().take(2000).collect::<String>()
                } else {
                    result.article.content.chars().take(2000).collect::<String>()
                };
                formatted.push(content);
                formatted.push("".to_string());
            }
            formatted.join("\n")
        }
        _ => {
            // No wiki results - that's fine, LLM can still work from its knowledge
            String::new()
        }
    };

    // Create ticket for fly-on-the-wall experience
    let dept_name = department::determine_department(question);
    let mut ticket = department::create_ticket(question, dept_name);
    push_and_send(writer, &mut dialogue, StepType::TicketCreated, ticket.case_number.clone(), gate)
        .await?;

    // Dispatch to appropriate specialist
    let specialist = department::get_specialist_for_topic(question);
    let assigned_spec_name = if let Some(spec) = specialist {
        ticket.assign(spec.name);
        department::update_ticket(&ticket);
        let assignment = team_speak::anna_assigns_to(spec, question);
        push_and_send(writer, &mut dialogue, StepType::TeamAssignment, assignment, gate).await?;
        let ack = team_speak::specialist_acknowledges(spec);
        push_and_send(writer, &mut dialogue, StepType::SpecialistWorking,
            format!("{}: {}", spec.name, ack), gate).await?;
        Some(spec.name.to_string())
    } else {
        None
    };

    // Track investigation probes with deduplication (Phase 22)
    let mut probe_ledger = ProbeLedger::new();
    let mut probe_count: usize = 0;
    let mut experiment_count: usize = 0;

    // Start investigation mode
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart, question.to_string(), gate)
        .await?;

    ticket.start_investigating();
    department::update_ticket(&ticket);

    // THE RALPH LOOP
    while iteration < criteria.max_iterations {
        iteration += 1;
        debug!("Ralph iteration {}/{}", iteration, criteria.max_iterations);

        // Ask LLM what to do next (with wiki research context)
        let next_action = get_next_action_with_research(model, question, &state, &wiki_research).await?;

        // Handle CONFIG: LLM recognized this as a system configuration request
        if matches!(next_action, NextAction::Config) {
            info!("LLM detected config request, investigating system state first");

            // v0.3.139: Investigate system BEFORE generating plan
            // Gather critical system info that plan generation needs
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
                "Investigating current system state...".to_string(), gate).await?;

            let system_state = investigate_system_state(&mut state, &mut dialogue, writer, gate).await?;

            // v0.3.185: Inject DE/WM context for desktop configuration requests
            let username = crate::user_context::get_real_user()
                .unwrap_or_else(|_| "root".to_string());
            let de_context = crate::ralph::config_handler::investigate_de_config(question, &username);
            let combined_state = if de_context.is_empty() || de_context.contains("Unknown") {
                system_state
            } else {
                format!("{}\n\n{}", system_state, de_context)
            };

            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
                "System investigation complete. Generating plan with real values...".to_string(), gate).await?;

            return handle_config_request_with_research(model, question, session_id, &wiki_research, &combined_state, writer, gate, &mut dialogue).await;
        }

        // v0.3.187: Agentic capability handlers — return AskResult
        macro_rules! agentic_result {
            ($answer:expr) => {{
                let answer = $answer;
                push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;
                return Ok(AskResult {
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
                });
            }};
        }

        if matches!(next_action, NextAction::ListCreated) {
            let registry = crate::artifact_registry::ArtifactRegistry::load();
            agentic_result!(registry.format_for_user());
        }

        if matches!(next_action, NextAction::AuditSsh) {
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
                "Reading sshd_config and researching Arch Wiki hardening guidelines...".to_string(), gate).await?;
            let result = crate::ssh_auditor::audit_ssh_config(model).await
                .unwrap_or_else(|e| format!("SSH audit failed: {}", e));
            agentic_result!(result);
        }

        if matches!(next_action, NextAction::SetWallpaper) {
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
                "Detecting your desktop environment and researching wallpaper tools...".to_string(), gate).await?;
            let result = crate::wallpaper::setup_wallpaper_automation(model).await
                .unwrap_or_else(|e| format!("Wallpaper setup failed: {}", e));
            agentic_result!(result);
        }

        if matches!(next_action, NextAction::CreateAutomation) {
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
                "Researching systemd timer best practices and generating unit files...".to_string(), gate).await?;
            let result = crate::automation_creator::create_and_register_automation(model, question).await
                .unwrap_or_else(|e| format!("Could not create automation: {}", e));
            agentic_result!(result);
        }

        if matches!(next_action, NextAction::ManageUser) {
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
                "Parsing user management request...".to_string(), gate).await?;
            let result = handle_user_management(model, question, writer, gate, &mut dialogue).await
                .unwrap_or_else(|e| format!("User management failed: {}", e));
            agentic_result!(result);
        }

        if matches!(next_action, NextAction::BuildKernel) {
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe,
                "Detecting hardware and researching Arch Wiki kernel compilation guide...".to_string(), gate).await?;
            let result = crate::kernel_builder::generate_kernel_build_plan(model, None).await
                .unwrap_or_else(|e| format!("Kernel plan generation failed: {}", e));
            agentic_result!(result);
        }

        let commands = match next_action {
            NextAction::Commands(cmds) => cmds,
            NextAction::None | NextAction::Config | NextAction::ListCreated
            | NextAction::CreateAutomation | NextAction::SetWallpaper | NextAction::AuditSsh
            | NextAction::ManageUser | NextAction::BuildKernel => Vec::new(),
        };

        for cmd in &commands {
            // Phase 22: Deduplicate probes via ProbeLedger
            if !probe_ledger.should_execute(cmd) {
                debug!("Skipping duplicate probe: {}", cmd);
                continue;
            }

            let risk = estimate_command_risk(cmd);
            let is_risky = risk > 0.3;

            probe_count += 1;
            push_and_send(writer, &mut dialogue, StepType::InvestigationProbe, cmd.clone(), gate)
                .await?;

            if is_risky {
                experiment_count += 1;
                ticket.start_experimenting();
                department::update_ticket(&ticket);
                push_and_send(writer, &mut dialogue, StepType::ExperimentStart,
                    format!("[risk={:.2}] expected=success", risk), gate).await?;
            }

            match execute_command(cmd) {
                Ok(output) => {
                    let clean_output = strip_ansi_codes(&output);
                    state.commands.push(cmd.clone());
                    state.outputs.push(clean_output.clone());
                    push_and_send(writer, &mut dialogue, StepType::InvestigationResult,
                        truncate(&clean_output, 500), gate).await?;
                    if is_risky {
                        let actual = if clean_output.contains("error") || clean_output.contains("failed")
                            { "failed" } else { "success" };
                        push_and_send(writer, &mut dialogue, StepType::ExperimentResult,
                            format!("actual={}", actual), gate).await?;
                        ticket.start_investigating();
                        department::update_ticket(&ticket);
                    }
                }
                Err(e) => {
                    if is_risky {
                        push_and_send(writer, &mut dialogue, StepType::ExperimentResult,
                            format!("actual=error ({})", e), gate).await?;
                        ticket.start_investigating();
                        department::update_ticket(&ticket);
                    }
                    state.feedback = Some(format!("Command '{}' failed: {}", cmd, e));
                }
            }
        }

        // v0.3.186: Track question topic for package suggestions (fire-and-forget)
        {
            let model_clone = model.to_string();
            let q_clone = question.to_string();
            tokio::spawn(async move {
                crate::pkg_suggestions::check_for_suggestions(&model_clone, &q_clone).await;
            });
        }

        // Phase 22: Wrap LLM calls with heartbeat emission
        // v0.3.131: Pass wiki research to answer generation
        let wiki_ref = if wiki_research.is_empty() { None } else { Some(wiki_research.as_str()) };
        let answer = with_heartbeat(writer, gate, generate_answer(model, question, &state, &criteria, wiki_ref)).await?;
        state.answer = Some(answer.clone());

        let eval = with_heartbeat(writer, gate, self_evaluate(model, question, &answer, &state, &criteria)).await?;
        state.confidence = eval.confidence;

        if eval.is_complete && eval.confidence >= criteria.min_confidence {
            return finish_streaming(
                writer,
                &mut dialogue,
                &mut ticket,
                &state,
                &answer,
                question,
                iteration,
                probe_count,
                experiment_count,
                &assigned_spec_name,
                eval.confidence,
                gate,
            )
            .await;
        }

        state.feedback = eval.suggestions;
        state.not_done_reason = eval.missing;
    }

    // Max iterations - return best effort
    push_and_send(writer, &mut dialogue, StepType::InvestigationComplete,
        format!("{} probes, {} experiments run (max iterations reached)", probe_count, experiment_count),
        gate).await?;

    let raw_answer = state.answer.unwrap_or_else(|| {
        "I couldn't fully answer your question. Please try rephrasing.".to_string()
    });
    let evidence: Vec<(String, String, i32)> = state
        .commands
        .iter()
        .zip(state.outputs.iter())
        .map(|(cmd, out)| (cmd.clone(), out.clone(), 0))
        .collect();
    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &evidence, debug_mode);
    let final_answer =
        super::streaming_helpers::build_final_answer(&verification.answer, &verification.evidence_line, None);
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, final_answer.clone(), gate).await?;

    // Phase 26: Determine if this is abstention vs failure
    // Abstention: max iterations + low confidence + no execution errors
    let has_execution_error = state.feedback.as_ref()
        .map(|f| f.contains("failed") || f.contains("error"))
        .unwrap_or(false);
    let is_abstained = state.confidence < 0.5 && !has_execution_error;

    let result = AskResult {
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
    };
    send_done(writer, &result).await?;

    Ok(result)
}

/// Wrapper to include wiki research in get_next_action.
pub async fn get_next_action_with_research(
    model: &str,
    question: &str,
    state: &IterationState,
    wiki_research: &str,
) -> Result<NextAction> {
    // For now, just call the original - we can enhance this later
    // The wiki research will be used in handle_config_request instead
    get_next_action(model, question, state).await
}

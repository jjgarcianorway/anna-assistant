//! Streaming Ralph loop with real-time progress updates.
//! LLM-first: no bypass paths. Every question goes through the LLM.

use anna_shared::experiment::estimate_command_risk;
use anna_shared::exposure::ExposureGate;
use chrono::Timelike;
use anna_shared::policy::get_policy;
use anna_shared::probe_ledger::ProbeLedger;
use anna_shared::rpc::{AskResult, DialogueStep, StepType};
use anna_shared::teaching;
use anyhow::Result;
use tracing::{debug, info};

use crate::core_loop::{execute_command, strip_ansi_codes};
use crate::department;
use crate::team_speak;

use super::commands::{generate_answer, get_next_action, NextAction, self_evaluate};
use super::criteria::{determine_criteria, IterationState};
use super::recipe_learning::{build_teaching_context, learn_recipe_from_answer};
use super::streaming_helpers::{build_final_answer, build_final_answer_with_confidence, push_and_send, send_done, with_heartbeat};
use super::verification::{truncate, verify_answer};

/// Streaming version of the Ralph loop with real-time progress updates.
/// LLM-first: all questions go through the full investigation loop.
pub async fn ralph_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    writer: &mut W,
) -> Result<AskResult> {
    let gate = ExposureGate::from_config();

    if gate.diagnostic_visible() {
        let policy = get_policy();
        let basis = policy.format_debug_basis();
        debug!("{}", basis);
        let step = DialogueStep {
            step_type: StepType::PolicyBasis,
            content: basis,
        };
        let _ = super::streaming_helpers::send_step(writer, step, &gate).await;
    }

    // Check for reminder requests first
    if let Some(result) = handle_reminder_request(question, writer, &gate).await? {
        return Ok(result);
    }

    // Check for morning briefing setup
    if let Some(result) = handle_morning_briefing_request(question, writer, &gate).await? {
        return Ok(result);
    }

    // v0.3.120: Handle natural language system queries
    if let Some(result) = handle_natural_system_query(question, writer, &gate).await? {
        return Ok(result);
    }

    // v0.3.123: Handle well-known error patterns with instant answers
    if let Some(result) = handle_pattern_match(question, writer, &gate).await? {
        return Ok(result);
    }

    // v0.3.125: Handle package history queries
    if let Some(result) = handle_package_history_query(question, writer, &gate).await? {
        return Ok(result);
    }

    // v0.3.121: Check for multi-domain questions that benefit from parallel investigation
    if let Some(result) = handle_multi_agent_query(question, writer, &gate).await? {
        return Ok(result);
    }

    // All other questions go through the full loop
    run_full_loop_streaming(model, question, writer, &gate).await
}

/// Handle "remind me in X" requests directly.
async fn handle_reminder_request<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::scheduler::{parse_reminder, ScheduledTask, TaskStore};
    use super::streaming_helpers::{push_and_send, send_done};

    let Some((message, when)) = parse_reminder(question) else {
        return Ok(None);
    };

    // Create and save the reminder
    let task = ScheduledTask::reminder(&message, when);
    let mut store = TaskStore::load();
    store.add(task);
    if let Err(e) = store.save() {
        tracing::warn!("Failed to save reminder: {}", e);
    }

    // Format when for display
    let duration = when - chrono::Utc::now();
    let when_str = if duration.num_hours() >= 1 {
        format!("{} hour(s)", duration.num_hours())
    } else {
        format!("{} minute(s)", duration.num_minutes())
    };

    let answer = format!("Got it. I'll remind you to \"{}\" in {}.", message, when_str);

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

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
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// Handle morning briefing setup requests.
async fn handle_morning_briefing_request<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::scheduler::{parse_morning_briefing, ScheduledTask, TaskStore};
    use super::streaming_helpers::{push_and_send, send_done};

    let Some(time) = parse_morning_briefing(question) else {
        return Ok(None);
    };

    // Create and save the morning briefing task
    let mut store = TaskStore::load();

    // Remove existing morning briefing if any
    store.remove_morning_briefing();

    let task = ScheduledTask::morning_briefing(time);
    store.add(task);

    if let Err(e) = store.save() {
        tracing::warn!("Failed to save morning briefing: {}", e);
    }

    let answer = format!(
        "Morning briefing set up. I'll send you a daily health check at {:02}:{:02}.",
        time.hour(), time.minute()
    );

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

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
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// v0.3.120: Handle natural language system queries (health, problems, status).
async fn handle_natural_system_query<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::natural_query::handle_natural_query;
    use super::streaming_helpers::{push_and_send, send_done};

    let Some(answer) = handle_natural_query(question) else {
        return Ok(None);
    };

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

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
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// v0.3.125: Handle package history queries.
async fn handle_package_history_query<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::package_history::PackageHistory;
    use super::streaming_helpers::{push_and_send, send_done};

    let q = question.to_lowercase();

    // Check if this is a package history query
    let is_package_query = (q.contains("package") || q.contains("installed") || q.contains("install"))
        && (q.contains("last") || q.contains("recent") || q.contains("history") || q.contains("show"));

    if !is_package_query {
        return Ok(None);
    }

    info!("Package history query detected");

    let history = PackageHistory::load();

    // Determine time range
    let days = if q.contains("week") {
        7
    } else if q.contains("month") || q.contains("30 days") {
        30
    } else if q.contains("6 month") || q.contains("180 days") {
        180
    } else if q.contains("year") {
        365
    } else {
        90 // Default to 3 months
    };

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart,
        format!("Analyzing package history (last {} days)", days), gate).await?;

    // Generate response
    let installations = history.installations_by_period(days);
    let total = installations.iter().map(|(_, count)| count).sum::<usize>();

    let mut answer = format!("Package installation summary (last {} days):\n\n", days);
    answer.push_str(&format!("Total installations: {}\n\n", total));

    if total > 0 {
        // Show chart
        let chart = history.chart_installations(days);
        answer.push_str(&chart);
        answer.push_str("\n\n");

        // Show most installed
        let top_packages = history.most_installed(5);
        if !top_packages.is_empty() {
            answer.push_str("Most installed packages:\n");
            for (pkg, count) in top_packages {
                answer.push_str(&format!("  {} ({} times)\n", pkg, count));
            }
            answer.push('\n');
        }

        // Show recent
        let recent = history.recent_installations(7, 10);
        if !recent.is_empty() {
            answer.push_str("Recent installations (last 7 days):\n");
            for event in recent {
                let date = chrono::DateTime::parse_from_rfc3339(&event.timestamp)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                answer.push_str(&format!("  {} - {} ({})\n", date, event.package, event.version));
            }
        }
    } else {
        answer.push_str("No package installations found in this period.\n");
    }

    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

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
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// v0.3.123: Handle well-known error patterns with instant, high-confidence answers.
async fn handle_pattern_match<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::patterns::{match_error_pattern, format_pattern_answer};
    use super::streaming_helpers::{push_and_send, send_done};

    let Some(pattern) = match_error_pattern(question) else {
        return Ok(None);
    };

    info!("Pattern match: {} (confidence={:.2})", pattern.pattern_id, pattern.confidence);

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;

    // Show that we recognized the pattern
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart,
        format!("Recognized common issue: {}", pattern.pattern_id), gate).await?;

    let answer = format_pattern_answer(&pattern);
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

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
        final_confidence: Some(pattern.confidence),
    };
    send_done(writer, &result).await?;

    Ok(Some(result))
}

/// v0.3.121: Handle multi-domain questions with parallel agent investigation.
async fn handle_multi_agent_query<W: tokio::io::AsyncWriteExt + Unpin>(
    question: &str,
    writer: &mut W,
    gate: &ExposureGate,
) -> Result<Option<AskResult>> {
    use anna_shared::config::AnnaConfig;
    use crate::orchestrator::{should_use_multi_agent, TaskAnalysis};
    use super::streaming_helpers::{push_and_send, send_done};

    let config = AnnaConfig::load().unwrap_or_default();

    // Only proceed if multi-agent mode is enabled and this is a multi-domain question
    if !should_use_multi_agent(question, &config) {
        return Ok(None);
    }

    let analysis = TaskAnalysis::analyze(question, &config);
    info!("Multi-agent mode: {} domains detected, using parallel investigation",
          analysis.domains.len());

    // Build orchestrator and solve
    use crate::agents::build_default_registry;
    use crate::orchestrator::AgentOrchestrator;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let registry = Arc::new(RwLock::new(build_default_registry()));
    let orchestrator = AgentOrchestrator::with_defaults(registry);
    let result = orchestrator.solve(question).await;

    let mut dialogue = Vec::new();
    push_and_send(writer, &mut dialogue, StepType::UserQuestion, question.to_string(), gate).await?;

    // Report parallel investigation
    let domain_list = analysis.domains.join(", ");
    push_and_send(writer, &mut dialogue, StepType::InvestigationStart,
        format!("Parallel investigation across: {}", domain_list), gate).await?;

    let answer = result.answer.unwrap_or_else(|| "Could not complete investigation.".to_string());
    let confidence = result.confidence;

    push_and_send(writer, &mut dialogue, StepType::InvestigationComplete,
        format!("{} agents contributed", analysis.domains.len()), gate).await?;
    push_and_send(writer, &mut dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

    let ask_result = AskResult {
        answer,
        success: result.success,
        iterations: 1,
        commands_executed: vec![],
        dialogue,
        needs_clarification: confidence < 0.3,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(confidence),
    };
    send_done(writer, &ask_result).await?;

    Ok(Some(ask_result))
}

/// Run the full Ralph loop with streaming progress.
async fn run_full_loop_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
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
            info!("LLM detected config request, generating ActionPlan with wiki research");
            return handle_config_request_with_research(model, question, &wiki_research, writer, gate, &mut dialogue).await;
        }

        let commands = match next_action {
            NextAction::Commands(cmds) => cmds,
            NextAction::None | NextAction::Config => Vec::new(),
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
        build_final_answer(&verification.answer, &verification.evidence_line, None);
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

/// Finish successful streaming loop with final answer.
#[allow(clippy::too_many_arguments)]
async fn finish_streaming<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W, dialogue: &mut Vec<DialogueStep>, ticket: &mut department::Ticket,
    state: &IterationState, answer: &str, question: &str, iteration: u32,
    probe_count: usize, experiment_count: usize, assigned_spec_name: &Option<String>,
    confidence: f32, gate: &ExposureGate,
) -> Result<AskResult> {
    // End investigation mode
    push_and_send(writer, dialogue, StepType::InvestigationComplete,
        format!("{} probes, {} experiments run", probe_count, experiment_count), gate).await?;

    // Specialist reports completion
    if let Some(ref spec_name) = assigned_spec_name {
        push_and_send(writer, dialogue, StepType::TeamDialogue,
            format!("{} -> Anna: I've got the answer.", spec_name), gate).await?;
    }

    // Verify answer through ClaimGate
    let evidence: Vec<(String, String, i32)> = state
        .commands
        .iter()
        .zip(state.outputs.iter())
        .map(|(cmd, out)| (cmd.clone(), out.clone(), 0))
        .collect();
    let config = anna_shared::config::AnnaConfig::load().ok();
    let debug_mode = config.as_ref().map(|c| c.debug_mode).unwrap_or(false);
    let teaching_mode = config.as_ref().map(|c| c.teaching_mode).unwrap_or(false);
    let verification = verify_answer(answer, question, &evidence, debug_mode);

    // Build teaching context
    let teaching_ctx = build_teaching_context(
        question,
        &state.commands,
        &state.outputs,
        experiment_count > 0,
        &verification.doc_citations,
    );

    // Generate teaching explanation if enabled
    let teaching_block = if teaching_mode {
        let explanation = teaching::generate_teaching(&teaching_ctx);
        teaching::format_teaching_block(&explanation, true)
    } else {
        None
    };

    // Build and send final answer (v0.3.113: with confidence indicator)
    let final_answer = build_final_answer_with_confidence(
        &verification.answer,
        &verification.evidence_line,
        teaching_block,
        Some(confidence),
    );
    push_and_send(writer, dialogue, StepType::FinalAnswer, final_answer.clone(), gate).await?;

    // Learn recipe and update ticket
    learn_recipe_from_answer(question, &state.commands, confidence);

    // v0.3.105: Also learn to Memory for semantic retrieval
    if confidence >= 0.7 && !state.commands.is_empty() {
        if let Ok(mut memory) = anna_shared::memory::Memory::load() {
            memory.learn(
                question,
                state.commands.clone(),
                &final_answer,
                anna_shared::memory::ExperienceContext::default(),
            );
            if let Err(e) = memory.save() {
                debug!("Failed to save memory: {}", e);
            } else {
                debug!("Learned experience to memory (confidence={:.2})", confidence);
            }
        }
    }

    let mut updated_ticket = ticket.clone();
    updated_ticket.resolve(&final_answer, 10);
    department::update_ticket(&updated_ticket);

    // v0.3.108: Generate proactive suggestions
    let suggestions = super::suggestions::generate_suggestions(question, &final_answer, &state.commands);
    let answer_with_suggestions = if let Some(suggestion_text) = super::suggestions::format_suggestions(&suggestions) {
        format!("{}{}", final_answer, suggestion_text)
    } else {
        final_answer.clone()
    };

    // Send done
    let result = AskResult {
        answer: answer_with_suggestions,
        success: true,
        iterations: iteration,
        commands_executed: state.commands.clone(),
        dialogue: dialogue.clone(),
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(confidence),
    };
    send_done(writer, &result).await?;

    Ok(result)
}

/// Wrapper to include wiki research in get_next_action.
async fn get_next_action_with_research(
    model: &str,
    question: &str,
    state: &IterationState,
    wiki_research: &str,
) -> Result<NextAction> {
    // For now, just call the original - we can enhance this later
    // The wiki research will be used in handle_config_request instead
    get_next_action(model, question, state).await
}

/// Handle a config request by generating an ActionPlan via LLM with wiki research.
async fn handle_config_request_with_research<W: tokio::io::AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    wiki_research: &str,
    writer: &mut W,
    gate: &ExposureGate,
    dialogue: &mut Vec<DialogueStep>,
) -> Result<AskResult> {
    use crate::dynamic_plan::{PLAN_GENERATION_PROMPT, parse_llm_plan, assess_plan_risk, RiskLevel};
    use crate::ollama;

    push_and_send(writer, dialogue, StepType::InvestigationProbe,
        "Generating configuration plan from research...".to_string(), gate).await?;

    // Include wiki research in the prompt
    let research_context = if !wiki_research.is_empty() {
        format!("\n\n{}", wiki_research)
    } else {
        String::new()
    };

    let full_prompt = format!("{}{}{}", PLAN_GENERATION_PROMPT, research_context, question);
    let llm_response = with_heartbeat(writer, gate,
        ollama::chat_with_timeout(model, &full_prompt, 60)
    ).await?;

    let plan = match parse_llm_plan(&llm_response, question) {
        Some(p) => p,
        None => {
            let answer = format!(
                "I understand you want to configure something, but I'm not confident enough \
                 to generate the right commands for: \"{}\". Could you be more specific?",
                question
            );
            push_and_send(writer, dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;
            let result = AskResult {
                answer,
                success: false,
                iterations: 1,
                commands_executed: vec![],
                dialogue: dialogue.clone(),
                needs_clarification: true,
                clarification_question: Some("Could you be more specific about what to configure?".to_string()),
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(0.3),
            };
            send_done(writer, &result).await?;
            return Ok(result);
        }
    };

    let risk = assess_plan_risk(&plan);
    info!("Config plan risk: {:?}", risk);

    match risk {
        RiskLevel::Blocked => {
            let answer = "I cannot execute this request - it contains potentially destructive operations.".to_string();
            push_and_send(writer, dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;
            let result = AskResult {
                answer,
                success: false,
                iterations: 1,
                commands_executed: vec![],
                dialogue: dialogue.clone(),
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: true,
                final_confidence: None,
            };
            send_done(writer, &result).await?;
            Ok(result)
        }
        RiskLevel::Low => {
            // Low risk: execute immediately without confirmation
            info!("Low risk plan - executing immediately");
            push_and_send(writer, dialogue, StepType::InvestigationProbe,
                "Executing configuration changes...".to_string(), gate).await?;

            let exec_result = crate::plan_executor::execute_plan(&plan);
            let commands_run: Vec<String> = plan.steps.iter().map(|s| s.command.clone()).collect();

            let answer = if exec_result.success {
                format!("Done. {}", plan.summary)
            } else {
                let errors: Vec<String> = exec_result.step_results.iter()
                    .filter(|r| !r.success)
                    .filter_map(|r| r.error.clone())
                    .collect();
                format!("Failed: {}", errors.join(", "))
            };

            push_and_send(writer, dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;
            let result = AskResult {
                answer,
                success: exec_result.success,
                iterations: 1,
                commands_executed: commands_run,
                dialogue: dialogue.clone(),
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(0.95),
            };
            send_done(writer, &result).await?;
            Ok(result)
        }
        RiskLevel::High => {
            // High risk: ask for confirmation
            let plan_text = format_plan_for_display(&plan);
            let answer = format!("{}\n\nThis requires elevated privileges. Proceed? (yes/no)", plan_text);
            push_and_send(writer, dialogue, StepType::FinalAnswer, answer.clone(), gate).await?;

            // Store plan for confirmation flow
            crate::plan_executor::set_pending_plan("default", plan);

            let result = AskResult {
                answer,
                success: true,
                iterations: 1,
                commands_executed: vec![],
                dialogue: dialogue.clone(),
                needs_clarification: true,
                clarification_question: Some("pending_plan".to_string()),
                cached: false,
                citations: vec![],
                abstained: false,
                final_confidence: Some(0.9),
            };
            send_done(writer, &result).await?;
            Ok(result)
        }
    }
}

fn format_plan_for_display(plan: &anna_shared::action_plan::ActionPlan) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Plan: {}", plan.summary));
    lines.push(String::new());
    lines.push("Commands to execute:".to_string());
    for (i, step) in plan.steps.iter().enumerate() {
        let privilege = if step.needs_sudo { " (requires root)" } else { "" };
        // Show the actual command first, then description
        lines.push(format!("  {}. {}{}", i + 1, step.command, privilege));
        if !step.description.is_empty() && step.description != step.command {
            lines.push(format!("     -> {}", step.description));
        }
    }
    lines.join("\n")
}

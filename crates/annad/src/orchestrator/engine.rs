//! Answer Engine v0.78.0
//!
//! v0.78.0: Senior JSON Fix - minimal prompt, robust parsing, fallback scoring
//!   Previously Senior ignored schema and we scored 0 even for correct answers.
//! v0.77.0: Dialog View - LLM prompts/responses streamed to annactl in real-time
//! v0.76.0: Minimal Junior Planner - radically reduced prompt for 4B model performance
//!
//! The main orchestration loop:
//! LLM-A (plan) -> Probes -> LLM-A (answer) -> LLM-B (audit) -> approve/fix/retry
//!
//! Key v0.17.0 changes:
//! - Use Senior's synthesized answer (text field) instead of Junior's draft
//! - Senior can now provide a direct answer that gets displayed to user
//! - Priority: Senior text > Senior fixed_answer > Junior draft_answer
//! - Fixes issue where useless "I will run probe..." answers were shown
//!
//! Key v0.71.0 changes:
//! - Fast path for simple probe-only questions (cpu, ram) - bypasses LLM for speed
//! - Overall orchestration timeout (10s for fast path, 60s for complex questions)
//! - Better debug output with probe lists and difficulty classification

use super::fallback;
use super::llm_client::OllamaClient;
use super::probe_executor;
use anna_common::{
    generate_llm_b_prompt, AuditScores, AuditVerdict,
    ConfidenceLevel, DebugIteration, DebugTrace, FinalAnswer, LoopState, ProbeCatalog,
    ProbeEvidenceV10, QuestionClassifier, QuestionDomain, ReliabilityScores, MAX_LOOPS,
    // v0.75.0: Complete Debug Output Contract
    DebugBlock, trace_is_debug_mode,
    // v0.76.0: Minimal Junior Planner
    generate_junior_prompt_v76,
};
use anyhow::Result;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Check if debug mode is enabled
fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
}

/// v0.76.0: Print debug step with clear labeling - FULL OUTPUT, NO TRUNCATION
fn debug_print(actor: &str, action: &str, content: &str) {
    if !is_debug_mode() {
        return;
    }
    use std::io::Write;
    let mut stderr = std::io::stderr();
    let separator = "=".repeat(78);
    let _ = writeln!(stderr, "\n{}", separator);
    let _ = writeln!(stderr, "[{}] {} ({} chars)", actor, action, content.len());
    let _ = writeln!(stderr, "{}", separator);
    // v0.76.0: FULL OUTPUT - NO TRUNCATION
    let _ = writeln!(stderr, "{}", content);
    let _ = writeln!(stderr, "{}", separator);
    let _ = stderr.flush();
}

/// Print iteration header for debug mode
fn print_iteration_header(iteration: usize, junior_model: &str, senior_model: &str) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    let _ = writeln!(stderr);
    let _ = writeln!(
        stderr,
        "████████████████████████████████████████████████████████████████████████████████"
    );
    let _ = writeln!(
        stderr,
        "██  ITERATION {}/{}  ██  Junior: {}  ██  Senior: {}  ██",
        iteration, MAX_LOOPS, junior_model, senior_model
    );
    let _ = writeln!(
        stderr,
        "████████████████████████████████████████████████████████████████████████████████"
    );
    let _ = stderr.flush();
}

/// Print probe execution info for debug mode
fn print_probes_executed(probes: &[String]) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    let _ = writeln!(stderr);
    let _ = writeln!(
        stderr,
        "┌─ [P]  PROBES EXECUTED ─────────────────────────────────────────────────────────┐"
    );
    for probe in probes {
        let _ = writeln!(stderr, "│  • {}", probe);
    }
    let _ = writeln!(
        stderr,
        "└────────────────────────────────────────────────────────────────────────────────┘"
    );
    let _ = stderr.flush();
}

/// Print verdict summary for debug mode
fn print_verdict_summary(verdict: &AuditVerdict, confidence: f64) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    let verdict_str = match verdict {
        AuditVerdict::Approve => "[+]  APPROVE",
        AuditVerdict::FixAndAccept => "[~]  FIX_AND_ACCEPT",
        AuditVerdict::NeedsMoreProbes => "[~]  NEEDS_MORE_PROBES",
        AuditVerdict::Refuse => "[-]  REFUSE",
    };

    let _ = writeln!(stderr);
    let _ = writeln!(
        stderr,
        "╔══════════════════════════════════════════════════════════════════════════════╗"
    );
    let _ = writeln!(
        stderr,
        "║  VERDICT: {}  │  Confidence: {:.0}%",
        verdict_str,
        confidence * 100.0
    );
    let _ = writeln!(
        stderr,
        "╚══════════════════════════════════════════════════════════════════════════════╝"
    );
    let _ = stderr.flush();
}

/// Answer engine - orchestrates the LLM-A/LLM-B loop
///
/// Supports role-specific models:
/// - Junior (LLM-A): Fast model for probe execution and command parsing
/// - Senior (LLM-B): Smarter model for reasoning and synthesis
pub struct AnswerEngine {
    llm_client: OllamaClient,
    catalog: ProbeCatalog,
}

impl AnswerEngine {
    /// Create engine with a single model for both roles (legacy/backwards compat)
    pub fn new(model: Option<String>) -> Self {
        Self {
            llm_client: OllamaClient::new(model),
            catalog: ProbeCatalog::standard(),
        }
    }

    /// Create engine with separate models for junior and senior roles
    pub fn with_role_models(junior_model: Option<String>, senior_model: Option<String>) -> Self {
        Self {
            llm_client: OllamaClient::with_role_models(junior_model, senior_model),
            catalog: ProbeCatalog::standard(),
        }
    }

    /// Get the junior (LLM-A) model name
    pub fn junior_model(&self) -> &str {
        self.llm_client.junior_model()
    }

    /// Get the senior (LLM-B) model name
    pub fn senior_model(&self) -> &str {
        self.llm_client.senior_model()
    }

    /// Get the model name being used (legacy - returns junior model)
    pub fn model(&self) -> &str {
        self.llm_client.junior_model()
    }

    /// Filter probe IDs to only valid ones from catalog
    fn filter_valid_probes(&self, probe_ids: &[String]) -> Vec<String> {
        probe_ids
            .iter()
            .filter(|id| {
                let valid = self.catalog.is_valid(id);
                if !valid {
                    warn!("Rejecting invalid probe_id: {} (not in catalog)", id);
                }
                valid
            })
            .cloned()
            .collect()
    }

    /// Process a user question and return the final answer
    pub async fn process_question(&self, question: &str) -> Result<FinalAnswer> {
        self.process_question_with_optional_emitter(question, None).await
    }

    /// v0.77.0: Internal processing with optional emitter for streaming LLM dialog
    async fn process_question_with_optional_emitter(
        &self,
        question: &str,
        emitter: Option<&dyn anna_common::DebugEventEmitter>,
    ) -> Result<FinalAnswer> {
        info!("Processing question: {}", question);
        let start_time = Instant::now();

        // v0.75.1: Show what Anna received
        debug_print("ANNA", "RECEIVED QUESTION", question);

        // v0.75.0: Initialize complete debug block
        let mut debug_block = DebugBlock::new(question, self.junior_model(), self.senior_model());

        // v0.29.0: Fast-path rejection for obviously unsupported questions
        // This avoids 100+ second LLM calls for questions like "what's the weather?"
        let classifier = QuestionClassifier::new();
        if let QuestionDomain::Unsupported { reason } = classifier.classify(question) {
            info!("[!]  Fast-path rejection: {}", reason);
            return Ok(FinalAnswer {
                question: question.to_string(),
                answer: format!(
                    "I cannot answer this question.\n\n\
                     Reason: {}\n\n\
                     My available probes are:\n  \
                     - cpu.info (CPU model, cores, threads)\n  \
                     - mem.info (memory usage)\n  \
                     - disk.lsblk (partitions, filesystems)\n  \
                     - hardware.gpu (GPU detection)\n  \
                     - drivers.gpu (GPU drivers)\n  \
                     - hardware.ram (RAM capacity)",
                    reason
                ),
                is_refusal: true,
                citations: vec![],
                scores: AuditScores::new(0.0, 0.0, 0.0),
                confidence_level: ConfidenceLevel::Red,
                problems: vec![reason],
                loop_iterations: 0,
                model_used: None,
                clarification_needed: None,
                debug_trace: Some(DebugTrace {
                    junior_model: self.junior_model().to_string(),
                    senior_model: self.senior_model().to_string(),
                    duration_secs: start_time.elapsed().as_secs_f64(),
                    iterations: vec![],
                }),
            });
        }

        // v0.73.0: Fast path DISABLED for Engineering Reset testing
        // The fast path bypasses Junior→Senior loop and rubber-stamps 95% scores
        // TODO: Re-enable after fixing fast path answer formatting
        // if let Some((probe_id, topic)) = self.try_fast_path(question) {
        //     match self.execute_fast_path(question, probe_id, topic, start_time).await {
        //         Ok(answer) => return Ok(answer),
        //         Err(e) => {
        //             // Fast path failed, fall through to normal orchestration
        //             warn!("[!]  Fast path failed ({}), using normal orchestration", e);
        //         }
        //     }
        // }
        info!("[*]  Fast path disabled - using full Junior→Senior orchestration");

        let mut loop_state = LoopState::default();
        let mut evidence: Vec<ProbeEvidenceV10> = vec![];
        let mut last_draft_answer: Option<String> = None;
        let mut last_scores: Option<AuditScores> = None;

        // Debug trace to capture full LLM dialog
        let mut debug_trace = DebugTrace {
            junior_model: self.junior_model().to_string(),
            senior_model: self.senior_model().to_string(),
            ..Default::default()
        };

        // Main orchestration loop
        while loop_state.can_continue() {
            loop_state.next_iteration();
            info!("Loop iteration {}/{}", loop_state.iteration, MAX_LOOPS);

            // v0.16.4: Print iteration header in debug mode
            if is_debug_mode() {
                print_iteration_header(
                    loop_state.iteration,
                    self.junior_model(),
                    self.senior_model(),
                );
            }

            // Initialize debug iteration
            let mut debug_iter = DebugIteration {
                iteration: loop_state.iteration,
                ..Default::default()
            };

            // Step 1: Call LLM-A with v0.76.0 minimal prompt
            let llm_a_prompt = generate_junior_prompt_v76(
                question,
                &self.catalog.available_probes(),
                &evidence,
                loop_state.iteration,
            );

            // v0.75.1: Show what Anna sends to Junior
            debug_print("ANNA", &format!("SENDING TO JUNIOR ({})", self.junior_model()), &llm_a_prompt);

            // Capture prompt for debug (truncated for very large prompts)
            debug_iter.llm_a_prompt = truncate_for_debug(&llm_a_prompt, 8000);

            // v0.77.0: Use emitter-enabled call if available (streams to annactl)
            let (llm_a_response, llm_a_raw) = if let Some(em) = emitter {
                self.llm_client.call_llm_a_with_emitter(&llm_a_prompt, loop_state.iteration, em).await?
            } else {
                self.llm_client.call_llm_a(&llm_a_prompt).await?
            };

            // v0.75.1: Show what Junior returned (raw)
            debug_print("JUNIOR", "RAW RESPONSE", &llm_a_raw);

            // Capture raw response for debug
            debug_iter.llm_a_response = llm_a_raw;
            debug_iter.llm_a_intent = llm_a_response.plan.intent.clone();
            debug_iter.llm_a_probes = llm_a_response
                .plan
                .probe_requests
                .iter()
                .map(|p| p.probe_id.clone())
                .collect();
            debug_iter.llm_a_has_draft = llm_a_response.draft_answer.is_some();

            info!(
                "[A]  LLM-A parsed: intent={}, probes={}, draft={}, needs_more={}, refuse={}",
                llm_a_response.plan.intent,
                llm_a_response.plan.probe_requests.len(),
                llm_a_response.draft_answer.is_some(),
                llm_a_response.needs_more_probes,
                llm_a_response.refuse_to_answer
            );

            // v0.75.1: Show what Anna parsed from Junior
            let parsed_summary = format!(
                "Intent: {}\n\
                 Probes requested: {:?}\n\
                 Has draft answer: {}\n\
                 Needs more probes: {}\n\
                 Refuses to answer: {}\n\
                 Draft: {}",
                llm_a_response.plan.intent,
                llm_a_response.plan.probe_requests.iter().map(|p| &p.probe_id).collect::<Vec<_>>(),
                llm_a_response.draft_answer.is_some(),
                llm_a_response.needs_more_probes,
                llm_a_response.refuse_to_answer,
                llm_a_response.draft_answer.as_ref().map(|d| d.text.chars().take(500).collect::<String>()).unwrap_or_else(|| "(none)".to_string())
            );
            debug_print("ANNA", "PARSED FROM JUNIOR", &parsed_summary);

            if let Some(ref draft) = llm_a_response.draft_answer {
                info!(
                    "[D]  Draft answer: {}",
                    &draft.text[..200.min(draft.text.len())]
                );
            }

            // v0.75.0: Capture Junior's plan in debug block
            let requested_probes: Vec<String> = llm_a_response.plan.probe_requests
                .iter()
                .map(|p| p.probe_id.clone())
                .collect();
            // Use draft answer text as reasoning if available
            let junior_reasoning = llm_a_response.draft_answer
                .as_ref()
                .map(|d| d.text.chars().take(200).collect::<String>())
                .unwrap_or_default();
            debug_block.set_junior_plan(
                &llm_a_response.plan.intent,
                requested_probes,
                &debug_iter.llm_a_response, // Use the already-stored raw response
                &junior_reasoning,
                llm_a_response.self_scores.as_ref().map(|s| s.overall()).unwrap_or(0.0),
            );
            // Capture raw Junior message for forensic log
            debug_block.raw_messages.junior_prompt = debug_iter.llm_a_prompt.clone();
            debug_block.raw_messages.junior_response = debug_iter.llm_a_response.clone();

            // Check for immediate refusal
            if llm_a_response.refuse_to_answer {
                // Only refuse if no evidence at all - otherwise try partial answer
                if evidence.is_empty() {
                    loop_state.mark_refused();
                    debug_trace.iterations.push(debug_iter);
                    debug_trace.duration_secs = start_time.elapsed().as_secs_f64();
                    return Ok(self.build_refusal_with_trace(
                        question,
                        llm_a_response
                            .refusal_reason
                            .as_deref()
                            .unwrap_or("Unable to answer"),
                        &evidence,
                        loop_state.iteration,
                        debug_trace,
                    ));
                }
                // If we have evidence, try to provide partial answer
                warn!("LLM-A wants to refuse but we have evidence - trying partial answer");
            }

            // Step 2: Execute any requested probes (validated)
            if llm_a_response.needs_more_probes || !llm_a_response.plan.probe_requests.is_empty() {
                let probe_ids: Vec<String> = llm_a_response
                    .plan
                    .probe_requests
                    .iter()
                    .map(|p| p.probe_id.clone())
                    .collect();

                // Filter to only valid probe IDs
                let valid_probes = self.filter_valid_probes(&probe_ids);

                if !valid_probes.is_empty() {
                    info!(
                        "Executing {} valid probes (rejected {})",
                        valid_probes.len(),
                        probe_ids.len() - valid_probes.len()
                    );
                    // Capture executed probes for debug
                    debug_iter.probes_executed = valid_probes.clone();

                    // v0.16.4: Show probes being executed in debug mode
                    if is_debug_mode() {
                        print_probes_executed(&valid_probes);
                    }

                    let probe_start = Instant::now();
                    let new_evidence =
                        probe_executor::execute_probes(&self.catalog, &valid_probes).await;
                    let probe_duration = probe_start.elapsed().as_millis() as u64;

                    // v0.75.0: Add probe executions to debug block
                    for ev in &new_evidence {
                        let per_probe_duration = probe_duration / new_evidence.len().max(1) as u64;
                        if ev.status.is_ok() {
                            let summary = ev.raw.as_deref().unwrap_or("");
                            debug_block.add_probe_execution(
                                &ev.probe_id,
                                &ev.command,
                                per_probe_duration,
                                summary,
                            );
                        } else {
                            debug_block.add_probe_failure(&ev.probe_id, ev.status.as_str());
                        }
                    }

                    // v0.75.1: Show probe results
                    let probe_results_summary = new_evidence.iter()
                        .map(|ev| {
                            let status = if ev.status.is_ok() { "OK" } else { "FAIL" };
                            let output_preview = ev.raw.as_deref()
                                .unwrap_or("(no output)")
                                .chars().take(500).collect::<String>();
                            format!("  [{}] {}\n    Command: {}\n    Output: {}",
                                status, ev.probe_id, ev.command, output_preview)
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    debug_print("PROBES", "EXECUTION RESULTS", &probe_results_summary);

                    evidence.extend(new_evidence);
                }

                // If needs more probes, continue to next iteration
                if llm_a_response.needs_more_probes && llm_a_response.draft_answer.is_none() {
                    debug_trace.iterations.push(debug_iter);
                    continue;
                }
            }

            // Step 3: Get draft answer
            let draft_answer = match llm_a_response.draft_answer {
                Some(draft) => draft,
                None => {
                    warn!("LLM-A did not provide draft answer");
                    debug_trace.iterations.push(debug_iter);
                    continue;
                }
            };

            // Store for potential partial answer
            last_draft_answer = Some(draft_answer.text.clone());

            // v0.73.0: CRITICAL - Must have evidence before sending to Senior
            // An answer without probe evidence is worthless - force another iteration
            if evidence.is_empty() {
                warn!("Draft answer has no evidence - cannot send to Senior without probes");
                debug_trace.iterations.push(debug_iter);
                // Loop continues - LLM-A must request probes in next iteration
                continue;
            }

            // v0.73.0: If Junior didn't self-score, use 0 (not 50%) to be honest with Senior
            let self_scores = llm_a_response
                .self_scores
                .unwrap_or_else(|| ReliabilityScores::new(0.0, 0.0, 0.0));

            // Step 4: Call LLM-B to audit
            let llm_b_prompt =
                generate_llm_b_prompt(question, &draft_answer, &evidence, &self_scores);

            // v0.75.1: Show what Anna sends to Senior
            debug_print("ANNA", &format!("SENDING TO SENIOR ({})", self.senior_model()), &llm_b_prompt);

            // Capture LLM-B prompt for debug (truncated for very large prompts)
            debug_iter.llm_b_prompt = Some(truncate_for_debug(&llm_b_prompt, 8000));

            // v0.77.0: Use emitter-enabled call if available (streams to annactl)
            let (llm_b_response, llm_b_raw) = if let Some(em) = emitter {
                self.llm_client.call_llm_b_with_emitter(&llm_b_prompt, loop_state.iteration, em).await?
            } else {
                self.llm_client.call_llm_b(&llm_b_prompt).await?
            };

            // v0.75.1: Show what Senior returned (raw)
            debug_print("SENIOR", "RAW RESPONSE", &llm_b_raw);

            // Capture LLM-B response for debug
            debug_iter.llm_b_response = Some(llm_b_raw);
            debug_iter.llm_b_verdict = Some(llm_b_response.verdict.as_str().to_string());
            debug_iter.llm_b_confidence = Some(llm_b_response.scores.overall);

            info!(
                "[=]  LLM-B parsed: verdict={:?}, score={:.2}, problems={}",
                llm_b_response.verdict,
                llm_b_response.scores.overall,
                llm_b_response.problems.len()
            );
            if let Some(ref fix) = llm_b_response.fixed_answer {
                info!("[~]  Fixed answer: {}", &fix[..200.min(fix.len())]);
            }

            // v0.75.1: Show what Anna parsed from Senior
            let senior_parsed_summary = format!(
                "Verdict: {:?}\n\
                 Scores:\n\
                   - Evidence: {:.2}\n\
                   - Reasoning: {:.2}\n\
                   - Coverage: {:.2}\n\
                   - Overall: {:.2}\n\
                 Problems: {:?}\n\
                 Fixed Answer: {}\n\
                 Text Override: {}",
                llm_b_response.verdict,
                llm_b_response.scores.evidence,
                llm_b_response.scores.reasoning,
                llm_b_response.scores.coverage,
                llm_b_response.scores.overall,
                llm_b_response.problems,
                llm_b_response.fixed_answer.as_deref().unwrap_or("(none)"),
                llm_b_response.text.as_deref().unwrap_or("(none)")
            );
            debug_print("ANNA", "PARSED FROM SENIOR", &senior_parsed_summary);

            // v0.75.0: Capture Senior verdict in debug block
            let citations: Vec<String> = evidence.iter().map(|e| e.probe_id.clone()).collect();
            debug_block.set_senior_verdict(
                llm_b_response.verdict.as_str(),
                &llm_b_response.problems.join("; "),
                llm_b_response.fixed_answer.clone(),
                citations,
                llm_b_response.scores.evidence,
                llm_b_response.scores.reasoning,
                llm_b_response.scores.coverage,
            );
            debug_block.input.iterations_used = loop_state.iteration;
            // Capture raw Senior message for forensic log
            debug_block.raw_messages.senior_prompt = debug_iter.llm_b_prompt.clone().unwrap_or_default();
            debug_block.raw_messages.senior_response = debug_iter.llm_b_response.clone().unwrap_or_default();

            // v0.16.4: Show verdict summary in debug mode
            if is_debug_mode() {
                print_verdict_summary(&llm_b_response.verdict, llm_b_response.scores.overall);
            }

            loop_state.record_score(llm_b_response.scores.overall);
            last_scores = Some(llm_b_response.scores.clone());

            // Step 5: Handle verdict
            // v0.17.0: Prefer Senior's answer (text/fixed_answer) over Junior's draft
            match llm_b_response.verdict {
                AuditVerdict::Approve => {
                    loop_state.mark_approved();
                    // v0.17.0: Use Senior's text if provided, otherwise fall back to draft
                    let answer_text = llm_b_response
                        .text
                        .as_deref()
                        .or(llm_b_response.fixed_answer.as_deref())
                        .unwrap_or(&draft_answer.text);

                    // v0.75.1: Show Anna's final decision
                    let decision_summary = format!(
                        "Decision: APPROVE\n\
                         Confidence: {:.0}%\n\
                         Answer Source: {}\n\
                         Final Answer:\n{}",
                        llm_b_response.scores.overall * 100.0,
                        if llm_b_response.text.is_some() { "Senior text override" }
                        else if llm_b_response.fixed_answer.is_some() { "Senior fixed_answer" }
                        else { "Junior draft" },
                        answer_text
                    );
                    debug_print("ANNA", "FINAL DECISION", &decision_summary);

                    debug_trace.iterations.push(debug_iter);
                    debug_trace.duration_secs = start_time.elapsed().as_secs_f64();

                    // v0.75.0: Finalize debug block and emit
                    debug_block.set_final_answer(answer_text, llm_b_response.scores.overall);
                    let _ = debug_block.write_all_logs();
                    if trace_is_debug_mode() {
                        eprintln!("{}", debug_block.format_cli());
                    }

                    return Ok(self.build_final_answer_with_trace(
                        question,
                        answer_text,
                        evidence,
                        llm_b_response.scores,
                        loop_state.iteration,
                        debug_trace,
                    ));
                }
                AuditVerdict::FixAndAccept => {
                    // LLM-B provided a fixed answer
                    loop_state.mark_approved();
                    // v0.17.0: Prefer text > fixed_answer > draft
                    let answer_text = llm_b_response
                        .text
                        .as_deref()
                        .or(llm_b_response.fixed_answer.as_deref())
                        .unwrap_or(&draft_answer.text);

                    // v0.75.1: Show Anna's final decision
                    let decision_summary = format!(
                        "Decision: FIX_AND_ACCEPT\n\
                         Confidence: {:.0}%\n\
                         Answer Source: {}\n\
                         Final Answer:\n{}",
                        llm_b_response.scores.overall * 100.0,
                        if llm_b_response.text.is_some() { "Senior text override" }
                        else if llm_b_response.fixed_answer.is_some() { "Senior fixed_answer" }
                        else { "Junior draft" },
                        answer_text
                    );
                    debug_print("ANNA", "FINAL DECISION", &decision_summary);

                    debug_trace.iterations.push(debug_iter);
                    debug_trace.duration_secs = start_time.elapsed().as_secs_f64();

                    // v0.75.0: Finalize debug block and emit
                    debug_block.set_final_answer(answer_text, llm_b_response.scores.overall);
                    let _ = debug_block.write_all_logs();
                    if trace_is_debug_mode() {
                        eprintln!("{}", debug_block.format_cli());
                    }

                    return Ok(self.build_final_answer_with_trace(
                        question,
                        answer_text,
                        evidence,
                        llm_b_response.scores,
                        loop_state.iteration,
                        debug_trace,
                    ));
                }
                AuditVerdict::Refuse => {
                    // Only refuse if we have no evidence and no draft
                    if evidence.is_empty() {
                        loop_state.mark_refused();
                        let reason = llm_b_response
                            .problems
                            .first()
                            .map(|s| s.as_str())
                            .unwrap_or("Auditor determined answer cannot be safely provided");

                        // v0.75.1: Show Anna's final decision
                        let decision_summary = format!(
                            "Decision: REFUSE (no evidence)\n\
                             Reason: {}",
                            reason
                        );
                        debug_print("ANNA", "FINAL DECISION", &decision_summary);

                        debug_trace.iterations.push(debug_iter);
                        debug_trace.duration_secs = start_time.elapsed().as_secs_f64();
                        return Ok(self.build_refusal_with_trace(
                            question,
                            reason,
                            &evidence,
                            loop_state.iteration,
                            debug_trace,
                        ));
                    }
                    // v0.73.0: LLM-B refused - we must honor that decision, not rubber-stamp
                    // If we have evidence but Senior refused, the scores must be 0 (unverified)
                    warn!("LLM-B refused - scores set to 0 (no rubber-stamping)");

                    // v0.75.1: Show Anna's decision to continue
                    let decision_summary = format!(
                        "Decision: REFUSE (but has evidence - will retry)\n\
                         Problems: {:?}\n\
                         Setting scores to 0 - will try again",
                        llm_b_response.problems
                    );
                    debug_print("ANNA", "ITERATION DECISION", &decision_summary);

                    last_scores = Some(AuditScores::new(0.0, 0.0, 0.0));
                    // Continue to see if we can improve
                    debug_trace.iterations.push(debug_iter);
                }
                AuditVerdict::NeedsMoreProbes => {
                    // Execute additional probes requested by auditor (validated)
                    let probe_ids: Vec<String> = llm_b_response
                        .probe_requests
                        .iter()
                        .map(|p| p.probe_id.clone())
                        .collect();

                    // v0.75.1: Show Anna's decision to gather more evidence
                    let decision_summary = format!(
                        "Decision: NEEDS_MORE_PROBES\n\
                         Senior requested: {:?}\n\
                         Problems: {:?}",
                        probe_ids,
                        llm_b_response.problems
                    );
                    debug_print("ANNA", "ITERATION DECISION", &decision_summary);

                    let valid_probes = self.filter_valid_probes(&probe_ids);

                    if !valid_probes.is_empty() {
                        info!(
                            "Auditor requested {} valid probes (rejected {})",
                            valid_probes.len(),
                            probe_ids.len() - valid_probes.len()
                        );
                        let new_evidence =
                            probe_executor::execute_probes(&self.catalog, &valid_probes).await;
                        evidence.extend(new_evidence);
                    }
                    // Save iteration and continue to next
                    debug_trace.iterations.push(debug_iter);
                }
            }
        }

        // Loop exhausted - provide partial answer instead of refusal
        loop_state.mark_exhausted();
        debug_trace.duration_secs = start_time.elapsed().as_secs_f64();

        // v0.73.0: CRITICAL - No answer without evidence
        // If we exhausted the loop without ever gathering evidence, refuse
        if evidence.is_empty() {
            warn!("Loop exhausted with no evidence - no probes were ever executed");
            return Ok(self.build_refusal_with_trace(
                question,
                "Unable to answer - no evidence was gathered (no probes executed)",
                &evidence,
                loop_state.iteration,
                debug_trace,
            ));
        }

        // If we have a draft answer, return it with honest low confidence
        if let Some(answer_text) = last_draft_answer {
            // v0.73.0: No more rubber-stamping - use actual scores or 0
            let scores = last_scores.unwrap_or_else(|| AuditScores::new(0.0, 0.0, 0.0));

            // v0.73.0: Reject answers with overall score 0
            if scores.overall < 0.01 {
                warn!("Draft answer has 0 score - refusing to deliver unverified answer");
                return Ok(self.build_refusal_with_trace(
                    question,
                    "Answer could not be verified (0% confidence)",
                    &evidence,
                    loop_state.iteration,
                    debug_trace,
                ));
            }

            info!(
                "Max loops reached - returning partial answer with confidence {:.2}",
                scores.overall
            );

            // v0.75.0: Finalize debug block and emit
            debug_block.set_final_answer(&answer_text, scores.overall);
            let _ = debug_block.write_all_logs();
            if trace_is_debug_mode() {
                eprintln!("{}", debug_block.format_cli());
            }

            return Ok(self.build_partial_answer_with_trace(
                question,
                &answer_text,
                evidence,
                scores,
                loop_state.iteration,
                debug_trace,
            ));
        }

        // No draft answer - try to extract basic facts from evidence as fallback
        // v0.73.0: Fallback answers are NOT Senior-reviewed, so they get 0 scores
        // This means they will be rejected by the 0-score check above - no rubber-stamping
        if !evidence.is_empty() {
            if let Some(_fallback) = fallback::extract_fallback_answer(question, &evidence) {
                warn!("Fallback answer extracted but not Senior-verified - cannot deliver");
                // Note: We could return the fallback with 0 scores, but it would be rejected.
                // Instead, fall through to the refusal path for clearer error messaging.
            }
        }

        // No draft answer at all - this is a true refusal
        Ok(self.build_refusal_with_trace(
            question,
            "Could not generate any answer after maximum iterations",
            &evidence,
            loop_state.iteration,
            debug_trace,
        ))
    }

    /// Build a final answer (approved) with debug trace
    fn build_final_answer_with_trace(
        &self,
        question: &str,
        answer_text: &str,
        evidence: Vec<ProbeEvidenceV10>,
        scores: AuditScores,
        loop_iterations: usize,
        debug_trace: DebugTrace,
    ) -> FinalAnswer {
        let confidence_level = ConfidenceLevel::from_score(scores.overall);

        FinalAnswer {
            question: question.to_string(),
            answer: answer_text.to_string(),
            is_refusal: false,
            citations: evidence,
            scores,
            confidence_level,
            problems: vec![],
            loop_iterations,
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: Some(debug_trace),
        }
    }

    /// Build a partial answer (max loops reached) with debug trace
    fn build_partial_answer_with_trace(
        &self,
        question: &str,
        answer_text: &str,
        evidence: Vec<ProbeEvidenceV10>,
        scores: AuditScores,
        loop_iterations: usize,
        debug_trace: DebugTrace,
    ) -> FinalAnswer {
        let confidence_level = ConfidenceLevel::from_score(scores.overall);
        let disclaimer = if confidence_level == ConfidenceLevel::Red {
            "\n\n[Note: This answer has limited verification. Confidence is low.]"
        } else {
            ""
        };

        FinalAnswer {
            question: question.to_string(),
            answer: format!("{}{}", answer_text, disclaimer),
            is_refusal: false,
            citations: evidence,
            scores,
            confidence_level,
            problems: vec!["Reached maximum verification loops".to_string()],
            loop_iterations,
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: Some(debug_trace),
        }
    }

    /// Build a refusal answer with debug trace
    fn build_refusal_with_trace(
        &self,
        question: &str,
        reason: &str,
        evidence: &[ProbeEvidenceV10],
        loop_iterations: usize,
        debug_trace: DebugTrace,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: format!("I cannot answer this question.\n\nReason: {}", reason),
            is_refusal: true,
            citations: evidence.to_vec(),
            scores: AuditScores::new(0.0, 0.0, 0.0),
            confidence_level: ConfidenceLevel::Red,
            problems: vec![reason.to_string()],
            loop_iterations,
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: Some(debug_trace),
        }
    }

    /// Build a final answer (approved) - legacy without trace
    fn build_final_answer(
        &self,
        question: &str,
        answer_text: &str,
        evidence: Vec<ProbeEvidenceV10>,
        scores: AuditScores,
        loop_iterations: usize,
    ) -> FinalAnswer {
        let confidence_level = ConfidenceLevel::from_score(scores.overall);

        FinalAnswer {
            question: question.to_string(),
            answer: answer_text.to_string(),
            is_refusal: false,
            citations: evidence,
            scores,
            confidence_level,
            problems: vec![],
            loop_iterations,
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: None,
        }
    }

    /// Build a partial answer - legacy without trace
    fn build_partial_answer(
        &self,
        question: &str,
        answer_text: &str,
        evidence: Vec<ProbeEvidenceV10>,
        scores: AuditScores,
        loop_iterations: usize,
    ) -> FinalAnswer {
        let confidence_level = ConfidenceLevel::from_score(scores.overall);
        let disclaimer = if confidence_level == ConfidenceLevel::Red {
            "\n\n[Note: This answer has limited verification. Confidence is low.]"
        } else {
            ""
        };

        FinalAnswer {
            question: question.to_string(),
            answer: format!("{}{}", answer_text, disclaimer),
            is_refusal: false,
            citations: evidence,
            scores,
            confidence_level,
            problems: vec!["Reached maximum verification loops".to_string()],
            loop_iterations,
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: None,
        }
    }

    /// Build a refusal answer - legacy without trace
    fn build_refusal(
        &self,
        question: &str,
        reason: &str,
        evidence: &[ProbeEvidenceV10],
        loop_iterations: usize,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: format!("I cannot answer this question.\n\nReason: {}", reason),
            is_refusal: true,
            citations: evidence.to_vec(),
            scores: AuditScores::new(0.0, 0.0, 0.0),
            confidence_level: ConfidenceLevel::Red,
            problems: vec![reason.to_string()],
            loop_iterations,
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: None,
        }
    }

    /// Check if LLM backend is available
    pub async fn is_available(&self) -> bool {
        self.llm_client.is_available().await
    }

    /// Get the catalog
    pub fn catalog(&self) -> &ProbeCatalog {
        &self.catalog
    }

    /// v0.71.0: Fast path for simple probe-only questions
    ///
    /// For questions like "What CPU do I have?" or "How much RAM?", we can:
    /// 1. Detect the simple pattern
    /// 2. Run the single appropriate probe
    /// 3. Format the answer directly from probe data
    /// 4. Return immediately without LLM calls
    ///
    /// This completes in <1 second vs 90+ seconds with full orchestration.
    fn try_fast_path(&self, question: &str) -> Option<(&'static str, &'static str)> {
        let q_lower = question.to_lowercase();

        // CPU questions
        if (q_lower.contains("cpu") || q_lower.contains("processor"))
           && (q_lower.contains("what") || q_lower.contains("how many")
               || q_lower.contains("which") || q_lower.contains("model")) {
            return Some(("cpu.info", "CPU"));
        }

        // RAM/Memory questions
        if (q_lower.contains("ram") || q_lower.contains("memory"))
           && (q_lower.contains("how much") || q_lower.contains("what")
               || q_lower.contains("free") || q_lower.contains("installed")) {
            return Some(("mem.info", "Memory"));
        }

        // Disk questions
        if (q_lower.contains("disk") || q_lower.contains("storage") || q_lower.contains("partition"))
           && (q_lower.contains("what") || q_lower.contains("how") || q_lower.contains("list")) {
            return Some(("disk.lsblk", "Disk"));
        }

        // v0.71.0: Annad logs questions
        if (q_lower.contains("annad") || q_lower.contains("anna"))
           && (q_lower.contains("log") || q_lower.contains("error") || q_lower.contains("warning")) {
            return Some(("logs.annad", "Logs"));
        }

        // v0.71.0: System updates questions
        if (q_lower.contains("update") || q_lower.contains("upgrade"))
           && (q_lower.contains("pending") || q_lower.contains("available")
               || q_lower.contains("check") || q_lower.contains("any")) {
            return Some(("updates.pending", "Updates"));
        }

        // v0.71.0: Self-diagnosis questions (use real health checks)
        if (q_lower.contains("health") || q_lower.contains("diagnose") || q_lower.contains("status"))
           && (q_lower.contains("yourself") || q_lower.contains("anna")
               || q_lower.contains("self") || q_lower.contains("your own")) {
            return Some(("self.health", "Self-Health"));
        }

        None
    }

    /// v0.71.0: Execute fast path for simple hardware questions
    /// Returns a FinalAnswer directly from probe data without LLM calls
    async fn execute_fast_path(
        &self,
        question: &str,
        probe_id: &str,
        topic: &str,
        start_time: Instant,
    ) -> Result<FinalAnswer> {
        info!("[!]  v0.71.0 Fast path: {} question -> {}", topic, probe_id);

        // Print debug info if enabled
        if is_debug_mode() {
            use std::io::Write;
            let mut stderr = std::io::stderr();
            let _ = writeln!(stderr);
            let _ = writeln!(stderr, "┌─ [*]  FAST PATH ENABLED ──────────────────────────────────────────────────┐");
            let _ = writeln!(stderr, "│  Topic: {}  │  Probe: {}", topic, probe_id);
            let _ = writeln!(stderr, "│  Skipping LLM orchestration for simple hardware query");
            let _ = writeln!(stderr, "└────────────────────────────────────────────────────────────────────────────┘");
            let _ = stderr.flush();
        }

        // v0.71.0: Special handling for self.health - use real health checks
        if probe_id == "self.health" {
            return self.execute_self_health_fast_path(question, start_time);
        }

        // Execute the single probe
        let evidence = probe_executor::execute_probes(&self.catalog, &[probe_id.to_string()]).await;

        if evidence.is_empty() {
            // Probe failed - fall back to normal path
            info!("[!]  Fast path probe failed, falling back to normal");
            return Err(anyhow::anyhow!("Fast path probe failed"));
        }

        // Format answer from probe evidence
        let probe_output = evidence.first()
            .and_then(|e| e.raw.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");
        let answer = self.format_fast_path_answer(topic, probe_id, probe_output);

        let debug_trace = DebugTrace {
            junior_model: "fast_path".to_string(),
            senior_model: "fast_path".to_string(),
            duration_secs: start_time.elapsed().as_secs_f64(),
            iterations: vec![DebugIteration {
                iteration: 1,
                llm_a_intent: format!("fast_path_{}", topic.to_lowercase()),
                llm_a_probes: vec![probe_id.to_string()],
                probes_executed: vec![probe_id.to_string()],
                llm_b_verdict: Some("fast_path_approve".to_string()),
                llm_b_confidence: Some(0.95),
                ..Default::default()
            }],
        };

        // v0.73.0 WARNING: Fast path is UNTRUSTED - no Senior review!
        // These scores are fabricated. After core pipeline is fixed and tested,
        // fast path should either: (a) route through Senior, or (b) use lower scores
        // For now, marking with a problem note to indicate lack of verification.
        Ok(FinalAnswer {
            question: question.to_string(),
            answer,
            is_refusal: false,
            citations: evidence,
            scores: AuditScores::new(0.95, 0.95, 0.95), // WARNING: Fabricated - no Senior review
            confidence_level: ConfidenceLevel::Green,
            problems: vec!["[UNTRUSTED] Fast path - not Senior-reviewed".to_string()],
            loop_iterations: 1,
            model_used: Some("fast_path".to_string()),
            clarification_needed: None,
            debug_trace: Some(debug_trace),
        })
    }

    /// v0.71.0: Format fast path answer from probe output
    fn format_fast_path_answer(&self, topic: &str, probe_id: &str, output: &str) -> String {
        match probe_id {
            "cpu.info" => {
                // Parse /proc/cpuinfo output
                let mut model = "Unknown";
                let mut cores = 0u32;
                let mut threads = 0u32;

                for line in output.lines() {
                    if line.starts_with("model name") {
                        if let Some(val) = line.split(':').nth(1) {
                            model = val.trim();
                        }
                    }
                    if line.starts_with("cpu cores") {
                        if let Some(val) = line.split(':').nth(1) {
                            cores = val.trim().parse().unwrap_or(0);
                        }
                    }
                    if line.starts_with("processor") {
                        threads += 1;
                    }
                }

                format!(
                    "Your CPU is: {}\n\n\
                     Physical cores: {}\n\
                     Threads (logical processors): {}\n\n\
                     Evidence: Retrieved from {} probe (reads /proc/cpuinfo)",
                    model, cores, threads, probe_id
                )
            }
            "mem.info" => {
                // Parse /proc/meminfo output
                let mut total_kb = 0u64;
                let mut free_kb = 0u64;
                let mut available_kb = 0u64;

                for line in output.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(val) = line.split_whitespace().nth(1) {
                            total_kb = val.parse().unwrap_or(0);
                        }
                    }
                    if line.starts_with("MemFree:") {
                        if let Some(val) = line.split_whitespace().nth(1) {
                            free_kb = val.parse().unwrap_or(0);
                        }
                    }
                    if line.starts_with("MemAvailable:") {
                        if let Some(val) = line.split_whitespace().nth(1) {
                            available_kb = val.parse().unwrap_or(0);
                        }
                    }
                }

                let total_gb = total_kb as f64 / 1024.0 / 1024.0;
                let free_gb = free_kb as f64 / 1024.0 / 1024.0;
                let available_gb = available_kb as f64 / 1024.0 / 1024.0;
                let used_gb = total_gb - available_gb;

                format!(
                    "RAM Information:\n\n\
                     Total installed: {:.1} GB\n\
                     Currently free: {:.1} GB\n\
                     Available for use: {:.1} GB\n\
                     In use: {:.1} GB\n\n\
                     Evidence: Retrieved from {} probe (reads /proc/meminfo)",
                    total_gb, free_gb, available_gb, used_gb, probe_id
                )
            }
            "disk.lsblk" => {
                // Summarize lsblk output
                let line_count = output.lines().count();
                format!(
                    "Disk/Storage Information:\n\n{}\n\n\
                     ({} devices shown)\n\n\
                     Evidence: Retrieved from {} probe (runs lsblk command)",
                    output.lines().take(20).collect::<Vec<_>>().join("\n"),
                    line_count,
                    probe_id
                )
            }
            "logs.annad" => {
                // Parse journalctl output for annad logs
                let line_count = output.lines().count();
                let error_count = output.lines()
                    .filter(|l| l.to_lowercase().contains("error") || l.to_lowercase().contains("err"))
                    .count();
                let warning_count = output.lines()
                    .filter(|l| l.to_lowercase().contains("warn"))
                    .count();

                let health_status = if error_count > 5 {
                    "Concerning - multiple errors detected"
                } else if error_count > 0 || warning_count > 3 {
                    "Some issues - review recommended"
                } else {
                    "Healthy - no significant issues"
                };

                format!(
                    "Anna Daemon (annad) Logs - Last 6 Hours:\n\n\
                     Status: {}\n\
                     Error entries: {}\n\
                     Warning entries: {}\n\
                     Total log lines: {}\n\n\
                     Recent entries:\n{}\n\n\
                     Evidence: Retrieved from {} probe (journalctl -u annad)",
                    health_status,
                    error_count,
                    warning_count,
                    line_count,
                    output.lines().rev().take(20).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n"),
                    probe_id
                )
            }
            "updates.pending" => {
                // Parse pacman -Qu output
                let update_count = output.lines().filter(|l| !l.is_empty()).count();

                if update_count == 0 || output.trim().is_empty() {
                    format!(
                        "System Updates:\n\n\
                         No pending updates. Your system is up to date.\n\n\
                         Evidence: Retrieved from {} probe (pacman -Qu)",
                        probe_id
                    )
                } else {
                    format!(
                        "System Updates:\n\n\
                         {} packages have pending updates:\n\n{}\n\n\
                         To update, run: sudo pacman -Syu\n\
                         (This is a state-changing operation - not executed automatically)\n\n\
                         Evidence: Retrieved from {} probe (pacman -Qu)",
                        update_count,
                        output.lines().take(30).collect::<Vec<_>>().join("\n"),
                        probe_id
                    )
                }
            }
            _ => {
                format!(
                    "{} Information:\n\n{}\n\n\
                     Evidence: Retrieved from {} probe",
                    topic,
                    output.lines().take(30).collect::<Vec<_>>().join("\n"),
                    probe_id
                )
            }
        }
    }

    /// v0.71.0: Execute self-health check using real health probes
    /// This runs the actual self_health::run_all_probes() function instead of LLM guessing
    fn execute_self_health_fast_path(
        &self,
        question: &str,
        start_time: Instant,
    ) -> Result<FinalAnswer> {
        use anna_common::self_health::{run_all_probes, ComponentStatus, OverallHealth};

        info!("[!]  v0.71.0 Self-health fast path: running real health checks");

        // Run actual health probes
        let health_report = run_all_probes();

        // Format the answer from real health data
        let mut problems: Vec<String> = Vec::new();
        let mut answer = String::from("Anna Self-Diagnosis Report:\n\n");

        // Overall status
        let (status_icon, status_text) = match health_report.overall {
            OverallHealth::Healthy => ("[+]", "Healthy"),
            OverallHealth::Degraded => ("[~]", "Degraded"),
            OverallHealth::Critical => ("[!]", "Critical"),
            OverallHealth::Unknown => ("[?]", "Unknown"),
        };
        answer.push_str(&format!("Overall Status: {} {}\n\n", status_icon, status_text));

        // Component details
        answer.push_str("Component Checks:\n");
        for component in &health_report.components {
            let (icon, status_str) = match component.status {
                ComponentStatus::Healthy => ("[+]", "Healthy"),
                ComponentStatus::Degraded => ("[~]", "Degraded"),
                ComponentStatus::Critical => ("[!]", "Critical"),
                ComponentStatus::Unknown => ("[?]", "Unknown"),
            };

            answer.push_str(&format!("  {} {} - {}", icon, component.name, status_str));
            if !component.message.is_empty() {
                answer.push_str(&format!(" ({})", component.message));
            }
            answer.push('\n');

            if !component.status.is_healthy() && !component.message.is_empty() {
                problems.push(format!("{}: {}", component.name, component.message));
            }
        }

        // Calculate reliability score based on health
        let healthy_count = health_report.components.iter()
            .filter(|c| c.status.is_healthy())
            .count();
        let total_count = health_report.components.len();
        let reliability = if total_count > 0 {
            healthy_count as f64 / total_count as f64
        } else {
            0.5
        };

        answer.push_str(&format!(
            "\nReliability Score: {:.0}%\n",
            reliability * 100.0
        ));

        answer.push_str("\nEvidence: Generated from real self_health::run_all_probes() checks\n");
        answer.push_str("(No LLM guessing - actual component inspection)");

        let confidence_level = if reliability >= 0.9 {
            ConfidenceLevel::Green
        } else if reliability >= 0.7 {
            ConfidenceLevel::Yellow
        } else {
            ConfidenceLevel::Red
        };

        let debug_trace = DebugTrace {
            junior_model: "fast_path_self_health".to_string(),
            senior_model: "fast_path_self_health".to_string(),
            duration_secs: start_time.elapsed().as_secs_f64(),
            iterations: vec![DebugIteration {
                iteration: 1,
                llm_a_intent: "self_health_check".to_string(),
                llm_a_probes: vec!["self.health".to_string()],
                probes_executed: vec!["self.health".to_string()],
                llm_b_verdict: Some("fast_path_health_check".to_string()),
                llm_b_confidence: Some(reliability),
                ..Default::default()
            }],
        };

        Ok(FinalAnswer {
            question: question.to_string(),
            answer,
            is_refusal: false,
            citations: vec![],
            scores: AuditScores::new(reliability, reliability, reliability),
            confidence_level,
            problems,
            loop_iterations: 1,
            model_used: Some("fast_path_self_health".to_string()),
            clarification_needed: None,
            debug_trace: Some(debug_trace),
        })
    }

    /// v0.77.0: Process a question with debug event emission for streaming
    ///
    /// This passes the emitter through to LLM calls, which emit prompts and responses
    /// as streaming events. The LLM dialog appears directly in annactl, not in logs.
    pub async fn process_question_with_emitter(
        &self,
        question: &str,
        emitter: &dyn anna_common::DebugEventEmitter,
    ) -> Result<FinalAnswer> {
        use anna_common::{DebugEvent, DebugEventData, DebugEventType};

        // Emit stream started event
        emitter.emit(DebugEvent::new(
            DebugEventType::StreamStarted,
            0,
            "Starting LLM orchestration",
        ).with_data(DebugEventData::StreamMeta {
            question: question.to_string(),
            junior_model: self.junior_model().to_string(),
            senior_model: self.senior_model().to_string(),
        }));

        // v0.77.0: Call internal method with emitter - LLM events stream to annactl
        let result = self.process_question_with_optional_emitter(question, Some(emitter)).await;

        // Emit completion events based on result
        match &result {
            Ok(answer) => {
                let iterations = answer
                    .debug_trace
                    .as_ref()
                    .map(|t| t.iterations.len())
                    .unwrap_or(1);

                // Emit answer ready
                emitter.answer_ready(
                    &format!("{:?}", answer.confidence_level),
                    answer.scores.overall,
                    iterations,
                );
            }
            Err(e) => {
                emitter.error(&format!("{}", e), false);
            }
        }

        result
    }
}

impl Default for AnswerEngine {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Truncate a string for debug display
fn truncate_for_debug(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!(
            "{}...[truncated {} chars]",
            &s[..max_len],
            s.len() - max_len
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AnswerEngine::default();
        assert!(!engine.catalog().available_probes().is_empty());
    }

    #[test]
    fn test_filter_valid_probes() {
        let engine = AnswerEngine::default();
        let probes = vec![
            "cpu.info".to_string(),
            "fake.probe".to_string(),
            "mem.info".to_string(),
            "invalid.id".to_string(),
        ];
        let valid = engine.filter_valid_probes(&probes);
        assert_eq!(valid.len(), 2);
        assert!(valid.contains(&"cpu.info".to_string()));
        assert!(valid.contains(&"mem.info".to_string()));
    }

    #[test]
    fn test_build_refusal() {
        let engine = AnswerEngine::default();
        let result = engine.build_refusal("test question", "no evidence", &[], 1);
        assert!(result.is_refusal);
        assert_eq!(result.confidence_level, ConfidenceLevel::Red);
        assert_eq!(result.loop_iterations, 1);
    }

    #[test]
    fn test_build_final_answer() {
        let engine = AnswerEngine::default();
        let scores = AuditScores::new(0.95, 0.90, 0.92);
        let result = engine.build_final_answer("test", "answer", vec![], scores, 2);
        assert!(!result.is_refusal);
        assert_eq!(result.confidence_level, ConfidenceLevel::Green);
        assert_eq!(result.loop_iterations, 2);
    }

    #[test]
    fn test_build_partial_answer() {
        let engine = AnswerEngine::default();
        let scores = AuditScores::new(0.5, 0.5, 0.5);
        let result = engine.build_partial_answer("test", "partial answer", vec![], scores, 3);
        assert!(!result.is_refusal);
        assert_eq!(result.confidence_level, ConfidenceLevel::Red);
        assert!(result.answer.contains("[Note:"));
        assert!(result
            .problems
            .contains(&"Reached maximum verification loops".to_string()));
    }
}

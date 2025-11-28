//! Answer Engine v0.17.0
//!
//! The main orchestration loop:
//! LLM-A (plan) -> Probes -> LLM-A (answer) -> LLM-B (audit) -> approve/fix/retry
//!
//! Key v0.17.0 changes:
//! - Use Senior's synthesized answer (text field) instead of Junior's draft
//! - Senior can now provide a direct answer that gets displayed to user
//! - Priority: Senior text > Senior fixed_answer > Junior draft_answer
//! - Fixes issue where useless "I will run probe..." answers were shown

use super::fallback;
use super::llm_client::OllamaClient;
use super::probe_executor;
use anna_common::{
    generate_llm_a_prompt_with_iteration, generate_llm_b_prompt, AuditScores, AuditVerdict,
    ConfidenceLevel, DebugIteration, DebugTrace, FinalAnswer, LoopState, ProbeCatalog,
    ProbeEvidenceV10, QuestionClassifier, QuestionDomain, ReliabilityScores, MAX_LOOPS,
};
use anyhow::Result;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Check if debug mode is enabled
fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
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
        info!("Processing question: {}", question);
        let start_time = Instant::now();

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

            // Step 1: Call LLM-A with iteration awareness
            let llm_a_prompt = generate_llm_a_prompt_with_iteration(
                question,
                &self.catalog.available_probes(),
                &evidence,
                loop_state.iteration,
            );

            // Capture prompt for debug (truncated for very large prompts)
            debug_iter.llm_a_prompt = truncate_for_debug(&llm_a_prompt, 8000);

            let (llm_a_response, llm_a_raw) = self.llm_client.call_llm_a(&llm_a_prompt).await?;

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
            if let Some(ref draft) = llm_a_response.draft_answer {
                info!(
                    "[D]  Draft answer: {}",
                    &draft.text[..200.min(draft.text.len())]
                );
            }

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

                    let new_evidence =
                        probe_executor::execute_probes(&self.catalog, &valid_probes).await;
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

            let self_scores = llm_a_response
                .self_scores
                .unwrap_or_else(|| ReliabilityScores::new(0.5, 0.5, 0.5));

            // Step 4: Call LLM-B to audit
            let llm_b_prompt =
                generate_llm_b_prompt(question, &draft_answer, &evidence, &self_scores);

            // Capture LLM-B prompt for debug (truncated for very large prompts)
            debug_iter.llm_b_prompt = Some(truncate_for_debug(&llm_b_prompt, 8000));

            let (llm_b_response, llm_b_raw) = self.llm_client.call_llm_b(&llm_b_prompt).await?;

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
                    debug_trace.iterations.push(debug_iter);
                    debug_trace.duration_secs = start_time.elapsed().as_secs_f64();
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
                    debug_trace.iterations.push(debug_iter);
                    debug_trace.duration_secs = start_time.elapsed().as_secs_f64();
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
                    // If we have evidence, try partial answer with low confidence
                    warn!("LLM-B wants to refuse but we have evidence - will try partial answer");
                    last_scores = Some(AuditScores::new(0.5, 0.5, 0.5));
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

        // If we have a draft answer, return it with honest low confidence
        if let Some(answer_text) = last_draft_answer {
            let scores = last_scores.unwrap_or_else(|| AuditScores::new(0.5, 0.5, 0.5));
            info!(
                "Max loops reached - returning partial answer with confidence {:.2}",
                scores.overall
            );
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
        if !evidence.is_empty() {
            if let Some(fallback) = fallback::extract_fallback_answer(question, &evidence) {
                info!("Using fallback answer extracted from evidence");
                return Ok(self.build_partial_answer_with_trace(
                    question,
                    &fallback,
                    evidence,
                    AuditScores::new(0.6, 0.6, 0.6),
                    loop_state.iteration,
                    debug_trace,
                ));
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

    /// v0.43.0: Process a question with debug event emission for streaming
    ///
    /// This wrapper emits streaming debug events during the orchestration loop.
    /// The emitter receives real-time updates as the Junior/Senior LLM loop progresses.
    pub async fn process_question_with_emitter(
        &self,
        question: &str,
        emitter: &dyn anna_common::DebugEventEmitter,
    ) -> Result<FinalAnswer> {
        use anna_common::{DebugEvent, DebugEventData, DebugEventType};

        // Emit iteration start
        emitter.emit(DebugEvent::new(
            DebugEventType::IterationStarted,
            1,
            "Starting orchestration",
        ));

        // Emit junior plan start
        emitter.emit(DebugEvent::new(
            DebugEventType::JuniorPlanStarted,
            1,
            "Junior LLM analyzing question",
        ));

        // Call the main process_question
        let result = self.process_question(question).await;

        // Emit completion events based on result
        match &result {
            Ok(answer) => {
                // Extract debug info from the answer if available
                let iterations = answer
                    .debug_trace
                    .as_ref()
                    .map(|t| t.iterations.len())
                    .unwrap_or(1);

                // Emit probes executed event
                let probes: Vec<String> = answer
                    .citations
                    .iter()
                    .map(|c| c.probe_id.clone())
                    .collect();
                if !probes.is_empty() {
                    let probe_results: Vec<anna_common::ProbeResultSnippet> = probes
                        .iter()
                        .map(|id| anna_common::ProbeResultSnippet {
                            probe_id: id.clone(),
                            success: true,
                            latency_ms: 0, // Not tracked at this level
                            snippet: "...".to_string(),
                        })
                        .collect();
                    emitter.emit(
                        DebugEvent::new(
                            DebugEventType::ProbesExecuted,
                            iterations,
                            "Probes executed",
                        )
                        .with_data(DebugEventData::ProbeResults {
                            probes: probe_results,
                            total_ms: 0,
                        }),
                    );
                }

                // Emit senior review
                emitter.emit(DebugEvent::new(
                    DebugEventType::SeniorReviewDone,
                    iterations,
                    &format!("Senior review: {:?}", answer.confidence_level),
                ));
            }
            Err(e) => {
                emitter.emit(DebugEvent::new(
                    DebugEventType::Error,
                    1,
                    &format!("Error: {}", e),
                ));
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

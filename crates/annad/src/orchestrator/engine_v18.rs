//! DEPRECATED: Legacy Answer Engine v0.18.0
//!
//! **WARNING**: This file is deprecated. Use `engine_v90.rs` (UnifiedEngine) instead.
//! See `docs/architecture.md` Section 9 for details.
//!
//! This module remains for backward compatibility but is not called in production.
//!
//! Step-by-step orchestration:
//! - ONE action per Junior iteration (probe, clarification, answer, escalate)
//! - Senior only called when Junior escalates or confidence is low
//! - Clear separation: Junior decides, Anna executes, Senior audits

use super::llm_client_v18::LlmClientV18;
use super::probe_executor;
use anna_common::{
    FinalAnswerV18, HistoryEntry, JuniorScores, JuniorStep, LoopStatus, ProbeResultV18,
    ProbeCatalog, QuestionLoopState, SeniorResponse, SeniorScores, MAX_ITERATIONS,
    MIN_SCORE_WITHOUT_SENIOR, SCORE_GREEN,
};
use anyhow::Result;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Check if debug mode is enabled
fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
}

/// Answer engine v0.18.0 - step-by-step orchestration
pub struct AnswerEngineV18 {
    llm_client: LlmClientV18,
    catalog: ProbeCatalog,
}

impl AnswerEngineV18 {
    /// Create engine with role-specific models
    pub fn with_role_models(junior_model: Option<String>, senior_model: Option<String>) -> Self {
        Self {
            llm_client: LlmClientV18::with_role_models(junior_model, senior_model),
            catalog: ProbeCatalog::standard(),
        }
    }

    /// Get available probe IDs
    fn available_probe_ids(&self) -> Vec<String> {
        self.catalog
            .available_probes()
            .iter()
            .map(|p| p.probe_id.clone())
            .collect()
    }

    /// Process a user question using step-by-step protocol
    pub async fn process_question(&self, question: &str) -> Result<FinalAnswerV18> {
        info!("v0.18.0 Processing: {}", question);
        let _start_time = Instant::now();

        let mut state = QuestionLoopState {
            question: question.to_string(),
            ..Default::default()
        };

        // Build history JSON for context
        let mut history: Vec<HistoryEntry> = vec![];

        // Main orchestration loop
        while state.iteration < MAX_ITERATIONS && state.status == LoopStatus::InProgress {
            state.iteration += 1;
            info!("Iteration {}/{}", state.iteration, MAX_ITERATIONS);

            if is_debug_mode() {
                print_iteration_header(state.iteration, &self.llm_client);
            }

            // Step 1: Ask Junior what to do next
            let history_json = serde_json::to_string_pretty(&history).unwrap_or_default();
            let junior_step = self
                .llm_client
                .call_junior(question, &self.available_probe_ids(), &history_json, state.iteration)
                .await?;

            // Step 2: Handle Junior's action
            match junior_step {
                JuniorStep::RunProbe { probe_id, reason } => {
                    info!("Junior requests probe: {} ({})", probe_id, reason);

                    // Validate probe exists
                    if !self.catalog.is_valid(&probe_id) {
                        warn!("Invalid probe_id: {} - skipping", probe_id);
                        history.push(HistoryEntry::ProbeResult {
                            probe_id: probe_id.clone(),
                            output: format!("ERROR: probe '{}' does not exist", probe_id),
                            success: false,
                        });
                        continue;
                    }

                    // Execute probe
                    let results = probe_executor::execute_probes(&self.catalog, &[probe_id.clone()]).await;
                    let result = results.first();

                    let (output, success) = match result {
                        Some(ev) => {
                            let out = ev.raw.clone().unwrap_or_else(|| "No output".to_string());
                            let ok = ev.status == anna_common::EvidenceStatus::Ok;
                            (out, ok)
                        }
                        None => ("Probe execution failed".to_string(), false),
                    };

                    state.probes_run.push(ProbeResultV18 {
                        probe_id: probe_id.clone(),
                        raw_output: output.clone(),
                        success,
                    });

                    history.push(HistoryEntry::ProbeResult {
                        probe_id,
                        output,
                        success,
                    });
                }

                JuniorStep::RunCommand { cmd, reason } => {
                    info!("Junior requests command: {} ({})", cmd, reason);
                    // For v0.18.0, commands are not yet supported
                    history.push(HistoryEntry::CommandResult {
                        cmd: cmd.clone(),
                        output: "ERROR: Custom commands not yet supported".to_string(),
                        success: false,
                    });
                }

                JuniorStep::AskClarification { question: clarify_q } => {
                    info!("Junior needs clarification: {}", clarify_q);
                    // For now, we skip clarification and ask Junior to proceed
                    // In future, this would be relayed to the user
                    history.push(HistoryEntry::Clarification {
                        question: clarify_q,
                        answer: "[User did not respond - please proceed with best guess]".to_string(),
                    });
                }

                JuniorStep::ProposeAnswer {
                    text,
                    citations,
                    scores,
                    ready_for_user,
                } => {
                    info!(
                        "Junior proposes answer (ready={}, overall={}): {}",
                        ready_for_user,
                        scores.overall,
                        &text[..100.min(text.len())]
                    );

                    // If high confidence and ready, deliver directly
                    if ready_for_user && scores.overall >= MIN_SCORE_WITHOUT_SENIOR {
                        state.status = LoopStatus::Complete;
                        return Ok(FinalAnswerV18 {
                            text,
                            reliability: scores.overall,
                            reliability_note: format!("Junior confidence: {}/100", scores.overall),
                            citations,
                            is_refusal: false,
                            iterations: state.iteration,
                            senior_reviewed: false,
                        });
                    }

                    // Otherwise, send to Senior for review
                    state.status = LoopStatus::AwaitingSenior;
                    let senior_response = self
                        .llm_client
                        .call_senior(
                            question,
                            &history_json,
                            Some(&text),
                            Some(&scores),
                            "Junior proposed answer with insufficient confidence",
                        )
                        .await?;

                    return self.handle_senior_response(senior_response, &text, &citations, state.iteration);
                }

                JuniorStep::EscalateToSenior { summary } => {
                    info!("Junior escalates to Senior: {}", summary.reason_for_escalation);
                    state.status = LoopStatus::AwaitingSenior;

                    let senior_response = self
                        .llm_client
                        .call_senior(
                            question,
                            &history_json,
                            summary.draft_answer.as_deref(),
                            summary.draft_scores.as_ref(),
                            &summary.reason_for_escalation,
                        )
                        .await?;

                    let draft = summary.draft_answer.unwrap_or_default();
                    let citations: Vec<String> = summary
                        .probes_run
                        .iter()
                        .map(|p| p.probe_id.clone())
                        .collect();

                    return self.handle_senior_response(senior_response, &draft, &citations, state.iteration);
                }
            }
        }

        // Max iterations reached
        state.status = LoopStatus::MaxIterations;
        warn!("Max iterations ({}) reached without answer", MAX_ITERATIONS);

        // Try to construct a partial answer from probes
        let probe_summary: Vec<String> = state
            .probes_run
            .iter()
            .map(|p| format!("{}: {}", p.probe_id, if p.success { "OK" } else { "FAILED" }))
            .collect();

        Ok(FinalAnswerV18 {
            text: format!(
                "I was unable to fully answer your question after {} iterations.\n\n\
                 Probes attempted: {}",
                state.iteration,
                probe_summary.join(", ")
            ),
            reliability: 30,
            reliability_note: "Partial answer - max iterations reached".to_string(),
            citations: state.probes_run.iter().map(|p| p.probe_id.clone()).collect(),
            is_refusal: false,
            iterations: state.iteration,
            senior_reviewed: false,
        })
    }

    /// Handle Senior's response and produce final answer
    fn handle_senior_response(
        &self,
        response: SeniorResponse,
        draft: &str,
        citations: &[String],
        iterations: usize,
    ) -> Result<FinalAnswerV18> {
        match response {
            SeniorResponse::ApproveAnswer { scores } => {
                Ok(FinalAnswerV18 {
                    text: draft.to_string(),
                    reliability: scores.overall,
                    reliability_note: scores.reliability_note,
                    citations: citations.to_vec(),
                    is_refusal: false,
                    iterations,
                    senior_reviewed: true,
                })
            }

            SeniorResponse::CorrectAnswer {
                text,
                scores,
                corrections,
            } => {
                info!("Senior corrected answer: {:?}", corrections);
                Ok(FinalAnswerV18 {
                    text,
                    reliability: scores.overall,
                    reliability_note: scores.reliability_note,
                    citations: citations.to_vec(),
                    is_refusal: false,
                    iterations,
                    senior_reviewed: true,
                })
            }

            SeniorResponse::RequestProbe { probe_id, reason } => {
                // For now, we don't loop back after Senior requests
                // This would require refactoring to async state machine
                warn!("Senior requested probe {} but loop continuation not implemented", probe_id);
                Ok(FinalAnswerV18 {
                    text: format!(
                        "{}\n\n[Note: Senior requested additional probe '{}' for: {}]",
                        draft, probe_id, reason
                    ),
                    reliability: 50,
                    reliability_note: "Incomplete - Senior needed more data".to_string(),
                    citations: citations.to_vec(),
                    is_refusal: false,
                    iterations,
                    senior_reviewed: true,
                })
            }

            SeniorResponse::RequestCommand { cmd, reason } => {
                warn!("Senior requested command {} but not implemented", cmd);
                Ok(FinalAnswerV18 {
                    text: format!(
                        "{}\n\n[Note: Senior requested command '{}' for: {}]",
                        draft, cmd, reason
                    ),
                    reliability: 50,
                    reliability_note: "Incomplete - Senior needed more data".to_string(),
                    citations: citations.to_vec(),
                    is_refusal: false,
                    iterations,
                    senior_reviewed: true,
                })
            }

            SeniorResponse::Refuse {
                reason,
                probes_attempted,
            } => {
                Ok(FinalAnswerV18 {
                    text: format!(
                        "I cannot answer this question.\n\nReason: {}\n\nProbes attempted: {}",
                        reason,
                        probes_attempted.join(", ")
                    ),
                    reliability: 0,
                    reliability_note: "Refused - insufficient evidence".to_string(),
                    citations: probes_attempted,
                    is_refusal: true,
                    iterations,
                    senior_reviewed: true,
                })
            }
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

    /// Get junior model name
    pub fn junior_model(&self) -> &str {
        self.llm_client.junior_model()
    }

    /// Get senior model name
    pub fn senior_model(&self) -> &str {
        self.llm_client.senior_model()
    }
}

impl Default for AnswerEngineV18 {
    fn default() -> Self {
        Self::with_role_models(None, None)
    }
}

/// Print iteration header for debug mode
fn print_iteration_header(iteration: usize, client: &LlmClientV18) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    let _ = writeln!(stderr);
    let _ = writeln!(
        stderr,
        "================================================================"
    );
    let _ = writeln!(
        stderr,
        "  v0.18.0 ITERATION {}/{}  |  Junior: {}  |  Senior: {}",
        iteration,
        MAX_ITERATIONS,
        client.junior_model(),
        client.senior_model()
    );
    let _ = writeln!(
        stderr,
        "================================================================"
    );
    let _ = stderr.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AnswerEngineV18::default();
        assert!(!engine.catalog().available_probes().is_empty());
    }

    #[test]
    fn test_available_probe_ids() {
        let engine = AnswerEngineV18::default();
        let probes = engine.available_probe_ids();
        assert!(probes.contains(&"cpu.info".to_string()));
        assert!(probes.contains(&"mem.info".to_string()));
    }
}

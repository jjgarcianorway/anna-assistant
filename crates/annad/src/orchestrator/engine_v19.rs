//! Answer Engine v0.19.0 - Subproblem Decomposition
//!
//! Key features:
//! - Junior decomposes questions into subproblems
//! - Fact-aware planning (check fact store first)
//! - Senior as mentor (guides, doesn't replace)
//! - Work on one subproblem at a time

use super::llm_client_v19::LlmClientV19;
use super::probe_executor;
use anna_common::{
    FinalAnswerV19, JuniorStepV19, ProbeResultV19, Subproblem, SubproblemStatus,
    SeniorMentor, ProbeCatalog, MAX_ITERATIONS_V19,
};
use anyhow::Result;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Check if debug mode is enabled
fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
}

/// Answer engine v0.19.0 - subproblem decomposition
pub struct AnswerEngineV19 {
    llm_client: LlmClientV19,
    catalog: ProbeCatalog,
}

impl AnswerEngineV19 {
    /// Create engine with role-specific models
    pub fn with_role_models(junior_model: Option<String>, senior_model: Option<String>) -> Self {
        Self {
            llm_client: LlmClientV19::with_role_models(junior_model, senior_model),
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

    /// Get known facts from the fact store (placeholder for now)
    fn get_known_facts(&self, _question: &str) -> String {
        // TODO: Query actual fact store
        // For now, return empty
        "No known facts available".to_string()
    }

    /// Process a user question using subproblem decomposition
    pub async fn process_question(&self, question: &str) -> Result<FinalAnswerV19> {
        info!("v0.19.0 Processing: {}", question);
        let _start_time = Instant::now();

        let mut subproblems: Vec<Subproblem> = vec![];
        let mut probes_executed: Vec<ProbeResultV19> = vec![];
        let mut senior_mentored = false;
        let mut iteration = 0;

        // Phase 1: Decomposition
        info!("Phase 1: Decomposition");
        let known_facts = self.get_known_facts(question);
        let decomposition = self
            .llm_client
            .call_junior_decompose(question, &known_facts, &self.available_probe_ids())
            .await?;

        if let JuniorStepV19::Decompose { decomposition: d } = decomposition {
            subproblems = d.subproblems;
            info!("Decomposed into {} subproblems", subproblems.len());
        }

        // Phase 2: Work on subproblems
        info!("Phase 2: Working on subproblems");
        while iteration < MAX_ITERATIONS_V19 {
            iteration += 1;
            info!("Iteration {}/{}", iteration, MAX_ITERATIONS_V19);

            if is_debug_mode() {
                print_iteration_header(iteration, &self.llm_client);
            }

            // Check if all subproblems are done
            let pending: Vec<_> = subproblems
                .iter()
                .filter(|sp| sp.status == SubproblemStatus::Pending || sp.status == SubproblemStatus::InProgress)
                .collect();

            if pending.is_empty() {
                info!("All subproblems resolved, moving to synthesis");
                break;
            }

            // Ask Junior for next action
            let subproblems_json = serde_json::to_string_pretty(&subproblems).unwrap_or_default();
            let probe_history = format_probe_history(&probes_executed);

            let action = self
                .llm_client
                .call_junior_work(question, &subproblems_json, &probe_history, iteration)
                .await?;

            match action {
                JuniorStepV19::WorkSubproblem {
                    subproblem_id,
                    probe_id,
                    reason,
                } => {
                    info!("Working on {} with probe {} ({})", subproblem_id, probe_id, reason);

                    // Validate probe
                    if !self.catalog.is_valid(&probe_id) {
                        warn!("Invalid probe: {}", probe_id);
                        continue;
                    }

                    // Mark subproblem as in progress
                    if let Some(sp) = subproblems.iter_mut().find(|sp| sp.id == subproblem_id) {
                        sp.status = SubproblemStatus::InProgress;
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

                    probes_executed.push(ProbeResultV19 {
                        probe_id: probe_id.clone(),
                        subproblem_id: Some(subproblem_id.clone()),
                        output: output.clone(),
                        success,
                    });

                    // Add evidence to subproblem
                    if let Some(sp) = subproblems.iter_mut().find(|sp| sp.id == subproblem_id) {
                        sp.evidence.push(format!("{}: {}", probe_id, &output[..output.len().min(200)]));
                    }
                }

                JuniorStepV19::SolveSubproblem {
                    subproblem_id,
                    partial_answer,
                    confidence,
                } => {
                    info!("Solved {} with confidence {}", subproblem_id, confidence);

                    if let Some(sp) = subproblems.iter_mut().find(|sp| sp.id == subproblem_id) {
                        sp.status = SubproblemStatus::Solved;
                        sp.partial_answer = Some(partial_answer);
                    }
                }

                JuniorStepV19::AskMentor { question: mentor_q, current_state } => {
                    info!("Junior asks mentor: {}", mentor_q);
                    senior_mentored = true;

                    let mentor_context = serde_json::to_string_pretty(&current_state).unwrap_or_default();
                    let mentor_response = self
                        .llm_client
                        .call_senior_mentor(question, &mentor_context, &mentor_q)
                        .await?;

                    // Apply mentor feedback
                    match mentor_response {
                        SeniorMentor::ApproveApproach { feedback } => {
                            info!("Mentor approves: {}", feedback);
                        }
                        SeniorMentor::RefineSubproblems {
                            feedback,
                            suggested_additions,
                            suggested_removals,
                            suggested_merges: _,
                        } => {
                            info!("Mentor suggests refinements: {}", feedback);

                            // Remove suggested removals
                            subproblems.retain(|sp| !suggested_removals.contains(&sp.id));

                            // Add suggested additions
                            for (i, addition) in suggested_additions.iter().enumerate() {
                                subproblems.push(Subproblem {
                                    id: format!("sp_new_{}", i),
                                    description: addition.description.clone(),
                                    required_probes: addition.suggested_probes.clone(),
                                    relevant_facts: vec![],
                                    status: SubproblemStatus::Pending,
                                    evidence: vec![],
                                    partial_answer: None,
                                });
                            }
                        }
                        SeniorMentor::SuggestApproach {
                            feedback,
                            new_approach: _,
                            key_subproblems,
                        } => {
                            info!("Mentor suggests new approach: {}", feedback);

                            // Replace subproblems with mentor's suggestions
                            subproblems.clear();
                            for (i, sp) in key_subproblems.iter().enumerate() {
                                subproblems.push(Subproblem {
                                    id: format!("sp_mentor_{}", i),
                                    description: sp.description.clone(),
                                    required_probes: sp.suggested_probes.clone(),
                                    relevant_facts: vec![],
                                    status: SubproblemStatus::Pending,
                                    evidence: vec![],
                                    partial_answer: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }

                JuniorStepV19::Synthesize { text, subproblem_summaries, scores } => {
                    info!("Junior synthesizes answer with score {}", scores.overall);

                    // Get Senior review
                    let summaries_json = serde_json::to_string_pretty(&subproblem_summaries).unwrap_or_default();
                    let scores_json = serde_json::to_string_pretty(&scores).unwrap_or_default();
                    let probes_list = probes_executed.iter().map(|p| p.probe_id.clone()).collect::<Vec<_>>().join(", ");

                    let review = self
                        .llm_client
                        .call_senior_review(question, &text, &summaries_json, &scores_json, &probes_list)
                        .await?;

                    let solved_count = subproblems.iter().filter(|sp| sp.status == SubproblemStatus::Solved).count();

                    match review {
                        SeniorMentor::ApproveAnswer { scores: senior_scores } => {
                            return Ok(FinalAnswerV19 {
                                text,
                                reliability: senior_scores.overall,
                                reliability_note: senior_scores.reliability_note,
                                subproblems_solved: solved_count,
                                subproblems_total: subproblems.len(),
                                probes_used: probes_executed.iter().map(|p| p.probe_id.clone()).collect(),
                                facts_used: vec![],
                                iterations: iteration,
                                senior_mentored,
                            });
                        }
                        SeniorMentor::CorrectAnswer { corrected_text, corrections, scores: senior_scores } => {
                            info!("Senior corrections: {:?}", corrections);
                            return Ok(FinalAnswerV19 {
                                text: corrected_text,
                                reliability: senior_scores.overall,
                                reliability_note: senior_scores.reliability_note,
                                subproblems_solved: solved_count,
                                subproblems_total: subproblems.len(),
                                probes_used: probes_executed.iter().map(|p| p.probe_id.clone()).collect(),
                                facts_used: vec![],
                                iterations: iteration,
                                senior_mentored,
                            });
                        }
                        _ => {}
                    }
                }

                JuniorStepV19::Decompose { .. } => {
                    warn!("Unexpected decompose action after initial phase");
                }
            }
        }

        // Max iterations reached - synthesize partial answer
        let solved_count = subproblems
            .iter()
            .filter(|sp| sp.status == SubproblemStatus::Solved)
            .count();

        let partial_answers: Vec<String> = subproblems
            .iter()
            .filter_map(|sp| sp.partial_answer.clone())
            .collect();

        Ok(FinalAnswerV19 {
            text: format!(
                "Partial answer after {} iterations:\n\n{}",
                iteration,
                partial_answers.join("\n\n")
            ),
            reliability: 40,
            reliability_note: "Partial - max iterations reached".to_string(),
            subproblems_solved: solved_count,
            subproblems_total: subproblems.len(),
            probes_used: probes_executed.iter().map(|p| p.probe_id.clone()).collect(),
            facts_used: vec![],
            iterations: iteration,
            senior_mentored,
        })
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

impl Default for AnswerEngineV19 {
    fn default() -> Self {
        Self::with_role_models(None, None)
    }
}

/// Format probe history for prompts
fn format_probe_history(probes: &[ProbeResultV19]) -> String {
    if probes.is_empty() {
        return "No probes executed yet".to_string();
    }

    probes
        .iter()
        .map(|p| {
            let sp = p.subproblem_id.as_deref().unwrap_or("global");
            let status = if p.success { "OK" } else { "FAILED" };
            format!("[{}] {} ({}): {}...", sp, p.probe_id, status, &p.output[..p.output.len().min(100)])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Print iteration header for debug mode
fn print_iteration_header(iteration: usize, client: &LlmClientV19) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    let _ = writeln!(stderr);
    let _ = writeln!(
        stderr,
        "================================================================"
    );
    let _ = writeln!(
        stderr,
        "  v0.19.0 ITERATION {}/{}  |  Junior: {}  |  Senior: {}",
        iteration,
        MAX_ITERATIONS_V19,
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
        let engine = AnswerEngineV19::default();
        assert!(!engine.catalog().available_probes().is_empty());
    }

    #[test]
    fn test_available_probe_ids() {
        let engine = AnswerEngineV19::default();
        let probes = engine.available_probe_ids();
        assert!(probes.contains(&"cpu.info".to_string()));
    }

    #[test]
    fn test_format_probe_history_empty() {
        let history = format_probe_history(&[]);
        assert_eq!(history, "No probes executed yet");
    }

    #[test]
    fn test_format_probe_history() {
        let probes = vec![ProbeResultV19 {
            probe_id: "cpu.info".to_string(),
            subproblem_id: Some("sp1".to_string()),
            output: "AMD Ryzen".to_string(),
            success: true,
        }];
        let history = format_probe_history(&probes);
        assert!(history.contains("cpu.info"));
        assert!(history.contains("sp1"));
    }
}

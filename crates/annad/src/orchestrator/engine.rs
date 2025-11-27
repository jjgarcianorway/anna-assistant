//! Answer Engine v0.10.0
//!
//! The main orchestration loop:
//! LLM-A (plan) -> Probes -> LLM-A (answer) -> LLM-B (audit) -> approve/refuse/retry

use super::llm_client::OllamaClient;
use super::probe_executor;
use anna_common::{
    generate_llm_a_prompt, generate_llm_b_prompt, AuditScores, AuditVerdict, ConfidenceLevel,
    FinalAnswer, LoopOutcome, LoopState, ProbeCatalog, ProbeEvidenceV10, ReliabilityScores,
    MINIMUM_ACCEPTABLE_SCORE, MAX_LOOPS,
};
use anyhow::Result;
use tracing::{debug, info, warn};

/// Answer engine - orchestrates the LLM-A/LLM-B loop
pub struct AnswerEngine {
    llm_client: OllamaClient,
    catalog: ProbeCatalog,
}

impl AnswerEngine {
    pub fn new(model: Option<String>) -> Self {
        Self {
            llm_client: OllamaClient::new(model),
            catalog: ProbeCatalog::standard(),
        }
    }

    /// Process a user question and return the final answer
    pub async fn process_question(&self, question: &str) -> Result<FinalAnswer> {
        info!("Processing question: {}", question);

        let mut loop_state = LoopState::default();
        let mut evidence: Vec<ProbeEvidenceV10> = vec![];

        // Main orchestration loop
        while loop_state.can_continue() {
            loop_state.next_iteration();
            info!("Loop iteration {}/{}", loop_state.iteration, MAX_LOOPS);

            // Step 1: Call LLM-A
            let llm_a_prompt =
                generate_llm_a_prompt(question, &self.catalog.available_probes(), &evidence);
            let llm_a_response = self.llm_client.call_llm_a(&llm_a_prompt).await?;

            debug!("LLM-A response: {:?}", llm_a_response);

            // Check for immediate refusal
            if llm_a_response.refuse_to_answer {
                loop_state.mark_refused();
                return Ok(self.build_refusal(
                    question,
                    llm_a_response.refusal_reason.as_deref().unwrap_or("Unable to answer"),
                    &evidence,
                    loop_state.iteration,
                ));
            }

            // Step 2: Execute any requested probes
            if llm_a_response.needs_more_probes || !llm_a_response.plan.probe_requests.is_empty() {
                let probe_ids: Vec<String> = llm_a_response
                    .plan
                    .probe_requests
                    .iter()
                    .map(|p| p.probe_id.clone())
                    .collect();

                if !probe_ids.is_empty() {
                    info!("Executing {} probes", probe_ids.len());
                    let new_evidence = probe_executor::execute_probes(&self.catalog, &probe_ids).await;
                    evidence.extend(new_evidence);
                }

                // If needs more probes, continue to next iteration
                if llm_a_response.needs_more_probes && llm_a_response.draft_answer.is_none() {
                    continue;
                }
            }

            // Step 3: Get draft answer
            let draft_answer = match llm_a_response.draft_answer {
                Some(draft) => draft,
                None => {
                    warn!("LLM-A did not provide draft answer");
                    continue;
                }
            };

            let self_scores = llm_a_response
                .self_scores
                .unwrap_or_else(|| ReliabilityScores::new(0.5, 0.5, 0.5));

            // Step 4: Call LLM-B to audit
            let llm_b_prompt = generate_llm_b_prompt(question, &draft_answer, &evidence, &self_scores);
            let llm_b_response = self.llm_client.call_llm_b(&llm_b_prompt).await?;

            debug!("LLM-B response: {:?}", llm_b_response);

            loop_state.record_score(llm_b_response.scores.overall);

            // Step 5: Handle verdict
            match llm_b_response.verdict {
                AuditVerdict::Approve => {
                    loop_state.mark_approved();
                    return Ok(self.build_final_answer(
                        question,
                        &draft_answer.text,
                        evidence,
                        llm_b_response.scores,
                        loop_state.iteration,
                    ));
                }
                AuditVerdict::Refuse => {
                    loop_state.mark_refused();
                    let reason = llm_b_response.problems.first().map(|s| s.as_str()).unwrap_or(
                        "Auditor determined answer cannot be safely provided",
                    );
                    return Ok(self.build_refusal(question, reason, &evidence, loop_state.iteration));
                }
                AuditVerdict::NeedsMoreProbes => {
                    // Execute additional probes requested by auditor
                    let probe_ids: Vec<String> = llm_b_response
                        .probe_requests
                        .iter()
                        .map(|p| p.probe_id.clone())
                        .collect();

                    if !probe_ids.is_empty() {
                        info!("Auditor requested {} more probes", probe_ids.len());
                        let new_evidence =
                            probe_executor::execute_probes(&self.catalog, &probe_ids).await;
                        evidence.extend(new_evidence);
                    }
                    // Continue to next iteration
                }
            }
        }

        // Loop exhausted - check if we have acceptable score
        loop_state.mark_exhausted();

        if loop_state.reached_acceptable {
            // Use last score
            let last_score = loop_state.score_history.last().copied().unwrap_or(0.0);
            return Ok(self.build_final_answer(
                question,
                "Unable to provide a fully verified answer within loop limit.",
                evidence,
                AuditScores::new(last_score, last_score, last_score),
                loop_state.iteration,
            ));
        }

        // No acceptable answer found
        Ok(self.build_refusal(
            question,
            "Could not reach acceptable confidence after maximum iterations",
            &evidence,
            loop_state.iteration,
        ))
    }

    /// Build a final answer
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
        }
    }

    /// Build a refusal answer
    fn build_refusal(
        &self,
        question: &str,
        reason: &str,
        evidence: &[ProbeEvidenceV10],
        loop_iterations: usize,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: format!(
                "I cannot answer this safely with the evidence available.\n\nReason: {}",
                reason
            ),
            is_refusal: true,
            citations: evidence.to_vec(),
            scores: AuditScores::new(0.0, 0.0, 0.0),
            confidence_level: ConfidenceLevel::Red,
            problems: vec![reason.to_string()],
            loop_iterations,
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
}

impl Default for AnswerEngine {
    fn default() -> Self {
        Self::new(None)
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
}

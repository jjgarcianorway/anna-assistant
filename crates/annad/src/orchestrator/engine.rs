//! Answer Engine v0.12.2
//!
//! The main orchestration loop:
//! LLM-A (plan) -> Probes -> LLM-A (answer) -> LLM-B (audit) -> approve/fix/retry
//!
//! Key v0.12.2 changes:
//! - Iteration-aware prompting (forces answer on iteration 2+)
//! - Fallback answer extraction from raw evidence
//! - More aggressive answer generation when evidence exists

use super::fallback;
use super::llm_client::OllamaClient;
use super::probe_executor;
use anna_common::{
    generate_llm_a_prompt_with_iteration, generate_llm_b_prompt, AuditScores, AuditVerdict,
    ConfidenceLevel, FinalAnswer, LoopState, ProbeCatalog, ProbeEvidenceV10, ReliabilityScores,
    MAX_LOOPS,
};
use anyhow::Result;
use tracing::{debug, info, warn};

/// Answer engine - orchestrates the LLM-A/LLM-B loop
pub struct AnswerEngine {
    llm_client: OllamaClient,
    catalog: ProbeCatalog,
    model_name: String,
}

impl AnswerEngine {
    pub fn new(model: Option<String>) -> Self {
        let model_name = model.clone().unwrap_or_else(|| "llama3.2:3b".to_string());
        Self {
            llm_client: OllamaClient::new(model),
            catalog: ProbeCatalog::standard(),
            model_name,
        }
    }

    /// Get the model name being used
    pub fn model(&self) -> &str {
        &self.model_name
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

        let mut loop_state = LoopState::default();
        let mut evidence: Vec<ProbeEvidenceV10> = vec![];
        let mut last_draft_answer: Option<String> = None;
        let mut last_scores: Option<AuditScores> = None;

        // Main orchestration loop
        while loop_state.can_continue() {
            loop_state.next_iteration();
            info!("Loop iteration {}/{}", loop_state.iteration, MAX_LOOPS);

            // Step 1: Call LLM-A with iteration awareness
            let llm_a_prompt = generate_llm_a_prompt_with_iteration(
                question,
                &self.catalog.available_probes(),
                &evidence,
                loop_state.iteration,
            );
            let llm_a_response = self.llm_client.call_llm_a(&llm_a_prompt).await?;

            info!(
                "🤖  LLM-A parsed: intent={}, probes={}, draft={}, needs_more={}, refuse={}",
                llm_a_response.plan.intent,
                llm_a_response.plan.probe_requests.len(),
                llm_a_response.draft_answer.is_some(),
                llm_a_response.needs_more_probes,
                llm_a_response.refuse_to_answer
            );
            if let Some(ref draft) = llm_a_response.draft_answer {
                info!(
                    "📄  Draft answer: {}",
                    &draft.text[..200.min(draft.text.len())]
                );
            }

            // Check for immediate refusal
            if llm_a_response.refuse_to_answer {
                // Only refuse if no evidence at all - otherwise try partial answer
                if evidence.is_empty() {
                    loop_state.mark_refused();
                    return Ok(self.build_refusal(
                        question,
                        llm_a_response
                            .refusal_reason
                            .as_deref()
                            .unwrap_or("Unable to answer"),
                        &evidence,
                        loop_state.iteration,
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
                    let new_evidence =
                        probe_executor::execute_probes(&self.catalog, &valid_probes).await;
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

            // Store for potential partial answer
            last_draft_answer = Some(draft_answer.text.clone());

            let self_scores = llm_a_response
                .self_scores
                .unwrap_or_else(|| ReliabilityScores::new(0.5, 0.5, 0.5));

            // Step 4: Call LLM-B to audit
            let llm_b_prompt =
                generate_llm_b_prompt(question, &draft_answer, &evidence, &self_scores);
            let llm_b_response = self.llm_client.call_llm_b(&llm_b_prompt).await?;

            info!(
                "🔍  LLM-B parsed: verdict={:?}, score={:.2}, problems={}",
                llm_b_response.verdict,
                llm_b_response.scores.overall,
                llm_b_response.problems.len()
            );
            if let Some(ref fix) = llm_b_response.fixed_answer {
                info!("🔧  Fixed answer: {}", &fix[..200.min(fix.len())]);
            }

            loop_state.record_score(llm_b_response.scores.overall);
            last_scores = Some(llm_b_response.scores.clone());

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
                AuditVerdict::FixAndAccept => {
                    // LLM-B provided a fixed answer
                    loop_state.mark_approved();
                    let answer_text = llm_b_response
                        .fixed_answer
                        .as_deref()
                        .unwrap_or(&draft_answer.text);
                    return Ok(self.build_final_answer(
                        question,
                        answer_text,
                        evidence,
                        llm_b_response.scores,
                        loop_state.iteration,
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
                        return Ok(self.build_refusal(
                            question,
                            reason,
                            &evidence,
                            loop_state.iteration,
                        ));
                    }
                    // If we have evidence, try partial answer with low confidence
                    warn!("LLM-B wants to refuse but we have evidence - will try partial answer");
                    last_scores = Some(AuditScores::new(0.5, 0.5, 0.5));
                    // Continue to see if we can improve
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
                    // Continue to next iteration
                }
            }
        }

        // Loop exhausted - provide partial answer instead of refusal
        loop_state.mark_exhausted();

        // If we have a draft answer, return it with honest low confidence
        if let Some(answer_text) = last_draft_answer {
            let scores = last_scores.unwrap_or_else(|| AuditScores::new(0.5, 0.5, 0.5));
            info!(
                "Max loops reached - returning partial answer with confidence {:.2}",
                scores.overall
            );
            return Ok(self.build_partial_answer(
                question,
                &answer_text,
                evidence,
                scores,
                loop_state.iteration,
            ));
        }

        // No draft answer - try to extract basic facts from evidence as fallback
        if !evidence.is_empty() {
            if let Some(fallback) = fallback::extract_fallback_answer(question, &evidence) {
                info!("Using fallback answer extracted from evidence");
                return Ok(self.build_partial_answer(
                    question,
                    &fallback,
                    evidence,
                    AuditScores::new(0.6, 0.6, 0.6),
                    loop_state.iteration,
                ));
            }
        }

        // No draft answer at all - this is a true refusal
        Ok(self.build_refusal(
            question,
            "Could not generate any answer after maximum iterations",
            &evidence,
            loop_state.iteration,
        ))
    }

    /// Build a final answer (approved)
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
            model_used: Some(self.model_name.clone()),
        }
    }

    /// Build a partial answer (max loops reached but have evidence)
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
            model_used: Some(self.model_name.clone()),
        }
    }

    /// Build a refusal answer (truly cannot answer)
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
            model_used: Some(self.model_name.clone()),
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

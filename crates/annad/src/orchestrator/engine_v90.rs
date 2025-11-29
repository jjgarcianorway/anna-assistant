//! Answer Engine v0.90.0 - Unified Orchestration Architecture
//!
//! Implements the specification from ANNA_SPEC.md:
//! 1. Brain Fast Path (NO LLMs) for simple questions (<150ms)
//! 2. Junior Planning (first LLM call) - discover needed probes
//! 3. Run probes (exactly once)
//! 4. Junior Draft Answer (second LLM call) - with evidence
//! 5. Senior Audit (optional for simple questions with score >= 80)
//! 6. Final Answer Assembly
//! 7. XP/Trust updates for all three actors
//!
//! Key constraints:
//! - Max 2 Junior calls, 1 Senior call
//! - Hard 10s time budget
//! - No infinite loops
//! - No repeated identical prompts

use super::llm_client::OllamaClient;
use super::probe_executor;
use anna_common::{
    AuditScores, ConfidenceLevel, DebugEventEmitter, FinalAnswer,
    ProbeCatalog, ProbeEvidenceV10,
    // v0.90.0: XP events
    XpEvent, XpEventType, XpLog,
    // v0.80.0: LLM prompts (reuse)
    generate_junior_prompt_v80, generate_senior_prompt_v80, ProbeSummary,
    // Probe summary helpers
    summarize_cpu_from_text, summarize_mem_from_text,
    // Brain fast path (free function, not a struct)
    try_fast_answer, FastAnswer,
};
use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

// ============================================================================
// Constants
// ============================================================================

/// v0.90.0: Hard time budget (10 seconds for full orchestration)
const ORCHESTRATION_TIMEOUT_SECS: u64 = 10;

/// v0.90.0: Brain fast path timeout (150ms target)
const BRAIN_TIMEOUT_MS: u64 = 150;

/// v0.90.0: High confidence threshold - skip Senior if Junior >= 80%
const SKIP_SENIOR_THRESHOLD: f64 = 0.80;

/// v0.90.0: Origin labels
const ORIGIN_BRAIN: &str = "Brain";
const ORIGIN_JUNIOR: &str = "Junior";
const ORIGIN_SENIOR: &str = "Senior";

// ============================================================================
// Answer Origin
// ============================================================================

/// v0.90.0: Track where the answer came from
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerOrigin {
    Brain,
    Junior,
    Senior,
}

impl AnswerOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnswerOrigin::Brain => ORIGIN_BRAIN,
            AnswerOrigin::Junior => ORIGIN_JUNIOR,
            AnswerOrigin::Senior => ORIGIN_SENIOR,
        }
    }
}

// ============================================================================
// Unified Engine v0.90.0
// ============================================================================

/// v0.90.0: Unified Answer Engine
///
/// Implements the exact flow specified in ANNA_SPEC.md:
/// Brain → Junior Plan → Probes → Junior Draft → Senior Audit → Answer
pub struct UnifiedEngine {
    llm_client: OllamaClient,
    catalog: ProbeCatalog,
    timeout: Duration,
    xp_log: XpLog,
    /// Current question for XP event logging
    current_question: String,
}

impl UnifiedEngine {
    /// Create engine with role-specific models
    pub fn new(junior_model: Option<String>, senior_model: Option<String>) -> Self {
        Self {
            llm_client: OllamaClient::with_role_models(junior_model, senior_model),
            catalog: ProbeCatalog::standard(),
            timeout: Duration::from_secs(ORCHESTRATION_TIMEOUT_SECS),
            xp_log: XpLog::new(),
            current_question: String::new(),
        }
    }

    /// Get the junior model name
    pub fn junior_model(&self) -> &str {
        self.llm_client.junior_model()
    }

    /// Get the senior model name
    pub fn senior_model(&self) -> &str {
        self.llm_client.senior_model()
    }

    /// Process a question following the v0.90.0 unified flow
    ///
    /// STEP 0: Start timer
    /// STEP 1: Brain Fast Path
    /// STEP 2: Junior Planning (if Brain failed)
    /// STEP 3: Run probes
    /// STEP 4: Junior Draft Answer
    /// STEP 5: Senior Audit (optional)
    /// STEP 6: Final Answer Assembly
    /// STEP 7: XP/Trust updates
    pub async fn process_question(&mut self, question: &str) -> Result<FinalAnswer> {
        self.process_question_with_emitter(question, None).await
    }

    /// Process question with optional debug event emitter
    pub async fn process_question_with_emitter(
        &mut self,
        question: &str,
        _emitter: Option<&dyn DebugEventEmitter>,
    ) -> Result<FinalAnswer> {
        // ================================================================
        // STEP 0: Start timer and bookkeeping
        // ================================================================
        let start_time = Instant::now();
        info!("[*]  v0.90.0 Unified Engine: {}", question);

        // Store current question for XP event logging
        self.current_question = question.to_string();

        let mut junior_total_ms: u64 = 0;
        let mut senior_total_ms: u64 = 0;
        let mut _origin = AnswerOrigin::Brain;
        let probe_ids: Vec<String>;

        // ================================================================
        // STEP 1: Brain Fast Path (NO LLMs)
        // ================================================================
        let brain_start = Instant::now();
        if let Some(brain_answer) = try_fast_answer(question) {
            let brain_ms = brain_start.elapsed().as_millis() as u64;
            info!(
                "[+]  Brain fast path succeeded in {}ms",
                brain_ms
            );

            // Record Anna XP for self-solve
            self.record_xp_event(XpEventType::BrainSelfSolve);

            return Ok(self.build_brain_answer(
                question,
                &brain_answer.text,
                brain_answer.reliability,
                start_time.elapsed(),
            ));
        }
        let brain_ms = brain_start.elapsed().as_millis() as u64;
        info!("[*]  Brain fast path did not match ({}ms)", brain_ms);

        // ================================================================
        // STEP 2: Junior Planning (First LLM call)
        // ================================================================
        // Check timeout before calling LLM
        if self.is_timed_out(&start_time) {
            return Ok(self.build_timeout_answer(question, start_time.elapsed(), junior_total_ms, senior_total_ms));
        }

        let available_probes: Vec<String> = self.catalog.available_probes()
            .iter()
            .map(|p| p.probe_id.clone())
            .collect();

        let junior_prompt_1 = generate_junior_prompt_v80(question, &available_probes, &[]);
        info!("[J1] Junior planning ({} chars)", junior_prompt_1.len());

        let junior_start_1 = Instant::now();
        let (junior_response_1, _raw) = match self.llm_client.call_junior_v80(&junior_prompt_1).await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("[!]  Junior planning failed: {}", e);
                self.record_xp_event(XpEventType::JuniorBadCommand);
                return Ok(self.build_error_answer(question, &e.to_string(), start_time.elapsed()));
            }
        };
        junior_total_ms += junior_start_1.elapsed().as_millis() as u64;

        // Extract requested probes
        probe_ids = junior_response_1
            .probe_requests
            .iter()
            .map(|p| p.probe_id.clone())
            .collect();
        info!("[J1] Junior requested {} probes: {:?}", probe_ids.len(), probe_ids);

        // ================================================================
        // STEP 3: Run probes (exactly once)
        // ================================================================
        if self.is_timed_out(&start_time) {
            return Ok(self.build_timeout_answer(question, start_time.elapsed(), junior_total_ms, senior_total_ms));
        }

        let mut evidence: Vec<ProbeEvidenceV10> = Vec::new();
        let mut summaries: Vec<ProbeSummary> = Vec::new();

        if !probe_ids.is_empty() {
            let valid_probes: Vec<String> = probe_ids
                .iter()
                .filter(|id| self.catalog.is_valid(id))
                .cloned()
                .collect();

            if !valid_probes.is_empty() {
                info!("[P]  Executing {} probes", valid_probes.len());
                evidence = probe_executor::execute_probes(&self.catalog, &valid_probes).await;

                // Precompute compact summaries for Junior
                for ev in &evidence {
                    if let Some(raw) = &ev.raw {
                        let compact = self.precompute_summary(&ev.probe_id, raw);
                        summaries.push(ProbeSummary::new(&ev.probe_id, &compact));
                    }
                }
                info!("[P]  Collected {} evidence items", evidence.len());
            }
        }

        // ================================================================
        // STEP 4: Junior Draft Answer (Second LLM call)
        // ================================================================
        if self.is_timed_out(&start_time) {
            return Ok(self.build_timeout_answer(question, start_time.elapsed(), junior_total_ms, senior_total_ms));
        }

        let junior_prompt_2 = generate_junior_prompt_v80(question, &available_probes, &summaries);
        info!("[J2] Junior drafting ({} chars)", junior_prompt_2.len());

        let junior_start_2 = Instant::now();
        let (junior_response_2, _raw) = match self.llm_client.call_junior_v80(&junior_prompt_2).await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("[!]  Junior draft failed: {}", e);
                self.record_xp_event(XpEventType::JuniorBadCommand);
                return Ok(self.build_error_answer(question, &e.to_string(), start_time.elapsed()));
            }
        };
        junior_total_ms += junior_start_2.elapsed().as_millis() as u64;

        // Check if Junior has a draft answer
        let junior_had_draft = junior_response_2.draft_answer.is_some()
            && junior_response_2.draft_answer.as_ref()
                .map(|d| !d.text.is_empty() && d.text != "null")
                .unwrap_or(false);

        let draft_text = match &junior_response_2.draft_answer {
            Some(draft) if draft.text != "null" && !draft.text.is_empty() => draft.text.clone(),
            _ => {
                warn!("[!]  Junior did not provide draft answer");
                self.record_xp_event(XpEventType::JuniorBadCommand);
                return Ok(self.build_refusal(
                    question,
                    "Could not generate answer - Junior failed to draft",
                    &evidence,
                    &probe_ids,
                    junior_had_draft,
                    junior_total_ms,
                    senior_total_ms,
                ));
            }
        };

        // Get Junior's confidence score (v0.80.0 DraftAnswerV80 doesn't have confidence field)
        // Use a default estimate based on whether evidence was collected
        let junior_confidence = if !evidence.is_empty() { 0.75 } else { 0.5 };

        info!(
            "[J2] Junior draft: {} chars, confidence={:.0}%",
            draft_text.len(),
            junior_confidence * 100.0
        );

        // ================================================================
        // STEP 5: Senior Audit (optional for high-confidence simple questions)
        // ================================================================
        // For simple domains with Junior confidence >= 80%, skip Senior
        let is_simple_domain = self.is_simple_domain(question);
        let skip_senior = is_simple_domain && junior_confidence >= SKIP_SENIOR_THRESHOLD;

        if skip_senior {
            info!(
                "[S]  Skipping Senior (simple domain, confidence={:.0}%)",
                junior_confidence * 100.0
            );
            _origin = AnswerOrigin::Junior;
            self.record_xp_event(XpEventType::JuniorCleanProposal);

            return Ok(self.build_junior_answer(
                question,
                &draft_text,
                junior_confidence,
                &evidence,
                &probe_ids,
                junior_had_draft,
                junior_total_ms,
                0,
                start_time.elapsed(),
            ));
        }

        // Check timeout before Senior call
        if self.is_timed_out(&start_time) {
            // Return Junior's answer with lower confidence since no Senior review
            info!("[!]  Timeout before Senior - returning Junior answer");
            _origin = AnswerOrigin::Junior;
            self.record_xp_event(XpEventType::LlmTimeoutFallback);
            return Ok(self.build_junior_answer(
                question,
                &draft_text,
                junior_confidence * 0.8, // Reduce confidence without Senior
                &evidence,
                &probe_ids,
                junior_had_draft,
                junior_total_ms,
                0,
                start_time.elapsed(),
            ));
        }

        // Call Senior for audit
        let probe_summary_pairs: Vec<(&str, &str)> = summaries
            .iter()
            .map(|s| (s.probe_id.as_str(), s.compact_json.as_str()))
            .collect();

        let senior_prompt = generate_senior_prompt_v80(question, &draft_text, &probe_summary_pairs);
        info!("[S]  Senior auditing ({} chars)", senior_prompt.len());

        let senior_start = Instant::now();
        let (senior_response, _raw) = match self.llm_client.call_senior_v80(&senior_prompt).await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("[!]  Senior audit failed: {}", e);
                // Fall back to Junior answer with reduced confidence
                _origin = AnswerOrigin::Junior;
                self.record_xp_event(XpEventType::LlmTimeoutFallback);
                return Ok(self.build_junior_answer(
                    question,
                    &draft_text,
                    junior_confidence * 0.7,
                    &evidence,
                    &probe_ids,
                    junior_had_draft,
                    junior_total_ms,
                    0,
                    start_time.elapsed(),
                ));
            }
        };
        senior_total_ms = senior_start.elapsed().as_millis() as u64;
        _origin = AnswerOrigin::Senior;

        // ================================================================
        // STEP 6: Final Answer Assembly
        // ================================================================
        let senior_verdict = senior_response.verdict.clone();
        let final_text = match senior_verdict.as_str() {
            "approve" => {
                self.record_xp_event(XpEventType::SeniorGreenApproval);
                self.record_xp_event(XpEventType::JuniorCleanProposal);
                senior_response.fixed_answer.unwrap_or(draft_text)
            }
            "fix_and_accept" => {
                // Senior fixed it but accepted - minor adjustment needed
                self.record_xp_event(XpEventType::SeniorGreenApproval);
                self.record_xp_event(XpEventType::SeniorRepeatedFix);
                senior_response.fixed_answer.unwrap_or(draft_text)
            }
            "refuse" => {
                self.record_xp_event(XpEventType::LowReliabilityRefusal);
                self.record_xp_event(XpEventType::JuniorBadCommand);
                return Ok(self.build_refusal(
                    question,
                    &senior_response.fixed_answer.unwrap_or_else(|| "Senior refused".to_string()),
                    &evidence,
                    &probe_ids,
                    junior_had_draft,
                    junior_total_ms,
                    senior_total_ms,
                ));
            }
            _ => draft_text,
        };

        let confidence = senior_response.scores_overall.max(0.0).min(1.0);
        let confidence_level = ConfidenceLevel::from_score(confidence);
        let elapsed = start_time.elapsed();

        info!(
            "[+]  Done in {:.2}s - verdict={}, confidence={:.0}%",
            elapsed.as_secs_f64(),
            senior_verdict,
            confidence * 100.0
        );

        // ================================================================
        // STEP 7: XP/Trust updates (done via record_xp_event calls above)
        // ================================================================
        // Note: XP events are saved automatically on each append() call

        Ok(FinalAnswer {
            question: question.to_string(),
            answer: final_text,
            is_refusal: false,
            citations: evidence,
            scores: AuditScores::new(confidence, confidence, confidence),
            confidence_level,
            problems: vec![],
            loop_iterations: 2, // Junior×2 + Senior×1
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms: junior_total_ms,
            senior_ms: senior_total_ms,
            junior_probes: probe_ids,
            junior_had_draft,
            senior_verdict: Some(senior_verdict),
            failure_cause: None,
        })
    }

    // ========================================================================
    // Brain Fast Path
    // ========================================================================

    /// Build answer from Brain fast path
    fn build_brain_answer(
        &self,
        question: &str,
        answer_text: &str,
        reliability: f64,
        _elapsed: Duration,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: answer_text.to_string(),
            is_refusal: false,
            citations: vec![],
            scores: AuditScores::new(reliability, reliability, reliability),
            confidence_level: ConfidenceLevel::from_score(reliability),
            problems: vec![],
            loop_iterations: 0,
            model_used: Some(ORIGIN_BRAIN.to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms: 0,
            senior_ms: 0,
            junior_probes: vec![],
            junior_had_draft: false,
            senior_verdict: None,
            failure_cause: None,
        }
    }

    // ========================================================================
    // Answer Building Helpers
    // ========================================================================

    /// Check if we've exceeded the time budget
    fn is_timed_out(&self, start_time: &Instant) -> bool {
        start_time.elapsed() > self.timeout
    }

    /// Check if the question is in a simple domain (hardware, RAM, CPU)
    fn is_simple_domain(&self, question: &str) -> bool {
        let q = question.to_lowercase();
        q.contains("cpu") || q.contains("ram") || q.contains("memory")
            || q.contains("disk") || q.contains("storage")
            || q.contains("core") || q.contains("thread")
    }

    /// Precompute compact JSON summary from raw probe output
    fn precompute_summary(&self, probe_id: &str, raw: &str) -> String {
        match probe_id {
            "cpu.info" => {
                let cpu = summarize_cpu_from_text(raw);
                cpu.to_compact_json()
            }
            "mem.info" => {
                let mem = summarize_mem_from_text(raw);
                mem.to_compact_json()
            }
            "disk.lsblk" => {
                let device_count = raw.lines()
                    .filter(|l| l.trim().starts_with("sd") || l.trim().starts_with("nvme"))
                    .count();
                format!(r#"{{"devices":{}}}"#, device_count)
            }
            "hardware.gpu" => {
                let has_nvidia = raw.to_lowercase().contains("nvidia");
                let has_amd = raw.to_lowercase().contains("amd") || raw.to_lowercase().contains("radeon");
                let has_intel = raw.to_lowercase().contains("intel");
                format!(
                    r#"{{"nvidia":{},"amd":{},"intel":{}}}"#,
                    has_nvidia, has_amd, has_intel
                )
            }
            _ => {
                let preview: String = raw.chars().take(200).collect();
                format!(r#"{{"preview":"{}"}}"#, preview.replace('"', "'").replace('\n', " "))
            }
        }
    }

    /// Build Junior-only answer (when Senior is skipped)
    fn build_junior_answer(
        &self,
        question: &str,
        answer_text: &str,
        confidence: f64,
        evidence: &[ProbeEvidenceV10],
        probe_ids: &[String],
        junior_had_draft: bool,
        junior_ms: u64,
        senior_ms: u64,
        _elapsed: Duration,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: answer_text.to_string(),
            is_refusal: false,
            citations: evidence.to_vec(),
            scores: AuditScores::new(confidence, confidence, confidence),
            confidence_level: ConfidenceLevel::from_score(confidence),
            problems: vec![],
            loop_iterations: 2,
            model_used: Some(self.junior_model().to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms,
            senior_ms,
            junior_probes: probe_ids.to_vec(),
            junior_had_draft,
            senior_verdict: Some("skipped".to_string()),
            failure_cause: None,
        }
    }

    /// Build a refusal answer
    fn build_refusal(
        &self,
        question: &str,
        reason: &str,
        evidence: &[ProbeEvidenceV10],
        probe_ids: &[String],
        junior_had_draft: bool,
        junior_ms: u64,
        senior_ms: u64,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: format!("I cannot answer this question.\n\nReason: {}", reason),
            is_refusal: true,
            citations: evidence.to_vec(),
            scores: AuditScores::new(0.0, 0.0, 0.0),
            confidence_level: ConfidenceLevel::Red,
            problems: vec![reason.to_string()],
            loop_iterations: 1,
            model_used: Some(self.senior_model().to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms,
            senior_ms,
            junior_probes: probe_ids.to_vec(),
            junior_had_draft,
            senior_verdict: Some("refuse".to_string()),
            failure_cause: Some("unsupported_domain".to_string()),
        }
    }

    /// Build a timeout answer
    fn build_timeout_answer(
        &self,
        question: &str,
        elapsed: Duration,
        junior_ms: u64,
        senior_ms: u64,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: format!(
                "Sorry, I couldn't answer in time ({:.1}s exceeded {}s budget).\n\n\
                 Try asking again or check system load.",
                elapsed.as_secs_f64(),
                ORCHESTRATION_TIMEOUT_SECS
            ),
            is_refusal: true,
            citations: vec![],
            scores: AuditScores::new(0.0, 0.0, 0.0),
            confidence_level: ConfidenceLevel::Red,
            problems: vec![format!("Timeout after {:.1}s", elapsed.as_secs_f64())],
            loop_iterations: 0,
            model_used: None,
            clarification_needed: None,
            debug_trace: None,
            junior_ms,
            senior_ms,
            junior_probes: vec![],
            junior_had_draft: false,
            senior_verdict: None,
            failure_cause: Some("timeout_or_latency".to_string()),
        }
    }

    /// Build an error answer
    fn build_error_answer(
        &self,
        question: &str,
        error: &str,
        _elapsed: Duration,
    ) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: format!(
                "Sorry, an error occurred while processing your question.\n\nError: {}",
                error
            ),
            is_refusal: true,
            citations: vec![],
            scores: AuditScores::new(0.0, 0.0, 0.0),
            confidence_level: ConfidenceLevel::Red,
            problems: vec![error.to_string()],
            loop_iterations: 0,
            model_used: None,
            clarification_needed: None,
            debug_trace: None,
            junior_ms: 0,
            senior_ms: 0,
            junior_probes: vec![],
            junior_had_draft: false,
            senior_verdict: None,
            failure_cause: Some("llm_error".to_string()),
        }
    }

    // ========================================================================
    // XP Tracking
    // ========================================================================

    /// Record an XP event for the current question
    fn record_xp_event(&self, event_type: XpEventType) {
        let event = XpEvent::new(event_type, &self.current_question);
        if let Err(e) = self.xp_log.append(&event) {
            warn!("[!]  Failed to record XP event: {}", e);
        }
    }

    /// Check if LLM backend is available
    pub async fn is_available(&self) -> bool {
        self.llm_client.is_available().await
    }
}

impl Default for UnifiedEngine {
    fn default() -> Self {
        Self::new(None, None)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = UnifiedEngine::default();
        assert_eq!(engine.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_simple_domain_detection() {
        let engine = UnifiedEngine::default();
        assert!(engine.is_simple_domain("how much RAM do I have?"));
        assert!(engine.is_simple_domain("what CPU model?"));
        assert!(engine.is_simple_domain("how many cores and threads?"));
        assert!(engine.is_simple_domain("disk space"));
        assert!(!engine.is_simple_domain("what's the weather?"));
    }

    #[test]
    fn test_precompute_cpu_summary() {
        let engine = UnifiedEngine::default();
        let raw = r#"
processor	: 0
model name	: AMD Ryzen 9 9950X 16-Core Processor
cpu cores	: 16
processor	: 1
model name	: AMD Ryzen 9 9950X 16-Core Processor
"#;
        let summary = engine.precompute_summary("cpu.info", raw);
        assert!(summary.contains("physical_cores"));
    }

    #[test]
    fn test_timeout_check() {
        let engine = UnifiedEngine::default();
        let start = Instant::now();
        // Should not be timed out immediately
        assert!(!engine.is_timed_out(&start));
    }
}

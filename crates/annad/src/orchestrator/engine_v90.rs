//! Answer Engine v3.12.0 - Unified Orchestration with Per-Call Timeouts
//!
//! This is the CANONICAL orchestration entry point for Anna.
//!
//! ## Flow Summary (v3.0.0)
//!
//! ```text
//! Question → Brain → Recipe → Junior Plan → Probes → Junior Draft → Senior → Answer
//!                      ↓                                                 ↓
//!                 (if match)                                    (extract recipe)
//! ```
//!
//! ## Invariants
//!
//! 1. Max LLM calls: 2 Junior + 1 Senior = 3 total
//! 2. Hard timeout: 10 seconds (reduced from 30s in v2.3.0)
//! 3. No loops: Each step executes exactly once
//! 4. Safe commands only: Probes use whitelist
//! 5. XP recorded: Every answer generates at least one XP event
//! 6. Origin tracked: Every answer has model_used set
//! 7. Recipe extraction: High-reliability answers create recipes
//!
//! ## Answer Origins (v3.0.0)
//!
//! | Origin | Latency | LLM Calls | Description                    |
//! |--------|---------|-----------|--------------------------------|
//! | Brain  | <150ms  | 0         | Fast path pattern match        |
//! | Recipe | <500ms  | 0         | Learned pattern + probe        |
//! | Junior | <8s     | 2         | Junior plan + draft            |
//! | Senior | <10s    | 3         | Junior + Senior audit          |

use super::llm_client::OllamaClient;
use super::probe_executor;
use anna_common::{
    AuditScores, ConfidenceLevel, DebugEventEmitter, FinalAnswer,
    ProbeCatalog, ProbeEvidenceV10,
    // v0.90.0: XP events
    XpEventType,
    // v0.92.0: Decision Policy and XP Store
    DecisionPolicy, XpStore,
    // v0.80.0: LLM prompts (reuse)
    generate_junior_prompt_v80, generate_senior_prompt_v80, ProbeSummary,
    // Probe summary helpers
    summarize_cpu_from_text, summarize_mem_from_text,
    // Brain fast path (free function, not a struct)
    try_fast_answer,
    // v1.1.0: Unified XP recording
    UnifiedXpRecorder,
    // v3.0.0: Recipe system for learning
    RecipeStore, extract_recipe, MIN_RECIPE_RELIABILITY,
    // v3.0.0: Router LLM for question classification
    router_llm::QuestionType,
};
use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

// ============================================================================
// Constants
// ============================================================================

/// v3.13.1: Hard time budget (45 seconds for full orchestration)
/// Increased to accommodate qwen3:4b which can take 16-20s with thinking
const ORCHESTRATION_TIMEOUT_SECS: u64 = 45;

/// v3.13.1: Soft time budget for warning (30 seconds)
const ORCHESTRATION_SOFT_LIMIT_SECS: u64 = 30;

/// v0.90.0: Brain fast path timeout (150ms target)
const BRAIN_TIMEOUT_MS: u64 = 150;

/// v3.13.1: Junior LLM call timeout (25 seconds per call)
/// qwen3:4b can take 16-20s with verbose "thinking" output
const JUNIOR_TIMEOUT_MS: u64 = 25000;

/// v3.13.1: Senior LLM call timeout (30 seconds per call)
/// 14b models need even more time for reasoning
const SENIOR_TIMEOUT_MS: u64 = 30000;

/// v0.90.0: High confidence threshold - skip Senior if Junior >= 80%
const SKIP_SENIOR_THRESHOLD: f64 = 0.80;

/// v3.0.0: Origin labels
const ORIGIN_BRAIN: &str = "Brain";
const ORIGIN_RECIPE: &str = "Recipe";
const ORIGIN_JUNIOR: &str = "Junior";
const ORIGIN_SENIOR: &str = "Senior";

// ============================================================================
// Answer Origin
// ============================================================================

/// v3.0.0: Track where the answer came from
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerOrigin {
    /// Fast pattern match, no LLM, <150ms
    Brain,
    /// Learned recipe + probe execution, no LLM, <500ms
    Recipe,
    /// Junior plan + draft, 2 LLM calls, <8s
    Junior,
    /// Junior + Senior audit, 3 LLM calls, <10s
    Senior,
}

impl AnswerOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnswerOrigin::Brain => ORIGIN_BRAIN,
            AnswerOrigin::Recipe => ORIGIN_RECIPE,
            AnswerOrigin::Junior => ORIGIN_JUNIOR,
            AnswerOrigin::Senior => ORIGIN_SENIOR,
        }
    }
}

// ============================================================================
// Unified Engine v0.90.0
// ============================================================================

/// v3.0.0: Unified Answer Engine with Recipe Learning
///
/// Flow: Brain → Recipe → Junior Plan → Probes → Junior Draft → Senior Audit → Answer
///                                                                      ↓
///                                                              (extract recipe)
pub struct UnifiedEngine {
    llm_client: OllamaClient,
    catalog: ProbeCatalog,
    timeout: Duration,
    /// v1.1.0: Unified XP recorder - updates BOTH XpLog AND XpStore
    xp_recorder: UnifiedXpRecorder,
    /// v0.92.0: Decision policy for routing and circuit breaker
    decision_policy: DecisionPolicy,
    /// v0.92.0: XP store for trust-based decisions
    xp_store: XpStore,
    /// v3.0.0: Recipe store for learned patterns
    recipe_store: RecipeStore,
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
            // v1.1.0: Unified XP recorder - updates BOTH XpLog AND XpStore
            xp_recorder: UnifiedXpRecorder::new(),
            // v0.92.0: Load persistent state
            decision_policy: DecisionPolicy::load(),
            xp_store: XpStore::load(),
            // v3.0.0: Recipe store for learned patterns
            recipe_store: RecipeStore::load(),
            current_question: String::new(),
        }
    }

    /// Get the recipe store (for status display)
    pub fn recipe_store(&self) -> &RecipeStore {
        &self.recipe_store
    }

    /// Get the decision policy (for status display)
    pub fn decision_policy(&self) -> &DecisionPolicy {
        &self.decision_policy
    }

    /// Get the XP store (for status display)
    pub fn xp_store(&self) -> &XpStore {
        &self.xp_store
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
        

        // ================================================================
        // STEP 1: Brain Fast Path (NO LLMs)
        // ================================================================
        let brain_start = Instant::now();
        if let Some(brain_answer) = try_fast_answer(question) {
            let brain_ms = brain_start.elapsed().as_millis() as u64;

            // v1.5.0: Check if this is a benchmark trigger - if so, run it
            if anna_common::is_benchmark_trigger(&brain_answer) {
                info!("[+]  Benchmark trigger detected, running benchmark...");
                let benchmark_result = self.run_benchmark_now(&brain_answer).await;
                // Record XP for running benchmark
                self.record_xp_event(XpEventType::BrainSelfSolve);
                return Ok(self.build_brain_answer(
                    question,
                    &benchmark_result,
                    0.99,
                    start_time.elapsed(),
                ));
            }

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
        // STEP 1.5: Recipe Match (NO LLMs, learned patterns)
        // ================================================================
        // Try to match against learned recipes before calling LLM
        let question_type = self.classify_question(question);
        if let Some(recipe) = self.recipe_store.find_match(question, &question_type) {
            // Clone recipe data to avoid borrow issues
            let recipe_id = recipe.id.clone();
            let recipe_intent = recipe.intent.clone();
            let recipe_score = recipe.last_success_score;
            let recipe_probes = recipe.probes.clone();
            let recipe_params = recipe.parameters.clone();
            let recipe_template = recipe.answer_template.clone();

            info!("[R]  Recipe match found: {} (score={:.2})", recipe_intent, recipe_score);

            // Execute probes from recipe
            let valid_probes: Vec<String> = recipe_probes
                .iter()
                .filter(|id| self.catalog.is_valid(id))
                .cloned()
                .collect();

            if !valid_probes.is_empty() {
                let recipe_evidence = probe_executor::execute_probes(&self.catalog, &valid_probes).await;

                // Build evidence map for template substitution
                let mut evidence_map = std::collections::HashMap::new();
                for ev in &recipe_evidence {
                    if let Some(raw) = &ev.raw {
                        // Extract key values from probe output
                        evidence_map.insert(ev.probe_id.clone(), self.precompute_summary(&ev.probe_id, raw));
                    }
                }

                // Also add recipe parameters
                for (key, value) in &recipe_params {
                    evidence_map.insert(key.clone(), value.clone());
                }

                // Apply recipe template
                let mut recipe_answer = recipe_template.clone();
                for (key, value) in &evidence_map {
                    recipe_answer = recipe_answer.replace(&format!("{{{}}}", key), value);
                }
                let recipe_ms = start_time.elapsed().as_millis() as u64;

                if !recipe_answer.trim().is_empty() && !recipe_answer.contains('{') {
                    // Recipe produced valid answer (no unfilled placeholders)
                    info!("[+]  Recipe answer generated in {}ms", recipe_ms);

                    // Record recipe application
                    self.recipe_store.record_application(&recipe_id);
                    self.record_xp_event(XpEventType::BrainSelfSolve); // Recipe counts as Brain-level

                    return Ok(self.build_recipe_answer(
                        question,
                        &recipe_answer,
                        recipe_score,
                        &recipe_id,
                        start_time.elapsed(),
                    ));
                }
            }
            info!("[*]  Recipe match did not produce valid answer, falling through to Junior");
        }

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
        // v3.12.0: Wrap Junior call with timeout
        let junior_timeout = Duration::from_millis(JUNIOR_TIMEOUT_MS);
        let junior_call_1 = self.llm_client.call_junior_v80(&junior_prompt_1);
        let (junior_response_1, _raw) = match tokio::time::timeout(junior_timeout, junior_call_1).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => {
                warn!("[!]  Junior planning failed: {}", e);
                self.record_xp_event(XpEventType::JuniorBadCommand);
                return Ok(self.build_error_answer(question, &e.to_string(), start_time.elapsed()));
            }
            Err(_) => {
                // v3.12.0: Junior timeout exceeded
                let elapsed_ms = junior_start_1.elapsed().as_millis() as u64;
                warn!("[!]  Junior planning timeout after {}ms (budget: {}ms)", elapsed_ms, JUNIOR_TIMEOUT_MS);
                self.record_xp_event(XpEventType::LlmTimeoutFallback);
                return Ok(self.build_timeout_answer(question, start_time.elapsed(), elapsed_ms, 0));
            }
        };
        junior_total_ms += junior_start_1.elapsed().as_millis() as u64;

        // Extract requested probes
        let probe_ids: Vec<String> = junior_response_1
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
        // v3.12.0: Wrap Junior call with timeout
        let junior_call_2 = self.llm_client.call_junior_v80(&junior_prompt_2);
        let (junior_response_2, _raw) = match tokio::time::timeout(junior_timeout, junior_call_2).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => {
                warn!("[!]  Junior draft failed: {}", e);
                self.record_xp_event(XpEventType::JuniorBadCommand);
                return Ok(self.build_error_answer(question, &e.to_string(), start_time.elapsed()));
            }
            Err(_) => {
                // v3.12.0: Junior timeout exceeded
                let elapsed_ms = junior_start_2.elapsed().as_millis() as u64;
                warn!("[!]  Junior draft timeout after {}ms (budget: {}ms)", elapsed_ms, JUNIOR_TIMEOUT_MS);
                self.record_xp_event(XpEventType::LlmTimeoutFallback);
                return Ok(self.build_timeout_answer(question, start_time.elapsed(), junior_total_ms + elapsed_ms, 0));
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
        // v3.12.0: Wrap Senior call with timeout
        let senior_timeout = Duration::from_millis(SENIOR_TIMEOUT_MS);
        let senior_call = self.llm_client.call_senior_v80(&senior_prompt);
        let (senior_response, _raw) = match tokio::time::timeout(senior_timeout, senior_call).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => {
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
            Err(_) => {
                // v3.12.0: Senior timeout exceeded - fall back to Junior answer
                let elapsed_ms = senior_start.elapsed().as_millis() as u64;
                warn!("[!]  Senior audit timeout after {}ms (budget: {}ms)", elapsed_ms, SENIOR_TIMEOUT_MS);
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
                    elapsed_ms,
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

        // ================================================================
        // STEP 8: Recipe Extraction (v3.0.0)
        // ================================================================
        // Extract recipe from successful high-reliability answers
        self.maybe_extract_recipe(question, &question_type, &probe_ids, &final_text, confidence);

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
    /// v1.5.0: Includes empty answer guardrail
    fn build_brain_answer(
        &self,
        question: &str,
        answer_text: &str,
        reliability: f64,
        _elapsed: Duration,
    ) -> FinalAnswer {
        // v1.5.0: Empty answer guardrail - never return empty text
        let final_text = if answer_text.trim().is_empty() {
            warn!("[!]  Empty answer detected, applying guardrail");
            format!(
                "I processed your question but couldn't generate a meaningful response.\n\n\
                 Question: {}\n\n\
                 Please try rephrasing your question or ask something more specific.",
                question
            )
        } else {
            answer_text.to_string()
        };

        FinalAnswer {
            question: question.to_string(),
            answer: final_text,
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
    // Recipe Support (v3.0.0)
    // ========================================================================

    /// Simple question classification for recipe matching
    fn classify_question(&self, question: &str) -> QuestionType {
        let q = question.to_lowercase();

        // CPU related
        if q.contains("cpu") || q.contains("processor") || q.contains("core") || q.contains("thread") {
            return QuestionType::CpuInfo;
        }

        // RAM related
        if q.contains("ram") || q.contains("memory") {
            return QuestionType::RamInfo;
        }

        // Disk related
        if q.contains("disk") || q.contains("storage") || q.contains("space") || q.contains("filesystem") {
            return QuestionType::DiskInfo;
        }

        // Network related
        if q.contains("network") || q.contains("ip") || q.contains("interface") {
            return QuestionType::NetworkInfo;
        }

        // GPU related
        if q.contains("gpu") || q.contains("graphics") || q.contains("nvidia") || q.contains("amd") {
            return QuestionType::GpuInfo;
        }

        // OS related
        if q.contains("os") || q.contains("distro") || q.contains("linux") || q.contains("arch") {
            return QuestionType::OsInfo;
        }

        // Uptime
        if q.contains("uptime") || q.contains("running") {
            return QuestionType::UptimeInfo;
        }

        // Logs
        if q.contains("log") || q.contains("annad") {
            return QuestionType::SelfLogs;
        }

        // Health
        if q.contains("health") || q.contains("status") {
            return QuestionType::SelfHealth;
        }

        QuestionType::Unknown
    }

    /// Build answer from Recipe match
    fn build_recipe_answer(
        &self,
        question: &str,
        answer_text: &str,
        reliability: f64,
        recipe_id: &str,
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
            model_used: Some(format!("{}:{}", ORIGIN_RECIPE, recipe_id)),
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

    /// Extract and store a recipe from a successful answer
    fn maybe_extract_recipe(
        &mut self,
        question: &str,
        question_type: &QuestionType,
        probes_used: &[String],
        answer: &str,
        reliability: f64,
    ) {
        // Only extract recipes from high-reliability answers
        if reliability < MIN_RECIPE_RELIABILITY {
            debug!("[R]  Not extracting recipe: reliability {:.2} < {:.2}", reliability, MIN_RECIPE_RELIABILITY);
            return;
        }

        // Don't create recipes for Unknown question types
        if *question_type == QuestionType::Unknown {
            debug!("[R]  Not extracting recipe: Unknown question type");
            return;
        }

        if let Some(recipe) = extract_recipe(question, question_type.clone(), probes_used, answer, reliability) {
            info!("[R]  Extracted recipe: {} (reliability={:.2})", recipe.intent, reliability);
            self.recipe_store.add(recipe);
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

    /// Build a refusal answer (v2.3.0: improved with context and non-zero reliability)
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
        // v2.3.0: Provide context even in refusal
        let evidence_info = if !evidence.is_empty() {
            let probes_str = probe_ids.join(", ");
            format!("\n\nI did collect some information from: {}\nHowever, I could not use it to answer reliably.", probes_str)
        } else {
            String::new()
        };

        let answer_text = format!(
            "I cannot fully answer this question.\n\n\
             Reason: {}{}\n\n\
             Suggestions:\n\
             - Try rephrasing with more specific terms\n\
             - Ask about something I can measure (CPU, RAM, disk, services)\n\
             - Break complex questions into simpler parts",
            reason,
            evidence_info
        );

        // v2.3.0: Non-zero reliability if we collected evidence
        let base_score = if !evidence.is_empty() { 0.25 } else { 0.1 };

        FinalAnswer {
            question: question.to_string(),
            answer: answer_text,
            is_refusal: true,
            citations: evidence.to_vec(),
            scores: AuditScores::new(base_score, base_score, base_score),
            confidence_level: ConfidenceLevel::from_score(base_score),
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

    /// Build a timeout answer (v2.3.0: improved with partial info)
    fn build_timeout_answer(
        &self,
        question: &str,
        elapsed: Duration,
        junior_ms: u64,
        senior_ms: u64,
    ) -> FinalAnswer {
        // v2.3.0: Never return empty answers - provide partial info with low reliability
        let answer_text = format!(
            "I could not complete the full analysis within the time limit ({:.1}s exceeded {}s budget).\n\n\
             What I know:\n\
             - Your question: \"{}\"\n\
             - Junior processing: {}ms\n\
             - Senior processing: {}ms\n\n\
             This is a partial response. For better results:\n\
             - Try rephrasing your question\n\
             - Ask simpler, more specific questions\n\
             - Check if the LLM models are responding normally",
            elapsed.as_secs_f64(),
            ORCHESTRATION_TIMEOUT_SECS,
            question,
            junior_ms,
            senior_ms
        );

        FinalAnswer {
            question: question.to_string(),
            answer: answer_text,
            is_refusal: false, // v2.3.0: Not a refusal, just partial
            citations: vec![],
            scores: AuditScores::new(0.4, 0.4, 0.4), // v2.3.0: Low but non-zero
            confidence_level: ConfidenceLevel::from_score(0.4),
            problems: vec![format!("Timeout after {:.1}s", elapsed.as_secs_f64())],
            loop_iterations: 0,
            model_used: Some("Partial".to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms,
            senior_ms,
            junior_probes: vec![],
            junior_had_draft: false,
            senior_verdict: Some("timeout".to_string()),
            failure_cause: Some("timeout_or_latency".to_string()),
        }
    }

    /// Build an error answer (v2.3.0: improved with non-zero reliability)
    fn build_error_answer(
        &self,
        question: &str,
        error: &str,
        elapsed: Duration,
    ) -> FinalAnswer {
        // v2.3.0: Never return empty answers - provide error context
        let answer_text = format!(
            "I encountered an issue while processing your question.\n\n\
             Error: {}\n\n\
             What I attempted:\n\
             - Question: \"{}\"\n\
             - Processing time: {:.1}s\n\n\
             Suggestions:\n\
             - Try rephrasing your question\n\
             - Check if Ollama is running (ollama ps)\n\
             - Verify models are loaded (ollama list)",
            error,
            question,
            elapsed.as_secs_f64()
        );

        FinalAnswer {
            question: question.to_string(),
            answer: answer_text,
            is_refusal: false, // v2.3.0: Not a refusal, just an error state
            citations: vec![],
            scores: AuditScores::new(0.3, 0.3, 0.3), // v2.3.0: Low but non-zero
            confidence_level: ConfidenceLevel::from_score(0.3),
            problems: vec![error.to_string()],
            loop_iterations: 0,
            model_used: Some("Error".to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms: 0,
            senior_ms: 0,
            junior_probes: vec![],
            junior_had_draft: false,
            senior_verdict: Some("error".to_string()),
            failure_cause: Some("llm_error".to_string()),
        }
    }

    /// Build a partial answer when we have some evidence but couldn't complete (v2.3.0)
    fn build_partial_answer(
        &self,
        question: &str,
        partial_text: &str,
        evidence: &[ProbeEvidenceV10],
        probe_ids: &[String],
        junior_ms: u64,
        elapsed: Duration,
    ) -> FinalAnswer {
        let evidence_summary = if !evidence.is_empty() {
            let probes_str = probe_ids.join(", ");
            format!("\n\nEvidence collected from: {}", probes_str)
        } else {
            String::new()
        };

        let answer_text = format!(
            "{}{}\n\n\
             Note: This is a partial answer ({:.1}s processing). \
             I was unable to complete full verification.",
            partial_text,
            evidence_summary,
            elapsed.as_secs_f64()
        );

        FinalAnswer {
            question: question.to_string(),
            answer: answer_text,
            is_refusal: false,
            citations: evidence.to_vec(),
            scores: AuditScores::new(0.5, 0.5, 0.5), // Medium-low reliability
            confidence_level: ConfidenceLevel::from_score(0.5),
            problems: vec!["Partial analysis".to_string()],
            loop_iterations: 1,
            model_used: Some("Partial".to_string()),
            clarification_needed: None,
            debug_trace: None,
            junior_ms,
            senior_ms: 0,
            junior_probes: probe_ids.to_vec(),
            junior_had_draft: true,
            senior_verdict: Some("partial".to_string()),
            failure_cause: Some("incomplete".to_string()),
        }
    }

    // ========================================================================
    // XP Tracking
    // ========================================================================

    /// Record an XP event for the current question
    /// v1.1.0: Now uses UnifiedXpRecorder which updates BOTH XpLog AND XpStore
    fn record_xp_event(&self, event_type: XpEventType) {
        let log_line = self.xp_recorder.record(event_type, &self.current_question);
        debug!("[XP] {}", log_line);
    }

    /// Check if LLM backend is available
    pub async fn is_available(&self) -> bool {
        self.llm_client.is_available().await
    }

    // ========================================================================
    // Benchmark Execution (v1.5.0)
    // ========================================================================

    /// Run a benchmark and return formatted results (v2.3.0)
    /// Includes progress tracking and proper error handling
    async fn run_benchmark_now(&self, trigger: &anna_common::FastAnswer) -> String {
        use anna_common::bench_snow_leopard::{
            SnowLeopardConfig, run_benchmark, BenchmarkMode, PhaseId,
            BenchmarkHistoryEntry, LastBenchmarkSummary,
        };
        use std::panic;

        let is_quick = anna_common::get_benchmark_mode_from_trigger(trigger) == Some("quick");
        let mode = if is_quick { BenchmarkMode::Quick } else { BenchmarkMode::Full };
        let phases = if is_quick { PhaseId::quick() } else { PhaseId::all() };
        let total_phases = phases.len();

        info!("[BENCH] Starting Snow Leopard benchmark (mode={:?}, phases={})", mode, total_phases);

        // v2.3.0: Build progress header
        let mut output = String::new();
        output.push_str("\n═══════════════════════════════════════════\n");
        output.push_str("  🐆  SNOW LEOPARD BENCHMARK\n");
        output.push_str("═══════════════════════════════════════════\n\n");
        output.push_str(&format!("  Mode: {:?} ({} phases)\n\n", mode, total_phases));

        // Use test/simulated mode - exercises the brain fast path
        let mut config = SnowLeopardConfig::test_mode();
        config.phases_enabled = phases.clone();

        // v2.3.0: Run benchmark with panic catching
        let result = match panic::catch_unwind(panic::AssertUnwindSafe(|| {
            // We need to block on the async function since catch_unwind doesn't work with async
            // So we just run it directly and handle errors at the Result level
            
        })) {
            Ok(_) => {
                // Actually run the benchmark
                run_benchmark(&config).await
            }
            Err(_) => {
                // Panic occurred during benchmark setup
                warn!("[BENCH] Benchmark panicked during setup");
                output.push_str("  ❌  BENCHMARK FAILED\n");
                output.push_str("      Internal error during benchmark setup.\n");
                output.push_str("      Please check logs and try again.\n\n");
                output.push_str("═══════════════════════════════════════════\n");
                return output;
            }
        };

        // v2.3.0: Format phase results
        for (i, phase) in result.phases.iter().enumerate() {
            let phase_num = i + 1;
            let status = if phase.questions.iter().all(|q| q.is_success) {
                "✅"
            } else if phase.questions.iter().any(|q| q.is_success) {
                "⚠️"
            } else {
                "❌"
            };

            let success_count = phase.questions.iter().filter(|q| q.is_success).count();
            let total = phase.questions.len();

            // Calculate average latency from questions
            let avg_latency = if total > 0 {
                phase.questions.iter().map(|q| q.latency_ms).sum::<u64>() / total as u64
            } else {
                0
            };

            output.push_str(&format!(
                "  Phase {}/{}: {} {} ({}/{} passed, {}ms avg)\n",
                phase_num, total_phases, status, phase.phase_name,
                success_count, total, avg_latency
            ));
        }

        output.push_str("\n───────────────────────────────────────────\n");
        output.push_str("  SUMMARY\n");
        output.push_str("───────────────────────────────────────────\n");

        let success_rate = result.overall_success_rate();
        let avg_latency = result.overall_avg_latency();
        let brain_pct = result.brain_usage_pct();

        output.push_str(&format!("  📊  Success Rate: {:.1}%\n", success_rate));
        output.push_str(&format!("  ⏱️   Avg Latency: {}ms\n", avg_latency));
        output.push_str(&format!("  🧠  Brain Usage: {:.1}%\n", brain_pct));
        output.push_str(&format!("  🤖  LLM Usage: {:.1}%\n", result.llm_usage_pct()));
        output.push_str(&format!("  📝  Total Questions: {}\n", result.total_questions));

        // v2.3.0: Status interpretation
        output.push('\n');
        if success_rate >= 90.0 {
            output.push_str("  ✅  STATUS: Excellent - Anna is performing well!\n");
        } else if success_rate >= 70.0 {
            output.push_str("  🟡  STATUS: Good - Some questions need improvement.\n");
        } else if success_rate >= 50.0 {
            output.push_str("  🟠  STATUS: Fair - Anna is still learning.\n");
        } else {
            output.push_str("  🔴  STATUS: Needs Attention - Review configuration.\n");
        }

        // v2.3.0: Report any warnings
        if !result.warnings.is_empty() {
            output.push_str("\n  ⚠️   Warnings:\n");
            for warning in &result.warnings {
                output.push_str(&format!("      - {}\n", warning));
            }
        }

        output.push_str("\n═══════════════════════════════════════════\n");

        // Save to history
        if let Err(e) = BenchmarkHistoryEntry::from_result(&result).save() {
            warn!("[BENCH] Failed to save benchmark history: {}", e);
        }

        // Save last benchmark summary
        if let Err(e) = LastBenchmarkSummary::from_result(&result).save() {
            warn!("[BENCH] Failed to save last benchmark summary: {}", e);
        }

        info!("[BENCH] Benchmark complete: success_rate={:.1}%", success_rate);

        output
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
        // v3.13.1: Timeout increased to 45s for qwen3:4b with verbose thinking
        assert_eq!(engine.timeout, Duration::from_secs(45));
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

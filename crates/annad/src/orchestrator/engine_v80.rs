//! Answer Engine v0.81.0 - Razorback Fast Path
//!
//! Optimized orchestrator for the razorback-fast profile.
//! Goal: Complete simple questions in <5 seconds with maximum reliability.
//!
//! Flow:
//! 1. Junior call 1 (no evidence) → get probe requests
//! 2. Execute probes, precompute summaries (CpuSummary, MemSummary)
//! 3. Junior call 2 (with compact summaries) → get draft answer
//! 4. Senior call 1 (with compact summaries) → approve/fix
//!
//! Max: 2 Junior calls, 1 Senior call, 5-second budget
//!
//! v0.81.0: Timing tracking for QA (junior_ms, senior_ms, dialog_trace)

use super::llm_client::OllamaClient;
use super::probe_executor;
use anna_common::{
    AuditScores, ConfidenceLevel, FinalAnswer, ProbeCatalog, ProbeEvidenceV10,
    // v0.80.0: Razorback prompts
    generate_junior_prompt_v80, generate_senior_prompt_v80, ProbeSummary,
    LLM_A_SYSTEM_PROMPT_V80, LLM_B_SYSTEM_PROMPT_V80,
    // Probe summary helpers
    summarize_cpu_from_text, summarize_mem_from_text,
};
use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// v0.80.0: Razorback fast timeout budget
const RAZORBACK_TIMEOUT_SECS: u64 = 5;

/// Check if debug mode is enabled
fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
}

/// v0.80.0: Razorback Fast Engine
///
/// Optimized for simple hardware questions with:
/// - Minimal prompts
/// - Precomputed probe summaries
/// - No looping beyond 2+1 LLM calls
/// - 5-second budget enforcement
pub struct RazorbackEngine {
    llm_client: OllamaClient,
    catalog: ProbeCatalog,
    timeout: Duration,
}

impl RazorbackEngine {
    /// Create engine with role-specific models
    pub fn new(junior_model: Option<String>, senior_model: Option<String>) -> Self {
        Self {
            llm_client: OllamaClient::with_role_models(junior_model, senior_model),
            catalog: ProbeCatalog::standard(),
            timeout: Duration::from_secs(RAZORBACK_TIMEOUT_SECS),
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

    /// Process a question using razorback-fast path
    ///
    /// 1. Junior call 1 (no evidence) → probe requests
    /// 2. Execute probes → precompute summaries
    /// 3. Junior call 2 (with summaries) → draft answer
    /// 4. Senior call 1 → approve/fix
    pub async fn process_question(&self, question: &str) -> Result<FinalAnswer> {
        let start_time = Instant::now();
        info!("[*]  v0.81.0 Razorback fast path: {}", question);

        // v0.81.0: Track timing for QA
        let mut junior_total_ms: u64 = 0;
        let mut senior_total_ms: u64 = 0;

        // Available probes for Junior (extract probe_id strings)
        let available_probes: Vec<String> = self.catalog.available_probes()
            .iter()
            .map(|p| p.probe_id.clone())
            .collect();

        // ============================================================
        // STEP 1: Junior call 1 - discover needed probes
        // ============================================================
        let junior_prompt_1 = generate_junior_prompt_v80(question, &available_probes, &[]);

        if is_debug_mode() {
            eprintln!("[RB]  Junior prompt 1 ({} chars)", junior_prompt_1.len());
        }

        let junior_start = Instant::now();
        let (junior_response_1, _raw) = self.llm_client.call_junior_v80(&junior_prompt_1).await?;
        junior_total_ms += junior_start.elapsed().as_millis() as u64;

        // Check timeout
        if start_time.elapsed() > self.timeout {
            warn!("[RB]  Timeout after Junior call 1");
            return Ok(self.build_timeout_answer(question, start_time.elapsed(), junior_total_ms, senior_total_ms));
        }

        // Extract probe requests
        let probe_ids: Vec<String> = junior_response_1
            .probe_requests
            .iter()
            .map(|p| p.probe_id.clone())
            .collect();

        if is_debug_mode() {
            eprintln!("[RB]  Junior requested probes: {:?}", probe_ids);
        }

        // ============================================================
        // STEP 2: Execute probes and precompute summaries
        // ============================================================
        let mut evidence: Vec<ProbeEvidenceV10> = Vec::new();
        let mut summaries: Vec<ProbeSummary> = Vec::new();

        if !probe_ids.is_empty() {
            // Filter to valid probes only
            let valid_probes: Vec<String> = probe_ids
                .iter()
                .filter(|id| self.catalog.is_valid(id))
                .cloned()
                .collect();

            if !valid_probes.is_empty() {
                evidence = probe_executor::execute_probes(&self.catalog, &valid_probes).await;

                // Precompute compact summaries
                for ev in &evidence {
                    if let Some(raw) = &ev.raw {
                        let compact = self.precompute_summary(&ev.probe_id, raw);
                        summaries.push(ProbeSummary::new(&ev.probe_id, &compact));
                    }
                }
            }
        }

        // Check timeout
        if start_time.elapsed() > self.timeout {
            warn!("[RB]  Timeout after probe execution");
            return Ok(self.build_timeout_answer(question, start_time.elapsed(), junior_total_ms, senior_total_ms));
        }

        if is_debug_mode() {
            eprintln!("[RB]  Collected {} summaries", summaries.len());
        }

        // ============================================================
        // STEP 3: Junior call 2 - generate draft answer with evidence
        // ============================================================
        let junior_prompt_2 = generate_junior_prompt_v80(question, &available_probes, &summaries);

        if is_debug_mode() {
            eprintln!("[RB]  Junior prompt 2 ({} chars)", junior_prompt_2.len());
        }

        let junior_start_2 = Instant::now();
        let (junior_response_2, _raw) = self.llm_client.call_junior_v80(&junior_prompt_2).await?;
        junior_total_ms += junior_start_2.elapsed().as_millis() as u64;

        // Check timeout
        if start_time.elapsed() > self.timeout {
            warn!("[RB]  Timeout after Junior call 2");
            return Ok(self.build_timeout_answer(question, start_time.elapsed(), junior_total_ms, senior_total_ms));
        }

        // Get draft answer
        let junior_had_draft = junior_response_2.draft_answer.is_some()
            && junior_response_2.draft_answer.as_ref().map(|d| !d.text.is_empty() && d.text != "null").unwrap_or(false);

        let draft_text = match &junior_response_2.draft_answer {
            Some(draft) if draft.text != "null" && !draft.text.is_empty() => draft.text.clone(),
            _ => {
                warn!("[RB]  No draft answer from Junior - refusing");
                return Ok(self.build_refusal(
                    question,
                    "Could not generate answer",
                    &evidence,
                    &probe_ids,
                    junior_had_draft,
                    junior_total_ms,
                    senior_total_ms
                ));
            }
        };

        if is_debug_mode() {
            eprintln!("[RB]  Draft answer: {}", &draft_text[..100.min(draft_text.len())]);
        }

        // ============================================================
        // STEP 4: Senior call - audit and approve/fix
        // ============================================================
        let probe_summary_pairs: Vec<(&str, &str)> = summaries
            .iter()
            .map(|s| (s.probe_id.as_str(), s.compact_json.as_str()))
            .collect();

        let senior_prompt = generate_senior_prompt_v80(question, &draft_text, &probe_summary_pairs);

        if is_debug_mode() {
            eprintln!("[RB]  Senior prompt ({} chars)", senior_prompt.len());
        }

        let senior_start = Instant::now();
        let (senior_response, _raw) = self.llm_client.call_senior_v80(&senior_prompt).await?;
        senior_total_ms = senior_start.elapsed().as_millis() as u64;

        // Check timeout (but still return answer if we have one)
        let elapsed = start_time.elapsed();
        if elapsed > self.timeout {
            warn!("[RB]  Timeout after Senior call ({:.2}s > {}s)", elapsed.as_secs_f64(), RAZORBACK_TIMEOUT_SECS);
        }

        // ============================================================
        // STEP 5: Build final answer
        // ============================================================
        let senior_verdict = senior_response.verdict.clone();
        let final_text = match senior_response.verdict.as_str() {
            "approve" => senior_response.fixed_answer.unwrap_or(draft_text),
            "fix_and_accept" => senior_response.fixed_answer.unwrap_or(draft_text),
            "refuse" => {
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

        info!(
            "[RB]  Done in {:.2}s - verdict={}, confidence={:.0}%",
            elapsed.as_secs_f64(),
            senior_verdict,
            confidence * 100.0
        );

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
            // v0.81.0: Timing and dialog trace fields
            junior_ms: junior_total_ms,
            senior_ms: senior_total_ms,
            junior_probes: probe_ids,
            junior_had_draft,
            senior_verdict: Some(senior_verdict),
        })
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
                // Simple disk summary - count devices
                let device_count = raw.lines()
                    .filter(|l| l.trim().starts_with("sd") || l.trim().starts_with("nvme"))
                    .count();
                format!(r#"{{"devices":{}}}"#, device_count)
            }
            "hardware.gpu" => {
                // GPU summary - extract vendor
                let has_nvidia = raw.to_lowercase().contains("nvidia");
                let has_amd = raw.to_lowercase().contains("amd") || raw.to_lowercase().contains("radeon");
                let has_intel = raw.to_lowercase().contains("intel");
                format!(
                    r#"{{"nvidia":{},"amd":{},"intel":{}}}"#,
                    has_nvidia, has_amd, has_intel
                )
            }
            _ => {
                // Generic: first 200 chars as preview
                let preview: String = raw.chars().take(200).collect();
                format!(r#"{{"preview":"{}"}}"#, preview.replace('"', "'").replace('\n', " "))
            }
        }
    }

    /// Build a refusal answer
    fn build_refusal(
        &self,
        question: &str,
        reason: &str,
        evidence: &[ProbeEvidenceV10],
        junior_probes: &[String],
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
            // v0.81.0: Timing and dialog trace fields
            junior_ms,
            senior_ms,
            junior_probes: junior_probes.to_vec(),
            junior_had_draft,
            senior_verdict: Some("refuse".to_string()),
        }
    }

    /// Build a timeout answer
    fn build_timeout_answer(&self, question: &str, elapsed: Duration, junior_ms: u64, senior_ms: u64) -> FinalAnswer {
        FinalAnswer {
            question: question.to_string(),
            answer: format!(
                "Sorry, I couldn't answer in time ({:.1}s exceeded {}s budget).\n\n\
                 Try asking again or check system load.",
                elapsed.as_secs_f64(),
                RAZORBACK_TIMEOUT_SECS
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
            // v0.81.0: Timing and dialog trace fields
            junior_ms,
            senior_ms,
            junior_probes: vec![],
            junior_had_draft: false,
            senior_verdict: None,
        }
    }

    /// Check if LLM backend is available
    pub async fn is_available(&self) -> bool {
        self.llm_client.is_available().await
    }
}

impl Default for RazorbackEngine {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = RazorbackEngine::default();
        assert_eq!(engine.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_precompute_cpu_summary() {
        let engine = RazorbackEngine::default();
        let raw = r#"
processor	: 0
model name	: AMD Ryzen 9 9950X 16-Core Processor
cpu cores	: 16
processor	: 1
model name	: AMD Ryzen 9 9950X 16-Core Processor
"#;
        let summary = engine.precompute_summary("cpu.info", raw);
        assert!(summary.contains("physical_cores"));
        assert!(summary.contains("threads_total"));
    }

    #[test]
    fn test_precompute_mem_summary() {
        let engine = RazorbackEngine::default();
        let raw = r#"
MemTotal:       32768000 kB
MemFree:         8000000 kB
MemAvailable:   24000000 kB
SwapTotal:       8000000 kB
"#;
        let summary = engine.precompute_summary("mem.info", raw);
        assert!(summary.contains("total_gib"));
        assert!(summary.contains("available_gib"));
    }

    #[test]
    fn test_precompute_gpu_summary() {
        let engine = RazorbackEngine::default();
        let raw = "NVIDIA Corporation GA102 [GeForce RTX 3090]";
        let summary = engine.precompute_summary("hardware.gpu", raw);
        assert!(summary.contains(r#""nvidia":true"#));
    }

    #[test]
    fn test_build_refusal() {
        let engine = RazorbackEngine::default();
        let answer = engine.build_refusal("test?", "no evidence", &[], &["cpu.info".to_string()], false, 100, 200);
        assert!(answer.is_refusal);
        assert_eq!(answer.confidence_level, ConfidenceLevel::Red);
        assert_eq!(answer.junior_ms, 100);
        assert_eq!(answer.senior_ms, 200);
        assert_eq!(answer.senior_verdict, Some("refuse".to_string()));
    }

    #[test]
    fn test_build_timeout_answer() {
        let engine = RazorbackEngine::default();
        let answer = engine.build_timeout_answer("test?", Duration::from_secs(6), 500, 0);
        assert!(answer.is_refusal);
        assert_eq!(answer.confidence_level, ConfidenceLevel::Red);
        assert_eq!(answer.junior_ms, 500);
        assert_eq!(answer.senior_ms, 0);
    }
}

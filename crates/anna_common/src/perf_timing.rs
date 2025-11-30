//! Performance Timing Infrastructure v3.4.0
//!
//! Lightweight timing helpers for precise performance instrumentation
//! across the entire pipeline.
//!
//! Key components:
//! - `PerfSpan`: RAII-style timing span
//! - `PipelineTimings`: Aggregated timings for a single question
//! - `GlobalBudget`: Per-question time budget enforcement
//! - `PerformanceHint`: Telemetry-driven runtime hints
//!
//! Usage:
//! ```ignore
//! let span = PerfSpan::start("brain_classify");
//! // ... do work ...
//! let elapsed_ms = span.end();
//! ```

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tracing::{debug, warn};

// ============================================================================
// Global Budget Constants (v3.4.0)
// ============================================================================

/// Default global time budget per question (15 seconds)
pub const DEFAULT_GLOBAL_BUDGET_MS: u64 = 15_000;

/// Brain + Recipe fast path budget (tiny slice - 500ms max)
pub const FAST_PATH_BUDGET_MS: u64 = 500;

/// Probe execution budget (4 seconds)
pub const PROBE_BUDGET_MS: u64 = 4_000;

/// Junior LLM soft timeout (triggers fallback consideration)
/// v3.13.0: Increased for realistic LLM response times (4b-7b models)
pub const JUNIOR_SOFT_TIMEOUT_MS: u64 = 8_000;

/// Junior LLM hard timeout (stops retries)
/// v3.13.0: Increased for realistic LLM response times
pub const JUNIOR_HARD_TIMEOUT_MS: u64 = 12_000;

/// Senior LLM soft timeout (triggers skip consideration)
/// v3.13.0: Increased for realistic LLM response times (14b models)
pub const SENIOR_SOFT_TIMEOUT_MS: u64 = 10_000;

/// Senior LLM hard timeout (use Junior answer as-is)
/// v3.13.0: Increased for realistic LLM response times
pub const SENIOR_HARD_TIMEOUT_MS: u64 = 15_000;

/// Maximum time to produce a degraded answer after timeout decision
pub const DEGRADED_ANSWER_BUDGET_MS: u64 = 2_000;

/// Unsupported question fast-fail budget (must be < 500ms)
pub const UNSUPPORTED_FAIL_FAST_MS: u64 = 500;

// ============================================================================
// PerfSpan - Lightweight Timing Helper
// ============================================================================

/// A lightweight timing span for measuring elapsed time
#[derive(Debug)]
pub struct PerfSpan {
    label: &'static str,
    start: Instant,
    ended: AtomicBool,
}

impl PerfSpan {
    /// Start a new timing span with the given label
    pub fn start(label: &'static str) -> Self {
        Self {
            label,
            start: Instant::now(),
            ended: AtomicBool::new(false),
        }
    }

    /// End the span and return elapsed milliseconds
    pub fn end(&self) -> u64 {
        self.ended.store(true, Ordering::SeqCst);
        self.elapsed_ms()
    }

    /// Get elapsed milliseconds without ending the span
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get the label
    pub fn label(&self) -> &'static str {
        self.label
    }

    /// Check if the span has ended
    pub fn is_ended(&self) -> bool {
        self.ended.load(Ordering::SeqCst)
    }

    /// Log timing if debug mode is enabled
    pub fn log_if_debug(&self, debug_enabled: bool) {
        if debug_enabled {
            debug!(
                label = self.label,
                elapsed_ms = self.elapsed_ms(),
                "PERF_SPAN"
            );
        }
    }
}

impl Drop for PerfSpan {
    fn drop(&mut self) {
        // Auto-end if not explicitly ended (useful for error paths)
        if !self.ended.load(Ordering::SeqCst) {
            let _ = self.end();
        }
    }
}

// ============================================================================
// PipelineTimings - Aggregated Timings
// ============================================================================

/// Aggregated timings for a complete question pipeline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineTimings {
    /// Total pipeline time (wall clock)
    pub total_ms: u64,
    /// Brain classification time
    pub brain_classify_ms: u64,
    /// Recipe lookup time
    pub recipe_lookup_ms: u64,
    /// Recipe apply time (if matched)
    pub recipe_apply_ms: u64,
    /// Total probe execution time
    pub probe_total_ms: u64,
    /// Per-probe timings (probe_id -> ms)
    pub probe_individual: Vec<(String, u64)>,
    /// Junior planning call time
    pub junior_plan_ms: u64,
    /// Junior drafting call time
    pub junior_draft_ms: u64,
    /// Senior audit call time
    pub senior_audit_ms: u64,
    /// Answer formatting time
    pub answer_format_ms: u64,
    /// Whether we hit a timeout
    pub hit_timeout: bool,
    /// Origin of the final answer
    pub origin: String,
}

impl PipelineTimings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total LLM time (Junior + Senior)
    pub fn llm_total_ms(&self) -> u64 {
        self.junior_plan_ms + self.junior_draft_ms + self.senior_audit_ms
    }

    /// Check if Brain/Recipe fast path was used (no LLM)
    pub fn used_fast_path(&self) -> bool {
        self.llm_total_ms() == 0 && !self.origin.is_empty()
    }

    /// Format as compact log line
    pub fn format_log(&self) -> String {
        format!(
            "PIPELINE_TIMING total={}ms brain={}ms recipe={}ms probe={}ms junior={}ms senior={}ms origin={} timeout={}",
            self.total_ms,
            self.brain_classify_ms,
            self.recipe_lookup_ms + self.recipe_apply_ms,
            self.probe_total_ms,
            self.junior_plan_ms + self.junior_draft_ms,
            self.senior_audit_ms,
            self.origin,
            self.hit_timeout
        )
    }

    /// Convert to telemetry-compatible format
    pub fn to_telemetry_fields(&self) -> Vec<(String, String)> {
        vec![
            ("timing_total_ms".to_string(), self.total_ms.to_string()),
            ("timing_brain_ms".to_string(), self.brain_classify_ms.to_string()),
            ("timing_recipe_ms".to_string(), (self.recipe_lookup_ms + self.recipe_apply_ms).to_string()),
            ("timing_probe_ms".to_string(), self.probe_total_ms.to_string()),
            ("timing_llm_ms".to_string(), self.llm_total_ms().to_string()),
            ("timing_origin".to_string(), self.origin.clone()),
            ("timing_timeout".to_string(), self.hit_timeout.to_string()),
        ]
    }
}

// ============================================================================
// GlobalBudget - Per-Question Time Budget
// ============================================================================

/// Per-question global time budget enforcer
#[derive(Debug)]
pub struct GlobalBudget {
    /// Total budget in milliseconds
    budget_ms: u64,
    /// When the budget started
    start: Instant,
    /// Whether we've already triggered degraded mode
    degraded: AtomicBool,
}

impl GlobalBudget {
    /// Create a new budget with the default global limit
    pub fn new() -> Self {
        Self::with_budget(DEFAULT_GLOBAL_BUDGET_MS)
    }

    /// Create a new budget with a custom limit
    pub fn with_budget(budget_ms: u64) -> Self {
        Self {
            budget_ms,
            start: Instant::now(),
            degraded: AtomicBool::new(false),
        }
    }

    /// Check remaining time in milliseconds
    pub fn remaining_ms(&self) -> u64 {
        let elapsed = self.start.elapsed().as_millis() as u64;
        self.budget_ms.saturating_sub(elapsed)
    }

    /// Check elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Check if budget is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.remaining_ms() == 0
    }

    /// Check if we should enter degraded mode
    pub fn should_degrade(&self) -> bool {
        // Degrade when less than 20% time remaining
        self.remaining_ms() < self.budget_ms / 5
    }

    /// Mark as degraded (only logs once)
    pub fn mark_degraded(&self) {
        if !self.degraded.swap(true, Ordering::SeqCst) {
            warn!(
                elapsed_ms = self.elapsed_ms(),
                remaining_ms = self.remaining_ms(),
                "BUDGET_DEGRADED: Entering degraded mode"
            );
        }
    }

    /// Check if already in degraded mode
    pub fn is_degraded(&self) -> bool {
        self.degraded.load(Ordering::SeqCst)
    }

    /// Check if we have enough time for an operation
    pub fn has_time_for(&self, operation_budget_ms: u64) -> bool {
        self.remaining_ms() >= operation_budget_ms
    }

    /// Get a sub-budget for a specific operation, respecting remaining time
    pub fn sub_budget_ms(&self, requested_ms: u64) -> u64 {
        self.remaining_ms().min(requested_ms)
    }
}

impl Default for GlobalBudget {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PerformanceHint - Telemetry-Driven Hints
// ============================================================================

/// Performance hint derived from recent telemetry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum PerformanceHint {
    /// System performing well - use full pipeline
    #[default]
    Good,
    /// Degraded performance - prefer Brain/Recipe, minimal Senior
    Degraded,
    /// Critical degradation - Brain/Recipe only, no Senior
    Critical,
}

impl PerformanceHint {
    /// Derive hint from rolling statistics
    pub fn from_stats(
        rolling_avg_latency_ms: u64,
        rolling_failure_rate: f64,
        rolling_timeout_rate: f64,
    ) -> Self {
        // Critical: High failure or timeout rate
        if rolling_failure_rate > 0.3 || rolling_timeout_rate > 0.2 {
            return Self::Critical;
        }

        // Degraded: Elevated latency or moderate issues
        if rolling_avg_latency_ms > 10_000
            || rolling_failure_rate > 0.1
            || rolling_timeout_rate > 0.05
        {
            return Self::Degraded;
        }

        // Good: Normal operation
        Self::Good
    }

    /// Check if we should skip Senior audit
    pub fn should_skip_senior(&self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Check if we should prefer fast paths
    pub fn prefer_fast_path(&self) -> bool {
        matches!(self, Self::Degraded | Self::Critical)
    }

    /// Get maximum LLM calls allowed under this hint
    pub fn max_llm_calls(&self) -> u32 {
        match self {
            Self::Good => 3,
            Self::Degraded => 2,
            Self::Critical => 1,
        }
    }

    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Good => "Good",
            Self::Degraded => "Degraded",
            Self::Critical => "Critical",
        }
    }
}


// ============================================================================
// LlmTimeoutResult - Tiered Timeout Tracking
// ============================================================================

/// Result of an LLM call with timeout tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmTimeoutResult {
    /// Completed within soft timeout
    Success { latency_ms: u64 },
    /// Completed but hit soft timeout (consider fallback)
    SoftTimeout { latency_ms: u64 },
    /// Hit hard timeout (stop further LLM use)
    HardTimeout { latency_ms: u64 },
}

impl LlmTimeoutResult {
    /// Evaluate Junior call result
    pub fn evaluate_junior(latency_ms: u64) -> Self {
        if latency_ms <= JUNIOR_SOFT_TIMEOUT_MS {
            Self::Success { latency_ms }
        } else if latency_ms <= JUNIOR_HARD_TIMEOUT_MS {
            Self::SoftTimeout { latency_ms }
        } else {
            Self::HardTimeout { latency_ms }
        }
    }

    /// Evaluate Senior call result
    pub fn evaluate_senior(latency_ms: u64) -> Self {
        if latency_ms <= SENIOR_SOFT_TIMEOUT_MS {
            Self::Success { latency_ms }
        } else if latency_ms <= SENIOR_HARD_TIMEOUT_MS {
            Self::SoftTimeout { latency_ms }
        } else {
            Self::HardTimeout { latency_ms }
        }
    }

    /// Check if this result suggests we should stop LLM usage
    pub fn should_stop_llm(&self) -> bool {
        matches!(self, Self::HardTimeout { .. })
    }

    /// Check if we hit any timeout
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::SoftTimeout { .. } | Self::HardTimeout { .. })
    }

    /// Get latency
    pub fn latency_ms(&self) -> u64 {
        match self {
            Self::Success { latency_ms }
            | Self::SoftTimeout { latency_ms }
            | Self::HardTimeout { latency_ms } => *latency_ms,
        }
    }
}

// ============================================================================
// UnsupportedQuestionResult - Fast-Fail Detection
// ============================================================================

/// Result of unsupported question detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsupportedResult {
    /// Confidence that the question is unsupported (0.0-1.0)
    pub confidence: f64,
    /// Reason for classification
    pub reason: UnsupportedReason,
    /// Time taken for classification
    pub classify_ms: u64,
}

/// Reasons a question may be unsupported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnsupportedReason {
    /// Empty or whitespace-only input
    EmptyInput,
    /// Pure gibberish/junk
    Gibberish,
    /// Conversational/philosophical with no system action
    Conversational,
    /// Request to modify system beyond capability
    BeyondCapability,
    /// Generic greeting with no actionable request
    Greeting,
    /// Question is supported
    Supported,
}

impl UnsupportedResult {
    /// Create a supported result
    pub fn supported(classify_ms: u64) -> Self {
        Self {
            confidence: 0.0,
            reason: UnsupportedReason::Supported,
            classify_ms,
        }
    }

    /// Create an unsupported result
    pub fn unsupported(reason: UnsupportedReason, confidence: f64, classify_ms: u64) -> Self {
        Self {
            confidence: confidence.clamp(0.0, 1.0),
            reason,
            classify_ms,
        }
    }

    /// Check if we should fail fast (high confidence unsupported)
    pub fn should_fail_fast(&self) -> bool {
        self.reason != UnsupportedReason::Supported && self.confidence >= 0.8
    }

    /// Get a human-readable explanation
    pub fn explanation(&self) -> &'static str {
        match self.reason {
            UnsupportedReason::EmptyInput => {
                "I received an empty question. Please ask something specific about your system."
            }
            UnsupportedReason::Gibberish => {
                "I could not understand the question. Please rephrase clearly."
            }
            UnsupportedReason::Conversational => {
                "I am a system assistant focused on Linux questions. I cannot help with general conversation or philosophy."
            }
            UnsupportedReason::BeyondCapability => {
                "This request is beyond my current capabilities. I can only answer questions and run safe read-only commands."
            }
            UnsupportedReason::Greeting => {
                "Hello! I'm Anna, your Linux system assistant. Ask me about your system - CPU, RAM, disk, services, logs, and more."
            }
            UnsupportedReason::Supported => "",
        }
    }
}

/// Classify if a question is unsupported (fast, no LLM)
pub fn classify_unsupported(question: &str) -> UnsupportedResult {
    let span = PerfSpan::start("unsupported_classify");
    let q = question.trim().to_lowercase();

    // Empty input
    if q.is_empty() {
        return UnsupportedResult::unsupported(
            UnsupportedReason::EmptyInput,
            1.0,
            span.end(),
        );
    }

    // Very short gibberish
    if q.len() < 3 && !q.chars().all(|c| c.is_alphabetic()) {
        return UnsupportedResult::unsupported(
            UnsupportedReason::Gibberish,
            0.9,
            span.end(),
        );
    }

    // Pure greetings (no question)
    let greeting_only = ["hi", "hello", "hey", "yo", "sup", "greetings"];
    if greeting_only.contains(&q.as_str()) {
        return UnsupportedResult::unsupported(
            UnsupportedReason::Greeting,
            0.95,
            span.end(),
        );
    }

    // Conversational/philosophical patterns
    let conversational_patterns = [
        "meaning of life",
        "what do you think about",
        "tell me a joke",
        "tell me a story",
        "how are you feeling",
        "do you have feelings",
        "are you alive",
        "who created you",
        "what's your favorite",
        "can you write me a poem",
        "write me a story",
        "let's chat",
        "let's talk",
        "what's the weather",
        "politics",
        "religion",
    ];

    for pattern in conversational_patterns {
        if q.contains(pattern) {
            return UnsupportedResult::unsupported(
                UnsupportedReason::Conversational,
                0.85,
                span.end(),
            );
        }
    }

    // Beyond capability patterns (destructive or not system-related)
    let beyond_patterns = [
        "delete everything",
        "format my disk",
        "rm -rf",
        "drop table",
        "hack ",
        "crack ",
        "break into",
        "exploit ",
        "ddos",
        "make me a website",
        "build me an app",
        "write code for",
        "create a program",
    ];

    for pattern in beyond_patterns {
        if q.contains(pattern) {
            return UnsupportedResult::unsupported(
                UnsupportedReason::BeyondCapability,
                0.9,
                span.end(),
            );
        }
    }

    // Supported
    UnsupportedResult::supported(span.end())
}

// ============================================================================
// Fast Degraded Answer Generator - Guarantee RED answers within budget
// ============================================================================

/// Result of a fast degraded answer generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradedAnswer {
    /// Human-readable answer text
    pub text: String,
    /// Reliability score (always < 0.7 for degraded)
    pub reliability: f64,
    /// Reason for degradation
    pub reason: DegradationReason,
    /// Time taken to generate (should be < DEGRADED_ANSWER_BUDGET_MS)
    pub generation_ms: u64,
    /// Origin tag
    pub origin: String,
}

/// Reasons for producing a degraded answer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationReason {
    /// LLM hit hard timeout
    LlmTimeout,
    /// Global budget exhausted
    BudgetExhausted,
    /// LLM returned invalid response
    LlmInvalid,
    /// All probes failed
    ProbesFailed,
    /// Backend unavailable
    BackendUnavailable,
    /// Emergency fallback (unknown error)
    EmergencyFallback,
}

impl DegradationReason {
    /// Get human-readable prefix for the degraded answer
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::LlmTimeout => "I ran out of time processing this question.",
            Self::BudgetExhausted => "Time budget exhausted before completing analysis.",
            Self::LlmInvalid => "I received an incomplete response from my reasoning engine.",
            Self::ProbesFailed => "I could not gather the system information needed.",
            Self::BackendUnavailable => "My reasoning engine is currently unavailable.",
            Self::EmergencyFallback => "An unexpected error occurred while processing.",
        }
    }

    /// Get reliability score for this degradation type
    pub fn reliability(&self) -> f64 {
        match self {
            Self::LlmTimeout => 0.40,
            Self::BudgetExhausted => 0.35,
            Self::LlmInvalid => 0.30,
            Self::ProbesFailed => 0.45,
            Self::BackendUnavailable => 0.20,
            Self::EmergencyFallback => 0.25,
        }
    }
}

impl DegradedAnswer {
    /// Create a degraded answer with the given reason
    /// Guaranteed to complete within DEGRADED_ANSWER_BUDGET_MS
    pub fn generate(
        question: &str,
        reason: DegradationReason,
        partial_evidence: Option<&str>,
    ) -> Self {
        let span = PerfSpan::start("degraded_answer_generate");

        // Build text: prefix + partial evidence if available
        let text = if let Some(evidence) = partial_evidence {
            format!(
                "{}\n\nHere's what I did gather before the timeout:\n{}",
                reason.prefix(),
                evidence.chars().take(500).collect::<String>()
            )
        } else {
            format!(
                "{}\n\nPlease try again or rephrase your question: \"{}\"",
                reason.prefix(),
                question.chars().take(100).collect::<String>()
            )
        };

        let generation_ms = span.end();

        // Log if we exceeded budget
        if generation_ms > DEGRADED_ANSWER_BUDGET_MS {
            tracing::warn!(
                generation_ms,
                budget_ms = DEGRADED_ANSWER_BUDGET_MS,
                reason = ?reason,
                "DEGRADED_ANSWER: Exceeded generation budget"
            );
        }

        Self {
            text,
            reliability: reason.reliability(),
            reason,
            generation_ms,
            origin: format!("Degraded-{:?}", reason),
        }
    }

    /// Create an emergency fallback answer (fastest path)
    /// Use when everything else has failed and we need an answer NOW
    pub fn emergency(question: &str) -> Self {
        Self {
            text: format!(
                "I could not process this question due to an unexpected error.\n\n\
                 Please try again: \"{}\"",
                question.chars().take(80).collect::<String>()
            ),
            reliability: 0.10,
            reason: DegradationReason::EmergencyFallback,
            generation_ms: 0,
            origin: "Emergency-Fallback".to_string(),
        }
    }

    /// Check if this is a RED answer (50-69%)
    pub fn is_red(&self) -> bool {
        self.reliability >= 0.50 && self.reliability < 0.70
    }

    /// Check if this is a REFUSED answer (< 50%)
    pub fn is_refused(&self) -> bool {
        self.reliability < 0.50
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_perf_span_basic() {
        let span = PerfSpan::start("test");
        sleep(Duration::from_millis(10));
        let elapsed = span.end();
        assert!(elapsed >= 10, "Elapsed should be at least 10ms");
        assert!(span.is_ended());
    }

    #[test]
    fn test_perf_span_auto_end() {
        let elapsed;
        {
            let span = PerfSpan::start("test");
            sleep(Duration::from_millis(5));
            elapsed = span.elapsed_ms();
        } // span dropped here, auto-ends
        assert!(elapsed >= 5);
    }

    #[test]
    fn test_global_budget_basic() {
        let budget = GlobalBudget::with_budget(100);
        assert!(!budget.is_exhausted());
        assert!(budget.remaining_ms() <= 100);

        sleep(Duration::from_millis(50));
        assert!(budget.remaining_ms() <= 50);
    }

    #[test]
    fn test_global_budget_exhausted() {
        let budget = GlobalBudget::with_budget(10);
        sleep(Duration::from_millis(15));
        assert!(budget.is_exhausted());
        assert_eq!(budget.remaining_ms(), 0);
    }

    #[test]
    fn test_performance_hint_good() {
        let hint = PerformanceHint::from_stats(5000, 0.05, 0.01);
        assert_eq!(hint, PerformanceHint::Good);
        assert!(!hint.should_skip_senior());
        assert!(!hint.prefer_fast_path());
    }

    #[test]
    fn test_performance_hint_degraded() {
        let hint = PerformanceHint::from_stats(12000, 0.05, 0.01);
        assert_eq!(hint, PerformanceHint::Degraded);
        assert!(!hint.should_skip_senior());
        assert!(hint.prefer_fast_path());
    }

    #[test]
    fn test_performance_hint_critical() {
        let hint = PerformanceHint::from_stats(5000, 0.35, 0.01);
        assert_eq!(hint, PerformanceHint::Critical);
        assert!(hint.should_skip_senior());
        assert!(hint.prefer_fast_path());
    }

    #[test]
    fn test_llm_timeout_result_junior() {
        // v3.13.0: Updated for realistic timeouts (8s soft, 12s hard)
        let success = LlmTimeoutResult::evaluate_junior(6000);
        assert!(matches!(success, LlmTimeoutResult::Success { .. }));
        assert!(!success.should_stop_llm());

        let soft = LlmTimeoutResult::evaluate_junior(10000);
        assert!(matches!(soft, LlmTimeoutResult::SoftTimeout { .. }));
        assert!(!soft.should_stop_llm());

        let hard = LlmTimeoutResult::evaluate_junior(15000);
        assert!(matches!(hard, LlmTimeoutResult::HardTimeout { .. }));
        assert!(hard.should_stop_llm());
    }

    #[test]
    fn test_unsupported_empty() {
        let result = classify_unsupported("");
        assert!(result.should_fail_fast());
        assert_eq!(result.reason, UnsupportedReason::EmptyInput);
    }

    #[test]
    fn test_unsupported_greeting() {
        let result = classify_unsupported("hello");
        assert!(result.should_fail_fast());
        assert_eq!(result.reason, UnsupportedReason::Greeting);
    }

    #[test]
    fn test_unsupported_conversational() {
        let result = classify_unsupported("What is the meaning of life?");
        assert!(result.should_fail_fast());
        assert_eq!(result.reason, UnsupportedReason::Conversational);
    }

    #[test]
    fn test_unsupported_beyond_capability() {
        let result = classify_unsupported("hack into my neighbor's wifi");
        assert!(result.should_fail_fast());
        assert_eq!(result.reason, UnsupportedReason::BeyondCapability);
    }

    #[test]
    fn test_supported_questions() {
        let questions = [
            "How much RAM do I have?",
            "What CPU is installed?",
            "Show me disk usage",
            "Check nginx status",
            "What's my IP address?",
        ];

        for q in questions {
            let result = classify_unsupported(q);
            assert!(
                !result.should_fail_fast(),
                "Question '{}' should be supported",
                q
            );
        }
    }

    #[test]
    fn test_unsupported_fast_classify() {
        // All unsupported classifications must be < 500ms
        let questions = [
            "",
            "hi",
            "What is the meaning of life?",
            "rm -rf everything",
        ];

        for q in questions {
            let result = classify_unsupported(q);
            assert!(
                result.classify_ms < UNSUPPORTED_FAIL_FAST_MS,
                "Classification took {}ms, should be < {}ms",
                result.classify_ms,
                UNSUPPORTED_FAIL_FAST_MS
            );
        }
    }

    #[test]
    fn test_pipeline_timings_format() {
        let mut timings = PipelineTimings::new();
        timings.total_ms = 5000;
        timings.brain_classify_ms = 10;
        timings.junior_plan_ms = 2000;
        timings.junior_draft_ms = 2000;
        timings.origin = "Junior".to_string();

        let log = timings.format_log();
        assert!(log.contains("total=5000ms"));
        assert!(log.contains("brain=10ms"));
        assert!(log.contains("junior=4000ms")); // plan + draft
    }

    #[test]
    fn test_pipeline_timings_fast_path() {
        let mut timings = PipelineTimings::new();
        timings.brain_classify_ms = 15;
        timings.origin = "Brain".to_string();

        assert!(timings.used_fast_path());
        assert_eq!(timings.llm_total_ms(), 0);
    }
}

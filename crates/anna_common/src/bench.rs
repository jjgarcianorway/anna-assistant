//! Benchmark Logging Module v0.84.0
//!
//! Structured logging for benchmark mode and metrics collection.
//! Enabled when ANNA_BENCH_MODE=1 is set.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Check if benchmark mode is enabled
pub fn is_bench_mode() -> bool {
    std::env::var("ANNA_BENCH_MODE").is_ok()
}

/// Failure cause classification for reliability analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureCause {
    /// Question requires a probe that doesn't exist
    NoProbeAvailable,
    /// Probe ran but data was misinterpreted
    ProbeDataMisread,
    /// LLM fabricated information not in evidence
    LlmHallucination,
    /// Exceeded time budget
    TimeoutOrLatency,
    /// Question is outside Anna's scope
    UnsupportedDomain,
    /// Internal error in answer pipeline
    OrchestrationBug,
    /// Junior proposed unsafe or invalid command
    BadCommandProposal,
}

impl FailureCause {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoProbeAvailable => "no_probe_available",
            Self::ProbeDataMisread => "probe_data_misread",
            Self::LlmHallucination => "llm_hallucination",
            Self::TimeoutOrLatency => "timeout_or_latency",
            Self::UnsupportedDomain => "unsupported_domain",
            Self::OrchestrationBug => "orchestration_bug",
            Self::BadCommandProposal => "bad_command_proposal",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "no_probe_available" => Some(Self::NoProbeAvailable),
            "probe_data_misread" => Some(Self::ProbeDataMisread),
            "llm_hallucination" => Some(Self::LlmHallucination),
            "timeout_or_latency" => Some(Self::TimeoutOrLatency),
            "unsupported_domain" => Some(Self::UnsupportedDomain),
            "orchestration_bug" => Some(Self::OrchestrationBug),
            "bad_command_proposal" => Some(Self::BadCommandProposal),
            _ => None,
        }
    }
}

impl std::fmt::Display for FailureCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Per-question benchmark event for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkEvent {
    /// ISO timestamp when question was received
    pub timestamp_start: String,
    /// ISO timestamp when answer was delivered
    pub timestamp_end: String,
    /// The question text
    pub question_text: String,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Time spent in self-solve attempt (no LLM)
    pub self_ms: u64,
    /// Time spent in Junior LLM calls
    pub junior_ms: u64,
    /// Time spent in Senior LLM calls
    pub senior_ms: u64,
    /// Time spent running commands/probes
    pub cmd_ms: u64,
    /// Number of Junior LLM invocations
    pub junior_calls: u32,
    /// Number of Senior LLM invocations
    pub senior_calls: u32,
    /// List of probes/tools used
    pub probes_used: Vec<String>,
    /// Final confidence score (0.0-1.0)
    pub final_confidence: f64,
    /// Whether the answer was successful
    pub success: bool,
    /// XP change from this answer
    pub xp_change: i32,
    /// Level after this answer
    pub level_after: u32,
    /// Failure cause if answer failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_cause: Option<String>,
}

impl Default for BenchmarkEvent {
    fn default() -> Self {
        Self {
            timestamp_start: String::new(),
            timestamp_end: String::new(),
            question_text: String::new(),
            duration_ms: 0,
            self_ms: 0,
            junior_ms: 0,
            senior_ms: 0,
            cmd_ms: 0,
            junior_calls: 0,
            senior_calls: 0,
            probes_used: Vec::new(),
            final_confidence: 0.0,
            success: false,
            xp_change: 0,
            level_after: 1,
            failure_cause: None,
        }
    }
}

/// Timing breakdown for a single answer
#[derive(Debug, Clone, Default)]
pub struct TimingBreakdown {
    /// Total answer time
    pub total_ms: u64,
    /// Self-solve attempt time
    pub self_ms: u64,
    /// Junior LLM time
    pub junior_ms: u64,
    /// Senior LLM time
    pub senior_ms: u64,
    /// Command execution time
    pub cmd_ms: u64,
    /// Start instant for duration calculation
    start: Option<Instant>,
}

impl TimingBreakdown {
    pub fn new() -> Self {
        Self {
            start: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Record self-solve timing
    pub fn record_self(&mut self, ms: u64) {
        self.self_ms = ms;
    }

    /// Record Junior timing
    pub fn record_junior(&mut self, ms: u64) {
        self.junior_ms += ms;
    }

    /// Record Senior timing
    pub fn record_senior(&mut self, ms: u64) {
        self.senior_ms += ms;
    }

    /// Record command execution timing
    pub fn record_cmd(&mut self, ms: u64) {
        self.cmd_ms += ms;
    }

    /// Finalize and calculate total
    pub fn finalize(&mut self) {
        if let Some(start) = self.start.take() {
            self.total_ms = start.elapsed().as_millis() as u64;
        }
    }

    /// Check if any budget is exceeded and return violations
    pub fn check_budgets(&self, budgets: &LatencyBudgets) -> Vec<BudgetViolation> {
        let mut violations = Vec::new();

        if self.self_ms > budgets.max_self_ms {
            violations.push(BudgetViolation {
                component: "self_ms".to_string(),
                limit_ms: budgets.max_self_ms,
                actual_ms: self.self_ms,
            });
        }
        if self.junior_ms > budgets.max_junior_ms {
            violations.push(BudgetViolation {
                component: "junior_ms".to_string(),
                limit_ms: budgets.max_junior_ms,
                actual_ms: self.junior_ms,
            });
        }
        if self.senior_ms > budgets.max_senior_ms {
            violations.push(BudgetViolation {
                component: "senior_ms".to_string(),
                limit_ms: budgets.max_senior_ms,
                actual_ms: self.senior_ms,
            });
        }
        if self.cmd_ms > budgets.max_cmd_ms {
            violations.push(BudgetViolation {
                component: "cmd_ms".to_string(),
                limit_ms: budgets.max_cmd_ms,
                actual_ms: self.cmd_ms,
            });
        }

        violations
    }
}

/// Latency budget configuration
#[derive(Debug, Clone)]
pub struct LatencyBudgets {
    pub max_self_ms: u64,
    pub max_junior_ms: u64,
    pub max_senior_ms: u64,
    pub max_cmd_ms: u64,
}

impl Default for LatencyBudgets {
    fn default() -> Self {
        // v0.83.0 razorback profile budgets
        Self {
            max_self_ms: 500,
            max_junior_ms: 5000,
            max_senior_ms: 6000,
            max_cmd_ms: 3000,
        }
    }
}

/// Budget violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub component: String,
    pub limit_ms: u64,
    pub actual_ms: u64,
}

impl std::fmt::Display for BudgetViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ANNA_WARN budget_exceeded component={} limit_ms={} actual_ms={}",
            self.component, self.limit_ms, self.actual_ms
        )
    }
}

/// Log a metrics line for a completed answer
pub fn log_metrics(
    question: &str,
    timing: &TimingBreakdown,
    confidence: f64,
    junior_calls: u32,
    senior_calls: u32,
) {
    // Escape question for logging
    let escaped_q = question.replace('"', "\\\"");
    eprintln!(
        "ANNA_METRICS question=\"{}\" total_ms={} self_ms={} junior_ms={} senior_ms={} cmd_ms={} confidence={:.2} junior_calls={} senior_calls={}",
        escaped_q,
        timing.total_ms,
        timing.self_ms,
        timing.junior_ms,
        timing.senior_ms,
        timing.cmd_ms,
        confidence,
        junior_calls,
        senior_calls
    );
}

/// Log budget violations as warnings
pub fn log_violations(violations: &[BudgetViolation]) {
    for v in violations {
        eprintln!("{}", v);
    }
}

/// XP event for progression tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpEvent {
    /// ISO timestamp
    pub timestamp: String,
    /// Reason for XP change
    pub reason: String,
    /// XP delta (positive or negative)
    pub delta: i32,
    /// Resulting level after change
    pub resulting_level: u32,
}

/// XP event reasons
pub mod xp_reasons {
    pub const NEW_METHOD_STORED: &str = "new_method_stored";
    pub const IMPROVED_METHOD: &str = "improved_method";
    pub const BAD_ASSUMPTION_JUNIOR: &str = "bad_assumption_junior";
    pub const SEVERE_ERROR_SENIOR: &str = "severe_error_senior";
    pub const SUCCESSFUL_ANSWER: &str = "successful_answer";
    pub const FAILED_ANSWER: &str = "failed_answer";
    pub const FAST_CACHED_ANSWER: &str = "fast_cached_answer";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_cause_roundtrip() {
        let causes = vec![
            FailureCause::NoProbeAvailable,
            FailureCause::ProbeDataMisread,
            FailureCause::LlmHallucination,
            FailureCause::TimeoutOrLatency,
            FailureCause::UnsupportedDomain,
            FailureCause::OrchestrationBug,
            FailureCause::BadCommandProposal,
        ];

        for cause in causes {
            let s = cause.as_str();
            let parsed = FailureCause::from_str(s).unwrap();
            assert_eq!(cause, parsed);
        }
    }

    #[test]
    fn test_timing_breakdown() {
        let mut timing = TimingBreakdown::new();
        timing.record_self(100);
        timing.record_junior(2000);
        timing.record_senior(1500);
        timing.record_cmd(500);
        timing.finalize();

        assert_eq!(timing.self_ms, 100);
        assert_eq!(timing.junior_ms, 2000);
        assert_eq!(timing.senior_ms, 1500);
        assert_eq!(timing.cmd_ms, 500);
        // Note: total_ms comes from elapsed time, not sum of components
        // In test, we verify components are recorded correctly
        let sum = timing.self_ms + timing.junior_ms + timing.senior_ms + timing.cmd_ms;
        assert_eq!(sum, 4100);
    }

    #[test]
    fn test_budget_violations() {
        let timing = TimingBreakdown {
            total_ms: 20000,
            self_ms: 100,
            junior_ms: 8000, // exceeds 5000
            senior_ms: 2000,
            cmd_ms: 500,
            start: None,
        };

        let budgets = LatencyBudgets::default();
        let violations = timing.check_budgets(&budgets);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].component, "junior_ms");
        assert_eq!(violations[0].limit_ms, 5000);
        assert_eq!(violations[0].actual_ms, 8000);
    }

    #[test]
    fn test_benchmark_event_default() {
        let event = BenchmarkEvent::default();
        assert_eq!(event.duration_ms, 0);
        assert_eq!(event.junior_calls, 0);
        assert_eq!(event.senior_calls, 0);
        assert!(!event.success);
        assert!(event.failure_cause.is_none());
    }
}

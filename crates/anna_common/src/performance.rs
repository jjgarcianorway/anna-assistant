//! Performance enforcement for Anna v0.85.0
//!
//! Strict runtime ceilings and budget tracking.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

// ============================================================================
// Part 4: Performance Ceilings
// ============================================================================

/// v0.85.0 performance limits for razorback profile
pub const MAX_TOTAL_MS: u64 = 12000;    // 12 seconds total
pub const MAX_BRAIN_MS: u64 = 500;       // 500ms for Brain
pub const MAX_JUNIOR_MS: u64 = 3000;     // 3 seconds Junior
pub const MAX_SENIOR_MS: u64 = 4000;     // 4 seconds Senior
pub const MAX_CMD_MS: u64 = 4000;        // 4 seconds command execution

/// Maximum LLM calls per question type
pub const MAX_LLM_CALLS_SIMPLE: u32 = 1;
pub const MAX_LLM_CALLS_MEDIUM: u32 = 2;
pub const MAX_LLM_CALLS_COMPLEX: u32 = 3;

// ============================================================================
// Part 5: Reliability Thresholds
// ============================================================================

/// v0.85.0 reliability thresholds
pub const MIN_EVIDENCE_SCORE: f64 = 0.80;
pub const MIN_COVERAGE_SCORE: f64 = 0.90;
pub const MIN_REASONING_SCORE: f64 = 0.90;

/// Performance budget tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudget {
    /// Total time allowed
    pub max_total_ms: u64,
    /// Brain time allowed
    pub max_brain_ms: u64,
    /// Junior LLM time allowed
    pub max_junior_ms: u64,
    /// Senior LLM time allowed
    pub max_senior_ms: u64,
    /// Command execution time allowed
    pub max_cmd_ms: u64,
    /// Maximum LLM calls allowed
    pub max_llm_calls: u32,
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        Self::razorback()
    }
}

impl PerformanceBudget {
    /// Razorback profile (fast, local LLM)
    pub fn razorback() -> Self {
        Self {
            max_total_ms: MAX_TOTAL_MS,
            max_brain_ms: MAX_BRAIN_MS,
            max_junior_ms: MAX_JUNIOR_MS,
            max_senior_ms: MAX_SENIOR_MS,
            max_cmd_ms: MAX_CMD_MS,
            max_llm_calls: MAX_LLM_CALLS_MEDIUM,
        }
    }

    /// Simple question profile
    pub fn simple() -> Self {
        Self {
            max_total_ms: 5000,
            max_brain_ms: MAX_BRAIN_MS,
            max_junior_ms: 2000,
            max_senior_ms: 2000,
            max_cmd_ms: 2000,
            max_llm_calls: MAX_LLM_CALLS_SIMPLE,
        }
    }

    /// Complex question profile
    pub fn complex() -> Self {
        Self {
            max_total_ms: 20000,
            max_brain_ms: MAX_BRAIN_MS,
            max_junior_ms: 5000,
            max_senior_ms: 6000,
            max_cmd_ms: 8000,
            max_llm_calls: MAX_LLM_CALLS_COMPLEX,
        }
    }
}

/// Live performance tracker
#[derive(Debug)]
pub struct PerformanceTracker {
    /// Budget to enforce
    budget: PerformanceBudget,
    /// When tracking started
    start: Instant,
    /// Time spent in Brain
    brain_ms: u64,
    /// Time spent in Junior
    junior_ms: u64,
    /// Time spent in Senior
    senior_ms: u64,
    /// Time spent in commands
    cmd_ms: u64,
    /// Number of LLM calls made
    llm_calls: u32,
    /// Violations detected
    violations: Vec<BudgetViolation>,
}

/// A budget violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub component: String,
    pub limit_ms: u64,
    pub actual_ms: u64,
    pub exceeded_by_ms: u64,
}

impl PerformanceTracker {
    pub fn new(budget: PerformanceBudget) -> Self {
        Self {
            budget,
            start: Instant::now(),
            brain_ms: 0,
            junior_ms: 0,
            senior_ms: 0,
            cmd_ms: 0,
            llm_calls: 0,
            violations: Vec::new(),
        }
    }

    /// Check if total time budget is exceeded
    pub fn is_total_exceeded(&self) -> bool {
        self.start.elapsed().as_millis() as u64 > self.budget.max_total_ms
    }

    /// Check if we can start another LLM call
    pub fn can_call_llm(&self) -> bool {
        !self.is_total_exceeded() && self.llm_calls < self.budget.max_llm_calls
    }

    /// Check remaining time budget
    pub fn remaining_ms(&self) -> u64 {
        let elapsed = self.start.elapsed().as_millis() as u64;
        self.budget.max_total_ms.saturating_sub(elapsed)
    }

    /// Record Brain time
    pub fn record_brain(&mut self, ms: u64) {
        self.brain_ms += ms;
        if self.brain_ms > self.budget.max_brain_ms {
            self.violations.push(BudgetViolation {
                component: "brain".to_string(),
                limit_ms: self.budget.max_brain_ms,
                actual_ms: self.brain_ms,
                exceeded_by_ms: self.brain_ms - self.budget.max_brain_ms,
            });
        }
    }

    /// Record Junior time and increment LLM calls
    pub fn record_junior(&mut self, ms: u64) {
        self.junior_ms += ms;
        self.llm_calls += 1;
        if self.junior_ms > self.budget.max_junior_ms {
            self.violations.push(BudgetViolation {
                component: "junior".to_string(),
                limit_ms: self.budget.max_junior_ms,
                actual_ms: self.junior_ms,
                exceeded_by_ms: self.junior_ms - self.budget.max_junior_ms,
            });
        }
    }

    /// Record Senior time and increment LLM calls
    pub fn record_senior(&mut self, ms: u64) {
        self.senior_ms += ms;
        self.llm_calls += 1;
        if self.senior_ms > self.budget.max_senior_ms {
            self.violations.push(BudgetViolation {
                component: "senior".to_string(),
                limit_ms: self.budget.max_senior_ms,
                actual_ms: self.senior_ms,
                exceeded_by_ms: self.senior_ms - self.budget.max_senior_ms,
            });
        }
    }

    /// Record command execution time
    pub fn record_cmd(&mut self, ms: u64) {
        self.cmd_ms += ms;
        if self.cmd_ms > self.budget.max_cmd_ms {
            self.violations.push(BudgetViolation {
                component: "cmd".to_string(),
                limit_ms: self.budget.max_cmd_ms,
                actual_ms: self.cmd_ms,
                exceeded_by_ms: self.cmd_ms - self.budget.max_cmd_ms,
            });
        }
    }

    /// Get all violations
    pub fn get_violations(&self) -> &[BudgetViolation] {
        &self.violations
    }

    /// Has any violation
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Get total elapsed time
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Format as ANNA_TIME_BUDGET log line
    pub fn format_log(&self) -> String {
        format!(
            "ANNA_TIME_BUDGET total={}ms brain={}ms junior={}ms senior={}ms cmd={}ms llm_calls={}",
            self.elapsed_ms(),
            self.brain_ms,
            self.junior_ms,
            self.senior_ms,
            self.cmd_ms,
            self.llm_calls
        )
    }

    /// Format violations as ANNA_LIMIT log lines
    pub fn format_violations(&self) -> Vec<String> {
        self.violations.iter().map(|v| {
            format!(
                "ANNA_LIMIT breach={} limit={} actual={}",
                v.component, v.limit_ms, v.actual_ms
            )
        }).collect()
    }
}

/// Reliability scores from Senior
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityCheck {
    pub evidence: f64,
    pub coverage: f64,
    pub reasoning: f64,
}

impl ReliabilityCheck {
    pub fn new(evidence: f64, coverage: f64, reasoning: f64) -> Self {
        Self { evidence, coverage, reasoning }
    }

    /// Check if all thresholds are met
    pub fn passes_thresholds(&self) -> bool {
        self.evidence >= MIN_EVIDENCE_SCORE
            && self.coverage >= MIN_COVERAGE_SCORE
            && self.reasoning >= MIN_REASONING_SCORE
    }

    /// Get which thresholds failed
    pub fn failed_thresholds(&self) -> Vec<String> {
        let mut failures = Vec::new();
        if self.evidence < MIN_EVIDENCE_SCORE {
            failures.push(format!("evidence: {:.0}% < {:.0}%", self.evidence * 100.0, MIN_EVIDENCE_SCORE * 100.0));
        }
        if self.coverage < MIN_COVERAGE_SCORE {
            failures.push(format!("coverage: {:.0}% < {:.0}%", self.coverage * 100.0, MIN_COVERAGE_SCORE * 100.0));
        }
        if self.reasoning < MIN_REASONING_SCORE {
            failures.push(format!("reasoning: {:.0}% < {:.0}%", self.reasoning * 100.0, MIN_REASONING_SCORE * 100.0));
        }
        failures
    }

    /// Get minimum score
    pub fn min_score(&self) -> f64 {
        self.evidence.min(self.coverage).min(self.reasoning)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_budget_default() {
        let budget = PerformanceBudget::default();
        assert_eq!(budget.max_total_ms, MAX_TOTAL_MS);
        assert_eq!(budget.max_junior_ms, MAX_JUNIOR_MS);
    }

    #[test]
    fn test_tracker_violations() {
        let budget = PerformanceBudget::simple();
        let mut tracker = PerformanceTracker::new(budget);

        // Record exceeding junior time
        tracker.record_junior(3000); // Exceeds 2000ms limit

        assert!(tracker.has_violations());
        assert_eq!(tracker.get_violations().len(), 1);
        assert_eq!(tracker.get_violations()[0].component, "junior");
    }

    #[test]
    fn test_reliability_thresholds() {
        let good = ReliabilityCheck::new(0.85, 0.92, 0.95);
        assert!(good.passes_thresholds());

        let bad = ReliabilityCheck::new(0.70, 0.92, 0.95);
        assert!(!bad.passes_thresholds());
        assert_eq!(bad.failed_thresholds().len(), 1);
    }

    #[test]
    fn test_tracker_log_format() {
        let budget = PerformanceBudget::default();
        let mut tracker = PerformanceTracker::new(budget);
        tracker.record_brain(100);
        tracker.record_junior(2000);

        let log = tracker.format_log();
        assert!(log.contains("brain=100ms"));
        assert!(log.contains("junior=2000ms"));
    }

    #[test]
    fn test_can_call_llm() {
        let budget = PerformanceBudget::simple(); // max 1 LLM call
        let mut tracker = PerformanceTracker::new(budget);

        assert!(tracker.can_call_llm());
        tracker.record_junior(100);
        assert!(!tracker.can_call_llm()); // Already made 1 call
    }
}

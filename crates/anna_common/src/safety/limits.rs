//! Safety Limits v0.15.0
//!
//! Hard limits for the research loop to prevent resource exhaustion.

use std::time::Duration;

// ============================================================================
// Research Loop Limits
// ============================================================================

/// Maximum LLM-A iterations per request
pub const MAX_LLM_A_ITERATIONS: usize = 6;

/// Maximum LLM-B review passes per iteration
pub const MAX_LLM_B_PASSES: usize = 2;

/// Maximum checks (commands) per single LLM-A iteration
pub const MAX_CHECKS_PER_ITERATION: usize = 5;

/// Maximum total checks per research session
pub const MAX_TOTAL_CHECKS: usize = 20;

/// Maximum concurrent check executions
pub const MAX_CONCURRENT_CHECKS: usize = 3;

// ============================================================================
// Timeout Limits
// ============================================================================

/// Default command execution timeout
pub const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum command timeout (can be overridden per-command)
pub const MAX_COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

/// LLM call timeout
pub const LLM_CALL_TIMEOUT: Duration = Duration::from_secs(120);

/// Total research session timeout
pub const SESSION_TIMEOUT: Duration = Duration::from_secs(300);

/// User question response timeout
pub const USER_QUESTION_TIMEOUT: Duration = Duration::from_secs(300);

// ============================================================================
// Size Limits
// ============================================================================

/// Maximum command output size (chars) before truncation
pub const MAX_OUTPUT_SIZE: usize = 8000;

/// Maximum evidence stored per check
pub const MAX_EVIDENCE_SIZE: usize = 4000;

/// Maximum total evidence size in research state
pub const MAX_TOTAL_EVIDENCE_SIZE: usize = 32000;

/// Maximum question text length
pub const MAX_QUESTION_LENGTH: usize = 500;

/// Maximum answer text length
pub const MAX_ANSWER_LENGTH: usize = 16000;

/// Maximum user free-text answer length
pub const MAX_USER_ANSWER_LENGTH: usize = 1000;

// ============================================================================
// Rate Limits
// ============================================================================

/// Maximum requests per minute (per client)
pub const MAX_REQUESTS_PER_MINUTE: u32 = 20;

/// Maximum concurrent sessions (total)
pub const MAX_CONCURRENT_SESSIONS: usize = 5;

/// Minimum delay between requests from same client (ms)
pub const MIN_REQUEST_INTERVAL_MS: u64 = 500;

// ============================================================================
// Resource Budget
// ============================================================================

/// Resource budget for a single research session
#[derive(Debug, Clone)]
pub struct ResourceBudget {
    /// Remaining LLM-A iterations
    pub iterations_remaining: usize,
    /// Remaining checks
    pub checks_remaining: usize,
    /// Remaining evidence bytes
    pub evidence_bytes_remaining: usize,
    /// Session start time
    pub started_at: std::time::Instant,
    /// Session timeout
    pub timeout: Duration,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceBudget {
    pub fn new() -> Self {
        Self {
            iterations_remaining: MAX_LLM_A_ITERATIONS,
            checks_remaining: MAX_TOTAL_CHECKS,
            evidence_bytes_remaining: MAX_TOTAL_EVIDENCE_SIZE,
            started_at: std::time::Instant::now(),
            timeout: SESSION_TIMEOUT,
        }
    }

    /// Check if the session has exceeded its timeout
    pub fn is_expired(&self) -> bool {
        self.started_at.elapsed() > self.timeout
    }

    /// Check if we can start another iteration
    pub fn can_iterate(&self) -> bool {
        !self.is_expired() && self.iterations_remaining > 0
    }

    /// Consume an iteration, returns false if none remaining
    pub fn consume_iteration(&mut self) -> bool {
        if self.iterations_remaining > 0 {
            self.iterations_remaining -= 1;
            true
        } else {
            false
        }
    }

    /// Consume checks, returns false if would exceed limit
    pub fn consume_checks(&mut self, count: usize) -> bool {
        if count <= self.checks_remaining {
            self.checks_remaining -= count;
            true
        } else {
            false
        }
    }

    /// Consume evidence bytes, returns actual bytes allowed
    pub fn consume_evidence(&mut self, bytes: usize) -> usize {
        let allowed = bytes.min(self.evidence_bytes_remaining);
        self.evidence_bytes_remaining -= allowed;
        allowed
    }

    /// Get remaining time before timeout
    pub fn remaining_time(&self) -> Duration {
        self.timeout.saturating_sub(self.started_at.elapsed())
    }
}

// ============================================================================
// Safety Checks
// ============================================================================

/// Result of a safety check
#[derive(Debug, Clone)]
pub enum SafetyCheckResult {
    /// Operation is allowed
    Allowed,
    /// Operation is denied
    Denied { reason: String },
    /// Operation is allowed but with warnings
    Warning { message: String },
}

impl SafetyCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            SafetyCheckResult::Allowed | SafetyCheckResult::Warning { .. }
        )
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, SafetyCheckResult::Denied { .. })
    }
}

/// Validate that a proposed check count is within limits
pub fn validate_check_count(proposed: usize, budget: &ResourceBudget) -> SafetyCheckResult {
    if proposed > MAX_CHECKS_PER_ITERATION {
        return SafetyCheckResult::Denied {
            reason: format!(
                "Too many checks requested ({}) - max {} per iteration",
                proposed, MAX_CHECKS_PER_ITERATION
            ),
        };
    }

    if proposed > budget.checks_remaining {
        return SafetyCheckResult::Denied {
            reason: format!(
                "Would exceed total check limit ({} remaining)",
                budget.checks_remaining
            ),
        };
    }

    if proposed > 3 {
        return SafetyCheckResult::Warning {
            message: format!("High check count ({}) may impact performance", proposed),
        };
    }

    SafetyCheckResult::Allowed
}

/// Validate command output size
pub fn validate_output_size(size: usize) -> (usize, bool) {
    if size > MAX_OUTPUT_SIZE {
        (MAX_OUTPUT_SIZE, true)
    } else {
        (size, false)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_budget_creation() {
        let budget = ResourceBudget::new();
        assert_eq!(budget.iterations_remaining, MAX_LLM_A_ITERATIONS);
        assert_eq!(budget.checks_remaining, MAX_TOTAL_CHECKS);
        assert!(budget.can_iterate());
    }

    #[test]
    fn test_consume_iteration() {
        let mut budget = ResourceBudget::new();
        for _ in 0..MAX_LLM_A_ITERATIONS {
            assert!(budget.consume_iteration());
        }
        assert!(!budget.consume_iteration());
        assert!(!budget.can_iterate());
    }

    #[test]
    fn test_consume_checks() {
        let mut budget = ResourceBudget::new();
        assert!(budget.consume_checks(5));
        assert_eq!(budget.checks_remaining, MAX_TOTAL_CHECKS - 5);
        assert!(!budget.consume_checks(MAX_TOTAL_CHECKS));
    }

    #[test]
    fn test_consume_evidence() {
        let mut budget = ResourceBudget::new();
        let allowed = budget.consume_evidence(1000);
        assert_eq!(allowed, 1000);

        // Try to consume more than remaining
        let remaining = budget.evidence_bytes_remaining;
        let allowed = budget.consume_evidence(remaining + 1000);
        assert_eq!(allowed, remaining);
    }

    #[test]
    fn test_validate_check_count() {
        let budget = ResourceBudget::new();

        // Normal count
        assert!(validate_check_count(2, &budget).is_allowed());

        // Too many per iteration
        assert!(validate_check_count(MAX_CHECKS_PER_ITERATION + 1, &budget).is_denied());

        // High but allowed
        if let SafetyCheckResult::Warning { .. } = validate_check_count(4, &budget) {
            // Expected
        } else {
            panic!("Expected warning for high check count");
        }
    }

    #[test]
    fn test_validate_output_size() {
        let (size, truncated) = validate_output_size(100);
        assert_eq!(size, 100);
        assert!(!truncated);

        let (size, truncated) = validate_output_size(MAX_OUTPUT_SIZE + 1000);
        assert_eq!(size, MAX_OUTPUT_SIZE);
        assert!(truncated);
    }
}

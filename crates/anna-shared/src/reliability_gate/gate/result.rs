//! Complete gate evaluation result.

use super::check::GateCheck;
use super::outcome::GateOutcome;
use serde::{Deserialize, Serialize};

/// Complete gate evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Overall outcome
    pub outcome: GateOutcome,
    /// Individual check results
    pub checks: Vec<GateCheck>,
    /// Request ID
    pub request_id: String,
    /// Evidence coverage (0.0 to 1.0)
    pub evidence_coverage: f32,
}

impl GateResult {
    /// Create a passing result.
    pub fn pass(request_id: &str, checks: Vec<GateCheck>, coverage: f32) -> Self {
        Self {
            outcome: GateOutcome::Pass,
            checks,
            request_id: request_id.to_string(),
            evidence_coverage: coverage,
        }
    }

    /// Create a failing result.
    pub fn fail(
        request_id: &str,
        outcome: GateOutcome,
        checks: Vec<GateCheck>,
        coverage: f32,
    ) -> Self {
        Self {
            outcome,
            checks,
            request_id: request_id.to_string(),
            evidence_coverage: coverage,
        }
    }

    /// Check if gate passed.
    pub fn passed(&self) -> bool {
        self.outcome.is_success()
    }

    /// Get first failed check.
    pub fn first_failure(&self) -> Option<&GateCheck> {
        self.checks.iter().find(|c| !c.passed)
    }
}

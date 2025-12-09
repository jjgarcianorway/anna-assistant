//! Stage outcome types (v0.0.178).

use serde::{Deserialize, Serialize};

/// Stage outcome for StageEnd events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageOutcome {
    Ok,
    Timeout,
    Error,
    Skipped,
    Deterministic, // Used when deterministic router answered
    /// Stage budget exceeded (METER phase)
    /// Distinct from Timeout: budget is stage-level, timeout is operation-level
    BudgetExceeded {
        /// Which stage exceeded its budget
        stage: String,
        /// Budget in milliseconds
        budget_ms: u64,
        /// Actual elapsed time in milliseconds
        elapsed_ms: u64,
    },
    /// Clarification required before proceeding (v0.45.5)
    /// Stage paused waiting for user to select from verified choices
    ClarificationRequired {
        /// The question prompt
        question: String,
        /// Available choices (verified against evidence)
        choices: Vec<String>,
    },
}

impl StageOutcome {
    /// Create a BudgetExceeded outcome.
    pub fn budget_exceeded(stage: impl Into<String>, budget_ms: u64, elapsed_ms: u64) -> Self {
        Self::BudgetExceeded {
            stage: stage.into(),
            budget_ms,
            elapsed_ms,
        }
    }

    /// Create a ClarificationRequired outcome (v0.45.5).
    pub fn clarification_required(question: impl Into<String>, choices: Vec<String>) -> Self {
        Self::ClarificationRequired {
            question: question.into(),
            choices,
        }
    }

    /// Check if this outcome represents a budget exceeded condition.
    pub fn is_budget_exceeded(&self) -> bool {
        matches!(self, Self::BudgetExceeded { .. })
    }

    /// Check if this outcome requires user clarification (v0.45.5).
    pub fn is_clarification_required(&self) -> bool {
        matches!(self, Self::ClarificationRequired { .. })
    }

    /// Check if this outcome allows the stage to proceed without user input.
    pub fn can_proceed(&self) -> bool {
        matches!(self, Self::Ok | Self::Deterministic)
    }
}

impl std::fmt::Display for StageOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Timeout => write!(f, "timeout"),
            Self::Error => write!(f, "error"),
            Self::Skipped => write!(f, "skipped"),
            Self::Deterministic => write!(f, "deterministic"),
            Self::BudgetExceeded {
                stage,
                budget_ms,
                elapsed_ms,
            } => {
                write!(
                    f,
                    "budget_exceeded({}: {}ms > {}ms)",
                    stage, elapsed_ms, budget_ms
                )
            }
            Self::ClarificationRequired { question, choices } => {
                write!(
                    f,
                    "clarification_required({}, {} choices)",
                    question,
                    choices.len()
                )
            }
        }
    }
}

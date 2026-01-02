//! Pipeline phases and phase budgets.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::constants::*;

/// Pipeline phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Translator intent extraction.
    TranslatorIntent,
    /// Probe collection.
    ProbeCollection,
    /// Junior specialist analysis.
    JuniorSpecialist,
    /// Senior specialist analysis (escalation).
    SeniorSpecialist,
    /// Renderer (final output).
    Renderer,
}

impl Phase {
    /// Get budget for this phase in milliseconds.
    pub fn budget_ms(&self) -> u64 {
        match self {
            Self::TranslatorIntent => TRANSLATOR_BUDGET_MS,
            Self::JuniorSpecialist => JUNIOR_BUDGET_MS,
            Self::SeniorSpecialist => SENIOR_BUDGET_MS,
            Self::ProbeCollection => PROBE_BUDGET_MS,
            Self::Renderer => RENDERER_BUDGET_MS,
        }
    }

    /// Get budget as Duration.
    pub fn budget(&self) -> Duration {
        Duration::from_millis(self.budget_ms())
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::TranslatorIntent => "intent extraction",
            Self::ProbeCollection => "probe collection",
            Self::JuniorSpecialist => "junior specialist",
            Self::SeniorSpecialist => "senior specialist",
            Self::Renderer => "rendering",
        }
    }
}

/// Budget for a single phase.
#[derive(Debug, Clone)]
pub struct PhaseBudget {
    /// Phase name.
    pub phase: Phase,
    /// Budget in milliseconds.
    pub budget_ms: u64,
    /// Actual time spent.
    pub actual_ms: Option<u64>,
    /// Whether this phase timed out.
    pub timed_out: bool,
    /// Whether this phase was skipped.
    pub skipped: bool,
}

impl PhaseBudget {
    /// Create a new phase budget.
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            budget_ms: phase.budget_ms(),
            actual_ms: None,
            timed_out: false,
            skipped: false,
        }
    }

    /// Mark as completed with actual time.
    pub fn complete(&mut self, actual_ms: u64) {
        self.actual_ms = Some(actual_ms);
        self.timed_out = actual_ms > self.budget_ms;
    }

    /// Mark as timed out.
    pub fn timeout(&mut self, actual_ms: u64) {
        self.actual_ms = Some(actual_ms);
        self.timed_out = true;
    }

    /// Mark as skipped.
    pub fn skip(&mut self) {
        self.skipped = true;
    }

    /// Check if within budget.
    pub fn within_budget(&self) -> bool {
        !self.timed_out
    }

    /// Get remaining budget.
    pub fn remaining_ms(&self, elapsed_ms: u64) -> u64 {
        if elapsed_ms >= self.budget_ms {
            0
        } else {
            self.budget_ms - elapsed_ms
        }
    }
}

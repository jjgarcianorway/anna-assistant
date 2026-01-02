//! Budget types and core structures.

use serde::{Deserialize, Serialize};

/// Budget for translator intent extraction.
pub const TRANSLATOR_BUDGET_MS: u64 = 700;

/// Budget for probe collection (parallel probes).
pub const PROBE_BUDGET_MS: u64 = 2500;

/// Budget for junior specialist.
pub const JUNIOR_BUDGET_MS: u64 = 1500;

/// Budget for senior specialist (escalation only).
pub const SENIOR_BUDGET_MS: u64 = 3500;

/// Budget for renderer.
pub const RENDERER_BUDGET_MS: u64 = 200;

/// Total budget for one-shot queries.
pub const TOTAL_BUDGET_MS: u64 = 6500;

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
    pub fn budget(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.budget_ms())
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

/// Result of a budget check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetResult {
    /// Within budget, proceed.
    Proceed,
    /// Exceeded budget, should cancel.
    Exceeded,
    /// Total budget exhausted.
    TotalExhausted,
}

/// Summary of budget usage.
#[derive(Debug, Clone)]
pub struct BudgetSummary {
    /// Total time spent.
    pub total_ms: u64,
    /// Per-phase timings: (phase, actual_ms, timed_out).
    pub phases: Vec<(Phase, u64, bool)>,
    /// Count of timed out phases.
    pub timed_out_phases: usize,
    /// Count of skipped phases.
    pub skipped_phases: usize,
    /// Whether total was within budget.
    pub within_budget: bool,
}

impl BudgetSummary {
    /// Format as display string.
    pub fn display(&self) -> String {
        let status = if self.within_budget { "OK" } else { "OVER" };
        let phase_str: Vec<String> = self
            .phases
            .iter()
            .map(|(p, ms, timeout)| {
                let marker = if *timeout { "!" } else { "" };
                format!("{}{}:{}ms", p.label(), marker, ms)
            })
            .collect();

        format!(
            "[{}] {}ms total | {} | timeouts:{} skipped:{}",
            status,
            self.total_ms,
            phase_str.join(" | "),
            self.timed_out_phases,
            self.skipped_phases
        )
    }
}

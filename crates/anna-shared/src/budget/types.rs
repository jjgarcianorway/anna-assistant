//! Budget types and constants (v0.0.199).

use serde::{Deserialize, Serialize};

// === LLM Token Budget Constants (v0.0.41) ===

/// Max tokens for LLM draft responses (keep tight for speed)
pub const LLM_MAX_DRAFT_TOKENS: u32 = 800;
/// Max tokens for LLM specialist responses
pub const LLM_MAX_SPECIALIST_TOKENS: u32 = 1200;
/// Max context tokens for local LLM (8k context models)
pub const LLM_MAX_CONTEXT_TOKENS: u32 = 6000;
/// Translator timeout in seconds (triggers fallback)
pub const TRANSLATOR_TIMEOUT_SECS: u64 = 30;
/// Specialist timeout in seconds (triggers graceful degradation)
pub const SPECIALIST_TIMEOUT_SECS: u64 = 45;

/// Stage names for budget tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Translator,
    Probes,
    Specialist,
    Supervisor,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Translator => write!(f, "translator"),
            Self::Probes => write!(f, "probes"),
            Self::Specialist => write!(f, "specialist"),
            Self::Supervisor => write!(f, "supervisor"),
        }
    }
}

/// Stage timing result for logging/diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    pub stage: Stage,
    pub elapsed_ms: u64,
    pub budget_ms: u64,
    pub exceeded: bool,
}

impl StageTiming {
    pub fn new(stage: Stage, elapsed_ms: u64, budget: &super::stage::StageBudget) -> Self {
        let budget_ms = budget.get(stage);
        Self {
            stage,
            elapsed_ms,
            budget_ms,
            exceeded: elapsed_ms > budget_ms,
        }
    }
}

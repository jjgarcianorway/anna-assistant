//! METER: Stage-level latency budgets with explicit degradation (v0.0.199).
//!
//! Provides configurable budgets per stage and budget enforcement.
//! Pure decision functions for testability.
//!
//! v0.0.36: Added ProbeBudget for controlling probe resource usage.
//! v0.0.41: Added LLM token budgets and timeout fallback constants.
//! v0.0.199: Modularized into domain-focused submodules.

mod llm;
mod probe;
mod stage;
mod types;

// Re-export all types and functions
pub use llm::{check_llm_fallback, LlmBudget, LlmFallback};
pub use probe::{ProbeBudget, ProbeBudgetCheck};
pub use stage::{
    check_stage_budget, check_total_budget, BudgetCheck, BudgetEnforcer, StageBudget,
};
pub use types::{
    Stage, StageTiming, LLM_MAX_CONTEXT_TOKENS, LLM_MAX_DRAFT_TOKENS, LLM_MAX_SPECIALIST_TOKENS,
    SPECIALIST_TIMEOUT_SECS, TRANSLATOR_TIMEOUT_SECS,
};

//! Fast Pipeline - v0.0.438.
//!
//! Eliminate timeout parse errors and make response time predictable and fast.
//! Key principles:
//! - Hard time budgets per phase
//! - Short JSON-only specialist outputs
//! - No streaming for parsed calls
//! - Probe-only fallback when specialists fail
//! - Parallel probe execution with caching
//! - Honest progress rendering
//! - Reality-based reliability stats

pub mod budget;
pub mod specialist_v2;
pub mod no_stream;
pub mod retry;
pub mod probe_fallback;
pub mod parallel_probes;
pub mod progress;
pub mod reliability_stats;

pub use budget::{
    PhaseBudget, TimeBudgets, BudgetResult, BudgetTracker, Phase,
    TRANSLATOR_BUDGET_MS, PROBE_BUDGET_MS, JUNIOR_BUDGET_MS,
    SENIOR_BUDGET_MS, RENDERER_BUDGET_MS, TOTAL_BUDGET_MS,
};
pub use specialist_v2::{
    SpecialistOutputV2, Verdict, AnswerPayload, SpecialistParser,
    MAX_SPECIALIST_TOKENS, MAX_SUMMARY_CHARS, MAX_NOTES_CHARS,
};
pub use no_stream::{CallPolicy, ModelCallConfig, enforce_no_stream};
pub use retry::{RetryStrategy, RetryResult, RetryConfig};
pub use probe_fallback::{ProbeFallbackEngine, FallbackAnswer, ProbeOnlyResult};
pub use parallel_probes::{ParallelProbeEngine, ProbeCache, ProbeBatch};
pub use progress::{PhaseProgress, ProgressRenderer, PhaseStatus};
pub use reliability_stats::{ReliabilityStats, ReliabilityOutcome};

/// Pipeline version.
pub const PIPELINE_VERSION: &str = "2";

/// Whether to enable slow deep analysis mode (user must explicitly opt-in).
pub const SLOW_ANALYSIS_ENABLED: bool = false;

/// Deep analysis budget (only if user opts in).
pub const DEEP_ANALYSIS_BUDGET_MS: u64 = 30_000;

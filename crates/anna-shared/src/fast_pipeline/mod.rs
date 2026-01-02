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
pub mod budget_tracker;
pub mod budget_types;
pub mod no_stream;
pub mod parallel_probes;
pub mod probe_fallback;
pub mod progress;
pub mod progress_pipeline;
pub mod progress_renderer;
pub mod progress_types;
pub mod reliability_stats;
pub mod retry;
pub mod specialist_v2;
pub mod specialist_v2_parser;
pub mod specialist_v2_types;

pub use budget::{
    BudgetResult, BudgetSummary, BudgetTracker, Phase, PhaseBudget, TimeBudgets, JUNIOR_BUDGET_MS,
    PROBE_BUDGET_MS, RENDERER_BUDGET_MS, SENIOR_BUDGET_MS, TOTAL_BUDGET_MS, TRANSLATOR_BUDGET_MS,
};
pub use no_stream::{enforce_no_stream, CallPolicy, ModelCallConfig};
pub use parallel_probes::{ParallelProbeEngine, ProbeBatch, ProbeCache};
pub use probe_fallback::{FallbackAnswer, ProbeFallbackEngine, ProbeOnlyResult};
pub use progress::{PhaseProgress, PhaseStatus, PipelineProgress, ProgressRenderer};
pub use reliability_stats::{ExecutionRecord, ReliabilityOutcome, ReliabilityStats, WindowedStats};
pub use retry::{RetryConfig, RetryResult, RetryStrategy};
pub use specialist_v2::{
    AnswerPayload, ParseResult, SpecialistOutputV2, SpecialistParser, ValidationResult, Verdict,
    MAX_ANSWER_FIELDS, MAX_NOTES_CHARS, MAX_SPECIALIST_TOKENS, MAX_SUMMARY_CHARS,
};

/// Pipeline version.
pub const PIPELINE_VERSION: &str = "2";

/// Whether to enable slow deep analysis mode (user must explicitly opt-in).
pub const SLOW_ANALYSIS_ENABLED: bool = false;

/// Deep analysis budget (only if user opts in).
pub const DEEP_ANALYSIS_BUDGET_MS: u64 = 30_000;

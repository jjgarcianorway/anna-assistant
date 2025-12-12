//! Robustness Layer (v0.0.433).
//!
//! Provides strict contracts, timeouts, failure handling, and truthful stats.
//! Key principles:
//! - Specialists cannot silently fail or timeout
//! - Renderer never pretends success when it's not
//! - Time budgets are enforced and visible
//! - Stats reflect actual outcomes, not any response

mod contract;
mod timeouts;
mod json_enforce;
mod outcome_messages;
mod stats_engine;
mod lifecycle;
mod fallback;
mod performance;
mod tests;

pub use contract::{
    EvidenceRef, ProposedStep, SpecialistResult, StepCategory, TicketMetrics, TicketOutcome,
};
pub use timeouts::{
    RetryStrategy, StageTiming, TimeBudget, TimeoutConfig, TimeoutEnforcer, TimeoutStage,
};
pub use json_enforce::{JsonEnforcer, JsonParseEvent, ParseResult, SchemaHint};
pub use outcome_messages::{OutcomeMessage, OutcomeRenderer, UserMessage};
pub use stats_engine::{
    DepartmentStats, FailureRecord, StaffStats, StatsEngine, TicketStats, TruthfulStats,
};
pub use lifecycle::{TicketLifecycle, TicketState, TicketTransition};
pub use fallback::{FallbackAnswer, FallbackGenerator, ProbeEvidence};
pub use performance::{PerformanceTracker, StreamingUpdate, TimingBreakdown};

/// Default time budgets (milliseconds).
pub const TRANSLATOR_HARD_CAP_MS: u64 = 1000;
pub const JUNIOR_SOFT_BUDGET_MS: u64 = 4000;
pub const JUNIOR_HARD_CAP_MS: u64 = 8000;
pub const SENIOR_SOFT_BUDGET_MS: u64 = 10000;
pub const SENIOR_HARD_CAP_MS: u64 = 20000;
pub const GLOBAL_HARD_CAP_MS: u64 = 25000;

/// Maximum retry attempts for parse errors.
pub const MAX_PARSE_RETRIES: usize = 1;

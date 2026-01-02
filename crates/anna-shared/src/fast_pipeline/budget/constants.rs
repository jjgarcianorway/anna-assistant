//! Hard Time Budget Constants - v0.0.438.
//!
//! Define explicit budgets per phase:
//! - translator_intent: 700ms
//! - probe_collection: 2500ms (can be parallel)
//! - junior_specialist: 1500ms
//! - senior_specialist: 3500ms (only on escalation)
//! - renderer: 200ms
//!
//! Overall max for one-shot queries: 6.5s

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

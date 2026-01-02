//! Deterministic-First Policy (v0.0.445).
//!
//! Before involving ANY LLM, check if deterministic probes can answer.
//!
//! LLMs are ONLY allowed for:
//! - Interpretation
//! - Diagnosis
//! - Explanation
//! - Multi-step reasoning
//!
//! This is a HARD routing rule.

mod domain;
mod formatters;
mod policy;
mod route;

// Re-exports
pub use domain::QueryDomain;
pub use formatters::format_deterministic_answer;
pub use policy::DeterministicPolicy;
pub use route::DeterministicRoute;

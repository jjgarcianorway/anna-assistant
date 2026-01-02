//! Probe Plan - Dynamic Probe Composition (v0.0.435).
//!
//! Anna builds a ProbePlan at runtime by selecting primitives based on
//! ticket intent and domain keywords.

mod executor;
mod output;
mod parsers;
mod plan;

pub use executor::ProbeExecutor;
pub use output::{ParsedKind, ParsedOutput, ProbeOutput};
pub use plan::{ProbePlan, ProbeSelection};

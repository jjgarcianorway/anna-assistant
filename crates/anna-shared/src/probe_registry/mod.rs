//! Probe Registry - Composable system probes (v0.0.410).
//!
//! Centralized definitions for all probes Anna can run.
//! Each probe has:
//! - Unique ID
//! - Shell command
//! - Domain/tags for matching
//! - Cost (cheap/medium/expensive)
//! - Selection predicates

mod builtin;
mod registry;
mod types;

// Re-export public API
pub use registry::ProbeRegistry;
pub use types::{ProbeCost, ProbeDef};

//! Integration with probes and specialists (v0.0.434).
//!
//! Connects hardware-aware system to probes and specialist responses.

pub mod probe;
pub mod specialist;

pub use probe::{ProbeCommand, ProbeHelper};
pub use specialist::{HelperSuggestion, ModelAvailability, SpecialistHelper};

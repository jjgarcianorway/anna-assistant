//! Deterministic route types.

use serde::{Deserialize, Serialize};

/// Deterministic route types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterministicRoute {
    /// Pure deterministic - probes only, no LLM
    ProbesOnly,
    /// Deterministic with formatting - probes + LLM for presentation
    ProbesWithFormat,
    /// Requires LLM - interpretation, diagnosis, or explanation
    RequiresLlm,
}

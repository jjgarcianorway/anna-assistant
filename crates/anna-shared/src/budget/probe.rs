//! Probe budget management (v0.0.199).

use serde::{Deserialize, Serialize};

/// Budget for probe resource usage.
/// Limits the number of probes and total output size to prevent runaway costs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProbeBudget {
    /// Maximum number of probes to run
    pub max_probes: usize,
    /// Maximum total probe output in bytes
    pub max_output_bytes: usize,
    /// Per-probe output cap in bytes
    pub per_probe_cap_bytes: usize,
}

impl Default for ProbeBudget {
    fn default() -> Self {
        Self {
            max_probes: 4,               // Match fast path probe count
            max_output_bytes: 64_000,    // 64KB total
            per_probe_cap_bytes: 16_000, // 16KB per probe
        }
    }
}

impl ProbeBudget {
    /// Create a minimal probe budget for fast path queries
    pub fn fast_path() -> Self {
        Self {
            max_probes: 4,
            max_output_bytes: 32_000,
            per_probe_cap_bytes: 8_000,
        }
    }

    /// Create a standard probe budget for specialist queries
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create an extended probe budget for complex queries
    pub fn extended() -> Self {
        Self {
            max_probes: 6,
            max_output_bytes: 128_000,
            per_probe_cap_bytes: 32_000,
        }
    }

    /// Check if adding output would exceed budget
    pub fn would_exceed(&self, current_bytes: usize, new_bytes: usize) -> bool {
        current_bytes + new_bytes > self.max_output_bytes
    }

    /// Cap output to per-probe limit
    pub fn cap_output(&self, output: &str) -> String {
        if output.len() <= self.per_probe_cap_bytes {
            output.to_string()
        } else {
            let truncated = &output[..self.per_probe_cap_bytes];
            format!(
                "{}... [truncated, {} bytes exceeded cap]",
                truncated,
                output.len() - self.per_probe_cap_bytes
            )
        }
    }
}

/// Result of probe budget check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeBudgetCheck {
    /// Within budget
    Ok,
    /// Probe count exceeded
    ProbeCountExceeded { limit: usize, attempted: usize },
    /// Output size exceeded
    OutputSizeExceeded { limit: usize, current: usize },
}

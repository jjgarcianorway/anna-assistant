//! Configuration for ClaimGate.

use serde::{Deserialize, Serialize};

/// Configuration for ClaimGate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimGateConfig {
    /// Require at least this many evidence items
    pub min_evidence_count: usize,
    /// Minimum confidence for any claim
    pub min_confidence: f32,
    /// Allow user-provided evidence to count
    pub allow_user_evidence: bool,
    /// Maximum age of cached evidence (seconds)
    pub evidence_cache_ttl: u64,
}

impl Default for ClaimGateConfig {
    fn default() -> Self {
        Self {
            min_evidence_count: 1,
            min_confidence: 0.6,
            allow_user_evidence: false,
            evidence_cache_ttl: 60,
        }
    }
}

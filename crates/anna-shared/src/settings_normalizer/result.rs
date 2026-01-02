// v0.0.645: Settings Normalizer Result (Phase 221)
// Result types for normalization operations

use serde::{Deserialize, Serialize};

use super::types::{NormalizationType, NormalizationRule};

/// Normalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    /// Original value
    pub original: String,
    /// Normalized value
    pub normalized: String,
    /// Normalization type
    pub normalization_type: NormalizationType,
    /// Rule applied
    pub rule_applied: NormalizationRule,
    /// Was modified
    pub modified: bool,
}

impl NormalizationResult {
    /// Create new result
    pub fn new(
        original: impl Into<String>,
        normalized: impl Into<String>,
        normalization_type: NormalizationType,
        rule_applied: NormalizationRule,
    ) -> Self {
        let orig = original.into();
        let norm = normalized.into();
        let modified = orig != norm;
        Self {
            original: orig,
            normalized: norm,
            normalization_type,
            rule_applied,
            modified,
        }
    }

    /// Was modified
    pub fn was_modified(&self) -> bool {
        self.modified
    }

    /// Get normalized value
    pub fn value(&self) -> &str {
        &self.normalized
    }
}

// v0.0.668: Denormalization Result
// Result type for denormalization operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Denormalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenormalizationResult {
    /// Denormalized settings
    pub settings: HashMap<String, String>,
    /// Keys expanded
    pub keys_expanded: usize,
    /// Keys prefixed
    pub keys_prefixed: usize,
    /// Keys suffixed
    pub keys_suffixed: usize,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
}

impl DenormalizationResult {
    /// Create success result
    pub fn success(settings: HashMap<String, String>) -> Self {
        Self {
            settings,
            keys_expanded: 0,
            keys_prefixed: 0,
            keys_suffixed: 0,
            success: true,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            settings: HashMap::new(),
            keys_expanded: 0,
            keys_prefixed: 0,
            keys_suffixed: 0,
            success: false,
            error: Some(error.into()),
        }
    }

    /// With counts
    pub fn with_counts(mut self, expanded: usize, prefixed: usize, suffixed: usize) -> Self {
        self.keys_expanded = expanded;
        self.keys_prefixed = prefixed;
        self.keys_suffixed = suffixed;
        self
    }

    /// Total changes
    pub fn total_changes(&self) -> usize {
        self.keys_expanded + self.keys_prefixed + self.keys_suffixed
    }
}

impl Default for DenormalizationResult {
    fn default() -> Self {
        Self::success(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_success() {
        let r = DenormalizationResult::success(HashMap::new());
        assert!(r.success);
    }

    #[test]
    fn test_result_failure() {
        let r = DenormalizationResult::failure("error");
        assert!(!r.success);
    }

    #[test]
    fn test_result_with_counts() {
        let r = DenormalizationResult::success(HashMap::new())
            .with_counts(5, 3, 2);
        assert_eq!(r.total_changes(), 10);
    }
}

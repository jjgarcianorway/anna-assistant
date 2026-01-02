// v0.0.680: Settings Expander Result (Phase 256)
// Result types for expansion operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ExpandMode;

/// Expand result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandResult {
    /// Expanded settings
    pub settings: HashMap<String, String>,
    /// Variables expanded
    pub variables_expanded: usize,
    /// Variables missing
    pub variables_missing: usize,
    /// Mode used
    pub mode: ExpandMode,
}

impl ExpandResult {
    /// Create new result
    pub fn new(settings: HashMap<String, String>, expanded: usize, missing: usize, mode: ExpandMode) -> Self {
        Self {
            settings,
            variables_expanded: expanded,
            variables_missing: missing,
            mode,
        }
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.variables_expanded + self.variables_missing;
        if total == 0 {
            1.0
        } else {
            self.variables_expanded as f64 / total as f64
        }
    }

    /// Has missing
    pub fn has_missing(&self) -> bool {
        self.variables_missing > 0
    }

    /// Get value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}

impl Default for ExpandResult {
    fn default() -> Self {
        Self::new(HashMap::new(), 0, 0, ExpandMode::Environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_new() {
        let r = ExpandResult::new(HashMap::new(), 5, 2, ExpandMode::Environment);
        assert_eq!(r.variables_expanded, 5);
        assert!(r.has_missing());
    }

    #[test]
    fn test_result_success_rate() {
        let r = ExpandResult::new(HashMap::new(), 8, 2, ExpandMode::Environment);
        assert!((r.success_rate() - 0.8).abs() < 0.001);
    }
}

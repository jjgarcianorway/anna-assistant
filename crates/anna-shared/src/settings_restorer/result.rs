// v0.0.659: Settings Restorer - Restore Result
// Results from restore operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::mode::RestoreMode;

/// Restore result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    /// Restored settings
    pub restored: HashMap<String, String>,
    /// Keys restored
    pub keys_restored: Vec<String>,
    /// Keys skipped
    pub keys_skipped: Vec<String>,
    /// Keys failed
    pub keys_failed: Vec<String>,
    /// Restore mode used
    pub mode: RestoreMode,
}

impl RestoreResult {
    /// Create new result
    pub fn new(mode: RestoreMode) -> Self {
        Self {
            restored: HashMap::new(),
            keys_restored: Vec::new(),
            keys_skipped: Vec::new(),
            keys_failed: Vec::new(),
            mode,
        }
    }

    /// Add restored
    pub fn add_restored(&mut self, key: String, value: String) {
        self.restored.insert(key.clone(), value);
        self.keys_restored.push(key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.keys_skipped.push(key);
    }

    /// Add failed
    pub fn add_failed(&mut self, key: String) {
        self.keys_failed.push(key);
    }

    /// Total restored
    pub fn total_restored(&self) -> usize {
        self.restored.len()
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.keys_failed.is_empty()
    }

    /// Success
    pub fn success(&self) -> bool {
        self.keys_failed.is_empty() && !self.keys_restored.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_new() {
        let r = RestoreResult::new(RestoreMode::Full);
        assert_eq!(r.total_restored(), 0);
    }

    #[test]
    fn test_result_add_restored() {
        let mut r = RestoreResult::new(RestoreMode::Full);
        r.add_restored("key1".to_string(), "value1".to_string());
        assert_eq!(r.total_restored(), 1);
    }
}

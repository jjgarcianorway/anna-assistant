// v0.0.643: Settings Sanitizer Result (Phase 219)
// Result types for sanitization operations

use serde::{Deserialize, Serialize};

/// Sanitization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationResult {
    /// Original value
    pub original: String,
    /// Sanitized value
    pub sanitized: String,
    /// Changed
    pub changed: bool,
    /// Operations applied
    pub operations: Vec<String>,
}

impl SanitizationResult {
    /// Create new result
    pub fn new(original: impl Into<String>, sanitized: impl Into<String>) -> Self {
        let orig = original.into();
        let san = sanitized.into();
        let changed = orig != san;
        Self {
            original: orig,
            sanitized: san,
            changed,
            operations: Vec::new(),
        }
    }

    /// Add operation
    pub fn add_operation(&mut self, op: impl Into<String>) {
        self.operations.push(op.into());
    }

    /// Was changed
    pub fn was_changed(&self) -> bool {
        self.changed
    }

    /// Operation count
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_new() {
        let r = SanitizationResult::new("  test  ", "test");
        assert!(r.was_changed());
    }

    #[test]
    fn test_result_unchanged() {
        let r = SanitizationResult::new("test", "test");
        assert!(!r.was_changed());
    }
}

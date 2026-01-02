// v0.0.666: Settings Transform Rules (Phase 242)
// Transform rule and result types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::TransformType;

/// Transform rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    /// Rule ID
    pub id: String,
    /// Source pattern
    pub source_pattern: String,
    /// Target pattern
    pub target_pattern: String,
    /// Transform type
    pub transform_type: TransformType,
    /// Enabled
    pub enabled: bool,
}

impl TransformRule {
    /// Create new rule
    pub fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source_pattern: source.into(),
            target_pattern: target.into(),
            transform_type: TransformType::Map,
            enabled: true,
        }
    }

    /// With transform type
    pub fn with_type(mut self, transform_type: TransformType) -> Self {
        self.transform_type = transform_type;
        self
    }

    /// Set enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Transform result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// Transformed settings
    pub settings: HashMap<String, String>,
    /// Rules applied
    pub rules_applied: Vec<String>,
    /// Keys transformed
    pub keys_transformed: usize,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
}

impl TransformResult {
    /// Create success result
    pub fn success(settings: HashMap<String, String>) -> Self {
        Self {
            settings,
            rules_applied: Vec::new(),
            keys_transformed: 0,
            success: true,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            settings: HashMap::new(),
            rules_applied: Vec::new(),
            keys_transformed: 0,
            success: false,
            error: Some(error.into()),
        }
    }

    /// With rules
    pub fn with_rules(mut self, rules: Vec<String>) -> Self {
        self.rules_applied = rules;
        self
    }

    /// With transformed count
    pub fn with_transformed(mut self, count: usize) -> Self {
        self.keys_transformed = count;
        self
    }
}

impl Default for TransformResult {
    fn default() -> Self {
        Self::success(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_new() {
        let r = TransformRule::new("r1", "src", "tgt");
        assert!(r.enabled);
    }

    #[test]
    fn test_rule_with_type() {
        let r = TransformRule::new("r1", "s", "t").with_type(TransformType::Filter);
        assert_eq!(r.transform_type, TransformType::Filter);
    }

    #[test]
    fn test_result_success() {
        let r = TransformResult::success(HashMap::new());
        assert!(r.success);
    }

    #[test]
    fn test_result_failure() {
        let r = TransformResult::failure("error");
        assert!(!r.success);
    }
}

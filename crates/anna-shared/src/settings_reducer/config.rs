// v0.0.677: Settings Reducer Config (Phase 253)
// Configuration for reducer operations

use serde::{Deserialize, Serialize};
use super::types::{ReduceOp, ReduceTarget};

/// Reducer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducerConfig {
    /// Default operation
    pub default_op: ReduceOp,
    /// Default target
    pub default_target: ReduceTarget,
    /// Concat separator
    pub concat_separator: String,
    /// Skip non-numeric
    pub skip_non_numeric: bool,
}

impl ReducerConfig {
    /// Create new config
    pub fn new(op: ReduceOp) -> Self {
        Self {
            default_op: op,
            default_target: ReduceTarget::Values,
            concat_separator: ", ".to_string(),
            skip_non_numeric: true,
        }
    }

    /// Set target
    pub fn target(mut self, target: ReduceTarget) -> Self {
        self.default_target = target;
        self
    }

    /// Set concat separator
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.concat_separator = sep.into();
        self
    }

    /// Set skip non-numeric
    pub fn skip_non_numeric(mut self, skip: bool) -> Self {
        self.skip_non_numeric = skip;
        self
    }
}

impl Default for ReducerConfig {
    fn default() -> Self {
        Self::new(ReduceOp::Count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ReducerConfig::new(ReduceOp::Sum);
        assert_eq!(c.default_op, ReduceOp::Sum);
    }

    #[test]
    fn test_config_builder() {
        let c = ReducerConfig::new(ReduceOp::Concat)
            .separator("; ")
            .target(ReduceTarget::Keys);
        assert_eq!(c.concat_separator, "; ");
        assert_eq!(c.default_target, ReduceTarget::Keys);
    }
}

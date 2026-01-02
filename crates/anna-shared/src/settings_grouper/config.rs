// v0.0.676: Settings Grouper - Configuration (Phase 252)
// Grouper configuration

use serde::{Deserialize, Serialize};
use super::types::GroupByField;

/// Grouper config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrouperConfig {
    /// Default group by field
    pub default_field: GroupByField,
    /// Prefix delimiter
    pub prefix_delimiter: String,
    /// Suffix delimiter
    pub suffix_delimiter: String,
    /// Min group size
    pub min_group_size: usize,
}

impl GrouperConfig {
    /// Create new config
    pub fn new(field: GroupByField) -> Self {
        Self {
            default_field: field,
            prefix_delimiter: ".".to_string(),
            suffix_delimiter: "_".to_string(),
            min_group_size: 1,
        }
    }

    /// Set prefix delimiter
    pub fn prefix_delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.prefix_delimiter = delimiter.into();
        self
    }

    /// Set suffix delimiter
    pub fn suffix_delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.suffix_delimiter = delimiter.into();
        self
    }

    /// Set min group size
    pub fn min_group_size(mut self, size: usize) -> Self {
        self.min_group_size = size;
        self
    }
}

impl Default for GrouperConfig {
    fn default() -> Self {
        Self::new(GroupByField::KeyPrefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = GrouperConfig::new(GroupByField::KeyPrefix);
        assert_eq!(c.prefix_delimiter, ".");
    }

    #[test]
    fn test_config_builder() {
        let c = GrouperConfig::new(GroupByField::Value)
            .prefix_delimiter(":")
            .min_group_size(2);
        assert_eq!(c.prefix_delimiter, ":");
        assert_eq!(c.min_group_size, 2);
    }
}

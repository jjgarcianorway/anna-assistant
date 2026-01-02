// v0.0.762: Settings Field Config (Phase 338)
// Field configuration

use serde::{Deserialize, Serialize};
use super::types::{FieldType, FieldStatus};

/// Field config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    /// Name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Status
    pub status: FieldStatus,
    /// Max crops
    pub max_crops: usize,
}

impl FieldConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            field_type: FieldType::Arable,
            status: FieldStatus::Prepared,
            max_crops: 100,
        }
    }

    /// Set type
    pub fn field_type(mut self, ft: FieldType) -> Self {
        self.field_type = ft;
        self
    }

    /// Set status
    pub fn status(mut self, s: FieldStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max crops
    pub fn max_crops(mut self, max: usize) -> Self {
        self.max_crops = max;
        self
    }
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = FieldConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = FieldConfig::new("test")
            .field_type(FieldType::Orchard)
            .status(FieldStatus::Growing);
        assert_eq!(c.field_type, FieldType::Orchard);
        assert_eq!(c.status, FieldStatus::Growing);
    }
}

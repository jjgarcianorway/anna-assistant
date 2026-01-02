// v0.0.641: Settings Inspector Config (Phase 217)
// Configuration for inspector behavior

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{InspectionDepth, InspectionType};

/// Inspector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorConfig {
    /// Inspection type
    pub inspection_type: InspectionType,
    /// Depth
    pub depth: InspectionDepth,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include metadata
    pub include_metadata: bool,
    /// Include defaults
    pub include_defaults: bool,
}

impl InspectorConfig {
    /// Create new config
    pub fn new(inspection_type: InspectionType) -> Self {
        Self {
            inspection_type,
            depth: InspectionDepth::Normal,
            category: None,
            include_metadata: true,
            include_defaults: false,
        }
    }

    /// Set depth
    pub fn depth(mut self, depth: InspectionDepth) -> Self {
        self.depth = depth;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include metadata
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set include defaults
    pub fn include_defaults(mut self, include: bool) -> Self {
        self.include_defaults = include;
        self
    }
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self::new(InspectionType::Structure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = InspectorConfig::new(InspectionType::Structure);
        assert!(c.include_metadata);
    }

    #[test]
    fn test_config_builder() {
        let c = InspectorConfig::new(InspectionType::Full)
            .depth(InspectionDepth::Deep)
            .include_defaults(true);
        assert_eq!(c.depth, InspectionDepth::Deep);
        assert!(c.include_defaults);
    }
}

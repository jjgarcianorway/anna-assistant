// v0.0.651: Settings Mapper Types (Phase 227)
// Type definitions for settings mapping

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Mapping type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MappingType {
    /// Direct mapping
    #[default]
    Direct,
    /// Rename mapping
    Rename,
    /// Transform mapping
    Transform,
    /// Merge mapping
    Merge,
    /// Split mapping
    Split,
}

impl std::fmt::Display for MappingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::Rename => write!(f, "rename"),
            Self::Transform => write!(f, "transform"),
            Self::Merge => write!(f, "merge"),
            Self::Split => write!(f, "split"),
        }
    }
}

/// Mapping direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MappingDirection {
    /// Forward mapping
    #[default]
    Forward,
    /// Reverse mapping
    Reverse,
    /// Bidirectional mapping
    Bidirectional,
}

impl std::fmt::Display for MappingDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forward => write!(f, "forward"),
            Self::Reverse => write!(f, "reverse"),
            Self::Bidirectional => write!(f, "bidirectional"),
        }
    }
}

/// Mapping rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingRule {
    /// Source key
    pub source: String,
    /// Target key
    pub target: String,
    /// Mapping type
    pub mapping_type: MappingType,
    /// Optional default value
    pub default: Option<String>,
}

impl MappingRule {
    /// Create new rule
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            mapping_type: MappingType::Direct,
            default: None,
        }
    }

    /// Set mapping type
    pub fn mapping_type(mut self, mapping_type: MappingType) -> Self {
        self.mapping_type = mapping_type;
        self
    }

    /// Set default value
    pub fn default_value(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// Mapper config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapperConfig {
    /// Mapping direction
    pub direction: MappingDirection,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Case sensitive
    pub case_sensitive: bool,
    /// Skip unmapped
    pub skip_unmapped: bool,
}

impl MapperConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            direction: MappingDirection::Forward,
            category: None,
            case_sensitive: true,
            skip_unmapped: false,
        }
    }

    /// Set direction
    pub fn direction(mut self, direction: MappingDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set case sensitive
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Set skip unmapped
    pub fn skip_unmapped(mut self, skip: bool) -> Self {
        self.skip_unmapped = skip;
        self
    }
}

impl Default for MapperConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Mapping result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingResult {
    /// Mapped values
    pub values: HashMap<String, String>,
    /// Unmapped keys
    pub unmapped: Vec<String>,
    /// Applied rules count
    pub rules_applied: usize,
    /// Direction used
    pub direction: MappingDirection,
}

impl MappingResult {
    /// Create new result
    pub fn new(direction: MappingDirection) -> Self {
        Self {
            values: HashMap::new(),
            unmapped: Vec::new(),
            rules_applied: 0,
            direction,
        }
    }

    /// Add mapped value
    pub fn add(&mut self, key: String, value: String) {
        self.values.insert(key, value);
    }

    /// Add unmapped key
    pub fn add_unmapped(&mut self, key: String) {
        self.unmapped.push(key);
    }

    /// Value count
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    /// Has unmapped
    pub fn has_unmapped(&self) -> bool {
        !self.unmapped.is_empty()
    }
}

/// Mapper stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapperStats {
    /// Total mappings
    pub total_mappings: usize,
    /// Total keys mapped
    pub keys_mapped: usize,
    /// Total unmapped
    pub keys_unmapped: usize,
    /// By direction
    pub by_direction: HashMap<String, usize>,
}

impl MapperStats {
    /// Record mapping
    pub fn record(&mut self, direction: MappingDirection, mapped: usize, unmapped: usize) {
        self.total_mappings += 1;
        self.keys_mapped += mapped;
        self.keys_unmapped += unmapped;
        *self.by_direction.entry(direction.to_string()).or_insert(0) += 1;
    }

    /// Mapping efficiency
    pub fn efficiency(&self) -> f64 {
        let total = self.keys_mapped + self.keys_unmapped;
        if total == 0 {
            0.0
        } else {
            self.keys_mapped as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_type_display() {
        assert_eq!(format!("{}", MappingType::Direct), "direct");
        assert_eq!(format!("{}", MappingType::Rename), "rename");
    }

    #[test]
    fn test_mapping_direction_display() {
        assert_eq!(format!("{}", MappingDirection::Forward), "forward");
        assert_eq!(format!("{}", MappingDirection::Reverse), "reverse");
    }

    #[test]
    fn test_rule_new() {
        let r = MappingRule::new("src", "dst");
        assert_eq!(r.source, "src");
        assert_eq!(r.target, "dst");
    }

    #[test]
    fn test_config_new() {
        let c = MapperConfig::new();
        assert!(c.case_sensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = MapperConfig::new()
            .direction(MappingDirection::Reverse)
            .skip_unmapped(true);
        assert_eq!(c.direction, MappingDirection::Reverse);
        assert!(c.skip_unmapped);
    }

    #[test]
    fn test_result_new() {
        let r = MappingResult::new(MappingDirection::Forward);
        assert_eq!(r.value_count(), 0);
    }

    #[test]
    fn test_stats_record() {
        let mut s = MapperStats::default();
        s.record(MappingDirection::Forward, 10, 2);
        assert_eq!(s.total_mappings, 1);
        assert_eq!(s.keys_mapped, 10);
    }
}

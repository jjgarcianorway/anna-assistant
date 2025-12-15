// v0.0.651: Settings Mapper (Phase 227)
// Mapper for key transformations and field mapping

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

/// Settings mapper
#[derive(Debug, Clone, Default)]
pub struct SettingsMapper {
    /// Config
    config: MapperConfig,
    /// Rules
    rules: Vec<MappingRule>,
    /// Results
    results: Vec<MappingResult>,
    /// Stats
    stats: MapperStats,
}

impl SettingsMapper {
    /// Create new mapper
    pub fn new(config: MapperConfig) -> Self {
        Self {
            config,
            rules: Vec::new(),
            results: Vec::new(),
            stats: MapperStats::default(),
        }
    }

    /// Add mapping rule
    pub fn add_rule(&mut self, rule: MappingRule) {
        self.rules.push(rule);
    }

    /// Map settings
    pub fn map(&mut self, settings: &HashMap<String, String>) -> MappingResult {
        let mut result = MappingResult::new(self.config.direction);

        for (key, value) in settings {
            let lookup_key = if self.config.case_sensitive {
                key.clone()
            } else {
                key.to_lowercase()
            };

            if let Some(rule) = self.find_rule(&lookup_key) {
                let target_key = rule.target.clone();
                result.add(target_key, value.clone());
                result.rules_applied += 1;
            } else if !self.config.skip_unmapped {
                result.add(key.clone(), value.clone());
            } else {
                result.add_unmapped(key.clone());
            }
        }

        self.stats.record(
            self.config.direction,
            result.value_count(),
            result.unmapped.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Find matching rule
    fn find_rule(&self, key: &str) -> Option<&MappingRule> {
        self.rules.iter().find(|r| {
            if self.config.case_sensitive {
                r.source == key
            } else {
                r.source.to_lowercase() == key.to_lowercase()
            }
        })
    }

    /// Get results
    pub fn results(&self) -> &[MappingResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &MapperStats {
        &self.stats
    }

    /// Rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

/// Settings mapper registry
#[derive(Debug, Clone, Default)]
pub struct SettingsMapperRegistry {
    /// Mappers by ID
    mappers: HashMap<String, SettingsMapper>,
}

impl SettingsMapperRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register mapper
    pub fn register(&mut self, id: impl Into<String>, mapper: SettingsMapper) {
        self.mappers.insert(id.into(), mapper);
    }

    /// Unregister mapper
    pub fn unregister(&mut self, id: &str) -> bool {
        self.mappers.remove(id).is_some()
    }

    /// Get mapper
    pub fn get(&self, id: &str) -> Option<&SettingsMapper> {
        self.mappers.get(id)
    }

    /// Get mapper mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMapper> {
        self.mappers.get_mut(id)
    }

    /// Mapper count
    pub fn count(&self) -> usize {
        self.mappers.len()
    }
}

/// Format mapper registry
pub fn format_mapper_registry(registry: &SettingsMapperRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Mapper Registry:\n");
    output.push_str(&format!("  Mappers: {}\n", registry.count()));
    output
}

/// Check if query is about mapper
pub fn is_mapper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("mapper") || lower.contains("map settings") || lower.contains("key mapping")
}

/// Fun fact about mapper
pub fn mapper_fun_fact() -> &'static str {
    "Anna's settings mappers transform keys between systems!"
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

    #[test]
    fn test_mapper_new() {
        let m = SettingsMapper::new(MapperConfig::new());
        assert_eq!(m.rule_count(), 0);
    }

    #[test]
    fn test_mapper_add_rule() {
        let mut m = SettingsMapper::new(MapperConfig::new());
        m.add_rule(MappingRule::new("old", "new"));
        assert_eq!(m.rule_count(), 1);
    }

    #[test]
    fn test_mapper_map() {
        let mut m = SettingsMapper::new(MapperConfig::new());
        m.add_rule(MappingRule::new("old_key", "new_key"));

        let mut settings = HashMap::new();
        settings.insert("old_key".to_string(), "value".to_string());

        let r = m.map(&settings);
        assert_eq!(r.values.get("new_key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsMapperRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsMapperRegistry::new();
        r.register("map1", SettingsMapper::new(MapperConfig::new()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_mapper_query() {
        assert!(is_mapper_query("settings mapper"));
        assert!(!is_mapper_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = mapper_fun_fact();
        assert!(fact.contains("mapper"));
    }
}

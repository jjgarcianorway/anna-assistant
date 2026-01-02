// v0.0.651: Settings Mapper Implementation (Phase 227)
// Core mapper logic for key transformations

use std::collections::HashMap;

use crate::settings_mapper::types::{
    MapperConfig, MapperStats, MappingResult, MappingRule,
};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}

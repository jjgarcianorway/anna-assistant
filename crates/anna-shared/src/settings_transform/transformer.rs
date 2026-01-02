// v0.0.666: Settings Transformer (Phase 242)
// Main transformer implementation

use std::collections::HashMap;
use super::config::TransformerConfig;
use super::rule::{TransformRule, TransformResult};
use super::stats::TransformerStats;

/// Settings transformer
#[derive(Debug, Clone, Default)]
pub struct SettingsTransformer {
    /// Config
    config: TransformerConfig,
    /// Rules
    rules: HashMap<String, TransformRule>,
    /// Stats
    stats: TransformerStats,
}

impl SettingsTransformer {
    /// Create new transformer
    pub fn new(config: TransformerConfig) -> Self {
        Self {
            config,
            rules: HashMap::new(),
            stats: TransformerStats::default(),
        }
    }

    /// Add rule
    pub fn add_rule(&mut self, rule: TransformRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Remove rule
    pub fn remove_rule(&mut self, id: &str) -> bool {
        self.rules.remove(id).is_some()
    }

    /// Get rule
    pub fn get_rule(&self, id: &str) -> Option<&TransformRule> {
        self.rules.get(id)
    }

    /// Transform settings
    pub fn transform(&mut self, settings: &HashMap<String, String>) -> TransformResult {
        let mut result_settings = if self.config.preserve_originals {
            settings.clone()
        } else {
            HashMap::new()
        };

        let enabled_rules: Vec<_> = self.rules.values()
            .filter(|r| r.enabled)
            .collect();

        let mut rules_applied = Vec::new();
        let mut keys_transformed = 0;

        for (key, value) in settings {
            for rule in &enabled_rules {
                if key.starts_with(&rule.source_pattern) {
                    let new_key = key.replacen(&rule.source_pattern, &rule.target_pattern, 1);
                    result_settings.insert(new_key, value.clone());
                    if !rules_applied.contains(&rule.id) {
                        rules_applied.push(rule.id.clone());
                    }
                    keys_transformed += 1;
                    self.stats.record_type(rule.transform_type);
                }
            }
            if !self.config.preserve_originals && keys_transformed == 0 {
                result_settings.insert(key.clone(), value.clone());
            }
        }

        let result = TransformResult::success(result_settings)
            .with_rules(rules_applied)
            .with_transformed(keys_transformed);

        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &TransformerStats {
        &self.stats
    }

    /// Rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Enabled count
    pub fn enabled_count(&self) -> usize {
        self.rules.values().filter(|r| r.enabled).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_new() {
        let t = SettingsTransformer::new(TransformerConfig::default());
        assert_eq!(t.rule_count(), 0);
    }

    #[test]
    fn test_transformer_add_rule() {
        let mut t = SettingsTransformer::new(TransformerConfig::default());
        t.add_rule(TransformRule::new("r1", "src", "tgt"));
        assert_eq!(t.rule_count(), 1);
    }

    #[test]
    fn test_transformer_transform() {
        let mut t = SettingsTransformer::new(TransformerConfig::default());
        t.add_rule(TransformRule::new("r1", "old.", "new."));
        
        let mut settings = HashMap::new();
        settings.insert("old.key".to_string(), "value".to_string());
        
        let result = t.transform(&settings);
        assert!(result.success);
        assert!(result.settings.contains_key("new.key"));
    }
}

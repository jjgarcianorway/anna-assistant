// v0.0.674: Settings Filter Implementation (Phase 250)
// Main filter implementation

use std::collections::HashMap;
use super::types::{FilterConfig, FilterPredicate, FilterRule, FilterResult, FilterStats, FilterType};

/// Settings filter
#[derive(Debug, Clone, Default)]
pub struct SettingsFilter {
    /// Config
    config: FilterConfig,
    /// Rules
    rules: Vec<FilterRule>,
    /// Stats
    stats: FilterStats,
}

impl SettingsFilter {
    /// Create new filter
    pub fn new(config: FilterConfig) -> Self {
        Self {
            config,
            rules: Vec::new(),
            stats: FilterStats::default(),
        }
    }

    /// Add rule
    pub fn add_rule(&mut self, rule: FilterRule) {
        self.rules.push(rule);
    }

    /// Clear rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Filter by predicate
    pub fn filter_by(&mut self, settings: &HashMap<String, String>, predicate: FilterPredicate) -> FilterResult {
        let mut passed = HashMap::new();
        let mut filtered_out = 0;

        for (key, value) in settings {
            let val = if self.config.trim_values { value.trim() } else { value.as_str() };
            let rule = FilterRule::predicate("temp", predicate);
            
            let matches = if self.config.default_type == FilterType::Include || 
                           self.config.default_type == FilterType::AllowList {
                rule.evaluate(val)
            } else {
                !rule.evaluate(val)
            };

            if matches {
                passed.insert(key.clone(), value.clone());
            } else {
                filtered_out += 1;
            }
        }

        let result = FilterResult::new(passed, filtered_out);
        self.stats.record(&result, self.config.default_type);
        result
    }

    /// Filter with rules
    pub fn filter(&mut self, settings: &HashMap<String, String>) -> FilterResult {
        let enabled_rules: Vec<_> = self.rules.iter().filter(|r| r.enabled).collect();
        let mut passed = HashMap::new();
        let mut filtered_out = 0;
        let mut rules_applied = Vec::new();

        for (key, value) in settings {
            let val = if self.config.trim_values { value.trim() } else { value.as_str() };
            
            let matches = if self.config.chain_filters {
                enabled_rules.iter().all(|r| {
                    if !rules_applied.contains(&r.id) {
                        rules_applied.push(r.id.clone());
                    }
                    r.evaluate(val)
                })
            } else {
                enabled_rules.iter().any(|r| {
                    if !rules_applied.contains(&r.id) {
                        rules_applied.push(r.id.clone());
                    }
                    r.evaluate(val)
                })
            };

            if matches {
                passed.insert(key.clone(), value.clone());
            } else {
                filtered_out += 1;
            }
        }

        let result = FilterResult::new(passed, filtered_out).with_rules(rules_applied);
        self.stats.record(&result, self.config.default_type);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &FilterStats {
        &self.stats
    }

    /// Rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

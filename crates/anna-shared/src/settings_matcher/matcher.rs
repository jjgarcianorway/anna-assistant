// v0.0.687: Settings Matcher Core (Phase 263)
// Core matcher implementation

use std::collections::HashMap;

use super::items::{MatchItem, MatchResult};
use super::stats::MatcherStats;
use super::types::{MatchRule, MatchTarget, MatchType, MatcherConfig};

/// Settings matcher
#[derive(Debug, Clone, Default)]
pub struct SettingsMatcher {
    /// Config
    config: MatcherConfig,
    /// Rules
    rules: Vec<MatchRule>,
    /// Stats
    stats: MatcherStats,
}

impl SettingsMatcher {
    /// Create new matcher
    pub fn new(config: MatcherConfig) -> Self {
        Self {
            config,
            rules: Vec::new(),
            stats: MatcherStats::default(),
        }
    }

    /// Add rule
    pub fn add_rule(&mut self, rule: MatchRule) {
        self.rules.push(rule);
    }

    /// Remove rule
    pub fn remove_rule(&mut self, id: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        self.rules.len() != before
    }

    /// Check single match
    fn matches(&self, target: &str, pattern: &str, match_type: MatchType) -> bool {
        let (t, p) = if self.config.case_insensitive {
            (target.to_lowercase(), pattern.to_lowercase())
        } else {
            (target.to_string(), pattern.to_string())
        };

        let result = match match_type {
            MatchType::Exact => t == p,
            MatchType::Prefix => t.starts_with(&p),
            MatchType::Suffix => t.ends_with(&p),
            MatchType::Contains => t.contains(&p),
        };

        if self.config.invert { !result } else { result }
    }

    /// Match against pattern
    pub fn match_pattern(&mut self, settings: &HashMap<String, String>, pattern: &str) -> MatchResult {
        let mut items = Vec::new();

        for (key, value) in settings {
            let key_matches = self.matches(key, pattern, self.config.match_type);
            let value_matches = self.matches(value, pattern, self.config.match_type);

            let matched = match self.config.target {
                MatchTarget::Key => key_matches,
                MatchTarget::Value => value_matches,
                MatchTarget::Both => key_matches && value_matches,
                MatchTarget::Either => key_matches || value_matches,
            };

            if matched {
                let target = if key_matches && value_matches {
                    MatchTarget::Both
                } else if key_matches {
                    MatchTarget::Key
                } else {
                    MatchTarget::Value
                };
                items.push(MatchItem::new(key.clone(), value.clone(), vec!["pattern".to_string()], target));
            }
        }

        let result = MatchResult::new(items, settings.len(), 1);
        self.stats.record(&result, self.config.match_type);
        result
    }

    /// Match against rules
    pub fn match_rules(&mut self, settings: &HashMap<String, String>) -> MatchResult {
        let mut items: HashMap<String, MatchItem> = HashMap::new();

        for (key, value) in settings {
            let mut matched_rules = Vec::new();
            let mut matched_target = MatchTarget::Key;

            for rule in &self.rules {
                let key_matches = self.matches(key, &rule.pattern, rule.match_type);
                let value_matches = self.matches(value, &rule.pattern, rule.match_type);

                let matched = match rule.target {
                    MatchTarget::Key => key_matches,
                    MatchTarget::Value => value_matches,
                    MatchTarget::Both => key_matches && value_matches,
                    MatchTarget::Either => key_matches || value_matches,
                };

                if matched {
                    matched_rules.push(rule.id.clone());
                    if key_matches && value_matches {
                        matched_target = MatchTarget::Both;
                    } else if value_matches {
                        matched_target = MatchTarget::Value;
                    }
                }
            }

            if !matched_rules.is_empty() {
                items.insert(key.clone(), MatchItem::new(key.clone(), value.clone(), matched_rules, matched_target));
            }
        }

        let result = MatchResult::new(items.into_values().collect(), settings.len(), self.rules.len());
        self.stats.record(&result, self.config.match_type);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &MatcherStats {
        &self.stats
    }

    /// Rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher_new() {
        let m = SettingsMatcher::new(MatcherConfig::default());
        assert_eq!(m.rule_count(), 0);
    }

    #[test]
    fn test_matcher_add_rule() {
        let mut m = SettingsMatcher::new(MatcherConfig::default());
        m.add_rule(MatchRule::new("r1", "test", MatchType::Contains));
        assert_eq!(m.rule_count(), 1);
    }

    #[test]
    fn test_matcher_match_pattern() {
        let mut m = SettingsMatcher::new(MatcherConfig::new(MatchType::Contains));
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = m.match_pattern(&settings, "app");
        assert_eq!(result.total_matched, 1);
    }

    #[test]
    fn test_matcher_match_rules() {
        let mut m = SettingsMatcher::new(MatcherConfig::default());
        m.add_rule(MatchRule::new("app_rule", "app", MatchType::Prefix));

        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = m.match_rules(&settings);
        assert_eq!(result.total_matched, 1);
    }
}

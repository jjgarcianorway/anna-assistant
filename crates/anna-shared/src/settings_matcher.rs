// v0.0.687: Settings Matcher (Phase 263)
// Match settings against patterns and rules

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Match type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MatchType {
    /// Exact match
    #[default]
    Exact,
    /// Prefix match
    Prefix,
    /// Suffix match
    Suffix,
    /// Contains match
    Contains,
}

impl std::fmt::Display for MatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "exact"),
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
            Self::Contains => write!(f, "contains"),
        }
    }
}

/// Match target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MatchTarget {
    /// Match key
    #[default]
    Key,
    /// Match value
    Value,
    /// Match both
    Both,
    /// Match either
    Either,
}

impl std::fmt::Display for MatchTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key => write!(f, "key"),
            Self::Value => write!(f, "value"),
            Self::Both => write!(f, "both"),
            Self::Either => write!(f, "either"),
        }
    }
}

/// Matcher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatcherConfig {
    /// Match type
    pub match_type: MatchType,
    /// Match target
    pub target: MatchTarget,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Invert match
    pub invert: bool,
}

impl MatcherConfig {
    /// Create new config
    pub fn new(match_type: MatchType) -> Self {
        Self {
            match_type,
            target: MatchTarget::Key,
            case_insensitive: true,
            invert: false,
        }
    }

    /// Set target
    pub fn target(mut self, target: MatchTarget) -> Self {
        self.target = target;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set invert
    pub fn invert(mut self, inv: bool) -> Self {
        self.invert = inv;
        self
    }
}

impl Default for MatcherConfig {
    fn default() -> Self {
        Self::new(MatchType::Contains)
    }
}

/// Match rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    /// Rule ID
    pub id: String,
    /// Pattern
    pub pattern: String,
    /// Match type
    pub match_type: MatchType,
    /// Target
    pub target: MatchTarget,
}

impl MatchRule {
    /// Create new rule
    pub fn new(id: impl Into<String>, pattern: impl Into<String>, match_type: MatchType) -> Self {
        Self {
            id: id.into(),
            pattern: pattern.into(),
            match_type,
            target: MatchTarget::Key,
        }
    }

    /// Set target
    pub fn target(mut self, target: MatchTarget) -> Self {
        self.target = target;
        self
    }
}

/// Match item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Matched rules
    pub matched_rules: Vec<String>,
    /// Match target
    pub matched_on: MatchTarget,
}

impl MatchItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, rules: Vec<String>, target: MatchTarget) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            matched_rules: rules,
            matched_on: target,
        }
    }

    /// How many rules matched
    pub fn rule_count(&self) -> usize {
        self.matched_rules.len()
    }
}

/// Match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Matched items
    pub items: Vec<MatchItem>,
    /// Total checked
    pub total_checked: usize,
    /// Total matched
    pub total_matched: usize,
    /// Rules applied
    pub rules_applied: usize,
}

impl MatchResult {
    /// Create new result
    pub fn new(items: Vec<MatchItem>, checked: usize, rules: usize) -> Self {
        let total_matched = items.len();
        Self {
            items,
            total_checked: checked,
            total_matched,
            rules_applied: rules,
        }
    }

    /// Has matches
    pub fn has_matches(&self) -> bool {
        !self.items.is_empty()
    }

    /// Match rate
    pub fn match_rate(&self) -> f64 {
        if self.total_checked == 0 {
            0.0
        } else {
            self.total_matched as f64 / self.total_checked as f64
        }
    }

    /// Filter by rule
    pub fn filter_by_rule(&self, rule_id: &str) -> Vec<&MatchItem> {
        self.items.iter().filter(|i| i.matched_rules.contains(&rule_id.to_string())).collect()
    }
}

impl Default for MatchResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, 0)
    }
}

/// Matcher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatcherStats {
    /// Total matches
    pub total_matches: usize,
    /// Total checked
    pub total_checked: usize,
    /// Total matched
    pub total_matched: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MatcherStats {
    /// Record match
    pub fn record(&mut self, result: &MatchResult, match_type: MatchType) {
        self.total_matches += 1;
        self.total_checked += result.total_checked;
        self.total_matched += result.total_matched;
        *self.by_type.entry(match_type.to_string()).or_insert(0) += 1;
    }

    /// Overall match rate
    pub fn overall_rate(&self) -> f64 {
        if self.total_checked == 0 {
            0.0
        } else {
            self.total_matched as f64 / self.total_checked as f64
        }
    }
}

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

/// Matcher registry
#[derive(Debug, Clone, Default)]
pub struct MatcherRegistry {
    /// Matchers by ID
    matchers: HashMap<String, SettingsMatcher>,
}

impl MatcherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register matcher
    pub fn register(&mut self, id: impl Into<String>, matcher: SettingsMatcher) {
        self.matchers.insert(id.into(), matcher);
    }

    /// Unregister matcher
    pub fn unregister(&mut self, id: &str) -> bool {
        self.matchers.remove(id).is_some()
    }

    /// Get matcher
    pub fn get(&self, id: &str) -> Option<&SettingsMatcher> {
        self.matchers.get(id)
    }

    /// Get matcher mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMatcher> {
        self.matchers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.matchers.len()
    }
}

/// Format matcher registry
pub fn format_matcher_registry(registry: &MatcherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Matcher Registry:\n");
    output.push_str(&format!("  Matchers: {}\n", registry.count()));
    output
}

/// Check if query is about matcher
pub fn is_matcher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("match settings") || lower.contains("settings matcher") || lower.contains("pattern match")
}

/// Fun fact about matcher
pub fn matcher_fun_fact() -> &'static str {
    "Anna's settings matcher finds exactly what you're looking for with flexible rules!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_type_display() {
        assert_eq!(format!("{}", MatchType::Exact), "exact");
        assert_eq!(format!("{}", MatchType::Contains), "contains");
    }

    #[test]
    fn test_match_target_display() {
        assert_eq!(format!("{}", MatchTarget::Key), "key");
        assert_eq!(format!("{}", MatchTarget::Both), "both");
    }

    #[test]
    fn test_config_new() {
        let c = MatcherConfig::new(MatchType::Prefix);
        assert_eq!(c.match_type, MatchType::Prefix);
    }

    #[test]
    fn test_config_builder() {
        let c = MatcherConfig::new(MatchType::Exact)
            .target(MatchTarget::Value)
            .invert(true);
        assert_eq!(c.target, MatchTarget::Value);
        assert!(c.invert);
    }

    #[test]
    fn test_rule_new() {
        let r = MatchRule::new("r1", "app.*", MatchType::Prefix);
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_rule_target() {
        let r = MatchRule::new("r1", "test", MatchType::Contains).target(MatchTarget::Value);
        assert_eq!(r.target, MatchTarget::Value);
    }

    #[test]
    fn test_match_item_new() {
        let i = MatchItem::new("key", "value", vec!["r1".to_string()], MatchTarget::Key);
        assert_eq!(i.rule_count(), 1);
    }

    #[test]
    fn test_result_new() {
        let r = MatchResult::new(vec![MatchItem::new("k", "v", vec![], MatchTarget::Key)], 10, 2);
        assert!(r.has_matches());
    }

    #[test]
    fn test_result_match_rate() {
        let items = vec![MatchItem::new("k", "v", vec![], MatchTarget::Key)];
        let r = MatchResult::new(items, 10, 1);
        assert!((r.match_rate() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_stats_record() {
        let mut s = MatcherStats::default();
        let r = MatchResult::new(vec![MatchItem::new("k", "v", vec![], MatchTarget::Key)], 10, 1);
        s.record(&r, MatchType::Contains);
        assert_eq!(s.total_matches, 1);
    }

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

    #[test]
    fn test_registry_new() {
        let r = MatcherRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MatcherRegistry::new();
        r.register("m1", SettingsMatcher::new(MatcherConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_matcher_query() {
        assert!(is_matcher_query("match settings"));
        assert!(!is_matcher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = matcher_fun_fact();
        assert!(fact.contains("matcher"));
    }
}

// v0.0.674: Settings Filter (Phase 250)
// Filter settings with predicates and conditions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FilterType {
    /// Include matching
    #[default]
    Include,
    /// Exclude matching
    Exclude,
    /// Allow list
    AllowList,
    /// Block list
    BlockList,
}

impl std::fmt::Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Include => write!(f, "include"),
            Self::Exclude => write!(f, "exclude"),
            Self::AllowList => write!(f, "allow_list"),
            Self::BlockList => write!(f, "block_list"),
        }
    }
}

/// Filter predicate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FilterPredicate {
    /// Is empty
    #[default]
    IsEmpty,
    /// Is not empty
    IsNotEmpty,
    /// Is numeric
    IsNumeric,
    /// Is boolean
    IsBoolean,
    /// Has value
    HasValue,
}

impl std::fmt::Display for FilterPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsEmpty => write!(f, "is_empty"),
            Self::IsNotEmpty => write!(f, "is_not_empty"),
            Self::IsNumeric => write!(f, "is_numeric"),
            Self::IsBoolean => write!(f, "is_boolean"),
            Self::HasValue => write!(f, "has_value"),
        }
    }
}

/// Filter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Default filter type
    pub default_type: FilterType,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Trim values before filtering
    pub trim_values: bool,
    /// Chain filters (AND)
    pub chain_filters: bool,
}

impl FilterConfig {
    /// Create new config
    pub fn new(filter_type: FilterType) -> Self {
        Self {
            default_type: filter_type,
            case_insensitive: true,
            trim_values: true,
            chain_filters: true,
        }
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set trim values
    pub fn trim_values(mut self, trim: bool) -> Self {
        self.trim_values = trim;
        self
    }

    /// Set chain filters
    pub fn chain_filters(mut self, chain: bool) -> Self {
        self.chain_filters = chain;
        self
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self::new(FilterType::Include)
    }
}

/// Filter rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Rule ID
    pub id: String,
    /// Predicate
    pub predicate: FilterPredicate,
    /// Pattern (for pattern-based rules)
    pub pattern: Option<String>,
    /// Enabled
    pub enabled: bool,
}

impl FilterRule {
    /// Create predicate rule
    pub fn predicate(id: impl Into<String>, predicate: FilterPredicate) -> Self {
        Self {
            id: id.into(),
            predicate,
            pattern: None,
            enabled: true,
        }
    }

    /// Create pattern rule
    pub fn pattern(id: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            predicate: FilterPredicate::HasValue,
            pattern: Some(pattern.into()),
            enabled: true,
        }
    }

    /// Evaluate rule
    pub fn evaluate(&self, value: &str) -> bool {
        match self.predicate {
            FilterPredicate::IsEmpty => value.is_empty(),
            FilterPredicate::IsNotEmpty => !value.is_empty(),
            FilterPredicate::IsNumeric => value.parse::<f64>().is_ok(),
            FilterPredicate::IsBoolean => value == "true" || value == "false",
            FilterPredicate::HasValue => {
                if let Some(pattern) = &self.pattern {
                    value.contains(pattern)
                } else {
                    !value.is_empty()
                }
            }
        }
    }

    /// Set enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Filter result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResult {
    /// Filtered settings
    pub settings: HashMap<String, String>,
    /// Passed count
    pub passed: usize,
    /// Filtered out count
    pub filtered_out: usize,
    /// Rules applied
    pub rules_applied: Vec<String>,
}

impl FilterResult {
    /// Create new result
    pub fn new(settings: HashMap<String, String>, filtered_out: usize) -> Self {
        let passed = settings.len();
        Self {
            settings,
            passed,
            filtered_out,
            rules_applied: Vec::new(),
        }
    }

    /// With rules
    pub fn with_rules(mut self, rules: Vec<String>) -> Self {
        self.rules_applied = rules;
        self
    }

    /// Filter rate
    pub fn filter_rate(&self) -> f64 {
        let total = self.passed + self.filtered_out;
        if total == 0 {
            0.0
        } else {
            self.filtered_out as f64 / total as f64
        }
    }
}

impl Default for FilterResult {
    fn default() -> Self {
        Self::new(HashMap::new(), 0)
    }
}

/// Filter stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterStats {
    /// Total filters
    pub total_filters: usize,
    /// Total passed
    pub total_passed: usize,
    /// Total filtered
    pub total_filtered: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl FilterStats {
    /// Record filter
    pub fn record(&mut self, result: &FilterResult, filter_type: FilterType) {
        self.total_filters += 1;
        self.total_passed += result.passed;
        self.total_filtered += result.filtered_out;
        *self.by_type.entry(filter_type.to_string()).or_insert(0) += 1;
    }

    /// Average filter rate
    pub fn average_filter_rate(&self) -> f64 {
        let total = self.total_passed + self.total_filtered;
        if total == 0 {
            0.0
        } else {
            self.total_filtered as f64 / total as f64
        }
    }
}

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

/// Filter registry
#[derive(Debug, Clone, Default)]
pub struct FilterRegistry {
    /// Filters by ID
    filters: HashMap<String, SettingsFilter>,
}

impl FilterRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register filter
    pub fn register(&mut self, id: impl Into<String>, filter: SettingsFilter) {
        self.filters.insert(id.into(), filter);
    }

    /// Unregister filter
    pub fn unregister(&mut self, id: &str) -> bool {
        self.filters.remove(id).is_some()
    }

    /// Get filter
    pub fn get(&self, id: &str) -> Option<&SettingsFilter> {
        self.filters.get(id)
    }

    /// Get filter mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFilter> {
        self.filters.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.filters.len()
    }
}

/// Format filter registry
pub fn format_filter_registry(registry: &FilterRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Filter Registry:\n");
    output.push_str(&format!("  Filters: {}\n", registry.count()));
    output
}

/// Check if query is about filter
pub fn is_filter_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("filter settings") || lower.contains("settings filter") || lower.contains("exclude empty")
}

/// Fun fact about filter
pub fn filter_fun_fact() -> &'static str {
    "Anna's settings filter removes unwanted settings with powerful predicates!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_type_display() {
        assert_eq!(format!("{}", FilterType::Include), "include");
        assert_eq!(format!("{}", FilterType::Exclude), "exclude");
    }

    #[test]
    fn test_predicate_display() {
        assert_eq!(format!("{}", FilterPredicate::IsEmpty), "is_empty");
        assert_eq!(format!("{}", FilterPredicate::IsNumeric), "is_numeric");
    }

    #[test]
    fn test_config_new() {
        let c = FilterConfig::new(FilterType::Include);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = FilterConfig::new(FilterType::Exclude)
            .trim_values(false);
        assert!(!c.trim_values);
    }

    #[test]
    fn test_rule_predicate() {
        let r = FilterRule::predicate("r1", FilterPredicate::IsNotEmpty);
        assert!(r.evaluate("value"));
        assert!(!r.evaluate(""));
    }

    #[test]
    fn test_rule_pattern() {
        let r = FilterRule::pattern("r1", "test");
        assert!(r.evaluate("this is a test"));
        assert!(!r.evaluate("hello world"));
    }

    #[test]
    fn test_rule_is_numeric() {
        let r = FilterRule::predicate("r1", FilterPredicate::IsNumeric);
        assert!(r.evaluate("123"));
        assert!(r.evaluate("12.5"));
        assert!(!r.evaluate("abc"));
    }

    #[test]
    fn test_result_new() {
        let mut settings = HashMap::new();
        settings.insert("k".to_string(), "v".to_string());
        let r = FilterResult::new(settings, 2);
        assert_eq!(r.passed, 1);
        assert_eq!(r.filtered_out, 2);
    }

    #[test]
    fn test_stats_record() {
        let mut s = FilterStats::default();
        let r = FilterResult::new(HashMap::new(), 5);
        s.record(&r, FilterType::Include);
        assert_eq!(s.total_filters, 1);
        assert_eq!(s.total_filtered, 5);
    }

    #[test]
    fn test_filter_new() {
        let f = SettingsFilter::new(FilterConfig::default());
        assert_eq!(f.rule_count(), 0);
    }

    #[test]
    fn test_filter_by_not_empty() {
        let mut f = SettingsFilter::new(FilterConfig::default());
        let mut settings = HashMap::new();
        settings.insert("k1".to_string(), "value".to_string());
        settings.insert("k2".to_string(), "".to_string());
        
        let result = f.filter_by(&settings, FilterPredicate::IsNotEmpty);
        assert_eq!(result.passed, 1);
        assert_eq!(result.filtered_out, 1);
    }

    #[test]
    fn test_filter_with_rules() {
        let mut f = SettingsFilter::new(FilterConfig::default());
        f.add_rule(FilterRule::predicate("r1", FilterPredicate::IsNumeric));
        
        let mut settings = HashMap::new();
        settings.insert("count".to_string(), "42".to_string());
        settings.insert("name".to_string(), "test".to_string());
        
        let result = f.filter(&settings);
        assert_eq!(result.passed, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = FilterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FilterRegistry::new();
        r.register("f1", SettingsFilter::new(FilterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_filter_query() {
        assert!(is_filter_query("filter settings"));
        assert!(!is_filter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = filter_fun_fact();
        assert!(fact.contains("filter"));
    }
}

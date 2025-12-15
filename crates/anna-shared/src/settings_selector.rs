// v0.0.673: Settings Selector (Phase 249)
// Select settings based on criteria and patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Selector type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SelectorType {
    /// Select by key pattern
    #[default]
    Pattern,
    /// Select by value
    ByValue,
    /// Select by index/position
    ByIndex,
    /// Select first N
    First,
    /// Select last N
    Last,
}

impl std::fmt::Display for SelectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "pattern"),
            Self::ByValue => write!(f, "by_value"),
            Self::ByIndex => write!(f, "by_index"),
            Self::First => write!(f, "first"),
            Self::Last => write!(f, "last"),
        }
    }
}

/// Match mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MatchMode {
    /// Exact match
    #[default]
    Exact,
    /// Prefix match
    Prefix,
    /// Suffix match
    Suffix,
    /// Contains
    Contains,
    /// Regex match
    Regex,
}

impl std::fmt::Display for MatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "exact"),
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
            Self::Contains => write!(f, "contains"),
            Self::Regex => write!(f, "regex"),
        }
    }
}

/// Selector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorConfig {
    /// Default selector type
    pub default_type: SelectorType,
    /// Default match mode
    pub default_match: MatchMode,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Max selections
    pub max_selections: usize,
}

impl SelectorConfig {
    /// Create new config
    pub fn new(selector_type: SelectorType) -> Self {
        Self {
            default_type: selector_type,
            default_match: MatchMode::Exact,
            case_insensitive: true,
            max_selections: 1000,
        }
    }

    /// Set match mode
    pub fn match_mode(mut self, mode: MatchMode) -> Self {
        self.default_match = mode;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set max selections
    pub fn max_selections(mut self, max: usize) -> Self {
        self.max_selections = max;
        self
    }
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self::new(SelectorType::Pattern)
    }
}

/// Selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCriteria {
    /// Pattern to match
    pub pattern: String,
    /// Match mode
    pub match_mode: MatchMode,
    /// Target (key or value)
    pub target: String,
}

impl SelectionCriteria {
    /// Create key criteria
    pub fn key(pattern: impl Into<String>, mode: MatchMode) -> Self {
        Self {
            pattern: pattern.into(),
            match_mode: mode,
            target: "key".to_string(),
        }
    }

    /// Create value criteria
    pub fn value(pattern: impl Into<String>, mode: MatchMode) -> Self {
        Self {
            pattern: pattern.into(),
            match_mode: mode,
            target: "value".to_string(),
        }
    }

    /// Check if matches
    pub fn matches(&self, key: &str, value: &str, case_insensitive: bool) -> bool {
        let target_str = if self.target == "key" { key } else { value };
        let (target, pattern) = if case_insensitive {
            (target_str.to_lowercase(), self.pattern.to_lowercase())
        } else {
            (target_str.to_string(), self.pattern.clone())
        };

        match self.match_mode {
            MatchMode::Exact => target == pattern,
            MatchMode::Prefix => target.starts_with(&pattern),
            MatchMode::Suffix => target.ends_with(&pattern),
            MatchMode::Contains => target.contains(&pattern),
            MatchMode::Regex => target.contains(&pattern), // Simplified
        }
    }
}

/// Selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    /// Selected entries
    pub entries: Vec<(String, String)>,
    /// Total selected
    pub total_selected: usize,
    /// Total scanned
    pub total_scanned: usize,
    /// Success
    pub success: bool,
}

impl SelectionResult {
    /// Create success result
    pub fn success(entries: Vec<(String, String)>, scanned: usize) -> Self {
        let total_selected = entries.len();
        Self {
            entries,
            total_selected,
            total_scanned: scanned,
            success: true,
        }
    }

    /// Has selections
    pub fn has_selections(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Selection rate
    pub fn selection_rate(&self) -> f64 {
        if self.total_scanned == 0 {
            0.0
        } else {
            self.total_selected as f64 / self.total_scanned as f64
        }
    }
}

impl Default for SelectionResult {
    fn default() -> Self {
        Self::success(Vec::new(), 0)
    }
}

/// Selector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectorStats {
    /// Total selections
    pub total_selections: usize,
    /// Total selected
    pub total_selected: usize,
    /// Total scanned
    pub total_scanned: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SelectorStats {
    /// Record selection
    pub fn record(&mut self, result: &SelectionResult, selector_type: SelectorType) {
        self.total_selections += 1;
        self.total_selected += result.total_selected;
        self.total_scanned += result.total_scanned;
        *self.by_type.entry(selector_type.to_string()).or_insert(0) += 1;
    }

    /// Average selection rate
    pub fn average_selection_rate(&self) -> f64 {
        if self.total_scanned == 0 {
            0.0
        } else {
            self.total_selected as f64 / self.total_scanned as f64
        }
    }
}

/// Settings selector
#[derive(Debug, Clone, Default)]
pub struct SettingsSelector {
    /// Config
    config: SelectorConfig,
    /// Stats
    stats: SelectorStats,
}

impl SettingsSelector {
    /// Create new selector
    pub fn new(config: SelectorConfig) -> Self {
        Self {
            config,
            stats: SelectorStats::default(),
        }
    }

    /// Select by criteria
    pub fn select(&mut self, settings: &HashMap<String, String>, criteria: &SelectionCriteria) -> SelectionResult {
        let scanned = settings.len();
        let entries: Vec<(String, String)> = settings.iter()
            .filter(|(k, v)| criteria.matches(k, v, self.config.case_insensitive))
            .take(self.config.max_selections)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let result = SelectionResult::success(entries, scanned);
        self.stats.record(&result, self.config.default_type);
        result
    }

    /// Select first N
    pub fn select_first(&mut self, settings: &HashMap<String, String>, n: usize) -> SelectionResult {
        let scanned = settings.len();
        let entries: Vec<(String, String)> = settings.iter()
            .take(n.min(self.config.max_selections))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let result = SelectionResult::success(entries, scanned);
        self.stats.record(&result, SelectorType::First);
        result
    }

    /// Select by key prefix
    pub fn select_by_prefix(&mut self, settings: &HashMap<String, String>, prefix: &str) -> SelectionResult {
        let criteria = SelectionCriteria::key(prefix, MatchMode::Prefix);
        self.select(settings, &criteria)
    }

    /// Get stats
    pub fn stats(&self) -> &SelectorStats {
        &self.stats
    }
}

/// Selector registry
#[derive(Debug, Clone, Default)]
pub struct SelectorRegistry {
    /// Selectors by ID
    selectors: HashMap<String, SettingsSelector>,
}

impl SelectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register selector
    pub fn register(&mut self, id: impl Into<String>, selector: SettingsSelector) {
        self.selectors.insert(id.into(), selector);
    }

    /// Unregister selector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.selectors.remove(id).is_some()
    }

    /// Get selector
    pub fn get(&self, id: &str) -> Option<&SettingsSelector> {
        self.selectors.get(id)
    }

    /// Get selector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSelector> {
        self.selectors.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.selectors.len()
    }
}

/// Format selector registry
pub fn format_selector_registry(registry: &SelectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Selector Registry:\n");
    output.push_str(&format!("  Selectors: {}\n", registry.count()));
    output
}

/// Check if query is about selector
pub fn is_selector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("select settings") || lower.contains("settings selector") || lower.contains("pick settings")
}

/// Fun fact about selector
pub fn selector_fun_fact() -> &'static str {
    "Anna's settings selector finds exactly the settings you need with flexible criteria!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_type_display() {
        assert_eq!(format!("{}", SelectorType::Pattern), "pattern");
        assert_eq!(format!("{}", SelectorType::First), "first");
    }

    #[test]
    fn test_match_mode_display() {
        assert_eq!(format!("{}", MatchMode::Exact), "exact");
        assert_eq!(format!("{}", MatchMode::Prefix), "prefix");
    }

    #[test]
    fn test_config_new() {
        let c = SelectorConfig::new(SelectorType::Pattern);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = SelectorConfig::new(SelectorType::ByValue)
            .match_mode(MatchMode::Contains)
            .max_selections(50);
        assert_eq!(c.default_match, MatchMode::Contains);
        assert_eq!(c.max_selections, 50);
    }

    #[test]
    fn test_criteria_key() {
        let c = SelectionCriteria::key("app.", MatchMode::Prefix);
        assert!(c.matches("app.name", "value", true));
        assert!(!c.matches("db.host", "value", true));
    }

    #[test]
    fn test_criteria_value() {
        let c = SelectionCriteria::value("localhost", MatchMode::Exact);
        assert!(c.matches("key", "localhost", true));
        assert!(!c.matches("key", "remote", true));
    }

    #[test]
    fn test_result_success() {
        let r = SelectionResult::success(vec![("k".to_string(), "v".to_string())], 10);
        assert!(r.has_selections());
        assert_eq!(r.total_selected, 1);
    }

    #[test]
    fn test_result_selection_rate() {
        let r = SelectionResult::success(vec![("k".to_string(), "v".to_string())], 10);
        assert!((r.selection_rate() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_stats_record() {
        let mut s = SelectorStats::default();
        let r = SelectionResult::success(vec![("k".to_string(), "v".to_string())], 10);
        s.record(&r, SelectorType::Pattern);
        assert_eq!(s.total_selections, 1);
    }

    #[test]
    fn test_selector_new() {
        let s = SettingsSelector::new(SelectorConfig::default());
        assert_eq!(s.stats().total_selections, 0);
    }

    #[test]
    fn test_selector_select_by_prefix() {
        let mut s = SettingsSelector::new(SelectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());
        
        let result = s.select_by_prefix(&settings, "app.");
        assert_eq!(result.total_selected, 2);
    }

    #[test]
    fn test_selector_select_first() {
        let mut s = SettingsSelector::new(SelectorConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());
        settings.insert("c".to_string(), "3".to_string());
        
        let result = s.select_first(&settings, 2);
        assert_eq!(result.total_selected, 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SelectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SelectorRegistry::new();
        r.register("s1", SettingsSelector::new(SelectorConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_selector_query() {
        assert!(is_selector_query("select settings"));
        assert!(!is_selector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = selector_fun_fact();
        assert!(fact.contains("selector"));
    }
}

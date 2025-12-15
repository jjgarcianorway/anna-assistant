// v0.0.685: Settings Finder (Phase 261)
// Find settings by various criteria

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Find mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FindMode {
    /// Find by exact key
    #[default]
    ExactKey,
    /// Find by key pattern
    KeyPattern,
    /// Find by value
    ByValue,
    /// Find by value pattern
    ValuePattern,
}

impl std::fmt::Display for FindMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExactKey => write!(f, "exact_key"),
            Self::KeyPattern => write!(f, "key_pattern"),
            Self::ByValue => write!(f, "by_value"),
            Self::ValuePattern => write!(f, "value_pattern"),
        }
    }
}

/// Find limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FindLimit {
    /// Find first match
    First,
    /// Find all matches
    #[default]
    All,
    /// Find up to N matches
    Max(usize),
}

impl std::fmt::Display for FindLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => write!(f, "first"),
            Self::All => write!(f, "all"),
            Self::Max(n) => write!(f, "max({})", n),
        }
    }
}

/// Finder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinderConfig {
    /// Find mode
    pub mode: FindMode,
    /// Find limit
    pub limit: FindLimit,
    /// Case insensitive
    pub case_insensitive: bool,
    /// Include partial matches
    pub partial_match: bool,
}

impl FinderConfig {
    /// Create new config
    pub fn new(mode: FindMode) -> Self {
        Self {
            mode,
            limit: FindLimit::All,
            case_insensitive: true,
            partial_match: true,
        }
    }

    /// Set limit
    pub fn limit(mut self, limit: FindLimit) -> Self {
        self.limit = limit;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Set partial match
    pub fn partial_match(mut self, partial: bool) -> Self {
        self.partial_match = partial;
        self
    }
}

impl Default for FinderConfig {
    fn default() -> Self {
        Self::new(FindMode::KeyPattern)
    }
}

/// Found item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Match score (0.0-1.0)
    pub score: f64,
    /// Match type
    pub match_type: FindMode,
}

impl FoundItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, score: f64, match_type: FindMode) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            score,
            match_type,
        }
    }

    /// Is exact match
    pub fn is_exact(&self) -> bool {
        (self.score - 1.0).abs() < 0.001
    }

    /// Is partial match
    pub fn is_partial(&self) -> bool {
        self.score > 0.0 && self.score < 1.0
    }
}

/// Find result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    /// Found items
    pub items: Vec<FoundItem>,
    /// Total searched
    pub total_searched: usize,
    /// Total found
    pub total_found: usize,
    /// Mode used
    pub mode: FindMode,
}

impl FindResult {
    /// Create new result
    pub fn new(items: Vec<FoundItem>, searched: usize, mode: FindMode) -> Self {
        let total_found = items.len();
        Self {
            items,
            total_searched: searched,
            total_found,
            mode,
        }
    }

    /// Has results
    pub fn has_results(&self) -> bool {
        !self.items.is_empty()
    }

    /// Get first
    pub fn first(&self) -> Option<&FoundItem> {
        self.items.first()
    }

    /// Best match
    pub fn best_match(&self) -> Option<&FoundItem> {
        self.items.iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Filter by score
    pub fn filter_by_score(&self, min_score: f64) -> Vec<&FoundItem> {
        self.items.iter().filter(|i| i.score >= min_score).collect()
    }
}

impl Default for FindResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, FindMode::ExactKey)
    }
}

/// Finder stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FinderStats {
    /// Total finds
    pub total_finds: usize,
    /// Total searched
    pub total_searched: usize,
    /// Total found
    pub total_found: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl FinderStats {
    /// Record find
    pub fn record(&mut self, result: &FindResult) {
        self.total_finds += 1;
        self.total_searched += result.total_searched;
        self.total_found += result.total_found;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.total_searched == 0 {
            0.0
        } else {
            self.total_found as f64 / self.total_searched as f64
        }
    }
}

/// Settings finder
#[derive(Debug, Clone, Default)]
pub struct SettingsFinder {
    /// Config
    config: FinderConfig,
    /// Stats
    stats: FinderStats,
}

impl SettingsFinder {
    /// Create new finder
    pub fn new(config: FinderConfig) -> Self {
        Self {
            config,
            stats: FinderStats::default(),
        }
    }

    /// Calculate match score
    fn calc_score(&self, target: &str, pattern: &str) -> f64 {
        let (t, p) = if self.config.case_insensitive {
            (target.to_lowercase(), pattern.to_lowercase())
        } else {
            (target.to_string(), pattern.to_string())
        };

        if t == p {
            1.0
        } else if t.contains(&p) {
            p.len() as f64 / t.len() as f64
        } else {
            0.0
        }
    }

    /// Apply limit
    fn apply_limit(&self, mut items: Vec<FoundItem>) -> Vec<FoundItem> {
        match self.config.limit {
            FindLimit::First => items.into_iter().take(1).collect(),
            FindLimit::All => items,
            FindLimit::Max(n) => {
                items.truncate(n);
                items
            }
        }
    }

    /// Find by key pattern
    pub fn find_by_key(&mut self, settings: &HashMap<String, String>, pattern: &str) -> FindResult {
        let mut items = Vec::new();

        for (key, value) in settings {
            let score = self.calc_score(key, pattern);
            if score > 0.0 || (self.config.partial_match && score == 0.0) {
                if score > 0.0 {
                    items.push(FoundItem::new(key.clone(), value.clone(), score, FindMode::KeyPattern));
                }
            }
        }

        // Sort by score descending
        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let items = self.apply_limit(items);

        let result = FindResult::new(items, settings.len(), FindMode::KeyPattern);
        self.stats.record(&result);
        result
    }

    /// Find by value pattern
    pub fn find_by_value(&mut self, settings: &HashMap<String, String>, pattern: &str) -> FindResult {
        let mut items = Vec::new();

        for (key, value) in settings {
            let score = self.calc_score(value, pattern);
            if score > 0.0 {
                items.push(FoundItem::new(key.clone(), value.clone(), score, FindMode::ValuePattern));
            }
        }

        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let items = self.apply_limit(items);

        let result = FindResult::new(items, settings.len(), FindMode::ValuePattern);
        self.stats.record(&result);
        result
    }

    /// Find exact key
    pub fn find_exact(&mut self, settings: &HashMap<String, String>, key: &str) -> FindResult {
        let items = if let Some(value) = settings.get(key) {
            vec![FoundItem::new(key.to_string(), value.clone(), 1.0, FindMode::ExactKey)]
        } else {
            Vec::new()
        };

        let result = FindResult::new(items, settings.len(), FindMode::ExactKey);
        self.stats.record(&result);
        result
    }

    /// Find by exact value
    pub fn find_by_exact_value(&mut self, settings: &HashMap<String, String>, target_value: &str) -> FindResult {
        let mut items = Vec::new();

        let target = if self.config.case_insensitive {
            target_value.to_lowercase()
        } else {
            target_value.to_string()
        };

        for (key, value) in settings {
            let v = if self.config.case_insensitive {
                value.to_lowercase()
            } else {
                value.clone()
            };

            if v == target {
                items.push(FoundItem::new(key.clone(), value.clone(), 1.0, FindMode::ByValue));
            }
        }

        let items = self.apply_limit(items);
        let result = FindResult::new(items, settings.len(), FindMode::ByValue);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &FinderStats {
        &self.stats
    }
}

/// Finder registry
#[derive(Debug, Clone, Default)]
pub struct FinderRegistry {
    /// Finders by ID
    finders: HashMap<String, SettingsFinder>,
}

impl FinderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register finder
    pub fn register(&mut self, id: impl Into<String>, finder: SettingsFinder) {
        self.finders.insert(id.into(), finder);
    }

    /// Unregister finder
    pub fn unregister(&mut self, id: &str) -> bool {
        self.finders.remove(id).is_some()
    }

    /// Get finder
    pub fn get(&self, id: &str) -> Option<&SettingsFinder> {
        self.finders.get(id)
    }

    /// Get finder mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsFinder> {
        self.finders.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.finders.len()
    }
}

/// Format finder registry
pub fn format_finder_registry(registry: &FinderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Finder Registry:\n");
    output.push_str(&format!("  Finders: {}\n", registry.count()));
    output
}

/// Check if query is about finder
pub fn is_finder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("find settings") || lower.contains("settings finder") || lower.contains("search settings")
}

/// Fun fact about finder
pub fn finder_fun_fact() -> &'static str {
    "Anna's settings finder locates exactly the settings you need with smart scoring!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_mode_display() {
        assert_eq!(format!("{}", FindMode::ExactKey), "exact_key");
        assert_eq!(format!("{}", FindMode::KeyPattern), "key_pattern");
    }

    #[test]
    fn test_find_limit_display() {
        assert_eq!(format!("{}", FindLimit::First), "first");
        assert_eq!(format!("{}", FindLimit::Max(5)), "max(5)");
    }

    #[test]
    fn test_config_new() {
        let c = FinderConfig::new(FindMode::ExactKey);
        assert!(c.case_insensitive);
    }

    #[test]
    fn test_config_builder() {
        let c = FinderConfig::new(FindMode::ValuePattern)
            .limit(FindLimit::Max(10))
            .case_insensitive(false);
        assert!(!c.case_insensitive);
    }

    #[test]
    fn test_found_item_new() {
        let i = FoundItem::new("key", "value", 1.0, FindMode::ExactKey);
        assert!(i.is_exact());
        assert!(!i.is_partial());
    }

    #[test]
    fn test_found_item_partial() {
        let i = FoundItem::new("key", "value", 0.5, FindMode::KeyPattern);
        assert!(!i.is_exact());
        assert!(i.is_partial());
    }

    #[test]
    fn test_result_new() {
        let r = FindResult::new(vec![FoundItem::new("k", "v", 1.0, FindMode::ExactKey)], 10, FindMode::ExactKey);
        assert!(r.has_results());
        assert_eq!(r.total_found, 1);
    }

    #[test]
    fn test_result_best_match() {
        let items = vec![
            FoundItem::new("k1", "v1", 0.5, FindMode::KeyPattern),
            FoundItem::new("k2", "v2", 0.8, FindMode::KeyPattern),
        ];
        let r = FindResult::new(items, 10, FindMode::KeyPattern);
        assert_eq!(r.best_match().unwrap().key, "k2");
    }

    #[test]
    fn test_stats_record() {
        let mut s = FinderStats::default();
        let r = FindResult::new(vec![FoundItem::new("k", "v", 1.0, FindMode::ExactKey)], 10, FindMode::ExactKey);
        s.record(&r);
        assert_eq!(s.total_finds, 1);
        assert_eq!(s.total_found, 1);
    }

    #[test]
    fn test_finder_new() {
        let f = SettingsFinder::new(FinderConfig::default());
        assert_eq!(f.stats().total_finds, 0);
    }

    #[test]
    fn test_finder_find_exact() {
        let mut f = SettingsFinder::new(FinderConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());

        let result = f.find_exact(&settings, "app.name");
        assert!(result.has_results());
        assert_eq!(result.first().unwrap().value, "test");
    }

    #[test]
    fn test_finder_find_by_key() {
        let mut f = SettingsFinder::new(FinderConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = f.find_by_key(&settings, "app");
        assert_eq!(result.total_found, 2);
    }

    #[test]
    fn test_finder_find_by_value() {
        let mut f = SettingsFinder::new(FinderConfig::default());
        let mut settings = HashMap::new();
        settings.insert("host".to_string(), "localhost".to_string());
        settings.insert("port".to_string(), "8080".to_string());

        let result = f.find_by_value(&settings, "local");
        assert_eq!(result.total_found, 1);
    }

    #[test]
    fn test_finder_with_limit() {
        let mut f = SettingsFinder::new(FinderConfig::default().limit(FindLimit::Max(1)));
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("ab".to_string(), "2".to_string());
        settings.insert("abc".to_string(), "3".to_string());

        let result = f.find_by_key(&settings, "a");
        assert_eq!(result.total_found, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = FinderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FinderRegistry::new();
        r.register("f1", SettingsFinder::new(FinderConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_finder_query() {
        assert!(is_finder_query("find settings"));
        assert!(!is_finder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = finder_fun_fact();
        assert!(fact.contains("finder"));
    }
}

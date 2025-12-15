// v0.0.683: Settings Zipper (Phase 259)
// Zip and unzip settings collections together

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Zip mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ZipMode {
    /// Zip by matching keys
    #[default]
    ByKey,
    /// Zip by position
    ByPosition,
    /// Zip all combinations
    Cartesian,
    /// Zip with default for missing
    WithDefault,
}

impl std::fmt::Display for ZipMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByKey => write!(f, "by_key"),
            Self::ByPosition => write!(f, "by_position"),
            Self::Cartesian => write!(f, "cartesian"),
            Self::WithDefault => write!(f, "with_default"),
        }
    }
}

/// Unzip mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnzipMode {
    /// Split by key prefix
    #[default]
    ByPrefix,
    /// Split alternating
    Alternating,
    /// Split by predicate (odd/even index)
    ByIndex,
}

impl std::fmt::Display for UnzipMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByPrefix => write!(f, "by_prefix"),
            Self::Alternating => write!(f, "alternating"),
            Self::ByIndex => write!(f, "by_index"),
        }
    }
}

/// Zipper config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipperConfig {
    /// Zip mode
    pub zip_mode: ZipMode,
    /// Unzip mode
    pub unzip_mode: UnzipMode,
    /// Default value for missing
    pub default_value: String,
    /// Pair separator
    pub pair_separator: String,
}

impl ZipperConfig {
    /// Create new config
    pub fn new(zip_mode: ZipMode) -> Self {
        Self {
            zip_mode,
            unzip_mode: UnzipMode::ByPrefix,
            default_value: "".to_string(),
            pair_separator: ":".to_string(),
        }
    }

    /// Set unzip mode
    pub fn unzip_mode(mut self, mode: UnzipMode) -> Self {
        self.unzip_mode = mode;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Set pair separator
    pub fn pair_separator(mut self, sep: impl Into<String>) -> Self {
        self.pair_separator = sep.into();
        self
    }
}

impl Default for ZipperConfig {
    fn default() -> Self {
        Self::new(ZipMode::ByKey)
    }
}

/// Zipped pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZippedPair {
    /// Key
    pub key: String,
    /// First value
    pub first: String,
    /// Second value
    pub second: String,
}

impl ZippedPair {
    /// Create new pair
    pub fn new(key: impl Into<String>, first: impl Into<String>, second: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            first: first.into(),
            second: second.into(),
        }
    }

    /// To tuple
    pub fn to_tuple(&self) -> (&str, &str, &str) {
        (&self.key, &self.first, &self.second)
    }

    /// Combined value
    pub fn combined(&self, sep: &str) -> String {
        format!("{}{}{}", self.first, sep, self.second)
    }
}

/// Zip result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipResult {
    /// Zipped pairs
    pub pairs: Vec<ZippedPair>,
    /// Total pairs
    pub total_pairs: usize,
    /// Matched count
    pub matched: usize,
    /// Unmatched count
    pub unmatched: usize,
    /// Mode used
    pub mode: ZipMode,
}

impl ZipResult {
    /// Create new result
    pub fn new(pairs: Vec<ZippedPair>, matched: usize, unmatched: usize, mode: ZipMode) -> Self {
        let total_pairs = pairs.len();
        Self {
            pairs,
            total_pairs,
            matched,
            unmatched,
            mode,
        }
    }

    /// Get pair
    pub fn get(&self, index: usize) -> Option<&ZippedPair> {
        self.pairs.get(index)
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Match rate
    pub fn match_rate(&self) -> f64 {
        let total = self.matched + self.unmatched;
        if total == 0 {
            1.0
        } else {
            self.matched as f64 / total as f64
        }
    }
}

impl Default for ZipResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, 0, ZipMode::ByKey)
    }
}

/// Unzip result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnzipResult {
    /// First collection
    pub first: HashMap<String, String>,
    /// Second collection
    pub second: HashMap<String, String>,
    /// Total split
    pub total_split: usize,
}

impl UnzipResult {
    /// Create new result
    pub fn new(first: HashMap<String, String>, second: HashMap<String, String>) -> Self {
        let total_split = first.len() + second.len();
        Self {
            first,
            second,
            total_split,
        }
    }

    /// Is balanced
    pub fn is_balanced(&self) -> bool {
        let diff = if self.first.len() > self.second.len() {
            self.first.len() - self.second.len()
        } else {
            self.second.len() - self.first.len()
        };
        diff <= 1
    }
}

impl Default for UnzipResult {
    fn default() -> Self {
        Self::new(HashMap::new(), HashMap::new())
    }
}

/// Zipper stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZipperStats {
    /// Total zip operations
    pub total_zips: usize,
    /// Total unzip operations
    pub total_unzips: usize,
    /// Total pairs created
    pub total_pairs: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl ZipperStats {
    /// Record zip
    pub fn record_zip(&mut self, result: &ZipResult) {
        self.total_zips += 1;
        self.total_pairs += result.total_pairs;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Record unzip
    pub fn record_unzip(&mut self, result: &UnzipResult) {
        self.total_unzips += 1;
        self.total_pairs += result.total_split;
    }
}

/// Settings zipper
#[derive(Debug, Clone, Default)]
pub struct SettingsZipper {
    /// Config
    config: ZipperConfig,
    /// Stats
    stats: ZipperStats,
}

impl SettingsZipper {
    /// Create new zipper
    pub fn new(config: ZipperConfig) -> Self {
        Self {
            config,
            stats: ZipperStats::default(),
        }
    }

    /// Zip by key
    pub fn zip_by_key(&mut self, first: &HashMap<String, String>, second: &HashMap<String, String>) -> ZipResult {
        let mut pairs = Vec::new();
        let mut matched = 0;
        let mut unmatched = 0;

        for (key, first_val) in first {
            if let Some(second_val) = second.get(key) {
                pairs.push(ZippedPair::new(key.clone(), first_val.clone(), second_val.clone()));
                matched += 1;
            } else {
                pairs.push(ZippedPair::new(key.clone(), first_val.clone(), self.config.default_value.clone()));
                unmatched += 1;
            }
        }

        // Add keys only in second
        for (key, second_val) in second {
            if !first.contains_key(key) {
                pairs.push(ZippedPair::new(key.clone(), self.config.default_value.clone(), second_val.clone()));
                unmatched += 1;
            }
        }

        let result = ZipResult::new(pairs, matched, unmatched, ZipMode::ByKey);
        self.stats.record_zip(&result);
        result
    }

    /// Zip by position
    pub fn zip_by_position(&mut self, first: &HashMap<String, String>, second: &HashMap<String, String>) -> ZipResult {
        let first_vec: Vec<_> = first.iter().collect();
        let second_vec: Vec<_> = second.iter().collect();
        let mut pairs = Vec::new();
        let matched = first_vec.len().min(second_vec.len());
        let unmatched = first_vec.len().max(second_vec.len()) - matched;

        for i in 0..first_vec.len().max(second_vec.len()) {
            let (key, first_val) = first_vec.get(i).map(|(k, v)| ((*k).clone(), (*v).clone()))
                .unwrap_or_else(|| (format!("key_{}", i), self.config.default_value.clone()));
            let second_val = second_vec.get(i).map(|(_, v)| (*v).clone())
                .unwrap_or_else(|| self.config.default_value.clone());
            pairs.push(ZippedPair::new(key, first_val, second_val));
        }

        let result = ZipResult::new(pairs, matched, unmatched, ZipMode::ByPosition);
        self.stats.record_zip(&result);
        result
    }

    /// Unzip by prefix
    pub fn unzip_by_prefix(&mut self, settings: &HashMap<String, String>, prefix: &str) -> UnzipResult {
        let mut first = HashMap::new();
        let mut second = HashMap::new();

        for (key, value) in settings {
            if key.starts_with(prefix) {
                first.insert(key.clone(), value.clone());
            } else {
                second.insert(key.clone(), value.clone());
            }
        }

        let result = UnzipResult::new(first, second);
        self.stats.record_unzip(&result);
        result
    }

    /// Unzip alternating
    pub fn unzip_alternating(&mut self, settings: &HashMap<String, String>) -> UnzipResult {
        let mut first = HashMap::new();
        let mut second = HashMap::new();

        for (i, (key, value)) in settings.iter().enumerate() {
            if i % 2 == 0 {
                first.insert(key.clone(), value.clone());
            } else {
                second.insert(key.clone(), value.clone());
            }
        }

        let result = UnzipResult::new(first, second);
        self.stats.record_unzip(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &ZipperStats {
        &self.stats
    }
}

/// Zipper registry
#[derive(Debug, Clone, Default)]
pub struct ZipperRegistry {
    /// Zippers by ID
    zippers: HashMap<String, SettingsZipper>,
}

impl ZipperRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register zipper
    pub fn register(&mut self, id: impl Into<String>, zipper: SettingsZipper) {
        self.zippers.insert(id.into(), zipper);
    }

    /// Unregister zipper
    pub fn unregister(&mut self, id: &str) -> bool {
        self.zippers.remove(id).is_some()
    }

    /// Get zipper
    pub fn get(&self, id: &str) -> Option<&SettingsZipper> {
        self.zippers.get(id)
    }

    /// Get zipper mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsZipper> {
        self.zippers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.zippers.len()
    }
}

/// Format zipper registry
pub fn format_zipper_registry(registry: &ZipperRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Zipper Registry:\n");
    output.push_str(&format!("  Zippers: {}\n", registry.count()));
    output
}

/// Check if query is about zipper
pub fn is_zipper_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("zip settings") || lower.contains("settings zipper") || lower.contains("combine settings")
}

/// Fun fact about zipper
pub fn zipper_fun_fact() -> &'static str {
    "Anna's settings zipper pairs up settings from different sources like a perfect match!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_mode_display() {
        assert_eq!(format!("{}", ZipMode::ByKey), "by_key");
        assert_eq!(format!("{}", ZipMode::ByPosition), "by_position");
    }

    #[test]
    fn test_unzip_mode_display() {
        assert_eq!(format!("{}", UnzipMode::ByPrefix), "by_prefix");
        assert_eq!(format!("{}", UnzipMode::Alternating), "alternating");
    }

    #[test]
    fn test_config_new() {
        let c = ZipperConfig::new(ZipMode::ByKey);
        assert_eq!(c.zip_mode, ZipMode::ByKey);
    }

    #[test]
    fn test_config_builder() {
        let c = ZipperConfig::new(ZipMode::WithDefault)
            .default_value("N/A")
            .pair_separator("|");
        assert_eq!(c.default_value, "N/A");
        assert_eq!(c.pair_separator, "|");
    }

    #[test]
    fn test_pair_new() {
        let p = ZippedPair::new("key", "val1", "val2");
        assert_eq!(p.key, "key");
        assert_eq!(p.combined(":"), "val1:val2");
    }

    #[test]
    fn test_result_new() {
        let r = ZipResult::new(vec![ZippedPair::new("k", "v1", "v2")], 1, 0, ZipMode::ByKey);
        assert_eq!(r.total_pairs, 1);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_result_match_rate() {
        let r = ZipResult::new(vec![], 8, 2, ZipMode::ByKey);
        assert!((r.match_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_unzip_result_balanced() {
        let mut first = HashMap::new();
        first.insert("a".to_string(), "1".to_string());
        let mut second = HashMap::new();
        second.insert("b".to_string(), "2".to_string());
        let r = UnzipResult::new(first, second);
        assert!(r.is_balanced());
    }

    #[test]
    fn test_stats_record_zip() {
        let mut s = ZipperStats::default();
        let r = ZipResult::new(vec![ZippedPair::new("k", "v1", "v2")], 1, 0, ZipMode::ByKey);
        s.record_zip(&r);
        assert_eq!(s.total_zips, 1);
        assert_eq!(s.total_pairs, 1);
    }

    #[test]
    fn test_zipper_new() {
        let z = SettingsZipper::new(ZipperConfig::default());
        assert_eq!(z.stats().total_zips, 0);
    }

    #[test]
    fn test_zipper_zip_by_key() {
        let mut z = SettingsZipper::new(ZipperConfig::default());

        let mut first = HashMap::new();
        first.insert("a".to_string(), "1".to_string());
        first.insert("b".to_string(), "2".to_string());

        let mut second = HashMap::new();
        second.insert("a".to_string(), "10".to_string());
        second.insert("c".to_string(), "30".to_string());

        let result = z.zip_by_key(&first, &second);
        assert_eq!(result.matched, 1); // "a" matches
        assert_eq!(result.total_pairs, 3); // a, b, c
    }

    #[test]
    fn test_zipper_unzip_by_prefix() {
        let mut z = SettingsZipper::new(ZipperConfig::default());

        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = z.unzip_by_prefix(&settings, "app.");
        assert_eq!(result.first.len(), 2);
        assert_eq!(result.second.len(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ZipperRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ZipperRegistry::new();
        r.register("z1", SettingsZipper::new(ZipperConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_zipper_query() {
        assert!(is_zipper_query("zip settings"));
        assert!(!is_zipper_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = zipper_fun_fact();
        assert!(fact.contains("zipper"));
    }
}

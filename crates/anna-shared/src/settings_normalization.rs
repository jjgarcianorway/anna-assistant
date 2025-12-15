// v0.0.667: Settings Normalization (Phase 243)
// Normalize settings to a canonical format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NormalizationType {
    /// Case normalization
    #[default]
    Case,
    /// Whitespace normalization
    Whitespace,
    /// Key format normalization
    KeyFormat,
    /// Value format normalization
    ValueFormat,
    /// Full normalization
    Full,
}

impl std::fmt::Display for NormalizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Case => write!(f, "case"),
            Self::Whitespace => write!(f, "whitespace"),
            Self::KeyFormat => write!(f, "key_format"),
            Self::ValueFormat => write!(f, "value_format"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Case style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CaseStyle {
    /// lowercase
    #[default]
    Lower,
    /// UPPERCASE
    Upper,
    /// camelCase
    Camel,
    /// snake_case
    Snake,
    /// kebab-case
    Kebab,
}

impl std::fmt::Display for CaseStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lower => write!(f, "lower"),
            Self::Upper => write!(f, "upper"),
            Self::Camel => write!(f, "camel"),
            Self::Snake => write!(f, "snake"),
            Self::Kebab => write!(f, "kebab"),
        }
    }
}

/// Normalizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizerConfig {
    /// Normalization type
    pub normalization_type: NormalizationType,
    /// Key case style
    pub key_case: CaseStyle,
    /// Trim whitespace
    pub trim_whitespace: bool,
    /// Collapse whitespace
    pub collapse_whitespace: bool,
    /// Remove empty values
    pub remove_empty: bool,
}

impl NormalizerConfig {
    /// Create new config
    pub fn new(normalization_type: NormalizationType) -> Self {
        Self {
            normalization_type,
            key_case: CaseStyle::Lower,
            trim_whitespace: true,
            collapse_whitespace: true,
            remove_empty: false,
        }
    }

    /// Set key case
    pub fn key_case(mut self, case: CaseStyle) -> Self {
        self.key_case = case;
        self
    }

    /// Set trim whitespace
    pub fn trim_whitespace(mut self, trim: bool) -> Self {
        self.trim_whitespace = trim;
        self
    }

    /// Set remove empty
    pub fn remove_empty(mut self, remove: bool) -> Self {
        self.remove_empty = remove;
        self
    }
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self::new(NormalizationType::Full)
    }
}

/// Normalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    /// Normalized settings
    pub settings: HashMap<String, String>,
    /// Keys normalized
    pub keys_normalized: usize,
    /// Values normalized
    pub values_normalized: usize,
    /// Keys removed
    pub keys_removed: usize,
    /// Success
    pub success: bool,
}

impl NormalizationResult {
    /// Create success result
    pub fn success(settings: HashMap<String, String>) -> Self {
        Self {
            settings,
            keys_normalized: 0,
            values_normalized: 0,
            keys_removed: 0,
            success: true,
        }
    }

    /// With counts
    pub fn with_counts(mut self, keys: usize, values: usize, removed: usize) -> Self {
        self.keys_normalized = keys;
        self.values_normalized = values;
        self.keys_removed = removed;
        self
    }

    /// Total changes
    pub fn total_changes(&self) -> usize {
        self.keys_normalized + self.values_normalized + self.keys_removed
    }
}

impl Default for NormalizationResult {
    fn default() -> Self {
        Self::success(HashMap::new())
    }
}

/// Normalizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizerStats {
    /// Total normalizations
    pub total_normalizations: usize,
    /// Keys normalized
    pub keys_normalized: usize,
    /// Values normalized
    pub values_normalized: usize,
    /// Keys removed
    pub keys_removed: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NormalizerStats {
    /// Record normalization
    pub fn record(&mut self, result: &NormalizationResult) {
        self.total_normalizations += 1;
        self.keys_normalized += result.keys_normalized;
        self.values_normalized += result.values_normalized;
        self.keys_removed += result.keys_removed;
    }

    /// Record by type
    pub fn record_type(&mut self, norm_type: NormalizationType) {
        *self.by_type.entry(norm_type.to_string()).or_insert(0) += 1;
    }

    /// Changes per normalization
    pub fn changes_per_normalization(&self) -> f64 {
        if self.total_normalizations == 0 {
            0.0
        } else {
            (self.keys_normalized + self.values_normalized) as f64 / self.total_normalizations as f64
        }
    }
}

/// Settings normalizer
#[derive(Debug, Clone, Default)]
pub struct SettingsNormalizer {
    /// Config
    config: NormalizerConfig,
    /// Stats
    stats: NormalizerStats,
}

impl SettingsNormalizer {
    /// Create new normalizer
    pub fn new(config: NormalizerConfig) -> Self {
        Self {
            config,
            stats: NormalizerStats::default(),
        }
    }

    /// Normalize key
    fn normalize_key(&self, key: &str) -> String {
        let mut result = key.to_string();
        
        if self.config.trim_whitespace {
            result = result.trim().to_string();
        }
        
        result = match self.config.key_case {
            CaseStyle::Lower => result.to_lowercase(),
            CaseStyle::Upper => result.to_uppercase(),
            CaseStyle::Snake => result.replace('-', "_").replace(' ', "_").to_lowercase(),
            CaseStyle::Kebab => result.replace('_', "-").replace(' ', "-").to_lowercase(),
            CaseStyle::Camel => result, // Would need more complex logic
        };
        
        result
    }

    /// Normalize value
    fn normalize_value(&self, value: &str) -> String {
        let mut result = value.to_string();
        
        if self.config.trim_whitespace {
            result = result.trim().to_string();
        }
        
        if self.config.collapse_whitespace {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }
        
        result
    }

    /// Normalize settings
    pub fn normalize(&mut self, settings: &HashMap<String, String>) -> NormalizationResult {
        let mut result_settings = HashMap::new();
        let mut keys_normalized = 0;
        let mut values_normalized = 0;
        let mut keys_removed = 0;

        for (key, value) in settings {
            let normalized_key = self.normalize_key(key);
            let normalized_value = self.normalize_value(value);

            if self.config.remove_empty && normalized_value.is_empty() {
                keys_removed += 1;
                continue;
            }

            if normalized_key != *key {
                keys_normalized += 1;
            }
            if normalized_value != *value {
                values_normalized += 1;
            }

            result_settings.insert(normalized_key, normalized_value);
        }

        self.stats.record_type(self.config.normalization_type);

        let result = NormalizationResult::success(result_settings)
            .with_counts(keys_normalized, values_normalized, keys_removed);

        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &NormalizerStats {
        &self.stats
    }

    /// Get config
    pub fn config(&self) -> &NormalizerConfig {
        &self.config
    }
}

/// Normalizer registry
#[derive(Debug, Clone, Default)]
pub struct NormalizerRegistry {
    /// Normalizers by ID
    normalizers: HashMap<String, SettingsNormalizer>,
}

impl NormalizerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register normalizer
    pub fn register(&mut self, id: impl Into<String>, normalizer: SettingsNormalizer) {
        self.normalizers.insert(id.into(), normalizer);
    }

    /// Unregister normalizer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.normalizers.remove(id).is_some()
    }

    /// Get normalizer
    pub fn get(&self, id: &str) -> Option<&SettingsNormalizer> {
        self.normalizers.get(id)
    }

    /// Get normalizer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsNormalizer> {
        self.normalizers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.normalizers.len()
    }
}

/// Format normalizer registry
pub fn format_normalizer_registry(registry: &NormalizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Normalizer Registry:\n");
    output.push_str(&format!("  Normalizers: {}\n", registry.count()));
    output
}

/// Check if query is about normalizer
pub fn is_normalizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("normalize") || lower.contains("settings normalizer") || lower.contains("canonical")
}

/// Fun fact about normalizer
pub fn normalizer_fun_fact() -> &'static str {
    "Anna's settings normalizer converts settings to a canonical format for consistency!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_type_display() {
        assert_eq!(format!("{}", NormalizationType::Case), "case");
        assert_eq!(format!("{}", NormalizationType::Full), "full");
    }

    #[test]
    fn test_case_style_display() {
        assert_eq!(format!("{}", CaseStyle::Lower), "lower");
        assert_eq!(format!("{}", CaseStyle::Snake), "snake");
    }

    #[test]
    fn test_config_new() {
        let c = NormalizerConfig::new(NormalizationType::Case);
        assert!(c.trim_whitespace);
    }

    #[test]
    fn test_config_builder() {
        let c = NormalizerConfig::new(NormalizationType::Full)
            .key_case(CaseStyle::Snake)
            .remove_empty(true);
        assert_eq!(c.key_case, CaseStyle::Snake);
        assert!(c.remove_empty);
    }

    #[test]
    fn test_result_success() {
        let r = NormalizationResult::success(HashMap::new());
        assert!(r.success);
    }

    #[test]
    fn test_result_with_counts() {
        let r = NormalizationResult::success(HashMap::new())
            .with_counts(5, 3, 2);
        assert_eq!(r.total_changes(), 10);
    }

    #[test]
    fn test_stats_record() {
        let mut s = NormalizerStats::default();
        let r = NormalizationResult::success(HashMap::new())
            .with_counts(2, 3, 1);
        s.record(&r);
        assert_eq!(s.total_normalizations, 1);
        assert_eq!(s.keys_normalized, 2);
    }

    #[test]
    fn test_normalizer_new() {
        let n = SettingsNormalizer::new(NormalizerConfig::default());
        assert_eq!(n.stats().total_normalizations, 0);
    }

    #[test]
    fn test_normalizer_normalize_key_case() {
        let mut n = SettingsNormalizer::new(NormalizerConfig::new(NormalizationType::Case));
        let mut settings = HashMap::new();
        settings.insert("MY_KEY".to_string(), "value".to_string());
        
        let result = n.normalize(&settings);
        assert!(result.settings.contains_key("my_key"));
    }

    #[test]
    fn test_normalizer_trim_whitespace() {
        let mut n = SettingsNormalizer::new(NormalizerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "  value  ".to_string());
        
        let result = n.normalize(&settings);
        assert_eq!(result.settings.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_normalizer_remove_empty() {
        let mut n = SettingsNormalizer::new(
            NormalizerConfig::default().remove_empty(true)
        );
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "   ".to_string());
        
        let result = n.normalize(&settings);
        assert!(!result.settings.contains_key("key"));
        assert_eq!(result.keys_removed, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = NormalizerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = NormalizerRegistry::new();
        r.register("n1", SettingsNormalizer::new(NormalizerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_normalizer_query() {
        assert!(is_normalizer_query("normalize settings"));
        assert!(!is_normalizer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = normalizer_fun_fact();
        assert!(fact.contains("normalizer"));
    }
}

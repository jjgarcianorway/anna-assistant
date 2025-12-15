// v0.0.668: Settings Denormalization (Phase 244)
// Denormalize settings from canonical to target format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Denormalization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DenormalizationType {
    /// Expand to target format
    #[default]
    Expand,
    /// Unflatten nested structure
    Unflatten,
    /// Add prefixes
    Prefix,
    /// Add suffixes
    Suffix,
    /// Full denormalization
    Full,
}

impl std::fmt::Display for DenormalizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expand => write!(f, "expand"),
            Self::Unflatten => write!(f, "unflatten"),
            Self::Prefix => write!(f, "prefix"),
            Self::Suffix => write!(f, "suffix"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Target format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetFormat {
    /// JSON format
    #[default]
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// INI format
    Ini,
    /// Env format
    Env,
}

impl std::fmt::Display for TargetFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Toml => write!(f, "toml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}

/// Denormalizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenormalizerConfig {
    /// Denormalization type
    pub denorm_type: DenormalizationType,
    /// Target format
    pub target_format: TargetFormat,
    /// Key prefix
    pub key_prefix: String,
    /// Key suffix
    pub key_suffix: String,
    /// Preserve original keys
    pub preserve_original: bool,
}

impl DenormalizerConfig {
    /// Create new config
    pub fn new(denorm_type: DenormalizationType) -> Self {
        Self {
            denorm_type,
            target_format: TargetFormat::Json,
            key_prefix: String::new(),
            key_suffix: String::new(),
            preserve_original: false,
        }
    }

    /// Set target format
    pub fn target_format(mut self, format: TargetFormat) -> Self {
        self.target_format = format;
        self
    }

    /// Set key prefix
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    /// Set key suffix
    pub fn key_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.key_suffix = suffix.into();
        self
    }

    /// Set preserve original
    pub fn preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }
}

impl Default for DenormalizerConfig {
    fn default() -> Self {
        Self::new(DenormalizationType::Expand)
    }
}

/// Denormalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenormalizationResult {
    /// Denormalized settings
    pub settings: HashMap<String, String>,
    /// Keys expanded
    pub keys_expanded: usize,
    /// Keys prefixed
    pub keys_prefixed: usize,
    /// Keys suffixed
    pub keys_suffixed: usize,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
}

impl DenormalizationResult {
    /// Create success result
    pub fn success(settings: HashMap<String, String>) -> Self {
        Self {
            settings,
            keys_expanded: 0,
            keys_prefixed: 0,
            keys_suffixed: 0,
            success: true,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            settings: HashMap::new(),
            keys_expanded: 0,
            keys_prefixed: 0,
            keys_suffixed: 0,
            success: false,
            error: Some(error.into()),
        }
    }

    /// With counts
    pub fn with_counts(mut self, expanded: usize, prefixed: usize, suffixed: usize) -> Self {
        self.keys_expanded = expanded;
        self.keys_prefixed = prefixed;
        self.keys_suffixed = suffixed;
        self
    }

    /// Total changes
    pub fn total_changes(&self) -> usize {
        self.keys_expanded + self.keys_prefixed + self.keys_suffixed
    }
}

impl Default for DenormalizationResult {
    fn default() -> Self {
        Self::success(HashMap::new())
    }
}

/// Denormalizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DenormalizerStats {
    /// Total denormalizations
    pub total_denormalizations: usize,
    /// Keys expanded
    pub keys_expanded: usize,
    /// Keys prefixed
    pub keys_prefixed: usize,
    /// Keys suffixed
    pub keys_suffixed: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DenormalizerStats {
    /// Record denormalization
    pub fn record(&mut self, result: &DenormalizationResult) {
        self.total_denormalizations += 1;
        self.keys_expanded += result.keys_expanded;
        self.keys_prefixed += result.keys_prefixed;
        self.keys_suffixed += result.keys_suffixed;
    }

    /// Record by type
    pub fn record_type(&mut self, denorm_type: DenormalizationType) {
        *self.by_type.entry(denorm_type.to_string()).or_insert(0) += 1;
    }

    /// Changes per denormalization
    pub fn changes_per_denormalization(&self) -> f64 {
        if self.total_denormalizations == 0 {
            0.0
        } else {
            (self.keys_expanded + self.keys_prefixed + self.keys_suffixed) as f64 
                / self.total_denormalizations as f64
        }
    }
}

/// Settings denormalizer
#[derive(Debug, Clone, Default)]
pub struct SettingsDenormalizer {
    /// Config
    config: DenormalizerConfig,
    /// Stats
    stats: DenormalizerStats,
}

impl SettingsDenormalizer {
    /// Create new denormalizer
    pub fn new(config: DenormalizerConfig) -> Self {
        Self {
            config,
            stats: DenormalizerStats::default(),
        }
    }

    /// Denormalize settings
    pub fn denormalize(&mut self, settings: &HashMap<String, String>) -> DenormalizationResult {
        let mut result_settings = HashMap::new();
        let mut keys_prefixed = 0;
        let mut keys_suffixed = 0;

        for (key, value) in settings {
            let mut new_key = key.clone();

            if !self.config.key_prefix.is_empty() {
                new_key = format!("{}{}", self.config.key_prefix, new_key);
                keys_prefixed += 1;
            }

            if !self.config.key_suffix.is_empty() {
                new_key = format!("{}{}", new_key, self.config.key_suffix);
                keys_suffixed += 1;
            }

            result_settings.insert(new_key, value.clone());

            if self.config.preserve_original && (keys_prefixed > 0 || keys_suffixed > 0) {
                result_settings.insert(key.clone(), value.clone());
            }
        }

        self.stats.record_type(self.config.denorm_type);

        let result = DenormalizationResult::success(result_settings)
            .with_counts(0, keys_prefixed, keys_suffixed);

        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &DenormalizerStats {
        &self.stats
    }

    /// Get config
    pub fn config(&self) -> &DenormalizerConfig {
        &self.config
    }
}

/// Denormalizer registry
#[derive(Debug, Clone, Default)]
pub struct DenormalizerRegistry {
    /// Denormalizers by ID
    denormalizers: HashMap<String, SettingsDenormalizer>,
}

impl DenormalizerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register denormalizer
    pub fn register(&mut self, id: impl Into<String>, denormalizer: SettingsDenormalizer) {
        self.denormalizers.insert(id.into(), denormalizer);
    }

    /// Unregister denormalizer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.denormalizers.remove(id).is_some()
    }

    /// Get denormalizer
    pub fn get(&self, id: &str) -> Option<&SettingsDenormalizer> {
        self.denormalizers.get(id)
    }

    /// Get denormalizer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDenormalizer> {
        self.denormalizers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.denormalizers.len()
    }
}

/// Format denormalizer registry
pub fn format_denormalizer_registry(registry: &DenormalizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Denormalizer Registry:\n");
    output.push_str(&format!("  Denormalizers: {}\n", registry.count()));
    output
}

/// Check if query is about denormalizer
pub fn is_denormalizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("denormalize") || lower.contains("expand settings") || lower.contains("unflatten")
}

/// Fun fact about denormalizer
pub fn denormalizer_fun_fact() -> &'static str {
    "Anna's settings denormalizer expands canonical settings to target formats!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denorm_type_display() {
        assert_eq!(format!("{}", DenormalizationType::Expand), "expand");
        assert_eq!(format!("{}", DenormalizationType::Unflatten), "unflatten");
    }

    #[test]
    fn test_target_format_display() {
        assert_eq!(format!("{}", TargetFormat::Json), "json");
        assert_eq!(format!("{}", TargetFormat::Yaml), "yaml");
    }

    #[test]
    fn test_config_new() {
        let c = DenormalizerConfig::new(DenormalizationType::Expand);
        assert!(!c.preserve_original);
    }

    #[test]
    fn test_config_builder() {
        let c = DenormalizerConfig::new(DenormalizationType::Prefix)
            .key_prefix("app.")
            .preserve_original(true);
        assert_eq!(c.key_prefix, "app.");
        assert!(c.preserve_original);
    }

    #[test]
    fn test_result_success() {
        let r = DenormalizationResult::success(HashMap::new());
        assert!(r.success);
    }

    #[test]
    fn test_result_failure() {
        let r = DenormalizationResult::failure("error");
        assert!(!r.success);
    }

    #[test]
    fn test_result_with_counts() {
        let r = DenormalizationResult::success(HashMap::new())
            .with_counts(5, 3, 2);
        assert_eq!(r.total_changes(), 10);
    }

    #[test]
    fn test_stats_record() {
        let mut s = DenormalizerStats::default();
        let r = DenormalizationResult::success(HashMap::new())
            .with_counts(2, 3, 1);
        s.record(&r);
        assert_eq!(s.total_denormalizations, 1);
        assert_eq!(s.keys_prefixed, 3);
    }

    #[test]
    fn test_denormalizer_new() {
        let d = SettingsDenormalizer::new(DenormalizerConfig::default());
        assert_eq!(d.stats().total_denormalizations, 0);
    }

    #[test]
    fn test_denormalizer_prefix() {
        let mut d = SettingsDenormalizer::new(
            DenormalizerConfig::new(DenormalizationType::Prefix)
                .key_prefix("app.")
        );
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());
        
        let result = d.denormalize(&settings);
        assert!(result.settings.contains_key("app.key"));
    }

    #[test]
    fn test_denormalizer_suffix() {
        let mut d = SettingsDenormalizer::new(
            DenormalizerConfig::new(DenormalizationType::Suffix)
                .key_suffix("_setting")
        );
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());
        
        let result = d.denormalize(&settings);
        assert!(result.settings.contains_key("key_setting"));
    }

    #[test]
    fn test_registry_new() {
        let r = DenormalizerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DenormalizerRegistry::new();
        r.register("d1", SettingsDenormalizer::new(DenormalizerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_denormalizer_query() {
        assert!(is_denormalizer_query("denormalize settings"));
        assert!(!is_denormalizer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = denormalizer_fun_fact();
        assert!(fact.contains("denormalizer"));
    }
}

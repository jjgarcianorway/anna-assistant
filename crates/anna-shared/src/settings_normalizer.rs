// v0.0.645: Settings Normalizer (Phase 221)
// Normalizer for standardizing settings values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Normalization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NormalizationType {
    /// String normalization
    #[default]
    String,
    /// Path normalization
    Path,
    /// URL normalization
    Url,
    /// Number normalization
    Number,
    /// Boolean normalization
    Boolean,
}

impl std::fmt::Display for NormalizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Path => write!(f, "path"),
            Self::Url => write!(f, "url"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
        }
    }
}

/// Normalization rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NormalizationRule {
    /// No transformation
    #[default]
    None,
    /// Lowercase
    Lowercase,
    /// Uppercase
    Uppercase,
    /// Trim
    Trim,
    /// Canonical form
    Canonical,
}

impl std::fmt::Display for NormalizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Lowercase => write!(f, "lowercase"),
            Self::Uppercase => write!(f, "uppercase"),
            Self::Trim => write!(f, "trim"),
            Self::Canonical => write!(f, "canonical"),
        }
    }
}

/// Normalizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizerConfig {
    /// Normalization type
    pub normalization_type: NormalizationType,
    /// Normalization rule
    pub rule: NormalizationRule,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Preserve original
    pub preserve_original: bool,
    /// Enabled
    pub enabled: bool,
}

impl NormalizerConfig {
    /// Create new config
    pub fn new(normalization_type: NormalizationType) -> Self {
        Self {
            normalization_type,
            rule: NormalizationRule::None,
            category: None,
            preserve_original: true,
            enabled: true,
        }
    }

    /// Set rule
    pub fn rule(mut self, rule: NormalizationRule) -> Self {
        self.rule = rule;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set preserve original
    pub fn preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }

    /// Set enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self::new(NormalizationType::String)
    }
}

/// Normalization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    /// Original value
    pub original: String,
    /// Normalized value
    pub normalized: String,
    /// Normalization type
    pub normalization_type: NormalizationType,
    /// Rule applied
    pub rule_applied: NormalizationRule,
    /// Was modified
    pub modified: bool,
}

impl NormalizationResult {
    /// Create new result
    pub fn new(
        original: impl Into<String>,
        normalized: impl Into<String>,
        normalization_type: NormalizationType,
        rule_applied: NormalizationRule,
    ) -> Self {
        let orig = original.into();
        let norm = normalized.into();
        let modified = orig != norm;
        Self {
            original: orig,
            normalized: norm,
            normalization_type,
            rule_applied,
            modified,
        }
    }

    /// Was modified
    pub fn was_modified(&self) -> bool {
        self.modified
    }

    /// Get normalized value
    pub fn value(&self) -> &str {
        &self.normalized
    }
}

/// Normalizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizerStats {
    /// Total normalized
    pub total_normalized: usize,
    /// Modified count
    pub modified: usize,
    /// Unmodified count
    pub unmodified: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
    /// By rule
    pub by_rule: HashMap<String, usize>,
}

impl NormalizerStats {
    /// Record normalization
    pub fn record(&mut self, normalization_type: NormalizationType, rule: NormalizationRule, modified: bool) {
        self.total_normalized += 1;
        if modified {
            self.modified += 1;
        } else {
            self.unmodified += 1;
        }
        *self.by_type.entry(normalization_type.to_string()).or_insert(0) += 1;
        *self.by_rule.entry(rule.to_string()).or_insert(0) += 1;
    }

    /// Modification rate
    pub fn modification_rate(&self) -> f64 {
        if self.total_normalized == 0 {
            0.0
        } else {
            self.modified as f64 / self.total_normalized as f64
        }
    }
}

/// Settings normalizer
#[derive(Debug, Clone, Default)]
pub struct SettingsNormalizer {
    /// Config
    config: NormalizerConfig,
    /// Results
    results: Vec<NormalizationResult>,
    /// Stats
    stats: NormalizerStats,
}

impl SettingsNormalizer {
    /// Create new normalizer
    pub fn new(config: NormalizerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: NormalizerStats::default(),
        }
    }

    /// Normalize value
    pub fn normalize(&mut self, value: impl Into<String>) -> NormalizationResult {
        let original = value.into();

        if !self.config.enabled {
            let result = NormalizationResult::new(
                original.clone(),
                original,
                self.config.normalization_type,
                NormalizationRule::None,
            );
            self.results.push(result.clone());
            return result;
        }

        let normalized = self.apply_rule(&original);
        self.stats.record(
            self.config.normalization_type,
            self.config.rule,
            normalized != original,
        );

        let result = NormalizationResult::new(
            original,
            normalized,
            self.config.normalization_type,
            self.config.rule,
        );
        self.results.push(result.clone());
        result
    }

    /// Apply normalization rule
    fn apply_rule(&self, value: &str) -> String {
        match self.config.rule {
            NormalizationRule::None => value.to_string(),
            NormalizationRule::Lowercase => value.to_lowercase(),
            NormalizationRule::Uppercase => value.to_uppercase(),
            NormalizationRule::Trim => value.trim().to_string(),
            NormalizationRule::Canonical => {
                // Canonical: trim + lowercase
                value.trim().to_lowercase()
            }
        }
    }

    /// Get results
    pub fn results(&self) -> &[NormalizationResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &NormalizerStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

/// Settings normalizer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsNormalizerRegistry {
    /// Normalizers by ID
    normalizers: HashMap<String, SettingsNormalizer>,
}

impl SettingsNormalizerRegistry {
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

    /// Normalizer count
    pub fn count(&self) -> usize {
        self.normalizers.len()
    }
}

/// Format normalizer registry
pub fn format_normalizer_registry(registry: &SettingsNormalizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Normalizer Registry:\n");
    output.push_str(&format!("  Normalizers: {}\n", registry.count()));
    output
}

/// Check if query is about normalizer
pub fn is_normalizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("normalizer") || lower.contains("normalize settings") || lower.contains("standardize")
}

/// Fun fact about normalizer
pub fn normalizer_fun_fact() -> &'static str {
    "Anna's settings normalizers standardize values for consistency!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_type_display() {
        assert_eq!(format!("{}", NormalizationType::String), "string");
        assert_eq!(format!("{}", NormalizationType::Path), "path");
    }

    #[test]
    fn test_normalization_rule_display() {
        assert_eq!(format!("{}", NormalizationRule::Lowercase), "lowercase");
        assert_eq!(format!("{}", NormalizationRule::Trim), "trim");
    }

    #[test]
    fn test_config_new() {
        let c = NormalizerConfig::new(NormalizationType::String);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = NormalizerConfig::new(NormalizationType::Path)
            .rule(NormalizationRule::Canonical)
            .preserve_original(false);
        assert_eq!(c.rule, NormalizationRule::Canonical);
        assert!(!c.preserve_original);
    }

    #[test]
    fn test_result_new() {
        let r = NormalizationResult::new(
            "TEST",
            "test",
            NormalizationType::String,
            NormalizationRule::Lowercase,
        );
        assert!(r.was_modified());
    }

    #[test]
    fn test_result_unchanged() {
        let r = NormalizationResult::new(
            "test",
            "test",
            NormalizationType::String,
            NormalizationRule::None,
        );
        assert!(!r.was_modified());
    }

    #[test]
    fn test_stats_record() {
        let mut s = NormalizerStats::default();
        s.record(NormalizationType::String, NormalizationRule::Lowercase, true);
        s.record(NormalizationType::String, NormalizationRule::None, false);
        assert_eq!(s.total_normalized, 2);
        assert_eq!(s.modified, 1);
    }

    #[test]
    fn test_normalizer_new() {
        let n = SettingsNormalizer::new(NormalizerConfig::new(NormalizationType::String));
        assert_eq!(n.result_count(), 0);
    }

    #[test]
    fn test_normalizer_normalize_lowercase() {
        let mut n = SettingsNormalizer::new(
            NormalizerConfig::new(NormalizationType::String)
                .rule(NormalizationRule::Lowercase),
        );
        let r = n.normalize("TEST");
        assert!(r.was_modified());
        assert_eq!(r.normalized, "test");
    }

    #[test]
    fn test_normalizer_normalize_canonical() {
        let mut n = SettingsNormalizer::new(
            NormalizerConfig::new(NormalizationType::String)
                .rule(NormalizationRule::Canonical),
        );
        let r = n.normalize("  TEST  ");
        assert!(r.was_modified());
        assert_eq!(r.normalized, "test");
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsNormalizerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsNormalizerRegistry::new();
        r.register("norm1", SettingsNormalizer::new(NormalizerConfig::new(NormalizationType::String)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_normalizer_query() {
        assert!(is_normalizer_query("settings normalizer"));
        assert!(!is_normalizer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = normalizer_fun_fact();
        assert!(fact.contains("normalizer"));
    }
}

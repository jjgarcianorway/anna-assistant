// v0.0.643: Settings Sanitizer (Phase 219)
// Sanitizer for cleaning and validating settings values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Sanitization type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SanitizationType {
    /// Trim whitespace
    #[default]
    Trim,
    /// Normalize case
    NormalizeCase,
    /// Remove special chars
    RemoveSpecial,
    /// Escape values
    Escape,
    /// Full sanitization
    Full,
}

impl std::fmt::Display for SanitizationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trim => write!(f, "trim"),
            Self::NormalizeCase => write!(f, "normalize_case"),
            Self::RemoveSpecial => write!(f, "remove_special"),
            Self::Escape => write!(f, "escape"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Case normalization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CaseNormalization {
    /// No change
    #[default]
    None,
    /// Lowercase
    Lower,
    /// Uppercase
    Upper,
    /// Title case
    Title,
}

impl std::fmt::Display for CaseNormalization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Lower => write!(f, "lower"),
            Self::Upper => write!(f, "upper"),
            Self::Title => write!(f, "title"),
        }
    }
}

/// Sanitizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizerConfig {
    /// Sanitization type
    pub sanitization_type: SanitizationType,
    /// Case normalization
    pub case_normalization: CaseNormalization,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Trim whitespace
    pub trim: bool,
    /// Remove empty
    pub remove_empty: bool,
}

impl SanitizerConfig {
    /// Create new config
    pub fn new(sanitization_type: SanitizationType) -> Self {
        Self {
            sanitization_type,
            case_normalization: CaseNormalization::None,
            category: None,
            trim: true,
            remove_empty: false,
        }
    }

    /// Set case normalization
    pub fn case_normalization(mut self, case: CaseNormalization) -> Self {
        self.case_normalization = case;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set trim
    pub fn trim(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }

    /// Set remove empty
    pub fn remove_empty(mut self, remove: bool) -> Self {
        self.remove_empty = remove;
        self
    }
}

/// Sanitization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationResult {
    /// Original value
    pub original: String,
    /// Sanitized value
    pub sanitized: String,
    /// Changed
    pub changed: bool,
    /// Operations applied
    pub operations: Vec<String>,
}

impl SanitizationResult {
    /// Create new result
    pub fn new(original: impl Into<String>, sanitized: impl Into<String>) -> Self {
        let orig = original.into();
        let san = sanitized.into();
        let changed = orig != san;
        Self {
            original: orig,
            sanitized: san,
            changed,
            operations: Vec::new(),
        }
    }

    /// Add operation
    pub fn add_operation(&mut self, op: impl Into<String>) {
        self.operations.push(op.into());
    }

    /// Was changed
    pub fn was_changed(&self) -> bool {
        self.changed
    }

    /// Operation count
    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }
}

/// Sanitizer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SanitizerStats {
    /// Total sanitized
    pub total_sanitized: usize,
    /// Changed count
    pub changed: usize,
    /// Unchanged count
    pub unchanged: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SanitizerStats {
    /// Record sanitization
    pub fn record(&mut self, sanitization_type: SanitizationType, changed: bool) {
        self.total_sanitized += 1;
        if changed {
            self.changed += 1;
        } else {
            self.unchanged += 1;
        }
        *self.by_type.entry(sanitization_type.to_string()).or_insert(0) += 1;
    }

    /// Change rate
    pub fn change_rate(&self) -> f64 {
        if self.total_sanitized == 0 {
            0.0
        } else {
            self.changed as f64 / self.total_sanitized as f64
        }
    }
}

/// Settings sanitizer
#[derive(Debug, Clone, Default)]
pub struct SettingsSanitizer {
    /// Config
    config: SanitizerConfig,
    /// Results
    results: Vec<SanitizationResult>,
    /// Stats
    stats: SanitizerStats,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self::new(SanitizationType::Trim)
    }
}

impl SettingsSanitizer {
    /// Create new sanitizer
    pub fn new(config: SanitizerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: SanitizerStats::default(),
        }
    }

    /// Sanitize value
    pub fn sanitize(&mut self, value: impl Into<String>) -> SanitizationResult {
        let original = value.into();
        let mut sanitized = original.clone();
        let mut operations = Vec::new();

        if self.config.trim {
            let trimmed = sanitized.trim().to_string();
            if trimmed != sanitized {
                operations.push("trim".to_string());
                sanitized = trimmed;
            }
        }

        match self.config.case_normalization {
            CaseNormalization::Lower => {
                let lower = sanitized.to_lowercase();
                if lower != sanitized {
                    operations.push("lowercase".to_string());
                    sanitized = lower;
                }
            }
            CaseNormalization::Upper => {
                let upper = sanitized.to_uppercase();
                if upper != sanitized {
                    operations.push("uppercase".to_string());
                    sanitized = upper;
                }
            }
            _ => {}
        }

        let changed = original != sanitized;
        self.stats.record(self.config.sanitization_type, changed);

        let mut result = SanitizationResult::new(original, sanitized);
        for op in operations {
            result.add_operation(op);
        }
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[SanitizationResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &SanitizerStats {
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

/// Settings sanitizer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsSanitizerRegistry {
    /// Sanitizers by ID
    sanitizers: HashMap<String, SettingsSanitizer>,
}

impl SettingsSanitizerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register sanitizer
    pub fn register(&mut self, id: impl Into<String>, sanitizer: SettingsSanitizer) {
        self.sanitizers.insert(id.into(), sanitizer);
    }

    /// Unregister sanitizer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.sanitizers.remove(id).is_some()
    }

    /// Get sanitizer
    pub fn get(&self, id: &str) -> Option<&SettingsSanitizer> {
        self.sanitizers.get(id)
    }

    /// Get sanitizer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsSanitizer> {
        self.sanitizers.get_mut(id)
    }

    /// Sanitizer count
    pub fn count(&self) -> usize {
        self.sanitizers.len()
    }
}

/// Format sanitizer registry
pub fn format_sanitizer_registry(registry: &SettingsSanitizerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Sanitizer Registry:\n");
    output.push_str(&format!("  Sanitizers: {}\n", registry.count()));
    output
}

/// Check if query is about sanitizer
pub fn is_sanitizer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("sanitizer") || lower.contains("sanitize settings") || lower.contains("clean")
}

/// Fun fact about sanitizer
pub fn sanitizer_fun_fact() -> &'static str {
    "Anna's settings sanitizers clean and normalize values!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitization_type_display() {
        assert_eq!(format!("{}", SanitizationType::Trim), "trim");
        assert_eq!(format!("{}", SanitizationType::Full), "full");
    }

    #[test]
    fn test_case_display() {
        assert_eq!(format!("{}", CaseNormalization::Lower), "lower");
        assert_eq!(format!("{}", CaseNormalization::Upper), "upper");
    }

    #[test]
    fn test_config_new() {
        let c = SanitizerConfig::new(SanitizationType::Trim);
        assert!(c.trim);
    }

    #[test]
    fn test_config_builder() {
        let c = SanitizerConfig::new(SanitizationType::Full)
            .case_normalization(CaseNormalization::Lower)
            .remove_empty(true);
        assert_eq!(c.case_normalization, CaseNormalization::Lower);
        assert!(c.remove_empty);
    }

    #[test]
    fn test_result_new() {
        let r = SanitizationResult::new("  test  ", "test");
        assert!(r.was_changed());
    }

    #[test]
    fn test_result_unchanged() {
        let r = SanitizationResult::new("test", "test");
        assert!(!r.was_changed());
    }

    #[test]
    fn test_stats_record() {
        let mut s = SanitizerStats::default();
        s.record(SanitizationType::Trim, true);
        s.record(SanitizationType::Trim, false);
        assert_eq!(s.total_sanitized, 2);
        assert_eq!(s.changed, 1);
    }

    #[test]
    fn test_sanitizer_new() {
        let s = SettingsSanitizer::new(SanitizerConfig::new(SanitizationType::Trim));
        assert_eq!(s.result_count(), 0);
    }

    #[test]
    fn test_sanitizer_sanitize_trim() {
        let mut s = SettingsSanitizer::new(SanitizerConfig::new(SanitizationType::Trim));
        let r = s.sanitize("  test  ");
        assert!(r.was_changed());
        assert_eq!(r.sanitized, "test");
    }

    #[test]
    fn test_sanitizer_sanitize_case() {
        let mut s = SettingsSanitizer::new(
            SanitizerConfig::new(SanitizationType::NormalizeCase)
                .case_normalization(CaseNormalization::Lower)
        );
        let r = s.sanitize("TEST");
        assert!(r.was_changed());
        assert_eq!(r.sanitized, "test");
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsSanitizerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsSanitizerRegistry::new();
        r.register("san1", SettingsSanitizer::new(SanitizerConfig::new(SanitizationType::Trim)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_sanitizer_query() {
        assert!(is_sanitizer_query("settings sanitizer"));
        assert!(!is_sanitizer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = sanitizer_fun_fact();
        assert!(fact.contains("sanitizer"));
    }
}

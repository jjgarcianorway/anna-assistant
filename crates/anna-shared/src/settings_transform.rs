// v0.0.666: Settings Transform (Phase 242)
// Transform settings between different formats and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TransformType {
    /// Map transformation
    #[default]
    Map,
    /// Filter transformation
    Filter,
    /// Reduce transformation
    Reduce,
    /// Flatten transformation
    Flatten,
    /// Group transformation
    Group,
}

impl std::fmt::Display for TransformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Map => write!(f, "map"),
            Self::Filter => write!(f, "filter"),
            Self::Reduce => write!(f, "reduce"),
            Self::Flatten => write!(f, "flatten"),
            Self::Group => write!(f, "group"),
        }
    }
}

/// Transform direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TransformDirection {
    /// Forward transformation
    #[default]
    Forward,
    /// Reverse transformation
    Reverse,
    /// Bidirectional
    Bidirectional,
}

impl std::fmt::Display for TransformDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forward => write!(f, "forward"),
            Self::Reverse => write!(f, "reverse"),
            Self::Bidirectional => write!(f, "bidirectional"),
        }
    }
}

/// Transformer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    /// Default transform type
    pub default_type: TransformType,
    /// Default direction
    pub default_direction: TransformDirection,
    /// Preserve originals
    pub preserve_originals: bool,
    /// Chain transforms
    pub chain_transforms: bool,
    /// Enable logging
    pub enable_logging: bool,
}

impl TransformerConfig {
    /// Create new config
    pub fn new(transform_type: TransformType) -> Self {
        Self {
            default_type: transform_type,
            default_direction: TransformDirection::Forward,
            preserve_originals: false,
            chain_transforms: true,
            enable_logging: false,
        }
    }

    /// Set direction
    pub fn direction(mut self, direction: TransformDirection) -> Self {
        self.default_direction = direction;
        self
    }

    /// Set preserve originals
    pub fn preserve_originals(mut self, preserve: bool) -> Self {
        self.preserve_originals = preserve;
        self
    }

    /// Set chain transforms
    pub fn chain_transforms(mut self, chain: bool) -> Self {
        self.chain_transforms = chain;
        self
    }
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self::new(TransformType::Map)
    }
}

/// Transform rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    /// Rule ID
    pub id: String,
    /// Source pattern
    pub source_pattern: String,
    /// Target pattern
    pub target_pattern: String,
    /// Transform type
    pub transform_type: TransformType,
    /// Enabled
    pub enabled: bool,
}

impl TransformRule {
    /// Create new rule
    pub fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source_pattern: source.into(),
            target_pattern: target.into(),
            transform_type: TransformType::Map,
            enabled: true,
        }
    }

    /// With transform type
    pub fn with_type(mut self, transform_type: TransformType) -> Self {
        self.transform_type = transform_type;
        self
    }

    /// Set enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Transform result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// Transformed settings
    pub settings: HashMap<String, String>,
    /// Rules applied
    pub rules_applied: Vec<String>,
    /// Keys transformed
    pub keys_transformed: usize,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
}

impl TransformResult {
    /// Create success result
    pub fn success(settings: HashMap<String, String>) -> Self {
        Self {
            settings,
            rules_applied: Vec::new(),
            keys_transformed: 0,
            success: true,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            settings: HashMap::new(),
            rules_applied: Vec::new(),
            keys_transformed: 0,
            success: false,
            error: Some(error.into()),
        }
    }

    /// With rules
    pub fn with_rules(mut self, rules: Vec<String>) -> Self {
        self.rules_applied = rules;
        self
    }

    /// With transformed count
    pub fn with_transformed(mut self, count: usize) -> Self {
        self.keys_transformed = count;
        self
    }
}

impl Default for TransformResult {
    fn default() -> Self {
        Self::success(HashMap::new())
    }
}

/// Transformer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformerStats {
    /// Total transformations
    pub total_transformations: usize,
    /// Keys transformed
    pub keys_transformed: usize,
    /// Rules applied
    pub rules_applied: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl TransformerStats {
    /// Record transformation
    pub fn record(&mut self, result: &TransformResult) {
        self.total_transformations += 1;
        self.keys_transformed += result.keys_transformed;
        self.rules_applied += result.rules_applied.len();
    }

    /// Record by type
    pub fn record_type(&mut self, transform_type: TransformType) {
        *self.by_type.entry(transform_type.to_string()).or_insert(0) += 1;
    }

    /// Keys per transformation
    pub fn keys_per_transformation(&self) -> f64 {
        if self.total_transformations == 0 {
            0.0
        } else {
            self.keys_transformed as f64 / self.total_transformations as f64
        }
    }

    /// Rules per transformation
    pub fn rules_per_transformation(&self) -> f64 {
        if self.total_transformations == 0 {
            0.0
        } else {
            self.rules_applied as f64 / self.total_transformations as f64
        }
    }
}

/// Settings transformer
#[derive(Debug, Clone, Default)]
pub struct SettingsTransformer {
    /// Config
    config: TransformerConfig,
    /// Rules
    rules: HashMap<String, TransformRule>,
    /// Stats
    stats: TransformerStats,
}

impl SettingsTransformer {
    /// Create new transformer
    pub fn new(config: TransformerConfig) -> Self {
        Self {
            config,
            rules: HashMap::new(),
            stats: TransformerStats::default(),
        }
    }

    /// Add rule
    pub fn add_rule(&mut self, rule: TransformRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Remove rule
    pub fn remove_rule(&mut self, id: &str) -> bool {
        self.rules.remove(id).is_some()
    }

    /// Get rule
    pub fn get_rule(&self, id: &str) -> Option<&TransformRule> {
        self.rules.get(id)
    }

    /// Transform settings
    pub fn transform(&mut self, settings: &HashMap<String, String>) -> TransformResult {
        let mut result_settings = if self.config.preserve_originals {
            settings.clone()
        } else {
            HashMap::new()
        };

        let enabled_rules: Vec<_> = self.rules.values()
            .filter(|r| r.enabled)
            .collect();

        let mut rules_applied = Vec::new();
        let mut keys_transformed = 0;

        for (key, value) in settings {
            for rule in &enabled_rules {
                if key.starts_with(&rule.source_pattern) {
                    let new_key = key.replacen(&rule.source_pattern, &rule.target_pattern, 1);
                    result_settings.insert(new_key, value.clone());
                    if !rules_applied.contains(&rule.id) {
                        rules_applied.push(rule.id.clone());
                    }
                    keys_transformed += 1;
                    self.stats.record_type(rule.transform_type);
                }
            }
            if !self.config.preserve_originals && keys_transformed == 0 {
                result_settings.insert(key.clone(), value.clone());
            }
        }

        let result = TransformResult::success(result_settings)
            .with_rules(rules_applied)
            .with_transformed(keys_transformed);

        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &TransformerStats {
        &self.stats
    }

    /// Rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Enabled count
    pub fn enabled_count(&self) -> usize {
        self.rules.values().filter(|r| r.enabled).count()
    }
}

/// Transformer registry
#[derive(Debug, Clone, Default)]
pub struct TransformerRegistry {
    /// Transformers by ID
    transformers: HashMap<String, SettingsTransformer>,
}

impl TransformerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register transformer
    pub fn register(&mut self, id: impl Into<String>, transformer: SettingsTransformer) {
        self.transformers.insert(id.into(), transformer);
    }

    /// Unregister transformer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.transformers.remove(id).is_some()
    }

    /// Get transformer
    pub fn get(&self, id: &str) -> Option<&SettingsTransformer> {
        self.transformers.get(id)
    }

    /// Get transformer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTransformer> {
        self.transformers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.transformers.len()
    }
}

/// Format transformer registry
pub fn format_transformer_registry(registry: &TransformerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Transformer Registry:\n");
    output.push_str(&format!("  Transformers: {}\n", registry.count()));
    output
}

/// Check if query is about transformer
pub fn is_transformer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("transform") || lower.contains("settings transformer") || lower.contains("convert settings")
}

/// Fun fact about transformer
pub fn transformer_fun_fact() -> &'static str {
    "Anna's settings transformer converts between different formats and structures!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_type_display() {
        assert_eq!(format!("{}", TransformType::Map), "map");
        assert_eq!(format!("{}", TransformType::Filter), "filter");
    }

    #[test]
    fn test_direction_display() {
        assert_eq!(format!("{}", TransformDirection::Forward), "forward");
        assert_eq!(format!("{}", TransformDirection::Reverse), "reverse");
    }

    #[test]
    fn test_config_new() {
        let c = TransformerConfig::new(TransformType::Map);
        assert!(!c.preserve_originals);
    }

    #[test]
    fn test_config_builder() {
        let c = TransformerConfig::new(TransformType::Filter)
            .direction(TransformDirection::Reverse)
            .preserve_originals(true);
        assert_eq!(c.default_direction, TransformDirection::Reverse);
        assert!(c.preserve_originals);
    }

    #[test]
    fn test_rule_new() {
        let r = TransformRule::new("r1", "src", "tgt");
        assert!(r.enabled);
    }

    #[test]
    fn test_rule_with_type() {
        let r = TransformRule::new("r1", "s", "t").with_type(TransformType::Filter);
        assert_eq!(r.transform_type, TransformType::Filter);
    }

    #[test]
    fn test_result_success() {
        let r = TransformResult::success(HashMap::new());
        assert!(r.success);
    }

    #[test]
    fn test_result_failure() {
        let r = TransformResult::failure("error");
        assert!(!r.success);
    }

    #[test]
    fn test_stats_record() {
        let mut s = TransformerStats::default();
        let r = TransformResult::success(HashMap::new()).with_transformed(5);
        s.record(&r);
        assert_eq!(s.total_transformations, 1);
        assert_eq!(s.keys_transformed, 5);
    }

    #[test]
    fn test_transformer_new() {
        let t = SettingsTransformer::new(TransformerConfig::default());
        assert_eq!(t.rule_count(), 0);
    }

    #[test]
    fn test_transformer_add_rule() {
        let mut t = SettingsTransformer::new(TransformerConfig::default());
        t.add_rule(TransformRule::new("r1", "src", "tgt"));
        assert_eq!(t.rule_count(), 1);
    }

    #[test]
    fn test_transformer_transform() {
        let mut t = SettingsTransformer::new(TransformerConfig::default());
        t.add_rule(TransformRule::new("r1", "old.", "new."));
        
        let mut settings = HashMap::new();
        settings.insert("old.key".to_string(), "value".to_string());
        
        let result = t.transform(&settings);
        assert!(result.success);
        assert!(result.settings.contains_key("new.key"));
    }

    #[test]
    fn test_registry_new() {
        let r = TransformerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TransformerRegistry::new();
        r.register("t1", SettingsTransformer::new(TransformerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_transformer_query() {
        assert!(is_transformer_query("transform settings"));
        assert!(!is_transformer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = transformer_fun_fact();
        assert!(fact.contains("transform"));
    }
}

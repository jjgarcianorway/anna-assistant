// v0.0.680: Settings Expander (Phase 256)
// Expand settings with variables and templates

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expand mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ExpandMode {
    /// Expand environment variables
    #[default]
    Environment,
    /// Expand references to other settings
    Reference,
    /// Expand template strings
    Template,
    /// Expand all
    All,
}

impl std::fmt::Display for ExpandMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Environment => write!(f, "environment"),
            Self::Reference => write!(f, "reference"),
            Self::Template => write!(f, "template"),
            Self::All => write!(f, "all"),
        }
    }
}

/// Variable syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VariableSyntax {
    /// Shell-style: ${VAR}
    #[default]
    Shell,
    /// Mustache-style: {{VAR}}
    Mustache,
    /// Percent-style: %VAR%
    Percent,
}

impl std::fmt::Display for VariableSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shell => write!(f, "shell"),
            Self::Mustache => write!(f, "mustache"),
            Self::Percent => write!(f, "percent"),
        }
    }
}

/// Expander config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpanderConfig {
    /// Expand mode
    pub mode: ExpandMode,
    /// Variable syntax
    pub syntax: VariableSyntax,
    /// Fail on missing
    pub fail_on_missing: bool,
    /// Default value for missing
    pub default_value: Option<String>,
}

impl ExpanderConfig {
    /// Create new config
    pub fn new(mode: ExpandMode) -> Self {
        Self {
            mode,
            syntax: VariableSyntax::Shell,
            fail_on_missing: false,
            default_value: None,
        }
    }

    /// Set syntax
    pub fn syntax(mut self, syntax: VariableSyntax) -> Self {
        self.syntax = syntax;
        self
    }

    /// Set fail on missing
    pub fn fail_on_missing(mut self, fail: bool) -> Self {
        self.fail_on_missing = fail;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Get variable pattern
    pub fn get_pattern(&self) -> (&str, &str) {
        match self.syntax {
            VariableSyntax::Shell => ("${", "}"),
            VariableSyntax::Mustache => ("{{", "}}"),
            VariableSyntax::Percent => ("%", "%"),
        }
    }
}

impl Default for ExpanderConfig {
    fn default() -> Self {
        Self::new(ExpandMode::Environment)
    }
}

/// Expand result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandResult {
    /// Expanded settings
    pub settings: HashMap<String, String>,
    /// Variables expanded
    pub variables_expanded: usize,
    /// Variables missing
    pub variables_missing: usize,
    /// Mode used
    pub mode: ExpandMode,
}

impl ExpandResult {
    /// Create new result
    pub fn new(settings: HashMap<String, String>, expanded: usize, missing: usize, mode: ExpandMode) -> Self {
        Self {
            settings,
            variables_expanded: expanded,
            variables_missing: missing,
            mode,
        }
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.variables_expanded + self.variables_missing;
        if total == 0 {
            1.0
        } else {
            self.variables_expanded as f64 / total as f64
        }
    }

    /// Has missing
    pub fn has_missing(&self) -> bool {
        self.variables_missing > 0
    }

    /// Get value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
}

impl Default for ExpandResult {
    fn default() -> Self {
        Self::new(HashMap::new(), 0, 0, ExpandMode::Environment)
    }
}

/// Expander stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpanderStats {
    /// Total expand operations
    pub total_operations: usize,
    /// Total variables expanded
    pub total_expanded: usize,
    /// Total variables missing
    pub total_missing: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl ExpanderStats {
    /// Record expand
    pub fn record(&mut self, result: &ExpandResult) {
        self.total_operations += 1;
        self.total_expanded += result.variables_expanded;
        self.total_missing += result.variables_missing;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn overall_success_rate(&self) -> f64 {
        let total = self.total_expanded + self.total_missing;
        if total == 0 {
            1.0
        } else {
            self.total_expanded as f64 / total as f64
        }
    }
}

/// Settings expander
#[derive(Debug, Clone, Default)]
pub struct SettingsExpander {
    /// Config
    config: ExpanderConfig,
    /// Stats
    stats: ExpanderStats,
    /// Custom variables
    variables: HashMap<String, String>,
}

impl SettingsExpander {
    /// Create new expander
    pub fn new(config: ExpanderConfig) -> Self {
        Self {
            config,
            stats: ExpanderStats::default(),
            variables: HashMap::new(),
        }
    }

    /// Set variable
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Clear variables
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// Expand with environment variables
    pub fn expand_env(&mut self, settings: &HashMap<String, String>) -> ExpandResult {
        let (prefix, suffix) = self.config.get_pattern();
        let mut expanded = HashMap::new();
        let mut expanded_count = 0;
        let mut missing_count = 0;

        for (key, value) in settings {
            let mut result = value.clone();
            let mut start_idx = 0;

            while let Some(start) = result[start_idx..].find(prefix) {
                let abs_start = start_idx + start;
                if let Some(end) = result[abs_start + prefix.len()..].find(suffix) {
                    let var_name = &result[abs_start + prefix.len()..abs_start + prefix.len() + end];

                    if let Ok(env_value) = std::env::var(var_name) {
                        result = format!(
                            "{}{}{}",
                            &result[..abs_start],
                            env_value,
                            &result[abs_start + prefix.len() + end + suffix.len()..]
                        );
                        expanded_count += 1;
                    } else if let Some(default) = &self.config.default_value {
                        result = format!(
                            "{}{}{}",
                            &result[..abs_start],
                            default,
                            &result[abs_start + prefix.len() + end + suffix.len()..]
                        );
                        missing_count += 1;
                    } else {
                        missing_count += 1;
                        start_idx = abs_start + prefix.len();
                    }
                } else {
                    break;
                }
            }

            expanded.insert(key.clone(), result);
        }

        let result = ExpandResult::new(expanded, expanded_count, missing_count, ExpandMode::Environment);
        self.stats.record(&result);
        result
    }

    /// Expand with custom variables
    pub fn expand_vars(&mut self, settings: &HashMap<String, String>) -> ExpandResult {
        let (prefix, suffix) = self.config.get_pattern();
        let mut expanded = HashMap::new();
        let mut expanded_count = 0;
        let mut missing_count = 0;

        for (key, value) in settings {
            let mut result = value.clone();

            for (var_name, var_value) in &self.variables {
                let pattern = format!("{}{}{}", prefix, var_name, suffix);
                if result.contains(&pattern) {
                    result = result.replace(&pattern, var_value);
                    expanded_count += 1;
                }
            }

            expanded.insert(key.clone(), result);
        }

        let result = ExpandResult::new(expanded, expanded_count, missing_count, ExpandMode::Template);
        self.stats.record(&result);
        result
    }

    /// Expand references to other settings
    pub fn expand_refs(&mut self, settings: &HashMap<String, String>) -> ExpandResult {
        let (prefix, suffix) = self.config.get_pattern();
        let mut expanded = settings.clone();
        let mut expanded_count = 0;
        let mut missing_count = 0;

        // Simple single-pass expansion
        for (key, value) in settings {
            let mut result = value.clone();
            let mut start_idx = 0;

            while let Some(start) = result[start_idx..].find(prefix) {
                let abs_start = start_idx + start;
                if let Some(end) = result[abs_start + prefix.len()..].find(suffix) {
                    let ref_key = &result[abs_start + prefix.len()..abs_start + prefix.len() + end];

                    if let Some(ref_value) = settings.get(ref_key) {
                        result = format!(
                            "{}{}{}",
                            &result[..abs_start],
                            ref_value,
                            &result[abs_start + prefix.len() + end + suffix.len()..]
                        );
                        expanded_count += 1;
                    } else {
                        missing_count += 1;
                        start_idx = abs_start + prefix.len();
                    }
                } else {
                    break;
                }
            }

            expanded.insert(key.clone(), result);
        }

        let result = ExpandResult::new(expanded, expanded_count, missing_count, ExpandMode::Reference);
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &ExpanderStats {
        &self.stats
    }
}

/// Expander registry
#[derive(Debug, Clone, Default)]
pub struct ExpanderRegistry {
    /// Expanders by ID
    expanders: HashMap<String, SettingsExpander>,
}

impl ExpanderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register expander
    pub fn register(&mut self, id: impl Into<String>, expander: SettingsExpander) {
        self.expanders.insert(id.into(), expander);
    }

    /// Unregister expander
    pub fn unregister(&mut self, id: &str) -> bool {
        self.expanders.remove(id).is_some()
    }

    /// Get expander
    pub fn get(&self, id: &str) -> Option<&SettingsExpander> {
        self.expanders.get(id)
    }

    /// Get expander mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsExpander> {
        self.expanders.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.expanders.len()
    }
}

/// Format expander registry
pub fn format_expander_registry(registry: &ExpanderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Expander Registry:\n");
    output.push_str(&format!("  Expanders: {}\n", registry.count()));
    output
}

/// Check if query is about expander
pub fn is_expander_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("expand settings") || lower.contains("settings expander") || lower.contains("interpolate settings")
}

/// Fun fact about expander
pub fn expander_fun_fact() -> &'static str {
    "Anna's settings expander substitutes variables and templates in your settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_mode_display() {
        assert_eq!(format!("{}", ExpandMode::Environment), "environment");
        assert_eq!(format!("{}", ExpandMode::Reference), "reference");
    }

    #[test]
    fn test_variable_syntax_display() {
        assert_eq!(format!("{}", VariableSyntax::Shell), "shell");
        assert_eq!(format!("{}", VariableSyntax::Mustache), "mustache");
    }

    #[test]
    fn test_config_new() {
        let c = ExpanderConfig::new(ExpandMode::Environment);
        assert_eq!(c.mode, ExpandMode::Environment);
    }

    #[test]
    fn test_config_builder() {
        let c = ExpanderConfig::new(ExpandMode::Template)
            .syntax(VariableSyntax::Mustache)
            .default_value("default");
        assert_eq!(c.syntax, VariableSyntax::Mustache);
        assert_eq!(c.default_value, Some("default".to_string()));
    }

    #[test]
    fn test_config_get_pattern() {
        assert_eq!(ExpanderConfig::new(ExpandMode::Environment).get_pattern(), ("${", "}"));
        let mustache = ExpanderConfig::new(ExpandMode::Environment).syntax(VariableSyntax::Mustache);
        assert_eq!(mustache.get_pattern(), ("{{", "}}"));
    }

    #[test]
    fn test_result_new() {
        let r = ExpandResult::new(HashMap::new(), 5, 2, ExpandMode::Environment);
        assert_eq!(r.variables_expanded, 5);
        assert!(r.has_missing());
    }

    #[test]
    fn test_result_success_rate() {
        let r = ExpandResult::new(HashMap::new(), 8, 2, ExpandMode::Environment);
        assert!((r.success_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ExpanderStats::default();
        let r = ExpandResult::new(HashMap::new(), 5, 1, ExpandMode::Environment);
        s.record(&r);
        assert_eq!(s.total_operations, 1);
        assert_eq!(s.total_expanded, 5);
    }

    #[test]
    fn test_expander_new() {
        let e = SettingsExpander::new(ExpanderConfig::default());
        assert_eq!(e.stats().total_operations, 0);
    }

    #[test]
    fn test_expander_set_variable() {
        let mut e = SettingsExpander::new(ExpanderConfig::default());
        e.set_variable("TEST", "value");
        assert!(e.variables.contains_key("TEST"));
    }

    #[test]
    fn test_expander_expand_vars() {
        let mut e = SettingsExpander::new(ExpanderConfig::default());
        e.set_variable("NAME", "Anna");

        let mut settings = HashMap::new();
        settings.insert("greeting".to_string(), "Hello, ${NAME}!".to_string());

        let result = e.expand_vars(&settings);
        assert_eq!(result.get("greeting").unwrap(), "Hello, Anna!");
    }

    #[test]
    fn test_expander_expand_refs() {
        let mut e = SettingsExpander::new(ExpanderConfig::default());

        let mut settings = HashMap::new();
        settings.insert("host".to_string(), "localhost".to_string());
        settings.insert("url".to_string(), "http://${host}:8080".to_string());

        let result = e.expand_refs(&settings);
        assert_eq!(result.get("url").unwrap(), "http://localhost:8080");
    }

    #[test]
    fn test_registry_new() {
        let r = ExpanderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ExpanderRegistry::new();
        r.register("e1", SettingsExpander::new(ExpanderConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_expander_query() {
        assert!(is_expander_query("expand settings"));
        assert!(!is_expander_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = expander_fun_fact();
        assert!(fact.contains("expander"));
    }
}

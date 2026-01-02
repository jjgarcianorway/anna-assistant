// v0.0.680: Settings Expander Core (Phase 256)
// Core expander implementation

use std::collections::HashMap;
use super::config::ExpanderConfig;
use super::result::ExpandResult;
use super::stats::ExpanderStats;
use super::types::ExpandMode;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}

//! Natural Language Configuration Mapper for Anna v0.5.0
//!
//! Maps user natural language requests to structured config changes.
//! LLM-A classifies requests, LLM-B validates changes.

use crate::config::{
    AnnaConfigV5, Channel, ConfigChange, ConfigMutation, CoreMode, LlmSelectionMode,
    MIN_UPDATE_INTERVAL,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Intent classification for user input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigIntent {
    /// Enable dev auto-update with interval
    EnableDevAutoUpdate { interval_seconds: Option<u64> },
    /// Disable auto-update
    DisableAutoUpdate,
    /// Set update channel
    SetChannel { channel: Channel },
    /// Switch to manual model selection
    SetManualModel { model: String },
    /// Switch back to auto model selection
    SetAutoModelSelection,
    /// Show current configuration
    ShowConfig,
    /// Set mode (normal/dev)
    SetMode { mode: CoreMode },
    /// Unknown/not a config request
    NotConfigRequest,
}

/// Result of intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub intent: ConfigIntent,
    pub confidence: f64,
    pub matched_pattern: Option<String>,
}

/// Pattern matcher for config requests
pub struct ConfigPatternMatcher {
    patterns: Vec<(Regex, ConfigIntent)>,
}

impl Default for ConfigPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigPatternMatcher {
    pub fn new() -> Self {
        let patterns = vec![
            // Enable dev auto-update patterns
            (
                Regex::new(r"(?i)enable\s+(?:dev\s+)?auto[- ]?update").unwrap(),
                ConfigIntent::EnableDevAutoUpdate { interval_seconds: None },
            ),
            (
                Regex::new(r"(?i)enable\s+(?:auto[- ]?)?updates?\s+every\s+(\d+)\s*(?:minute|min)").unwrap(),
                ConfigIntent::EnableDevAutoUpdate { interval_seconds: Some(600) },
            ),
            (
                Regex::new(r"(?i)turn\s+on\s+(?:dev\s+)?auto[- ]?update").unwrap(),
                ConfigIntent::EnableDevAutoUpdate { interval_seconds: None },
            ),
            (
                Regex::new(r"(?i)(?:start|activate)\s+(?:dev\s+)?auto[- ]?update").unwrap(),
                ConfigIntent::EnableDevAutoUpdate { interval_seconds: None },
            ),
            // Disable auto-update patterns
            (
                Regex::new(r"(?i)(?:disable|turn\s+off|stop)\s+(?:dev\s+)?auto[- ]?update").unwrap(),
                ConfigIntent::DisableAutoUpdate,
            ),
            (
                Regex::new(r"(?i)no\s+(?:more\s+)?auto[- ]?update").unwrap(),
                ConfigIntent::DisableAutoUpdate,
            ),
            // Manual model selection
            (
                Regex::new(r"(?i)(?:use|switch\s+to|set)\s+(?:model\s+)?([a-zA-Z0-9\.:_-]+)").unwrap(),
                ConfigIntent::SetManualModel { model: String::new() },
            ),
            (
                Regex::new(r"(?i)manual\s+model").unwrap(),
                ConfigIntent::SetManualModel { model: String::new() },
            ),
            // Auto model selection
            (
                Regex::new(r"(?i)(?:go\s+back\s+to|switch\s+to|use)\s+auto(?:matic)?\s+(?:model\s+)?selection").unwrap(),
                ConfigIntent::SetAutoModelSelection,
            ),
            (
                Regex::new(r"(?i)auto(?:matic)?\s+model\s+selection").unwrap(),
                ConfigIntent::SetAutoModelSelection,
            ),
            // Show config
            (
                Regex::new(r"(?i)show\s+(?:me\s+)?(?:your\s+)?(?:current\s+)?config(?:uration)?").unwrap(),
                ConfigIntent::ShowConfig,
            ),
            (
                Regex::new(r"(?i)what(?:'s|\s+is)\s+(?:your\s+)?(?:current\s+)?config(?:uration)?").unwrap(),
                ConfigIntent::ShowConfig,
            ),
            (
                Regex::new(r"(?i)display\s+(?:your\s+)?settings?").unwrap(),
                ConfigIntent::ShowConfig,
            ),
            // Set mode
            (
                Regex::new(r"(?i)(?:switch\s+to|enter|enable)\s+dev(?:elop(?:ment|er))?\s+mode").unwrap(),
                ConfigIntent::SetMode { mode: CoreMode::Dev },
            ),
            (
                Regex::new(r"(?i)(?:switch\s+to|enter|enable)\s+normal\s+mode").unwrap(),
                ConfigIntent::SetMode { mode: CoreMode::Normal },
            ),
        ];

        Self { patterns }
    }

    /// Classify user input to determine config intent
    pub fn classify(&self, input: &str) -> IntentResult {
        let input_lower = input.to_lowercase();

        // Check for interval specification
        let interval = self.extract_interval(&input_lower);

        // Check for model name
        let model_name = self.extract_model_name(input);

        for (regex, intent) in &self.patterns {
            if regex.is_match(input) {
                let matched_intent = match intent {
                    ConfigIntent::EnableDevAutoUpdate { .. } => {
                        ConfigIntent::EnableDevAutoUpdate {
                            interval_seconds: interval,
                        }
                    }
                    ConfigIntent::SetManualModel { .. } => {
                        if let Some(model) = model_name.clone() {
                            ConfigIntent::SetManualModel { model }
                        } else {
                            continue; // Need a model name
                        }
                    }
                    other => other.clone(),
                };

                return IntentResult {
                    intent: matched_intent,
                    confidence: 0.9,
                    matched_pattern: Some(regex.to_string()),
                };
            }
        }

        IntentResult {
            intent: ConfigIntent::NotConfigRequest,
            confidence: 0.95,
            matched_pattern: None,
        }
    }

    fn extract_interval(&self, input: &str) -> Option<u64> {
        // Look for "every X minutes" or "every X min"
        let interval_regex = Regex::new(r"every\s+(\d+)\s*(?:minute|min)").unwrap();
        if let Some(caps) = interval_regex.captures(input) {
            if let Some(num) = caps.get(1) {
                if let Ok(minutes) = num.as_str().parse::<u64>() {
                    return Some(minutes * 60);
                }
            }
        }

        // Look for "every X seconds" or "every X sec"
        let seconds_regex = Regex::new(r"every\s+(\d+)\s*(?:second|sec)").unwrap();
        if let Some(caps) = seconds_regex.captures(input) {
            if let Some(num) = caps.get(1) {
                if let Ok(seconds) = num.as_str().parse::<u64>() {
                    return Some(seconds);
                }
            }
        }

        None
    }

    fn extract_model_name(&self, input: &str) -> Option<String> {
        // Look for model names like "qwen2.5:14b", "llama3.2:3b"
        // Pattern: name + optional version (digits/dots) + optional :size
        let model_regex = Regex::new(r"([a-zA-Z]+\d*(?:\.\d+)?(?::[a-zA-Z0-9.]+)?)").unwrap();

        // Common model patterns
        let model_patterns = [
            "qwen", "llama", "mistral", "phi", "gemma", "codellama", "deepseek", "yi",
        ];

        for caps in model_regex.captures_iter(input) {
            if let Some(m) = caps.get(1) {
                let candidate = m.as_str().to_string();
                // Ensure it has a colon (indicating size like :14b)
                if candidate.contains(':') {
                    for pattern in &model_patterns {
                        if candidate.to_lowercase().contains(pattern) {
                            return Some(candidate);
                        }
                    }
                }
            }
        }

        // Fallback: match without colon requirement
        for caps in model_regex.captures_iter(input) {
            if let Some(m) = caps.get(1) {
                let candidate = m.as_str().to_string();
                for pattern in &model_patterns {
                    if candidate.to_lowercase().contains(pattern) {
                        return Some(candidate);
                    }
                }
            }
        }

        None
    }
}

/// Apply config intent to current config
pub fn apply_intent(config: &AnnaConfigV5, intent: &ConfigIntent) -> Option<ConfigMutation> {
    match intent {
        ConfigIntent::EnableDevAutoUpdate { interval_seconds } => {
            let mut changes = Vec::new();

            // Set dev mode if not already
            if config.core.mode != CoreMode::Dev {
                changes.push(ConfigChange::new(
                    "core.mode",
                    config.core.mode.as_str(),
                    "dev",
                ));
            }

            // Enable auto-update if not already
            if !config.update.enabled {
                changes.push(ConfigChange::new("update.enabled", "false", "true"));
            }

            // Set interval (enforce minimum)
            let interval = interval_seconds.unwrap_or(MIN_UPDATE_INTERVAL);
            let effective_interval = interval.max(MIN_UPDATE_INTERVAL);

            if config.update.interval_seconds != effective_interval {
                changes.push(ConfigChange::new(
                    "update.interval_seconds",
                    config.update.interval_seconds,
                    effective_interval,
                ));
            }

            if changes.is_empty() {
                return None;
            }

            let summary = if interval < MIN_UPDATE_INTERVAL && interval_seconds.is_some() {
                format!(
                    "Enabling dev auto-update. Interval capped at {} seconds (minimum enforced).",
                    MIN_UPDATE_INTERVAL
                )
            } else {
                format!(
                    "Enabling dev auto-update every {} seconds.",
                    effective_interval
                )
            };

            Some(ConfigMutation {
                changes,
                summary,
                requires_confirmation: false,
            })
        }

        ConfigIntent::DisableAutoUpdate => {
            if !config.update.enabled {
                return None;
            }

            Some(ConfigMutation {
                changes: vec![ConfigChange::new("update.enabled", "true", "false")],
                summary: "Disabling auto-update.".to_string(),
                requires_confirmation: false,
            })
        }

        ConfigIntent::SetChannel { channel } => {
            if config.update.channel == *channel {
                return None;
            }

            Some(ConfigMutation {
                changes: vec![ConfigChange::new(
                    "update.channel",
                    config.update.channel.as_str(),
                    channel.as_str(),
                )],
                summary: format!("Setting update channel to {}.", channel.as_str()),
                requires_confirmation: false,
            })
        }

        ConfigIntent::SetManualModel { model } => {
            let mut changes = Vec::new();

            if config.llm.selection_mode != LlmSelectionMode::Manual {
                changes.push(ConfigChange::new("llm.selection_mode", "auto", "manual"));
            }

            if config.llm.preferred_model != *model {
                changes.push(ConfigChange::new(
                    "llm.preferred_model",
                    &config.llm.preferred_model,
                    model,
                ));
            }

            if changes.is_empty() {
                return None;
            }

            Some(ConfigMutation {
                changes,
                summary: format!("Switching to manual model selection with {}.", model),
                requires_confirmation: true, // Changing model is significant
            })
        }

        ConfigIntent::SetAutoModelSelection => {
            if config.llm.selection_mode == LlmSelectionMode::Auto {
                return None;
            }

            Some(ConfigMutation {
                changes: vec![ConfigChange::new("llm.selection_mode", "manual", "auto")],
                summary: "Switching to automatic model selection based on hardware.".to_string(),
                requires_confirmation: false,
            })
        }

        ConfigIntent::SetMode { mode } => {
            if config.core.mode == *mode {
                return None;
            }

            let requires_confirmation = *mode == CoreMode::Dev;

            Some(ConfigMutation {
                changes: vec![ConfigChange::new(
                    "core.mode",
                    config.core.mode.as_str(),
                    mode.as_str(),
                )],
                summary: format!("Switching to {} mode.", mode.as_str()),
                requires_confirmation,
            })
        }

        ConfigIntent::ShowConfig | ConfigIntent::NotConfigRequest => None,
    }
}

/// Apply mutation to config and return new config
pub fn apply_mutation(config: &AnnaConfigV5, mutation: &ConfigMutation) -> AnnaConfigV5 {
    let mut new_config = config.clone();

    for change in &mutation.changes {
        match change.path.as_str() {
            "core.mode" => {
                new_config.core.mode = match change.to.as_str() {
                    "dev" => CoreMode::Dev,
                    _ => CoreMode::Normal,
                };
            }
            "llm.selection_mode" => {
                new_config.llm.selection_mode = match change.to.as_str() {
                    "manual" => LlmSelectionMode::Manual,
                    _ => LlmSelectionMode::Auto,
                };
            }
            "llm.preferred_model" => {
                new_config.llm.preferred_model = change.to.clone();
            }
            "update.enabled" => {
                new_config.update.enabled = change.to == "true";
            }
            "update.interval_seconds" => {
                if let Ok(interval) = change.to.parse() {
                    new_config.update.interval_seconds = interval;
                }
            }
            "update.channel" => {
                new_config.update.channel = match change.to.as_str() {
                    "stable" => Channel::Stable,
                    "beta" => Channel::Beta,
                    "dev" => Channel::Dev,
                    _ => Channel::Main,
                };
            }
            _ => {}
        }
    }

    new_config
}

/// Format config for display
pub fn format_config_display(config: &AnnaConfigV5) -> String {
    format!(
        r#"[source: config.core]
  mode: {}

[source: config.llm]
  selection_mode: {}
  preferred_model: {}
  fallback_model: {}

[source: config.update]
  enabled: {}
  interval_seconds: {}
  channel: {}"#,
        config.core.mode.as_str(),
        config.llm.selection_mode.as_str(),
        config.llm.preferred_model,
        config.llm.fallback_model,
        config.update.enabled,
        config.update.interval_seconds,
        config.update.channel.as_str()
    )
}

/// Format mutation diff for display
pub fn format_mutation_diff(mutation: &ConfigMutation) -> String {
    let mut diff = String::new();

    for change in &mutation.changes {
        diff.push_str(&format!(
            "[config.change]\npath = \"{}\"\nfrom = \"{}\"\nto   = \"{}\"\n\n",
            change.path, change.from, change.to
        ));
    }

    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_enable_dev_auto_update() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("enable dev auto-update every 10 minutes");
        assert!(matches!(
            result.intent,
            ConfigIntent::EnableDevAutoUpdate { interval_seconds: Some(600) }
        ));

        let result = matcher.classify("turn on auto update");
        assert!(matches!(
            result.intent,
            ConfigIntent::EnableDevAutoUpdate { .. }
        ));
    }

    #[test]
    fn test_classify_disable_auto_update() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("disable auto-update");
        assert_eq!(result.intent, ConfigIntent::DisableAutoUpdate);

        let result = matcher.classify("turn off auto update entirely");
        assert_eq!(result.intent, ConfigIntent::DisableAutoUpdate);
    }

    #[test]
    fn test_classify_manual_model() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("switch to manual model selection and use qwen2.5:14b");
        match &result.intent {
            ConfigIntent::SetManualModel { model } => {
                assert_eq!(model, "qwen2.5:14b");
            }
            _ => panic!("Expected SetManualModel, got {:?}", result.intent),
        }
    }

    #[test]
    fn test_classify_auto_model_selection() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("go back to automatic model selection");
        assert_eq!(result.intent, ConfigIntent::SetAutoModelSelection);
    }

    #[test]
    fn test_classify_show_config() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("show me your current configuration");
        assert_eq!(result.intent, ConfigIntent::ShowConfig);
    }

    #[test]
    fn test_classify_not_config_request() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("How many CPU cores do I have?");
        assert_eq!(result.intent, ConfigIntent::NotConfigRequest);
    }

    #[test]
    fn test_apply_enable_dev_auto_update() {
        let config = AnnaConfigV5::default();
        let intent = ConfigIntent::EnableDevAutoUpdate {
            interval_seconds: Some(600),
        };

        let mutation = apply_intent(&config, &intent).unwrap();
        assert!(!mutation.changes.is_empty());
        assert!(mutation.summary.contains("600"));
    }

    #[test]
    fn test_apply_interval_minimum_enforced() {
        let config = AnnaConfigV5::default();
        let intent = ConfigIntent::EnableDevAutoUpdate {
            interval_seconds: Some(60), // Below minimum
        };

        let mutation = apply_intent(&config, &intent).unwrap();
        assert!(mutation.summary.contains("capped"));
    }

    #[test]
    fn test_apply_mutation() {
        let config = AnnaConfigV5::default();
        let mutation = ConfigMutation {
            changes: vec![
                ConfigChange::new("core.mode", "normal", "dev"),
                ConfigChange::new("update.enabled", "false", "true"),
            ],
            summary: "Test".to_string(),
            requires_confirmation: false,
        };

        let new_config = apply_mutation(&config, &mutation);
        assert_eq!(new_config.core.mode, CoreMode::Dev);
        assert!(new_config.update.enabled);
    }

    #[test]
    fn test_format_config_display() {
        let config = AnnaConfigV5::default();
        let display = format_config_display(&config);

        assert!(display.contains("[source: config.core]"));
        assert!(display.contains("[source: config.llm]"));
        assert!(display.contains("[source: config.update]"));
    }

    #[test]
    fn test_format_mutation_diff() {
        let mutation = ConfigMutation {
            changes: vec![ConfigChange::new("update.enabled", "false", "true")],
            summary: "Test".to_string(),
            requires_confirmation: false,
        };

        let diff = format_mutation_diff(&mutation);
        assert!(diff.contains("[config.change]"));
        assert!(diff.contains("update.enabled"));
    }
}

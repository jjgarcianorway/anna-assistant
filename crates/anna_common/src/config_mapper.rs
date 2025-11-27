//! Natural Language Configuration Mapper for Anna v0.8.0
//!
//! Maps user natural language requests to structured config changes.
//! LLM-A classifies requests, LLM-B validates changes.
//! v0.8.0: Added log configuration support.

use crate::config::{
    AnnaConfigV5, Channel, ConfigChange, ConfigMutation, CoreMode, LlmSelectionMode,
    MIN_UPDATE_INTERVAL,
};
use crate::logging::{parse_log_config_intent, LogConfigIntent, LogLevel};
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
    /// v0.8.0: Set log level
    SetLogLevel { level: LogLevel },
    /// v0.8.0: Enable/disable LLM logging
    SetLlmLogging { enabled: bool },
    /// v0.8.0: Enable/disable daemon logging
    SetDaemonLogging { enabled: bool },
    /// v0.8.0: Enable/disable request logging
    SetRequestLogging { enabled: bool },
    /// v0.8.0: Show log configuration
    ShowLogConfig,
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

        // v0.8.0: Check for log configuration intents first
        let log_intent = parse_log_config_intent(input);
        match log_intent {
            LogConfigIntent::SetLevel(level) => {
                return IntentResult {
                    intent: ConfigIntent::SetLogLevel { level },
                    confidence: 0.9,
                    matched_pattern: Some("log level pattern".to_string()),
                };
            }
            LogConfigIntent::SetLlmEnabled(enabled) => {
                return IntentResult {
                    intent: ConfigIntent::SetLlmLogging { enabled },
                    confidence: 0.9,
                    matched_pattern: Some("llm logging pattern".to_string()),
                };
            }
            LogConfigIntent::SetDaemonEnabled(enabled) => {
                return IntentResult {
                    intent: ConfigIntent::SetDaemonLogging { enabled },
                    confidence: 0.9,
                    matched_pattern: Some("daemon logging pattern".to_string()),
                };
            }
            LogConfigIntent::SetRequestsEnabled(enabled) => {
                return IntentResult {
                    intent: ConfigIntent::SetRequestLogging { enabled },
                    confidence: 0.9,
                    matched_pattern: Some("request logging pattern".to_string()),
                };
            }
            LogConfigIntent::ShowConfig => {
                return IntentResult {
                    intent: ConfigIntent::ShowLogConfig,
                    confidence: 0.9,
                    matched_pattern: Some("show log config pattern".to_string()),
                };
            }
            LogConfigIntent::NotLogConfig => {
                // Continue with other config patterns
            }
        }

        // Check for interval specification
        let interval = self.extract_interval(&input_lower);

        // Check for model name
        let model_name = self.extract_model_name(input);

        for (regex, intent) in &self.patterns {
            if regex.is_match(input) {
                let matched_intent = match intent {
                    ConfigIntent::EnableDevAutoUpdate { .. } => ConfigIntent::EnableDevAutoUpdate {
                        interval_seconds: interval,
                    },
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
            "qwen",
            "llama",
            "mistral",
            "phi",
            "gemma",
            "codellama",
            "deepseek",
            "yi",
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

        ConfigIntent::SetLogLevel { level } => {
            if config.log.level == *level {
                return None;
            }

            Some(ConfigMutation {
                changes: vec![ConfigChange::new(
                    "log.level",
                    config.log.level.as_str(),
                    level.as_str(),
                )],
                summary: format!("Setting log level to {}.", level.as_str()),
                requires_confirmation: false,
            })
        }

        ConfigIntent::SetLlmLogging { enabled } => {
            if config.log.llm_enabled == *enabled {
                return None;
            }

            Some(ConfigMutation {
                changes: vec![ConfigChange::new(
                    "log.llm_enabled",
                    config.log.llm_enabled,
                    enabled,
                )],
                summary: format!(
                    "{} LLM orchestration logging.",
                    if *enabled { "Enabling" } else { "Disabling" }
                ),
                requires_confirmation: false,
            })
        }

        ConfigIntent::SetDaemonLogging { enabled } => {
            if config.log.daemon_enabled == *enabled {
                return None;
            }

            Some(ConfigMutation {
                changes: vec![ConfigChange::new(
                    "log.daemon_enabled",
                    config.log.daemon_enabled,
                    enabled,
                )],
                summary: format!(
                    "{} daemon logging.",
                    if *enabled { "Enabling" } else { "Disabling" }
                ),
                requires_confirmation: false,
            })
        }

        ConfigIntent::SetRequestLogging { enabled } => {
            if config.log.requests_enabled == *enabled {
                return None;
            }

            Some(ConfigMutation {
                changes: vec![ConfigChange::new(
                    "log.requests_enabled",
                    config.log.requests_enabled,
                    enabled,
                )],
                summary: format!(
                    "{} request logging.",
                    if *enabled { "Enabling" } else { "Disabling" }
                ),
                requires_confirmation: false,
            })
        }

        ConfigIntent::ShowConfig | ConfigIntent::ShowLogConfig | ConfigIntent::NotConfigRequest => {
            None
        }
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
            // v0.8.0: Log configuration changes
            "log.level" => {
                new_config.log.level = match change.to.as_str() {
                    "trace" => LogLevel::Trace,
                    "debug" => LogLevel::Debug,
                    "info" => LogLevel::Info,
                    "warn" => LogLevel::Warn,
                    "error" => LogLevel::Error,
                    _ => LogLevel::Debug,
                };
            }
            "log.llm_enabled" => {
                new_config.log.llm_enabled = change.to == "true";
            }
            "log.daemon_enabled" => {
                new_config.log.daemon_enabled = change.to == "true";
            }
            "log.requests_enabled" => {
                new_config.log.requests_enabled = change.to == "true";
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
  junior_model: {}
  senior_model: {}
  preferred_model: {}
  fallback_model: {}

[source: config.update]
  enabled: {}
  interval_seconds: {}
  channel: {}

[source: config.log]
  level: {}
  daemon_enabled: {}
  requests_enabled: {}
  llm_enabled: {}
  log_dir: {}
  journal_enabled: {}"#,
        config.core.mode.as_str(),
        config.llm.selection_mode.as_str(),
        config
            .llm
            .junior_model
            .as_deref()
            .unwrap_or("(uses preferred_model)"),
        config
            .llm
            .senior_model
            .as_deref()
            .unwrap_or("(uses preferred_model)"),
        config.llm.preferred_model,
        config.llm.fallback_model,
        config.update.enabled,
        config.update.interval_seconds,
        config.update.channel.as_str(),
        config.log.level.as_str(),
        config.log.daemon_enabled,
        config.log.requests_enabled,
        config.log.llm_enabled,
        config.log.log_dir.display(),
        config.log.journal_enabled
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
            ConfigIntent::EnableDevAutoUpdate {
                interval_seconds: Some(600)
            }
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
        assert!(display.contains("[source: config.log]"));
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

    // v0.8.0: Log configuration tests
    #[test]
    fn test_classify_set_log_level() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("set log level to info");
        assert!(matches!(
            result.intent,
            ConfigIntent::SetLogLevel {
                level: LogLevel::Info
            }
        ));

        let result = matcher.classify("set logging level to debug");
        assert!(matches!(
            result.intent,
            ConfigIntent::SetLogLevel {
                level: LogLevel::Debug
            }
        ));
    }

    #[test]
    fn test_classify_show_log_config() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("show current logging configuration");
        assert_eq!(result.intent, ConfigIntent::ShowLogConfig);

        let result = matcher.classify("what is the log level");
        assert_eq!(result.intent, ConfigIntent::ShowLogConfig);
    }

    #[test]
    fn test_classify_enable_llm_logging() {
        let matcher = ConfigPatternMatcher::new();

        let result = matcher.classify("enable detailed llm logging");
        assert!(matches!(
            result.intent,
            ConfigIntent::SetLlmLogging { enabled: true }
        ));

        let result = matcher.classify("disable llm logging");
        assert!(matches!(
            result.intent,
            ConfigIntent::SetLlmLogging { enabled: false }
        ));
    }

    #[test]
    fn test_apply_set_log_level() {
        let config = AnnaConfigV5::default();
        let intent = ConfigIntent::SetLogLevel {
            level: LogLevel::Info,
        };

        let mutation = apply_intent(&config, &intent).unwrap();
        assert!(!mutation.changes.is_empty());
        assert!(mutation.summary.contains("INFO"));
    }

    #[test]
    fn test_apply_log_mutation() {
        let config = AnnaConfigV5::default();
        let mutation = ConfigMutation {
            changes: vec![
                ConfigChange::new("log.level", "debug", "info"),
                ConfigChange::new("log.llm_enabled", "true", "false"),
            ],
            summary: "Test".to_string(),
            requires_confirmation: false,
        };

        let new_config = apply_mutation(&config, &mutation);
        assert_eq!(new_config.log.level, LogLevel::Info);
        assert!(!new_config.log.llm_enabled);
    }
}

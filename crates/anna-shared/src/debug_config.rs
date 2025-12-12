//! Debug Configuration via Natural Language.
//!
//! Allows users to configure debug settings through natural language:
//! - "enable debug mode"
//! - "set debug level to trace"
//! - "show full debug output"
//! - "disable debug"
//!
//! Per VISION.md: "All settings changeable through annactl in natural language"

use crate::debug_mode::{DebugConfig, DebugLevel};

/// Debug configuration change from natural language
#[derive(Debug, Clone, PartialEq)]
pub enum DebugConfigChange {
    /// Set debug level
    SetLevel(DebugLevel),
    /// Enable debug log to file
    EnableLogFile(bool),
    /// Set redaction for private IPs
    RedactIPs(bool),
    /// Set redaction for emails
    RedactEmails(bool),
    /// Set redaction for secrets
    RedactSecrets(bool),
    /// Set max probe output lines
    MaxProbeLines(usize),
}

impl DebugConfigChange {
    /// Human-readable description
    pub fn description(&self) -> String {
        match self {
            DebugConfigChange::SetLevel(level) => {
                format!("Set debug level to {} ({})", level.name(), level.description())
            }
            DebugConfigChange::EnableLogFile(true) => "Enabled debug log file".to_string(),
            DebugConfigChange::EnableLogFile(false) => "Disabled debug log file".to_string(),
            DebugConfigChange::RedactIPs(true) => "Enabled IP address redaction".to_string(),
            DebugConfigChange::RedactIPs(false) => "Disabled IP address redaction".to_string(),
            DebugConfigChange::RedactEmails(true) => "Enabled email redaction".to_string(),
            DebugConfigChange::RedactEmails(false) => "Disabled email redaction".to_string(),
            DebugConfigChange::RedactSecrets(true) => "Enabled secrets redaction".to_string(),
            DebugConfigChange::RedactSecrets(false) => "Disabled secrets redaction".to_string(),
            DebugConfigChange::MaxProbeLines(n) => {
                format!("Set max probe output lines to {}", n)
            }
        }
    }
}

/// Detect debug configuration changes from natural language
pub fn detect_debug_config(query: &str) -> Option<DebugConfigChange> {
    let lower = query.to_lowercase();

    // Log file changes (check FIRST - more specific patterns)
    if matches_any(&lower, &["log debug to file", "enable debug log", "save debug"]) {
        return Some(DebugConfigChange::EnableLogFile(true));
    }

    if matches_any(&lower, &["no debug log", "disable debug log", "don't log debug", "stop debug log"]) {
        return Some(DebugConfigChange::EnableLogFile(false));
    }

    // Debug level changes (check after log file patterns)
    if matches_any(&lower, &["debug off", "disable debug mode", "no debug", "turn off debug"]) {
        return Some(DebugConfigChange::SetLevel(DebugLevel::Off));
    }

    if matches_any(&lower, &["debug summary", "debug level 1", "enable summary"]) {
        return Some(DebugConfigChange::SetLevel(DebugLevel::Summary));
    }

    if matches_any(&lower, &[
        "debug trace", "debug level 2", "enable trace",
        "enable debug", "turn on debug", "debug mode on"
    ]) {
        return Some(DebugConfigChange::SetLevel(DebugLevel::Trace));
    }

    if matches_any(&lower, &[
        "debug full", "debug level 3", "full debug",
        "verbose debug", "maximum debug", "debug everything"
    ]) {
        return Some(DebugConfigChange::SetLevel(DebugLevel::Full));
    }

    // Redaction changes
    if matches_any(&lower, &["hide ips", "redact ips", "hide ip addresses"]) {
        return Some(DebugConfigChange::RedactIPs(true));
    }

    if matches_any(&lower, &["show ips", "don't redact ips", "show ip addresses"]) {
        return Some(DebugConfigChange::RedactIPs(false));
    }

    if matches_any(&lower, &["hide emails", "redact emails"]) {
        return Some(DebugConfigChange::RedactEmails(true));
    }

    if matches_any(&lower, &["show emails", "don't redact emails"]) {
        return Some(DebugConfigChange::RedactEmails(false));
    }

    if matches_any(&lower, &["hide secrets", "redact secrets", "hide keys"]) {
        return Some(DebugConfigChange::RedactSecrets(true));
    }

    if matches_any(&lower, &["show secrets", "don't redact secrets"]) {
        return Some(DebugConfigChange::RedactSecrets(false));
    }

    None
}

/// Check if query is asking about debug settings
pub fn is_show_debug_settings(query: &str) -> bool {
    let lower = query.to_lowercase();
    matches_any(&lower, &[
        "show debug", "debug settings", "debug status", "what debug",
        "current debug", "debug config", "debug level"
    ])
}

/// Apply debug config change
pub fn apply_debug_change(config: &mut DebugConfig, change: &DebugConfigChange) {
    match change {
        DebugConfigChange::SetLevel(level) => {
            config.level = *level;
        }
        DebugConfigChange::EnableLogFile(enabled) => {
            config.log_to_file = *enabled;
        }
        DebugConfigChange::RedactIPs(enabled) => {
            config.redact.redact_private_ips = *enabled;
        }
        DebugConfigChange::RedactEmails(enabled) => {
            config.redact.redact_emails = *enabled;
        }
        DebugConfigChange::RedactSecrets(enabled) => {
            config.redact.redact_secrets = *enabled;
        }
        DebugConfigChange::MaxProbeLines(lines) => {
            config.redact.max_probe_lines = *lines;
        }
    }
}

/// Format debug settings for display
pub fn format_debug_settings(config: &DebugConfig) -> String {
    let mut lines = vec![
        format!("level             {} ({})", config.level.name(), config.level.description()),
        format!("log_to_file       {}", if config.log_to_file { "enabled" } else { "disabled" }),
    ];

    if config.log_to_file {
        lines.push(format!("log_path          {}", config.log_path));
    }

    lines.push(String::new());
    lines.push("[redaction]".to_string());
    lines.push(format!("  private_ips     {}", if config.redact.redact_private_ips { "hidden" } else { "shown" }));
    lines.push(format!("  emails          {}", if config.redact.redact_emails { "hidden" } else { "shown" }));
    lines.push(format!("  secrets         {}", if config.redact.redact_secrets { "hidden" } else { "shown" }));
    lines.push(format!("  max_probe_lines {}", config.redact.max_probe_lines));

    lines.push(String::new());
    lines.push("Configure via natural language:".to_string());
    lines.push("  \"enable debug\" or \"debug trace\"".to_string());
    lines.push("  \"debug full\" for maximum detail".to_string());
    lines.push("  \"disable debug\" to turn off".to_string());

    lines.join("\n")
}

/// Get debug level from natural language description
pub fn parse_debug_level(s: &str) -> Option<DebugLevel> {
    let lower = s.to_lowercase();

    if matches_any(&lower, &["off", "none", "disabled", "0"]) {
        return Some(DebugLevel::Off);
    }
    if matches_any(&lower, &["summary", "basic", "1"]) {
        return Some(DebugLevel::Summary);
    }
    if matches_any(&lower, &["trace", "debug", "normal", "2"]) {
        return Some(DebugLevel::Trace);
    }
    if matches_any(&lower, &["full", "verbose", "maximum", "all", "3"]) {
        return Some(DebugLevel::Full);
    }

    None
}

fn matches_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_debug_off() {
        let change = detect_debug_config("disable debug mode");
        assert_eq!(change, Some(DebugConfigChange::SetLevel(DebugLevel::Off)));
    }

    #[test]
    fn test_detect_debug_trace() {
        let change = detect_debug_config("enable debug");
        assert_eq!(change, Some(DebugConfigChange::SetLevel(DebugLevel::Trace)));
    }

    #[test]
    fn test_detect_debug_full() {
        let change = detect_debug_config("full debug output");
        assert_eq!(change, Some(DebugConfigChange::SetLevel(DebugLevel::Full)));
    }

    #[test]
    fn test_detect_log_file() {
        let change = detect_debug_config("log debug to file");
        assert_eq!(change, Some(DebugConfigChange::EnableLogFile(true)));

        let change = detect_debug_config("disable debug log");
        assert_eq!(change, Some(DebugConfigChange::EnableLogFile(false)));
    }

    #[test]
    fn test_detect_redaction() {
        let change = detect_debug_config("hide ip addresses");
        assert_eq!(change, Some(DebugConfigChange::RedactIPs(true)));

        let change = detect_debug_config("show emails");
        assert_eq!(change, Some(DebugConfigChange::RedactEmails(false)));
    }

    #[test]
    fn test_is_show_debug() {
        assert!(is_show_debug_settings("show debug settings"));
        assert!(is_show_debug_settings("what is my debug level"));
        assert!(!is_show_debug_settings("how much disk space"));
    }

    #[test]
    fn test_apply_level_change() {
        let mut config = DebugConfig::default();
        assert_eq!(config.level, DebugLevel::Off);

        apply_debug_change(&mut config, &DebugConfigChange::SetLevel(DebugLevel::Trace));
        assert_eq!(config.level, DebugLevel::Trace);
    }

    #[test]
    fn test_apply_redaction_change() {
        let mut config = DebugConfig::default();
        assert!(config.redact.redact_private_ips);

        apply_debug_change(&mut config, &DebugConfigChange::RedactIPs(false));
        assert!(!config.redact.redact_private_ips);
    }

    #[test]
    fn test_format_settings() {
        let config = DebugConfig::default();
        let output = format_debug_settings(&config);
        assert!(output.contains("level"));
        assert!(output.contains("OFF"));
        assert!(output.contains("redaction"));
    }

    #[test]
    fn test_parse_debug_level() {
        assert_eq!(parse_debug_level("off"), Some(DebugLevel::Off));
        assert_eq!(parse_debug_level("trace"), Some(DebugLevel::Trace));
        assert_eq!(parse_debug_level("full"), Some(DebugLevel::Full));
        assert_eq!(parse_debug_level("invalid"), None);
    }

    #[test]
    fn test_description() {
        let change = DebugConfigChange::SetLevel(DebugLevel::Trace);
        let desc = change.description();
        assert!(desc.contains("TRACE"));
    }
}

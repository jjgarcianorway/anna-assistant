//! Debug configuration (v0.0.446).
//!
//! Config file: /etc/anna/debug.toml or merged into main config.
//!
//! Debug levels:
//! - 0 (off):     Normal user output only
//! - 1 (summary): Domain, intent, probes, outcome, reliability score, failure reason
//! - 2 (trace):   Above + probe commands, exit codes, parsed values, LLM tokens, gate report
//! - 3 (full):    Above + full prompts/responses, raw probe output, parser errors

use serde::{Deserialize, Serialize};

/// Debug verbosity level (0-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum DebugLevel {
    /// Level 0: Normal user output only.
    Off = 0,

    /// Level 1: Summary - domain, intent, probes, outcome, reliability, failures.
    Summary = 1,

    /// Level 2: Trace - probe commands/exits/parsed, LLM model/tokens, gate report.
    Trace = 2,

    /// Level 3: Full - raw prompts/responses, raw probe output, parser errors.
    Full = 3,
}

impl DebugLevel {
    /// Create from integer (0-3).
    pub fn from_u8(level: u8) -> Self {
        match level {
            0 => Self::Off,
            1 => Self::Summary,
            2 => Self::Trace,
            3 => Self::Full,
            _ => Self::Off,
        }
    }

    /// Get as integer.
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Check if this level includes summary info.
    pub fn includes_summary(&self) -> bool {
        *self >= Self::Summary
    }

    /// Check if this level includes trace info.
    pub fn includes_trace(&self) -> bool {
        *self >= Self::Trace
    }

    /// Check if this level includes full output.
    pub fn includes_full(&self) -> bool {
        *self >= Self::Full
    }

    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Summary => "SUMMARY",
            Self::Trace => "TRACE",
            Self::Full => "FULL",
        }
    }

    /// Get description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Off => "normal output only",
            Self::Summary => "domain, intent, outcome, failures",
            Self::Trace => "probe details, LLM tokens, gate report",
            Self::Full => "raw prompts, raw outputs, parser errors",
        }
    }
}

impl Default for DebugLevel {
    fn default() -> Self {
        Self::Off
    }
}

impl std::fmt::Display for DebugLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.as_u8(), self.name())
    }
}

impl std::str::FromStr for DebugLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "0" | "off" | "none" => Ok(Self::Off),
            "1" | "summary" => Ok(Self::Summary),
            "2" | "trace" | "debug" => Ok(Self::Trace),
            "3" | "full" | "verbose" => Ok(Self::Full),
            _ => Err(format!(
                "Invalid debug level: '{}' (use 0-3 or off/summary/trace/full)",
                s
            )),
        }
    }
}

/// Redaction configuration for sanitization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactConfig {
    /// Redact private IPs (10.x, 192.168.x, etc.)
    #[serde(default = "default_redact_ips")]
    pub redact_private_ips: bool,

    /// Redact email addresses
    #[serde(default = "default_redact_emails")]
    pub redact_emails: bool,

    /// Redact API keys and tokens (KEY=, TOKEN=, Authorization:)
    #[serde(default = "default_redact_secrets")]
    pub redact_secrets: bool,

    /// Redact SSH keys and shadow file content (always true, not configurable)
    #[serde(skip)]
    pub redact_sensitive_files: bool,

    /// Maximum lines of probe output to show
    #[serde(default = "default_max_probe_lines")]
    pub max_probe_lines: usize,

    /// Maximum characters of LLM output to show
    #[serde(default = "default_max_llm_output")]
    pub max_llm_output_chars: usize,
}

fn default_redact_ips() -> bool {
    true
}

fn default_redact_emails() -> bool {
    true
}

fn default_redact_secrets() -> bool {
    true
}

fn default_max_probe_lines() -> usize {
    50
}

fn default_max_llm_output() -> usize {
    4000
}

impl Default for RedactConfig {
    fn default() -> Self {
        Self {
            redact_private_ips: default_redact_ips(),
            redact_emails: default_redact_emails(),
            redact_secrets: default_redact_secrets(),
            redact_sensitive_files: true, // Always true
            max_probe_lines: default_max_probe_lines(),
            max_llm_output_chars: default_max_llm_output(),
        }
    }
}

/// Full debug configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Debug level (0=off, 1=trace, 2=full)
    #[serde(default)]
    pub level: DebugLevel,

    /// Redaction settings
    #[serde(default)]
    pub redact: RedactConfig,

    /// Store debug output to file (in addition to display)
    #[serde(default)]
    pub log_to_file: bool,

    /// Path to debug log file
    #[serde(default = "default_debug_log_path")]
    pub log_path: String,
}

fn default_debug_log_path() -> String {
    "/var/lib/anna/debug.log".to_string()
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            level: DebugLevel::Off,
            redact: RedactConfig::default(),
            log_to_file: false,
            log_path: default_debug_log_path(),
        }
    }
}

impl DebugConfig {
    /// Create config with specific level.
    pub fn with_level(level: DebugLevel) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    /// Check if debug output should be shown.
    pub fn should_show_debug(&self) -> bool {
        self.level.includes_trace()
    }

    /// Check if full prompts should be shown.
    pub fn should_show_full(&self) -> bool {
        self.level.includes_full()
    }

    /// Load from TOML file.
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read debug config: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse debug config: {}", e))
    }

    /// Try to load from default paths, fall back to defaults.
    pub fn load() -> Self {
        Self::load_from_file("/etc/anna/debug.toml")
            .or_else(|_| Self::load_from_file("/var/lib/anna/debug.toml"))
            .unwrap_or_default()
    }

    /// Generate example config file content.
    pub fn example_toml() -> String {
        r#"# Anna Debug Configuration
# debug.level: 0=OFF, 1=SUMMARY, 2=TRACE, 3=FULL
#
# Level 0 (off):     Normal user output only
# Level 1 (summary): Domain, intent, probes, outcome, reliability, failures
# Level 2 (trace):   Above + probe commands, exit codes, parsed values, LLM tokens, gate report
# Level 3 (full):    Above + full prompts/responses, raw probe output, parser errors

level = "off"

[redact]
redact_private_ips = true
redact_emails = true
redact_secrets = true
max_probe_lines = 50
max_llm_output_chars = 4000

# Optional: log debug output to file
log_to_file = false
log_path = "/var/lib/anna/debug.log"
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_levels() {
        assert_eq!(DebugLevel::Off.as_u8(), 0);
        assert_eq!(DebugLevel::Summary.as_u8(), 1);
        assert_eq!(DebugLevel::Trace.as_u8(), 2);
        assert_eq!(DebugLevel::Full.as_u8(), 3);

        assert!(!DebugLevel::Off.includes_summary());
        assert!(DebugLevel::Summary.includes_summary());
        assert!(DebugLevel::Trace.includes_summary());
        assert!(DebugLevel::Full.includes_summary());

        assert!(!DebugLevel::Off.includes_trace());
        assert!(!DebugLevel::Summary.includes_trace());
        assert!(DebugLevel::Trace.includes_trace());
        assert!(DebugLevel::Full.includes_trace());

        assert!(!DebugLevel::Off.includes_full());
        assert!(!DebugLevel::Summary.includes_full());
        assert!(!DebugLevel::Trace.includes_full());
        assert!(DebugLevel::Full.includes_full());
    }

    #[test]
    fn test_debug_level_from_str() {
        assert_eq!("0".parse::<DebugLevel>().unwrap(), DebugLevel::Off);
        assert_eq!("off".parse::<DebugLevel>().unwrap(), DebugLevel::Off);
        assert_eq!("1".parse::<DebugLevel>().unwrap(), DebugLevel::Summary);
        assert_eq!("summary".parse::<DebugLevel>().unwrap(), DebugLevel::Summary);
        assert_eq!("2".parse::<DebugLevel>().unwrap(), DebugLevel::Trace);
        assert_eq!("trace".parse::<DebugLevel>().unwrap(), DebugLevel::Trace);
        assert_eq!("3".parse::<DebugLevel>().unwrap(), DebugLevel::Full);
        assert_eq!("full".parse::<DebugLevel>().unwrap(), DebugLevel::Full);
    }

    #[test]
    fn test_debug_level_ordering() {
        assert!(DebugLevel::Off < DebugLevel::Summary);
        assert!(DebugLevel::Summary < DebugLevel::Trace);
        assert!(DebugLevel::Trace < DebugLevel::Full);
    }

    #[test]
    fn test_debug_level_display() {
        assert_eq!(format!("{}", DebugLevel::Off), "0 (OFF)");
        assert_eq!(format!("{}", DebugLevel::Summary), "1 (SUMMARY)");
        assert_eq!(format!("{}", DebugLevel::Trace), "2 (TRACE)");
        assert_eq!(format!("{}", DebugLevel::Full), "3 (FULL)");
    }

    #[test]
    fn test_config_defaults() {
        let config = DebugConfig::default();
        assert_eq!(config.level, DebugLevel::Off);
        assert!(config.redact.redact_private_ips);
        assert!(config.redact.redact_emails);
        assert!(config.redact.redact_secrets);
    }

    #[test]
    fn test_parse_toml() {
        let toml = r#"
level = "trace"

[redact]
redact_private_ips = false
max_probe_lines = 100
"#;
        let config: DebugConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.level, DebugLevel::Trace);
        assert!(!config.redact.redact_private_ips);
        assert_eq!(config.redact.max_probe_lines, 100);
    }
}

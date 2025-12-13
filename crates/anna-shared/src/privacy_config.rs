// v0.0.550: Privacy Config (Phase 126)
// Configurable privacy settings per VISION.md

use serde::{Deserialize, Serialize};

/// Data collection level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DataCollectionLevel {
    None,
    Minimal,
    #[default]
    Standard,
    Full,
}

impl std::fmt::Display for DataCollectionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None (no data stored)"),
            Self::Minimal => write!(f, "Minimal (errors only)"),
            Self::Standard => write!(f, "Standard"),
            Self::Full => write!(f, "Full (detailed telemetry)"),
        }
    }
}

/// Log retention policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LogRetention {
    Session,
    Day,
    #[default]
    Week,
    Month,
    Forever,
}

impl std::fmt::Display for LogRetention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Session => write!(f, "Session only"),
            Self::Day => write!(f, "1 day"),
            Self::Week => write!(f, "1 week"),
            Self::Month => write!(f, "1 month"),
            Self::Forever => write!(f, "Forever"),
        }
    }
}

/// Sensitive data handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SensitiveDataHandling {
    Redact,
    #[default]
    Mask,
    Hash,
    Allow,
}

impl std::fmt::Display for SensitiveDataHandling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Redact => write!(f, "Redact (remove)"),
            Self::Mask => write!(f, "Mask (****)"),
            Self::Hash => write!(f, "Hash (one-way)"),
            Self::Allow => write!(f, "Allow (keep)"),
        }
    }
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub data_collection: DataCollectionLevel,
    pub log_retention: LogRetention,
    pub sensitive_handling: SensitiveDataHandling,
    pub store_query_history: bool,
    pub store_command_history: bool,
    pub store_file_paths: bool,
    pub anonymize_usernames: bool,
    pub anonymize_hostnames: bool,
    pub allow_telemetry: bool,
    pub allow_crash_reports: bool,
    pub clear_on_exit: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            data_collection: DataCollectionLevel::Standard,
            log_retention: LogRetention::Week,
            sensitive_handling: SensitiveDataHandling::Mask,
            store_query_history: true,
            store_command_history: true,
            store_file_paths: true,
            anonymize_usernames: false,
            anonymize_hostnames: false,
            allow_telemetry: false,
            allow_crash_reports: true,
            clear_on_exit: false,
        }
    }
}

impl PrivacyConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Maximum privacy preset
    pub fn maximum() -> Self {
        Self {
            data_collection: DataCollectionLevel::None,
            log_retention: LogRetention::Session,
            sensitive_handling: SensitiveDataHandling::Redact,
            store_query_history: false,
            store_command_history: false,
            store_file_paths: false,
            anonymize_usernames: true,
            anonymize_hostnames: true,
            allow_telemetry: false,
            allow_crash_reports: false,
            clear_on_exit: true,
        }
    }

    /// Balanced privacy preset
    pub fn balanced() -> Self {
        Self {
            data_collection: DataCollectionLevel::Minimal,
            log_retention: LogRetention::Day,
            sensitive_handling: SensitiveDataHandling::Mask,
            store_query_history: true,
            store_command_history: false,
            store_file_paths: false,
            anonymize_usernames: true,
            anonymize_hostnames: false,
            allow_telemetry: false,
            allow_crash_reports: true,
            clear_on_exit: false,
        }
    }

    /// Convenience preset (less privacy)
    pub fn convenience() -> Self {
        Self {
            data_collection: DataCollectionLevel::Full,
            log_retention: LogRetention::Month,
            sensitive_handling: SensitiveDataHandling::Allow,
            store_query_history: true,
            store_command_history: true,
            store_file_paths: true,
            anonymize_usernames: false,
            anonymize_hostnames: false,
            allow_telemetry: true,
            allow_crash_reports: true,
            clear_on_exit: false,
        }
    }

    /// Is privacy maximized?
    pub fn is_maximum_privacy(&self) -> bool {
        self.data_collection == DataCollectionLevel::None
    }

    /// Should store history?
    pub fn should_store_history(&self) -> bool {
        self.store_query_history || self.store_command_history
    }

    /// Should anonymize data?
    pub fn should_anonymize(&self) -> bool {
        self.anonymize_usernames || self.anonymize_hostnames
    }

    /// Get retention days (0 = session only, u32::MAX = forever)
    pub fn retention_days(&self) -> u32 {
        match self.log_retention {
            LogRetention::Session => 0,
            LogRetention::Day => 1,
            LogRetention::Week => 7,
            LogRetention::Month => 30,
            LogRetention::Forever => u32::MAX,
        }
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Preset changes
        if lower.contains("maximum privacy") || lower.contains("most private") || lower.contains("paranoid") {
            *self = Self::maximum();
            return Some("Maximum privacy enabled - no data stored.".to_string());
        }
        if lower.contains("balanced privacy") || lower.contains("moderate privacy") {
            *self = Self::balanced();
            return Some("Balanced privacy settings applied.".to_string());
        }
        if lower.contains("convenience") || lower.contains("remember everything") {
            *self = Self::convenience();
            return Some("Convenience mode - full history stored.".to_string());
        }

        // Individual toggles
        if lower.contains("store history") || lower.contains("remember queries") {
            self.store_query_history = true;
            self.store_command_history = true;
            return Some("History will be stored.".to_string());
        }
        if lower.contains("don't store") || lower.contains("no history") || lower.contains("forget") {
            self.store_query_history = false;
            self.store_command_history = false;
            return Some("History will not be stored.".to_string());
        }
        if lower.contains("anonymize") || lower.contains("hide my name") {
            self.anonymize_usernames = true;
            self.anonymize_hostnames = true;
            return Some("Usernames and hostnames will be anonymized.".to_string());
        }
        if lower.contains("clear on exit") || lower.contains("delete on close") {
            self.clear_on_exit = true;
            return Some("Data will be cleared on exit.".to_string());
        }
        if lower.contains("keep data") || lower.contains("persist") {
            self.clear_on_exit = false;
            return Some("Data will persist between sessions.".to_string());
        }
        if lower.contains("allow telemetry") || lower.contains("enable telemetry") {
            self.allow_telemetry = true;
            return Some("Telemetry enabled.".to_string());
        }
        if lower.contains("disable telemetry") || lower.contains("no telemetry") {
            self.allow_telemetry = false;
            return Some("Telemetry disabled.".to_string());
        }

        None
    }
}

/// Format privacy config
pub fn format_privacy_config(config: &PrivacyConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Privacy Configuration ===\n\n");

    output.push_str(&format!("Data Collection: {}\n", config.data_collection));
    output.push_str(&format!("Log Retention: {}\n", config.log_retention));
    output.push_str(&format!("Sensitive Data: {}\n", config.sensitive_handling));
    output.push_str(&format!("Store Query History: {}\n", config.store_query_history));
    output.push_str(&format!("Store Command History: {}\n", config.store_command_history));
    output.push_str(&format!("Store File Paths: {}\n", config.store_file_paths));
    output.push_str(&format!("Anonymize Usernames: {}\n", config.anonymize_usernames));
    output.push_str(&format!("Anonymize Hostnames: {}\n", config.anonymize_hostnames));
    output.push_str(&format!("Allow Telemetry: {}\n", config.allow_telemetry));
    output.push_str(&format!("Allow Crash Reports: {}\n", config.allow_crash_reports));
    output.push_str(&format!("Clear on Exit: {}\n", config.clear_on_exit));

    output
}

/// Check if query is privacy-related
pub fn is_privacy_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("privacy")
        || lower.contains("data collection")
        || lower.contains("telemetry")
        || lower.contains("anonymize")
        || lower.contains("history storage")
}

/// Fun fact about privacy
pub fn privacy_fun_fact() -> &'static str {
    "The first data protection law was passed in Hesse, Germany in 1970 - privacy has been a concern for over 50 years!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_collection_display() {
        assert_eq!(format!("{}", DataCollectionLevel::None), "None (no data stored)");
        assert_eq!(format!("{}", DataCollectionLevel::Full), "Full (detailed telemetry)");
    }

    #[test]
    fn test_default_config() {
        let config = PrivacyConfig::default();
        assert_eq!(config.data_collection, DataCollectionLevel::Standard);
        assert!(config.store_query_history);
    }

    #[test]
    fn test_maximum_preset() {
        let config = PrivacyConfig::maximum();
        assert!(config.is_maximum_privacy());
        assert!(!config.store_query_history);
        assert!(config.clear_on_exit);
    }

    #[test]
    fn test_balanced_preset() {
        let config = PrivacyConfig::balanced();
        assert_eq!(config.data_collection, DataCollectionLevel::Minimal);
        assert!(config.anonymize_usernames);
    }

    #[test]
    fn test_convenience_preset() {
        let config = PrivacyConfig::convenience();
        assert_eq!(config.data_collection, DataCollectionLevel::Full);
        assert!(config.allow_telemetry);
    }

    #[test]
    fn test_retention_days() {
        let config = PrivacyConfig::default();
        assert_eq!(config.retention_days(), 7);
        let max = PrivacyConfig::maximum();
        assert_eq!(max.retention_days(), 0);
    }

    #[test]
    fn test_should_store_history() {
        let config = PrivacyConfig::default();
        assert!(config.should_store_history());
        let max = PrivacyConfig::maximum();
        assert!(!max.should_store_history());
    }

    #[test]
    fn test_apply_maximum() {
        let mut config = PrivacyConfig::default();
        let result = config.apply_change("use maximum privacy");
        assert!(result.is_some());
        assert!(config.is_maximum_privacy());
    }

    #[test]
    fn test_apply_anonymize() {
        let mut config = PrivacyConfig::default();
        config.apply_change("anonymize my data");
        assert!(config.anonymize_usernames);
    }

    #[test]
    fn test_is_privacy_query() {
        assert!(is_privacy_query("Show privacy settings"));
        assert!(is_privacy_query("Is telemetry enabled?"));
        assert!(!is_privacy_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = privacy_fun_fact();
        assert!(fact.contains("1970"));
    }
}

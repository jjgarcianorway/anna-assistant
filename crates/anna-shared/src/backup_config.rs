// v0.0.551: Backup Config (Phase 127)
// Configurable backup settings per VISION.md

use serde::{Deserialize, Serialize};

/// Backup frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BackupFrequency {
    Manual,
    Hourly,
    #[default]
    Daily,
    Weekly,
    Monthly,
}

impl std::fmt::Display for BackupFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "Manual only"),
            Self::Hourly => write!(f, "Hourly"),
            Self::Daily => write!(f, "Daily"),
            Self::Weekly => write!(f, "Weekly"),
            Self::Monthly => write!(f, "Monthly"),
        }
    }
}

/// Backup type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BackupType {
    #[default]
    Incremental,
    Full,
    Differential,
    Snapshot,
}

impl std::fmt::Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incremental => write!(f, "Incremental"),
            Self::Full => write!(f, "Full"),
            Self::Differential => write!(f, "Differential"),
            Self::Snapshot => write!(f, "Snapshot"),
        }
    }
}

/// Backup target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BackupTarget {
    #[default]
    Local,
    Network,
    Cloud,
    External,
}

impl std::fmt::Display for BackupTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Network => write!(f, "Network"),
            Self::Cloud => write!(f, "Cloud"),
            Self::External => write!(f, "External Drive"),
        }
    }
}

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CompressionLevel {
    None,
    Fast,
    #[default]
    Balanced,
    Maximum,
}

impl std::fmt::Display for CompressionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Fast => write!(f, "Fast"),
            Self::Balanced => write!(f, "Balanced"),
            Self::Maximum => write!(f, "Maximum"),
        }
    }
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub frequency: BackupFrequency,
    pub backup_type: BackupType,
    pub target: BackupTarget,
    pub compression: CompressionLevel,
    pub encrypt_backups: bool,
    pub verify_after_backup: bool,
    pub keep_versions: u32,
    pub backup_configs: bool,
    pub backup_recipes: bool,
    pub backup_history: bool,
    pub notify_on_complete: bool,
    pub notify_on_failure: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            frequency: BackupFrequency::Daily,
            backup_type: BackupType::Incremental,
            target: BackupTarget::Local,
            compression: CompressionLevel::Balanced,
            encrypt_backups: false,
            verify_after_backup: true,
            keep_versions: 7,
            backup_configs: true,
            backup_recipes: true,
            backup_history: true,
            notify_on_complete: false,
            notify_on_failure: true,
        }
    }
}

impl BackupConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Minimal backup preset
    pub fn minimal() -> Self {
        Self {
            frequency: BackupFrequency::Weekly,
            backup_type: BackupType::Incremental,
            target: BackupTarget::Local,
            compression: CompressionLevel::Fast,
            encrypt_backups: false,
            verify_after_backup: false,
            keep_versions: 3,
            backup_configs: true,
            backup_recipes: false,
            backup_history: false,
            notify_on_complete: false,
            notify_on_failure: true,
        }
    }

    /// Comprehensive backup preset
    pub fn comprehensive() -> Self {
        Self {
            frequency: BackupFrequency::Daily,
            backup_type: BackupType::Full,
            target: BackupTarget::Local,
            compression: CompressionLevel::Maximum,
            encrypt_backups: true,
            verify_after_backup: true,
            keep_versions: 30,
            backup_configs: true,
            backup_recipes: true,
            backup_history: true,
            notify_on_complete: true,
            notify_on_failure: true,
        }
    }

    /// Manual backup preset
    pub fn manual_only() -> Self {
        Self {
            frequency: BackupFrequency::Manual,
            backup_type: BackupType::Full,
            target: BackupTarget::Local,
            compression: CompressionLevel::Balanced,
            encrypt_backups: false,
            verify_after_backup: true,
            keep_versions: 5,
            backup_configs: true,
            backup_recipes: true,
            backup_history: true,
            notify_on_complete: false,
            notify_on_failure: false,
        }
    }

    /// Is automatic backup enabled?
    pub fn is_automatic(&self) -> bool {
        self.frequency != BackupFrequency::Manual
    }

    /// Get backup interval in hours
    pub fn interval_hours(&self) -> Option<u32> {
        match self.frequency {
            BackupFrequency::Manual => None,
            BackupFrequency::Hourly => Some(1),
            BackupFrequency::Daily => Some(24),
            BackupFrequency::Weekly => Some(168),
            BackupFrequency::Monthly => Some(720),
        }
    }

    /// Should encrypt?
    pub fn should_encrypt(&self) -> bool {
        self.encrypt_backups
    }

    /// Should verify?
    pub fn should_verify(&self) -> bool {
        self.verify_after_backup
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Preset changes
        if lower.contains("minimal backup") || lower.contains("simple backup") {
            *self = Self::minimal();
            return Some("Minimal backup settings applied.".to_string());
        }
        if lower.contains("comprehensive") || lower.contains("full backup") {
            *self = Self::comprehensive();
            return Some("Comprehensive backup settings applied.".to_string());
        }
        if lower.contains("manual backup") || lower.contains("no automatic") {
            *self = Self::manual_only();
            return Some("Manual backup mode - no automatic backups.".to_string());
        }

        // Frequency changes
        if lower.contains("backup hourly") || lower.contains("every hour") {
            self.frequency = BackupFrequency::Hourly;
            return Some("Backups will run hourly.".to_string());
        }
        if lower.contains("backup daily") || lower.contains("every day") {
            self.frequency = BackupFrequency::Daily;
            return Some("Backups will run daily.".to_string());
        }
        if lower.contains("backup weekly") || lower.contains("every week") {
            self.frequency = BackupFrequency::Weekly;
            return Some("Backups will run weekly.".to_string());
        }

        // Feature toggles
        if lower.contains("encrypt") && lower.contains("backup") || lower.contains("secure backup") {
            self.encrypt_backups = true;
            return Some("Backup encryption enabled.".to_string());
        }
        if lower.contains("no encrypt") || lower.contains("unencrypted") {
            self.encrypt_backups = false;
            return Some("Backup encryption disabled.".to_string());
        }
        if lower.contains("verify backup") || lower.contains("check backup") {
            self.verify_after_backup = true;
            return Some("Backup verification enabled.".to_string());
        }
        if lower.contains("skip verify") || lower.contains("no verify") {
            self.verify_after_backup = false;
            return Some("Backup verification disabled.".to_string());
        }
        if lower.contains("notify complete") || lower.contains("tell me when") {
            self.notify_on_complete = true;
            return Some("You'll be notified when backups complete.".to_string());
        }

        None
    }
}

/// Format backup config
pub fn format_backup_config(config: &BackupConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Backup Configuration ===\n\n");

    output.push_str(&format!("Frequency: {}\n", config.frequency));
    output.push_str(&format!("Backup Type: {}\n", config.backup_type));
    output.push_str(&format!("Target: {}\n", config.target));
    output.push_str(&format!("Compression: {}\n", config.compression));
    output.push_str(&format!("Encrypt: {}\n", config.encrypt_backups));
    output.push_str(&format!("Verify: {}\n", config.verify_after_backup));
    output.push_str(&format!("Keep Versions: {}\n", config.keep_versions));
    output.push_str(&format!("Backup Configs: {}\n", config.backup_configs));
    output.push_str(&format!("Backup Recipes: {}\n", config.backup_recipes));
    output.push_str(&format!("Backup History: {}\n", config.backup_history));
    output.push_str(&format!("Notify Complete: {}\n", config.notify_on_complete));
    output.push_str(&format!("Notify Failure: {}\n", config.notify_on_failure));

    output
}

/// Check if query is backup-related
pub fn is_backup_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("backup")
        || lower.contains("restore")
        || lower.contains("snapshot")
        || lower.contains("recovery")
}

/// Fun fact about backups
pub fn backup_fun_fact() -> &'static str {
    "World Backup Day is March 31st - the day before April Fools' Day, because losing data is no joke!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_display() {
        assert_eq!(format!("{}", BackupFrequency::Daily), "Daily");
        assert_eq!(format!("{}", BackupFrequency::Manual), "Manual only");
    }

    #[test]
    fn test_default_config() {
        let config = BackupConfig::default();
        assert_eq!(config.frequency, BackupFrequency::Daily);
        assert!(config.verify_after_backup);
    }

    #[test]
    fn test_minimal_preset() {
        let config = BackupConfig::minimal();
        assert_eq!(config.frequency, BackupFrequency::Weekly);
        assert_eq!(config.keep_versions, 3);
    }

    #[test]
    fn test_comprehensive_preset() {
        let config = BackupConfig::comprehensive();
        assert!(config.encrypt_backups);
        assert_eq!(config.keep_versions, 30);
    }

    #[test]
    fn test_manual_only_preset() {
        let config = BackupConfig::manual_only();
        assert!(!config.is_automatic());
    }

    #[test]
    fn test_is_automatic() {
        let config = BackupConfig::default();
        assert!(config.is_automatic());
        let manual = BackupConfig::manual_only();
        assert!(!manual.is_automatic());
    }

    #[test]
    fn test_interval_hours() {
        let config = BackupConfig::default();
        assert_eq!(config.interval_hours(), Some(24));
        let manual = BackupConfig::manual_only();
        assert_eq!(manual.interval_hours(), None);
    }

    #[test]
    fn test_apply_comprehensive() {
        let mut config = BackupConfig::default();
        let result = config.apply_change("use comprehensive backup");
        assert!(result.is_some());
        assert!(config.encrypt_backups);
    }

    #[test]
    fn test_apply_encrypt() {
        let mut config = BackupConfig::default();
        config.apply_change("encrypt my backups");
        assert!(config.should_encrypt());
    }

    #[test]
    fn test_is_backup_query() {
        assert!(is_backup_query("Configure backups"));
        assert!(is_backup_query("How to restore?"));
        assert!(!is_backup_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = backup_fun_fact();
        assert!(fact.contains("March 31"));
    }
}

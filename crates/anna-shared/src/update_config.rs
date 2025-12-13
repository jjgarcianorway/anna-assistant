// v0.0.552: Update Config (Phase 128)
// Configurable update settings per VISION.md

use serde::{Deserialize, Serialize};

/// Update check frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum UpdateCheckFrequency {
    Never,
    Daily,
    #[default]
    Startup,
    Hourly,
    Manual,
}

impl std::fmt::Display for UpdateCheckFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Never => write!(f, "Never"),
            Self::Daily => write!(f, "Daily"),
            Self::Startup => write!(f, "On Startup"),
            Self::Hourly => write!(f, "Hourly"),
            Self::Manual => write!(f, "Manual Only"),
        }
    }
}

/// Update channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum UpdateChannel {
    #[default]
    Stable,
    Beta,
    Nightly,
}

impl std::fmt::Display for UpdateChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "Stable"),
            Self::Beta => write!(f, "Beta"),
            Self::Nightly => write!(f, "Nightly"),
        }
    }
}

/// Update action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum UpdateAction {
    #[default]
    Notify,
    Download,
    AutoInstall,
}

impl std::fmt::Display for UpdateAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Notify => write!(f, "Notify only"),
            Self::Download => write!(f, "Download (manual install)"),
            Self::AutoInstall => write!(f, "Auto-install"),
        }
    }
}

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub check_frequency: UpdateCheckFrequency,
    pub channel: UpdateChannel,
    pub action: UpdateAction,
    pub verify_checksum: bool,
    pub backup_before_update: bool,
    pub notify_available: bool,
    pub notify_installed: bool,
    pub show_changelog: bool,
    pub skip_major_versions: bool,
    pub update_timeout_seconds: u64,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_frequency: UpdateCheckFrequency::Startup,
            channel: UpdateChannel::Stable,
            action: UpdateAction::Notify,
            verify_checksum: true,
            backup_before_update: true,
            notify_available: true,
            notify_installed: true,
            show_changelog: true,
            skip_major_versions: true,
            update_timeout_seconds: 60,
        }
    }
}

impl UpdateConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Conservative preset - minimal updates
    pub fn conservative() -> Self {
        Self {
            check_frequency: UpdateCheckFrequency::Manual,
            channel: UpdateChannel::Stable,
            action: UpdateAction::Notify,
            verify_checksum: true,
            backup_before_update: true,
            notify_available: true,
            notify_installed: true,
            show_changelog: true,
            skip_major_versions: true,
            update_timeout_seconds: 120,
        }
    }

    /// Automatic preset - auto-update enabled
    pub fn automatic() -> Self {
        Self {
            check_frequency: UpdateCheckFrequency::Hourly,
            channel: UpdateChannel::Stable,
            action: UpdateAction::AutoInstall,
            verify_checksum: true,
            backup_before_update: true,
            notify_available: false,
            notify_installed: true,
            show_changelog: false,
            skip_major_versions: true,
            update_timeout_seconds: 60,
        }
    }

    /// Bleeding edge preset - nightly with auto-install
    pub fn bleeding_edge() -> Self {
        Self {
            check_frequency: UpdateCheckFrequency::Hourly,
            channel: UpdateChannel::Nightly,
            action: UpdateAction::AutoInstall,
            verify_checksum: true,
            backup_before_update: false,
            notify_available: false,
            notify_installed: true,
            show_changelog: true,
            skip_major_versions: false,
            update_timeout_seconds: 30,
        }
    }

    /// Is auto-update enabled?
    pub fn is_auto_update(&self) -> bool {
        self.action == UpdateAction::AutoInstall
    }

    /// Is automatic checking enabled?
    pub fn is_auto_check(&self) -> bool {
        !matches!(
            self.check_frequency,
            UpdateCheckFrequency::Never | UpdateCheckFrequency::Manual
        )
    }

    /// Get check interval in seconds
    pub fn check_interval_seconds(&self) -> Option<u64> {
        match self.check_frequency {
            UpdateCheckFrequency::Never | UpdateCheckFrequency::Manual => None,
            UpdateCheckFrequency::Startup => Some(0), // Special case
            UpdateCheckFrequency::Hourly => Some(3600),
            UpdateCheckFrequency::Daily => Some(86400),
        }
    }

    /// Should notify about available updates?
    pub fn should_notify_available(&self) -> bool {
        self.notify_available && !self.is_auto_update()
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Preset changes
        if lower.contains("conservative") || lower.contains("manual update") {
            *self = Self::conservative();
            return Some("Conservative update mode - manual updates only.".to_string());
        }
        if lower.contains("automatic update") || lower.contains("auto update") || lower.contains("auto-update") {
            *self = Self::automatic();
            return Some("Automatic updates enabled.".to_string());
        }
        if lower.contains("bleeding edge") || lower.contains("nightly") || lower.contains("latest") {
            *self = Self::bleeding_edge();
            return Some("Bleeding edge mode - nightly updates.".to_string());
        }

        // Channel changes
        if lower.contains("stable channel") || lower.contains("stable version") {
            self.channel = UpdateChannel::Stable;
            return Some("Update channel set to stable.".to_string());
        }
        if lower.contains("beta channel") || lower.contains("beta version") {
            self.channel = UpdateChannel::Beta;
            return Some("Update channel set to beta.".to_string());
        }

        // Frequency changes
        if lower.contains("check hourly") || lower.contains("every hour") {
            self.check_frequency = UpdateCheckFrequency::Hourly;
            return Some("Will check for updates hourly.".to_string());
        }
        if lower.contains("check daily") || lower.contains("every day") {
            self.check_frequency = UpdateCheckFrequency::Daily;
            return Some("Will check for updates daily.".to_string());
        }
        if lower.contains("check on startup") || lower.contains("when starting") {
            self.check_frequency = UpdateCheckFrequency::Startup;
            return Some("Will check for updates on startup.".to_string());
        }
        if lower.contains("never check") || lower.contains("disable check") {
            self.check_frequency = UpdateCheckFrequency::Never;
            return Some("Update checks disabled.".to_string());
        }

        // Feature toggles
        if lower.contains("notify me") || lower.contains("tell me about update") {
            self.notify_available = true;
            return Some("You'll be notified about available updates.".to_string());
        }
        if lower.contains("silent update") || lower.contains("don't notify") {
            self.notify_available = false;
            return Some("Update notifications disabled.".to_string());
        }
        if lower.contains("show changelog") || lower.contains("what's new") {
            self.show_changelog = true;
            return Some("Changelog will be shown after updates.".to_string());
        }

        None
    }
}

/// Format update config
pub fn format_update_config(config: &UpdateConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Update Configuration ===\n\n");

    output.push_str(&format!("Check Frequency: {}\n", config.check_frequency));
    output.push_str(&format!("Channel: {}\n", config.channel));
    output.push_str(&format!("Action: {}\n", config.action));
    output.push_str(&format!("Verify Checksum: {}\n", config.verify_checksum));
    output.push_str(&format!("Backup Before Update: {}\n", config.backup_before_update));
    output.push_str(&format!("Notify Available: {}\n", config.notify_available));
    output.push_str(&format!("Notify Installed: {}\n", config.notify_installed));
    output.push_str(&format!("Show Changelog: {}\n", config.show_changelog));
    output.push_str(&format!("Skip Major Versions: {}\n", config.skip_major_versions));
    output.push_str(&format!("Timeout: {}s\n", config.update_timeout_seconds));

    output
}

/// Check if query is update-related
pub fn is_update_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("update")
        || lower.contains("upgrade")
        || lower.contains("new version")
        || lower.contains("changelog")
}

/// Fun fact about updates
pub fn update_fun_fact() -> &'static str {
    "The first software update over the internet was likely in 1989 when NeXT released patches via FTP!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_display() {
        assert_eq!(format!("{}", UpdateCheckFrequency::Startup), "On Startup");
        assert_eq!(format!("{}", UpdateCheckFrequency::Never), "Never");
    }

    #[test]
    fn test_default_config() {
        let config = UpdateConfig::default();
        assert_eq!(config.check_frequency, UpdateCheckFrequency::Startup);
        assert_eq!(config.channel, UpdateChannel::Stable);
    }

    #[test]
    fn test_conservative_preset() {
        let config = UpdateConfig::conservative();
        assert_eq!(config.check_frequency, UpdateCheckFrequency::Manual);
        assert!(!config.is_auto_update());
    }

    #[test]
    fn test_automatic_preset() {
        let config = UpdateConfig::automatic();
        assert!(config.is_auto_update());
        assert_eq!(config.action, UpdateAction::AutoInstall);
    }

    #[test]
    fn test_bleeding_edge_preset() {
        let config = UpdateConfig::bleeding_edge();
        assert_eq!(config.channel, UpdateChannel::Nightly);
        assert!(!config.skip_major_versions);
    }

    #[test]
    fn test_is_auto_check() {
        let config = UpdateConfig::default();
        assert!(config.is_auto_check());
        let conservative = UpdateConfig::conservative();
        assert!(!conservative.is_auto_check());
    }

    #[test]
    fn test_check_interval() {
        let config = UpdateConfig::automatic();
        assert_eq!(config.check_interval_seconds(), Some(3600));
        let manual = UpdateConfig::conservative();
        assert_eq!(manual.check_interval_seconds(), None);
    }

    #[test]
    fn test_apply_automatic() {
        let mut config = UpdateConfig::default();
        let result = config.apply_change("enable automatic updates");
        assert!(result.is_some());
        assert!(config.is_auto_update());
    }

    #[test]
    fn test_apply_stable_channel() {
        let mut config = UpdateConfig::bleeding_edge();
        config.apply_change("use stable channel");
        assert_eq!(config.channel, UpdateChannel::Stable);
    }

    #[test]
    fn test_is_update_query() {
        assert!(is_update_query("Check for updates"));
        assert!(is_update_query("Show changelog"));
        assert!(!is_update_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = update_fun_fact();
        assert!(fact.contains("1989"));
    }
}

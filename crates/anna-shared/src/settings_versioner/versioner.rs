// v0.0.660: Settings Versioner - Main Versioner
// The main SettingsVersioner implementation

use super::version_types::BumpType;
use super::version_config::VersionerConfig;
use super::version_core::{SettingsVersion, VersionResult, VersionerStats};

/// Settings versioner
#[derive(Debug, Clone, Default)]
pub struct SettingsVersioner {
    /// Config
    config: VersionerConfig,
    /// Current version
    current: SettingsVersion,
    /// History
    history: Vec<SettingsVersion>,
    /// Stats
    stats: VersionerStats,
}

impl SettingsVersioner {
    /// Create new versioner
    pub fn new(config: VersionerConfig) -> Self {
        Self {
            config,
            current: SettingsVersion::default(),
            history: Vec::new(),
            stats: VersionerStats::default(),
        }
    }

    /// Get current version
    pub fn current(&self) -> &SettingsVersion {
        &self.current
    }

    /// Bump version
    pub fn bump(&mut self, bump_type: BumpType) -> VersionResult {
        let previous = self.current.clone();
        let new_version = self.current.bump(bump_type);

        if self.config.track_history {
            self.history.push(previous.clone());
            if self.history.len() > self.config.max_history {
                self.history.remove(0);
            }
        }

        self.current = new_version.clone();
        self.stats.record(bump_type, &self.current.version);

        VersionResult::new(new_version, bump_type).with_previous(previous)
    }

    /// Bump with description
    pub fn bump_with_description(&mut self, bump_type: BumpType, description: &str) -> VersionResult {
        let mut result = self.bump(bump_type);
        self.current.description = Some(description.to_string());
        result.current.description = Some(description.to_string());
        result
    }

    /// Set version
    pub fn set_version(&mut self, version: SettingsVersion) {
        if self.config.track_history {
            self.history.push(self.current.clone());
        }
        self.current = version;
    }

    /// Get history
    pub fn history(&self) -> &[SettingsVersion] {
        &self.history
    }

    /// Get stats
    pub fn stats(&self) -> &VersionerStats {
        &self.stats
    }

    /// History count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

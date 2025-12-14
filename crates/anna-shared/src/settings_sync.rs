// v0.0.564: Settings Sync (Phase 140)
// Handles synchronizing settings across multiple instances

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::settings_persistence::{SettingsError, SettingsResult};
use crate::unified_settings::UnifiedSettings;

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Settings are in sync
    #[default]
    Synced,
    /// Local changes pending upload
    LocalAhead,
    /// Remote changes available
    RemoteAhead,
    /// Conflict between local and remote
    Conflict,
    /// Sync not configured
    NotConfigured,
    /// Sync in progress
    InProgress,
    /// Sync failed
    Failed,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Synced => write!(f, "Synced"),
            Self::LocalAhead => write!(f, "Local changes pending"),
            Self::RemoteAhead => write!(f, "Updates available"),
            Self::Conflict => write!(f, "Conflict"),
            Self::NotConfigured => write!(f, "Not configured"),
            Self::InProgress => write!(f, "Syncing..."),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// Sync provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SyncProvider {
    /// No sync provider
    #[default]
    None,
    /// File-based sync (e.g., Dropbox, Syncthing folder)
    File,
    /// Git repository
    Git,
    /// Custom URL endpoint
    Custom,
}

impl std::fmt::Display for SyncProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::File => write!(f, "File"),
            Self::Git => write!(f, "Git"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Keep local changes
    KeepLocal,
    /// Accept remote changes
    #[default]
    AcceptRemote,
    /// Merge (remote wins on conflict)
    MergeRemoteWins,
    /// Merge (local wins on conflict)
    MergeLocalWins,
    /// Ask user
    Ask,
}

impl std::fmt::Display for ConflictResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeepLocal => write!(f, "Keep local"),
            Self::AcceptRemote => write!(f, "Accept remote"),
            Self::MergeRemoteWins => write!(f, "Merge (remote wins)"),
            Self::MergeLocalWins => write!(f, "Merge (local wins)"),
            Self::Ask => write!(f, "Ask"),
        }
    }
}

/// Sync configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync provider
    pub provider: SyncProvider,
    /// Sync path/URL
    pub location: Option<String>,
    /// Auto-sync enabled
    pub auto_sync: bool,
    /// Sync interval in seconds
    pub interval_secs: u64,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
    /// Last sync timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    /// Last sync status
    pub last_status: SyncStatus,
}

impl SyncConfig {
    /// Create new sync config
    pub fn new() -> Self {
        Self {
            interval_secs: 300, // 5 minutes
            last_status: SyncStatus::NotConfigured,
            ..Default::default()
        }
    }

    /// Configure file-based sync
    pub fn file_sync(path: impl Into<String>) -> Self {
        Self {
            provider: SyncProvider::File,
            location: Some(path.into()),
            auto_sync: true,
            interval_secs: 300,
            conflict_resolution: ConflictResolution::AcceptRemote,
            last_sync: None,
            last_status: SyncStatus::NotConfigured,
        }
    }

    /// Configure git sync
    pub fn git_sync(repo_path: impl Into<String>) -> Self {
        Self {
            provider: SyncProvider::Git,
            location: Some(repo_path.into()),
            auto_sync: false,
            interval_secs: 3600,
            conflict_resolution: ConflictResolution::Ask,
            last_sync: None,
            last_status: SyncStatus::NotConfigured,
        }
    }

    /// Is sync configured?
    pub fn is_configured(&self) -> bool {
        self.provider != SyncProvider::None && self.location.is_some()
    }

    /// Is auto-sync enabled?
    pub fn is_auto_sync(&self) -> bool {
        self.auto_sync && self.is_configured()
    }

    /// Time since last sync
    pub fn time_since_sync(&self) -> Option<chrono::Duration> {
        self.last_sync.map(|t| chrono::Utc::now() - t)
    }

    /// Is sync due?
    pub fn is_sync_due(&self) -> bool {
        if !self.is_auto_sync() {
            return false;
        }
        match self.time_since_sync() {
            Some(duration) => duration.num_seconds() >= self.interval_secs as i64,
            None => true,
        }
    }
}

/// Sync manager
#[derive(Debug, Clone, Default)]
pub struct SyncManager {
    /// Sync configuration
    pub config: SyncConfig,
    /// Local settings version
    local_version: u64,
    /// Remote settings version
    remote_version: u64,
}

impl SyncManager {
    /// Create new sync manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure sync
    pub fn configure(&mut self, config: SyncConfig) {
        self.config = config;
    }

    /// Get current sync status
    pub fn status(&self) -> SyncStatus {
        if !self.config.is_configured() {
            return SyncStatus::NotConfigured;
        }

        if self.local_version > self.remote_version {
            SyncStatus::LocalAhead
        } else if self.remote_version > self.local_version {
            SyncStatus::RemoteAhead
        } else {
            SyncStatus::Synced
        }
    }

    /// Check for remote changes (file-based)
    pub fn check_remote(&mut self) -> SettingsResult<SyncStatus> {
        if !self.config.is_configured() {
            return Ok(SyncStatus::NotConfigured);
        }

        match self.config.provider {
            SyncProvider::File => self.check_file_remote(),
            SyncProvider::Git => self.check_git_remote(),
            _ => Ok(SyncStatus::NotConfigured),
        }
    }

    /// Check file-based remote
    fn check_file_remote(&self) -> SettingsResult<SyncStatus> {
        let path = self.config.location.as_ref()
            .ok_or(SettingsError::PathUnavailable)?;
        let sync_path = PathBuf::from(path);

        if !sync_path.exists() {
            return Ok(SyncStatus::LocalAhead);
        }

        // Check modification time
        let metadata = std::fs::metadata(&sync_path)?;
        let modified = metadata.modified()?;
        let modified_dt: chrono::DateTime<chrono::Utc> = modified.into();

        match self.config.last_sync {
            Some(last) if modified_dt > last => Ok(SyncStatus::RemoteAhead),
            _ => Ok(SyncStatus::Synced),
        }
    }

    /// Check git-based remote (placeholder)
    fn check_git_remote(&self) -> SettingsResult<SyncStatus> {
        // Git sync would require running git commands
        Ok(SyncStatus::NotConfigured)
    }

    /// Push local settings to remote
    pub fn push(&mut self, settings: &UnifiedSettings) -> SettingsResult<()> {
        if !self.config.is_configured() {
            return Err(SettingsError::PathUnavailable);
        }

        match self.config.provider {
            SyncProvider::File => self.push_file(settings),
            _ => Ok(()),
        }
    }

    /// Push to file-based remote
    fn push_file(&mut self, settings: &UnifiedSettings) -> SettingsResult<()> {
        let path = self.config.location.as_ref()
            .ok_or(SettingsError::PathUnavailable)?;
        let sync_path = PathBuf::from(path);

        if let Some(parent) = sync_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| SettingsError::Serde(e.to_string()))?;
        std::fs::write(&sync_path, content)?;

        self.config.last_sync = Some(chrono::Utc::now());
        self.config.last_status = SyncStatus::Synced;
        self.local_version += 1;
        self.remote_version = self.local_version;

        Ok(())
    }

    /// Pull settings from remote
    pub fn pull(&mut self) -> SettingsResult<UnifiedSettings> {
        if !self.config.is_configured() {
            return Err(SettingsError::PathUnavailable);
        }

        match self.config.provider {
            SyncProvider::File => self.pull_file(),
            _ => Err(SettingsError::PathUnavailable),
        }
    }

    /// Pull from file-based remote
    fn pull_file(&mut self) -> SettingsResult<UnifiedSettings> {
        let path = self.config.location.as_ref()
            .ok_or(SettingsError::PathUnavailable)?;
        let sync_path = PathBuf::from(path);

        let content = std::fs::read_to_string(&sync_path)?;
        let settings: UnifiedSettings = serde_json::from_str(&content)
            .map_err(|e| SettingsError::Serde(e.to_string()))?;

        self.config.last_sync = Some(chrono::Utc::now());
        self.config.last_status = SyncStatus::Synced;
        self.remote_version += 1;
        self.local_version = self.remote_version;

        Ok(settings)
    }

    /// Sync (push or pull based on status)
    pub fn sync(&mut self, local: &UnifiedSettings) -> SettingsResult<Option<UnifiedSettings>> {
        let status = self.check_remote()?;

        match status {
            SyncStatus::LocalAhead => {
                self.push(local)?;
                Ok(None)
            }
            SyncStatus::RemoteAhead => {
                let remote = self.pull()?;
                Ok(Some(remote))
            }
            SyncStatus::Conflict => {
                match self.config.conflict_resolution {
                    ConflictResolution::KeepLocal => {
                        self.push(local)?;
                        Ok(None)
                    }
                    ConflictResolution::AcceptRemote => {
                        let remote = self.pull()?;
                        Ok(Some(remote))
                    }
                    _ => {
                        // For merge strategies, just accept remote for now
                        let remote = self.pull()?;
                        Ok(Some(remote))
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// Mark local as changed
    pub fn mark_local_changed(&mut self) {
        self.local_version += 1;
    }
}

/// Format sync status for display
pub fn format_sync_status(manager: &SyncManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Sync ===\n\n");
    output.push_str(&format!("Provider: {}\n", manager.config.provider));
    output.push_str(&format!("Status: {}\n", manager.status()));

    if let Some(location) = &manager.config.location {
        output.push_str(&format!("Location: {}\n", location));
    }

    if let Some(last) = manager.config.last_sync {
        output.push_str(&format!("Last sync: {}\n", last.format("%Y-%m-%d %H:%M:%S UTC")));
    }

    output.push_str(&format!("Auto-sync: {}\n", manager.config.auto_sync));

    if manager.config.is_sync_due() {
        output.push_str("\nSync is due!\n");
    }

    output
}

/// Fun fact about settings sync
pub fn settings_sync_fun_fact() -> &'static str {
    "Anna can sync your settings across machines using a shared folder or git repository!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_display() {
        assert_eq!(format!("{}", SyncStatus::Synced), "Synced");
        assert_eq!(format!("{}", SyncStatus::LocalAhead), "Local changes pending");
        assert_eq!(format!("{}", SyncStatus::Conflict), "Conflict");
    }

    #[test]
    fn test_sync_provider_display() {
        assert_eq!(format!("{}", SyncProvider::File), "File");
        assert_eq!(format!("{}", SyncProvider::Git), "Git");
    }

    #[test]
    fn test_conflict_resolution_display() {
        assert_eq!(format!("{}", ConflictResolution::KeepLocal), "Keep local");
        assert_eq!(format!("{}", ConflictResolution::AcceptRemote), "Accept remote");
    }

    #[test]
    fn test_sync_config_new() {
        let config = SyncConfig::new();
        assert_eq!(config.provider, SyncProvider::None);
        assert!(!config.is_configured());
    }

    #[test]
    fn test_sync_config_file() {
        let config = SyncConfig::file_sync("/tmp/anna_sync");
        assert_eq!(config.provider, SyncProvider::File);
        assert!(config.is_configured());
        assert!(config.is_auto_sync());
    }

    #[test]
    fn test_sync_config_git() {
        let config = SyncConfig::git_sync("/path/to/repo");
        assert_eq!(config.provider, SyncProvider::Git);
        assert!(config.is_configured());
        assert!(!config.is_auto_sync()); // Git defaults to manual
    }

    #[test]
    fn test_sync_manager_new() {
        let manager = SyncManager::new();
        assert_eq!(manager.status(), SyncStatus::NotConfigured);
    }

    #[test]
    fn test_sync_manager_configure() {
        let mut manager = SyncManager::new();
        manager.configure(SyncConfig::file_sync("/tmp/sync"));
        assert!(manager.config.is_configured());
    }

    #[test]
    fn test_sync_manager_mark_changed() {
        let mut manager = SyncManager::new();
        manager.configure(SyncConfig::file_sync("/tmp/sync"));
        manager.mark_local_changed();
        assert_eq!(manager.status(), SyncStatus::LocalAhead);
    }

    #[test]
    fn test_is_sync_due() {
        let mut config = SyncConfig::file_sync("/tmp/sync");
        config.last_sync = None;
        assert!(config.is_sync_due());

        config.last_sync = Some(chrono::Utc::now());
        assert!(!config.is_sync_due());
    }

    #[test]
    fn test_format_sync_status() {
        let manager = SyncManager::new();
        let output = format_sync_status(&manager);
        assert!(output.contains("Sync"));
        assert!(output.contains("Not configured"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_sync_fun_fact();
        assert!(fact.contains("sync"));
    }
}

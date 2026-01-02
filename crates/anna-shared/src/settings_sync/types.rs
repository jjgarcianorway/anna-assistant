// v0.0.564: Settings Sync - Types (Phase 140)
// Sync types, enums, and configuration

use serde::{Deserialize, Serialize};

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
    fn test_is_sync_due() {
        let mut config = SyncConfig::file_sync("/tmp/sync");
        config.last_sync = None;
        assert!(config.is_sync_due());

        config.last_sync = Some(chrono::Utc::now());
        assert!(!config.is_sync_due());
    }
}

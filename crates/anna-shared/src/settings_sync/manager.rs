// v0.0.564: Settings Sync - Manager (Phase 140)
// Sync manager for handling synchronization operations

use std::path::PathBuf;

use crate::settings_persistence::{SettingsError, SettingsResult};
use crate::unified_settings::UnifiedSettings;

use super::types::{ConflictResolution, SyncConfig, SyncProvider, SyncStatus};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}

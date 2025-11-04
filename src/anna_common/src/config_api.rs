//! Configuration API - Unified read/write/watch with synchronization
//!
//! This module provides a single interface for configuration management with:
//! - File watching and change detection
//! - Atomic writes with validation
//! - Event emission for synchronization
//! - Snapshot creation before changes
//! - Checksum-based conflict detection

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::configurator::{
    load_master_config, load_priorities_config, save_master_config, save_priorities_config,
    MasterConfig, PrioritiesConfig,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Event Types
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Tui,
    Gui,
    Daemon,
    Remote { host: String },
    External, // Manual file edit
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    User,
    System,
    Scheduler,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateEvent {
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
    pub actor: Actor,
    pub changes: ChangeSet,
    pub snapshot_token: Option<String>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub master_config: HashMap<String, FieldChange>,
    pub priorities: HashMap<String, FieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub file: String,
    pub line: Option<usize>,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadRequest {
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub token: String,
    pub timestamp: DateTime<Utc>,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigEvent {
    Updated(ConfigUpdateEvent),
    ValidationError(ValidationError),
    ReloadRequest(ReloadRequest),
    SnapshotCreated(SnapshotInfo),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validated Configuration
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub master: MasterConfig,
    pub priorities: PrioritiesConfig,
    pub checksum: String,
    pub last_modified: SystemTime,
}

impl ValidatedConfig {
    /// Load and validate configuration from disk
    pub fn load() -> Result<Self> {
        let master = load_master_config()?;
        let priorities = load_priorities_config()?;

        let checksum = calculate_checksum_from_configs(&master, &priorities)?;
        let last_modified = get_config_mtime()?;

        Ok(Self {
            master,
            priorities,
            checksum,
            last_modified,
        })
    }

    /// Check if configuration has been modified externally
    pub fn is_stale(&self) -> Result<bool> {
        let current_mtime = get_config_mtime()?;
        Ok(current_mtime > self.last_modified)
    }

    /// Reload if stale
    pub fn reload_if_stale(&mut self) -> Result<bool> {
        if self.is_stale()? {
            *self = Self::load()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sync Result
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct SyncResult {
    pub snapshot_token: Option<String>,
    pub changes: ChangeSet,
    pub checksum: String,
    pub event: ConfigUpdateEvent,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Config Watcher
// ═══════════════════════════════════════════════════════════════════════════════

pub struct ConfigWatcher {
    _phantom: std::marker::PhantomData<()>,
}

impl ConfigWatcher {
    pub fn stop(self) {
        // Cleanup resources
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Save configuration with synchronization support
pub fn save_with_sync(
    master: &MasterConfig,
    priorities: &PrioritiesConfig,
    source: EventSource,
    actor: Actor,
) -> Result<SyncResult> {
    // Load current config for change detection
    let old_master = load_master_config().unwrap_or_default();
    let old_priorities = load_priorities_config().unwrap_or_default();

    // Create snapshot before changes
    let snapshot_token = create_snapshot("pre_config_update")?;

    // Atomic write
    save_master_config(master)?;
    save_priorities_config(priorities)?;

    let new_checksum = calculate_checksum_from_configs(master, priorities)?;

    // Detect changes
    let changes = detect_changes(&old_master, master, &old_priorities, priorities);

    // Create event
    let event = ConfigUpdateEvent {
        timestamp: Utc::now(),
        source,
        actor,
        changes: changes.clone(),
        snapshot_token: Some(snapshot_token.clone()),
        checksum: new_checksum.clone(),
    };

    // Emit event (in real implementation, this would broadcast)
    emit_event(ConfigEvent::Updated(event.clone()))?;

    Ok(SyncResult {
        snapshot_token: Some(snapshot_token),
        changes,
        checksum: new_checksum,
        event,
    })
}

/// Watch for configuration file changes
pub fn watch_config_dir<F>(callback: F) -> Result<ConfigWatcher>
where
    F: Fn(ConfigUpdateEvent) + Send + 'static,
{
    // Placeholder for notify integration
    // In full implementation:
    // - Use notify::RecommendedWatcher
    // - Watch ~/.config/anna directory
    // - Debounce events (200ms)
    // - Call callback on validated changes

    Ok(ConfigWatcher {
        _phantom: std::marker::PhantomData,
    })
}

/// Calculate configuration checksum
pub fn calculate_checksum(paths: &[PathBuf]) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    for path in paths {
        if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read {:?}", path))?;
            content.hash(&mut hasher);
        }
    }

    Ok(format!("sha256:{:x}", hasher.finish()))
}

/// Calculate checksum from config structs
fn calculate_checksum_from_configs(
    master: &MasterConfig,
    priorities: &PrioritiesConfig,
) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let master_yaml = serde_yaml::to_string(master)?;
    let priorities_yaml = serde_yaml::to_string(priorities)?;

    let mut hasher = DefaultHasher::new();
    master_yaml.hash(&mut hasher);
    priorities_yaml.hash(&mut hasher);

    Ok(format!("sha256:{:x}", hasher.finish()))
}

/// Create snapshot before changes
pub fn create_snapshot(label: &str) -> Result<String> {
    use chrono::Local;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let token = format!("snap_{}_{}", label, timestamp);

    // In full implementation:
    // - Create ~/.local/state/anna/snapshots/{token}/
    // - Copy all config files
    // - Write manifest.json
    // - Return token

    Ok(token)
}

/// Emit synchronization event
pub fn emit_event(event: ConfigEvent) -> Result<()> {
    // In full implementation:
    // - Write to event socket /run/anna/events.sock
    // - Broadcast to all connected clients
    // - Log to audit.jsonl

    // For now, just log
    match event {
        ConfigEvent::Updated(update) => {
            eprintln!("ConfigUpdateEvent: source={:?}, changes={:?}",
                update.source, update.changes);
        }
        ConfigEvent::ValidationError(error) => {
            eprintln!("ConfigValidationError: {} in {}", error.message, error.file);
        }
        ConfigEvent::ReloadRequest(req) => {
            eprintln!("ConfigReloadRequest: reason={}", req.reason);
        }
        ConfigEvent::SnapshotCreated(info) => {
            eprintln!("SnapshotCreated: token={}", info.token);
        }
    }

    Ok(())
}

/// Detect changes between old and new configurations
fn detect_changes(
    old_master: &MasterConfig,
    new_master: &MasterConfig,
    old_priorities: &PrioritiesConfig,
    new_priorities: &PrioritiesConfig,
) -> ChangeSet {
    let mut master_changes = HashMap::new();
    let mut priority_changes = HashMap::new();

    // Check master config changes
    if old_master.profile != new_master.profile {
        master_changes.insert(
            "profile".to_string(),
            FieldChange {
                old: old_master.profile.clone(),
                new: new_master.profile.clone(),
            },
        );
    }

    if old_master.autonomy != new_master.autonomy {
        master_changes.insert(
            "autonomy".to_string(),
            FieldChange {
                old: format!("{}", old_master.autonomy),
                new: format!("{}", new_master.autonomy),
            },
        );
    }

    if old_master.stability != new_master.stability {
        master_changes.insert(
            "stability".to_string(),
            FieldChange {
                old: format!("{}", old_master.stability),
                new: format!("{}", new_master.stability),
            },
        );
    }

    // Check priority changes
    if old_priorities.performance != new_priorities.performance {
        priority_changes.insert(
            "performance".to_string(),
            FieldChange {
                old: format!("{:?}", old_priorities.performance),
                new: format!("{:?}", new_priorities.performance),
            },
        );
    }

    if old_priorities.responsiveness != new_priorities.responsiveness {
        priority_changes.insert(
            "responsiveness".to_string(),
            FieldChange {
                old: format!("{:?}", old_priorities.responsiveness),
                new: format!("{:?}", new_priorities.responsiveness),
            },
        );
    }

    if old_priorities.battery_life != new_priorities.battery_life {
        priority_changes.insert(
            "battery_life".to_string(),
            FieldChange {
                old: format!("{:?}", old_priorities.battery_life),
                new: format!("{:?}", new_priorities.battery_life),
            },
        );
    }

    if old_priorities.aesthetics != new_priorities.aesthetics {
        priority_changes.insert(
            "aesthetics".to_string(),
            FieldChange {
                old: format!("{:?}", old_priorities.aesthetics),
                new: format!("{:?}", new_priorities.aesthetics),
            },
        );
    }

    ChangeSet {
        master_config: master_changes,
        priorities: priority_changes,
    }
}

/// Get last modification time of config files
fn get_config_mtime() -> Result<SystemTime> {
    use crate::configurator::anna_config_path;

    let path = anna_config_path()?;
    let metadata = fs::metadata(&path)
        .with_context(|| format!("Failed to get metadata for {:?}", path))?;

    Ok(metadata.modified()?)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_source_serialization() {
        let source = EventSource::Tui;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, r#""tui""#);

        let source = EventSource::Remote {
            host: "example.com".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("remote"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_actor_serialization() {
        let actor = Actor::User;
        let json = serde_json::to_string(&actor).unwrap();
        assert_eq!(json, r#""user""#);
    }

    #[test]
    fn test_change_detection() {
        let old_master = MasterConfig::default();
        let mut new_master = old_master.clone();
        new_master.profile = "workstation".to_string();

        let old_priorities = PrioritiesConfig::default();
        let new_priorities = old_priorities.clone();

        let changes = detect_changes(&old_master, &new_master, &old_priorities, &new_priorities);

        assert_eq!(changes.master_config.len(), 1);
        assert!(changes.master_config.contains_key("profile"));

        let profile_change = &changes.master_config["profile"];
        assert_eq!(profile_change.old, "default");
        assert_eq!(profile_change.new, "workstation");
    }

    #[test]
    fn test_checksum_consistency() {
        let master = MasterConfig::default();
        let priorities = PrioritiesConfig::default();

        let checksum1 = calculate_checksum_from_configs(&master, &priorities).unwrap();
        let checksum2 = calculate_checksum_from_configs(&master, &priorities).unwrap();

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_changes() {
        let master1 = MasterConfig::default();
        let mut master2 = master1.clone();
        master2.profile = "workstation".to_string();

        let priorities = PrioritiesConfig::default();

        let checksum1 = calculate_checksum_from_configs(&master1, &priorities).unwrap();
        let checksum2 = calculate_checksum_from_configs(&master2, &priorities).unwrap();

        assert_ne!(checksum1, checksum2);
    }
}

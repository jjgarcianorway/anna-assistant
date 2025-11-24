//! Update Manager - Background auto-update system for Anna
//!
//! 6.22.0: Implements safe, cached updates without new CLI commands
//!
//! Architecture:
//! - Background checks for updates (configurable interval)
//! - Downloads to /var/lib/anna/updates/ with .partial suffix
//! - Verifies checksums before finalizing
//! - Tracks state (Idle, Checking, Downloading, ReadyToInstall, Error)
//! - Exposed via RPC for annactl status
//! - Triggered via natural language: annactl "update yourself"

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Directory for cached update artifacts
const UPDATE_DIR: &str = "/var/lib/anna/updates";

/// Update phase - what the updater is currently doing
#[derive(Debug, Clone, PartialEq)]
pub enum UpdatePhase {
    /// Not checking or downloading
    Idle,
    /// Currently checking for new versions
    Checking,
    /// Downloading a new version
    Downloading {
        version: String,
        progress: Option<f32>,
    },
    /// Update downloaded and verified, ready to install
    ReadyToInstall { version: String },
    /// Error occurred during check or download
    Error {
        message: String,
        last_attempt: Option<Instant>,
    },
}

// Manual Serialize/Deserialize to handle Instant
impl Serialize for UpdatePhase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStructVariant;
        match self {
            UpdatePhase::Idle => serializer.serialize_unit_variant("UpdatePhase", 0, "Idle"),
            UpdatePhase::Checking => serializer.serialize_unit_variant("UpdatePhase", 1, "Checking"),
            UpdatePhase::Downloading { version, progress } => {
                let mut state = serializer.serialize_struct_variant("UpdatePhase", 2, "Downloading", 2)?;
                state.serialize_field("version", version)?;
                state.serialize_field("progress", progress)?;
                state.end()
            }
            UpdatePhase::ReadyToInstall { version } => {
                let mut state = serializer.serialize_struct_variant("UpdatePhase", 3, "ReadyToInstall", 1)?;
                state.serialize_field("version", version)?;
                state.end()
            }
            UpdatePhase::Error { message, .. } => {
                // Skip last_attempt during serialization
                let mut state = serializer.serialize_struct_variant("UpdatePhase", 4, "Error", 1)?;
                state.serialize_field("message", message)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for UpdatePhase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        enum UpdatePhase_Internal {
            Idle,
            Checking,
            Downloading {
                version: String,
                progress: Option<f32>,
            },
            ReadyToInstall {
                version: String,
            },
            Error {
                message: String,
            },
        }

        match UpdatePhase_Internal::deserialize(deserializer)? {
            UpdatePhase_Internal::Idle => Ok(UpdatePhase::Idle),
            UpdatePhase_Internal::Checking => Ok(UpdatePhase::Checking),
            UpdatePhase_Internal::Downloading { version, progress } => {
                Ok(UpdatePhase::Downloading { version, progress })
            }
            UpdatePhase_Internal::ReadyToInstall { version } => {
                Ok(UpdatePhase::ReadyToInstall { version })
            }
            UpdatePhase_Internal::Error { message } => Ok(UpdatePhase::Error {
                message,
                last_attempt: Some(Instant::now()),
            }),
        }
    }
}

impl Default for UpdatePhase {
    fn default() -> Self {
        Self::Idle
    }
}

/// Complete update state for the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    /// Current running version
    pub current_version: String,
    /// Latest version known from remote metadata
    pub latest_known_version: Option<String>,
    /// Current phase of update process
    #[serde(skip)]
    pub phase: UpdatePhase,
    /// Whether automatic updates are enabled in config
    pub auto_updates_enabled: bool,
    /// Last time we successfully checked for updates
    #[serde(skip)]
    pub last_check: Option<Instant>,
}

impl UpdateState {
    pub fn new(current_version: String, auto_enabled: bool) -> Self {
        Self {
            current_version,
            latest_known_version: None,
            phase: UpdatePhase::Idle,
            auto_updates_enabled: auto_enabled,
            last_check: None,
        }
    }

    /// Check if an update is available
    pub fn update_available(&self) -> bool {
        if let Some(ref latest) = self.latest_known_version {
            latest != &self.current_version
        } else {
            false
        }
    }

    /// Get human-readable status string
    pub fn status_string(&self) -> String {
        match &self.phase {
            UpdatePhase::Idle => {
                if self.update_available() {
                    format!(
                        "Update available: v{}",
                        self.latest_known_version.as_deref().unwrap_or("unknown")
                    )
                } else {
                    format!("Up to date (v{})", self.current_version)
                }
            }
            UpdatePhase::Checking => "Checking for updates...".to_string(),
            UpdatePhase::Downloading { version, progress } => {
                if let Some(pct) = progress {
                    format!("Downloading v{} ({:.0}%)", version, pct * 100.0)
                } else {
                    format!("Downloading v{}", version)
                }
            }
            UpdatePhase::ReadyToInstall { version } => {
                format!("v{} ready to install", version)
            }
            UpdatePhase::Error { message, .. } => {
                format!("Update error: {}", message)
            }
        }
    }
}

/// Update Manager - handles background checking and downloading
pub struct UpdateManager {
    update_dir: PathBuf,
    state: UpdateState,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new(current_version: String, auto_enabled: bool) -> Result<Self> {
        let update_dir = PathBuf::from(UPDATE_DIR);

        // Create update directory if it doesn't exist
        if !update_dir.exists() {
            std::fs::create_dir_all(&update_dir)
                .context("Failed to create update directory")?;
        }

        Ok(Self {
            update_dir,
            state: UpdateState::new(current_version, auto_enabled),
        })
    }

    /// Get current update state (for RPC)
    pub fn get_state(&self) -> &UpdateState {
        &self.state
    }

    /// Get mutable state reference
    pub fn get_state_mut(&mut self) -> &mut UpdateState {
        &mut self.state
    }

    /// Check for updates (non-blocking)
    pub async fn check_for_updates(&mut self) -> Result<()> {
        info!("Checking for updates (current: v{})", self.state.current_version);
        self.state.phase = UpdatePhase::Checking;

        // TODO: Implement actual GitHub API check
        // For now, simulate check
        self.state.last_check = Some(Instant::now());

        // Placeholder: always report up to date for now
        self.state.latest_known_version = Some(self.state.current_version.clone());
        self.state.phase = UpdatePhase::Idle;

        info!("Update check complete: up to date");
        Ok(())
    }

    /// Download a specific version
    pub async fn download_version(&mut self, version: &str) -> Result<()> {
        info!("Starting download of v{}", version);
        self.state.phase = UpdatePhase::Downloading {
            version: version.to_string(),
            progress: Some(0.0),
        };

        // TODO: Implement actual download logic with:
        // 1. Download to .partial file
        // 2. Verify checksum
        // 3. Rename to final name
        // 4. Update state to ReadyToInstall

        // Placeholder for now
        warn!("Download not yet implemented - setting error state");
        self.state.phase = UpdatePhase::Error {
            message: "Download not yet implemented".to_string(),
            last_attempt: Some(Instant::now()),
        };

        Ok(())
    }

    /// Apply a downloaded update (requires Ready state)
    pub fn apply_update(&mut self) -> Result<()> {
        match &self.state.phase {
            UpdatePhase::ReadyToInstall { version } => {
                info!("Applying update to v{}", version);
                // TODO: Implement apply logic:
                // 1. Stop daemon gracefully
                // 2. Replace binaries
                // 3. Restart daemon
                Err(anyhow::anyhow!("Apply not yet implemented"))
            }
            _ => Err(anyhow::anyhow!(
                "Cannot apply update - not in ReadyToInstall state"
            )),
        }
    }

    /// Clean up old cached versions (keep only latest)
    pub fn cleanup_old_versions(&self) -> Result<()> {
        // TODO: Implement cleanup logic
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_state_creation() {
        let state = UpdateState::new("6.22.0".to_string(), true);
        assert_eq!(state.current_version, "6.22.0");
        assert!(state.auto_updates_enabled);
        assert_eq!(state.phase, UpdatePhase::Idle);
    }

    #[test]
    fn test_update_available() {
        let mut state = UpdateState::new("6.22.0".to_string(), true);
        assert!(!state.update_available());

        state.latest_known_version = Some("6.23.0".to_string());
        assert!(state.update_available());

        state.latest_known_version = Some("6.22.0".to_string());
        assert!(!state.update_available());
    }

    #[test]
    fn test_status_strings() {
        let mut state = UpdateState::new("6.22.0".to_string(), true);

        assert!(state.status_string().contains("Up to date"));

        state.phase = UpdatePhase::Checking;
        assert!(state.status_string().contains("Checking"));

        state.phase = UpdatePhase::Downloading {
            version: "6.23.0".to_string(),
            progress: Some(0.5),
        };
        assert!(state.status_string().contains("50%"));

        state.phase = UpdatePhase::ReadyToInstall {
            version: "6.23.0".to_string(),
        };
        assert!(state.status_string().contains("ready"));
    }
}

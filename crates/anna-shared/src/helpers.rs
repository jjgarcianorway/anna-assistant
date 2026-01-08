//! Helper package tracking.
//!
//! Tracks packages installed by Anna so they can be removed on uninstall.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

use crate::config::anna_data_dir;

/// Source of a helper installation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallSource {
    /// Installed by Anna automatically
    Anna,
    /// Was already installed by user
    User,
    /// Bundled with the system
    System,
}

/// A helper package tracked by Anna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Helper {
    /// Package name (e.g., "ethtool", "lm_sensors")
    pub name: String,
    /// Why Anna installed it
    pub reason: String,
    /// Who installed it
    pub source: InstallSource,
    /// When it was registered
    pub registered_at: DateTime<Utc>,
    /// Binary path (for quick availability check)
    pub binary_path: Option<String>,
}

/// Registry of all tracked helpers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HelpersRegistry {
    pub helpers: HashMap<String, Helper>,
}

impl HelpersRegistry {
    /// Load registry from disk
    pub fn load() -> Result<Self> {
        let path = helpers_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let registry: HelpersRegistry = serde_json::from_str(&content)?;
            Ok(registry)
        } else {
            Ok(HelpersRegistry::default())
        }
    }

    /// Save registry to disk
    pub fn save(&self) -> Result<()> {
        let path = helpers_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Register a helper as installed by Anna
    pub fn register_anna_installed(&mut self, name: &str, reason: &str, binary_path: Option<&str>) -> Result<()> {
        let helper = Helper {
            name: name.to_string(),
            reason: reason.to_string(),
            source: InstallSource::Anna,
            registered_at: Utc::now(),
            binary_path: binary_path.map(|s| s.to_string()),
        };
        self.helpers.insert(name.to_string(), helper);
        self.save()?;
        Ok(())
    }

    /// Register a helper that was already installed by user
    pub fn register_user_installed(&mut self, name: &str, binary_path: Option<&str>) -> Result<()> {
        let helper = Helper {
            name: name.to_string(),
            reason: "Pre-existing installation".to_string(),
            source: InstallSource::User,
            registered_at: Utc::now(),
            binary_path: binary_path.map(|s| s.to_string()),
        };
        self.helpers.insert(name.to_string(), helper);
        self.save()?;
        Ok(())
    }

    /// Get all helpers installed by Anna (for uninstall cleanup)
    pub fn anna_installed(&self) -> Vec<&Helper> {
        self.helpers
            .values()
            .filter(|h| h.source == InstallSource::Anna)
            .collect()
    }

    /// Check if a helper is available
    pub fn is_available(&self, name: &str) -> bool {
        if let Some(helper) = self.helpers.get(name) {
            if let Some(path) = &helper.binary_path {
                return std::path::Path::new(path).exists();
            }
        }
        // Check if binary exists in PATH
        which::which(name).is_ok()
    }

    /// Remove a helper from registry
    pub fn unregister(&mut self, name: &str) -> Result<()> {
        self.helpers.remove(name);
        self.save()?;
        Ok(())
    }
}

/// Get helpers registry path
pub fn helpers_path() -> PathBuf {
    anna_data_dir().join("helpers.json")
}

/// Check if a command/tool is available on the system
pub fn tool_available(name: &str) -> bool {
    which::which(name).is_ok()
}

/// Install a package using pacman (requires sudo)
pub async fn install_package(name: &str) -> Result<()> {
    use tokio::process::Command;

    let output = Command::new("sudo")
        .args(["pacman", "-S", "--noconfirm", name])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to install {}: {}", name, stderr)
    }
}

/// Remove a package using pacman (requires sudo)
pub async fn remove_package(name: &str) -> Result<()> {
    use tokio::process::Command;

    let output = Command::new("sudo")
        .args(["pacman", "-R", "--noconfirm", name])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to remove {}: {}", name, stderr)
    }
}

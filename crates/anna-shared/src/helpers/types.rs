//! Helper package types (v0.0.221).
//! v0.0.466: Enhanced with last_used, hardware_requirement per Phase 32.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// How a helper package was installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum InstallSource {
    /// Installed by Anna (auto-install)
    Anna,
    /// Installed by user (system package manager, manual, etc.)
    User,
    /// Bundled with Anna
    Bundled,
    /// Unknown source
    #[default]
    Unknown,
}

impl std::fmt::Display for InstallSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anna => write!(f, "anna"),
            Self::User => write!(f, "user"),
            Self::Bundled => write!(f, "bundled"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// A helper package that Anna depends on.
/// v0.0.466: Enhanced with last_used, hardware_requirement per Phase 32.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelperPackage {
    /// Package identifier (e.g., "ollama")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Package version if known
    pub version: Option<String>,
    /// How the package was installed
    pub install_source: InstallSource,
    /// Whether the package is currently available
    pub available: bool,
    /// Path to the package binary if known
    pub binary_path: Option<PathBuf>,
    /// Whether this is a required dependency
    pub required: bool,
    /// v0.0.466: Last time this helper was used (Unix timestamp)
    #[serde(default)]
    pub last_used: Option<u64>,
    /// v0.0.466: Hardware requirement (e.g., "ethernet" for ethtool)
    #[serde(default)]
    pub hardware_requirement: Option<String>,
}

impl HelperPackage {
    /// Create a new helper package
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: None,
            install_source: InstallSource::Unknown,
            available: false,
            binary_path: None,
            required: false,
            last_used: None,
            hardware_requirement: None,
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the install source
    pub fn with_source(mut self, source: InstallSource) -> Self {
        self.install_source = source;
        self
    }

    /// Set availability
    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    /// Set binary path
    pub fn with_binary_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = Some(path.into());
        self
    }

    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// v0.0.466: Set hardware requirement
    pub fn with_hardware_requirement(mut self, req: impl Into<String>) -> Self {
        self.hardware_requirement = Some(req.into());
        self
    }

    /// v0.0.466: Record usage timestamp
    pub fn record_usage(&mut self) {
        self.last_used = Some(now_secs());
    }

    /// v0.0.466: Get time since last use in seconds
    pub fn secs_since_use(&self) -> Option<u64> {
        self.last_used.map(|ts| now_secs().saturating_sub(ts))
    }

    /// v0.0.466: Format last used for display
    pub fn last_used_display(&self) -> String {
        match self.last_used {
            None => "never".to_string(),
            Some(ts) => {
                let secs = now_secs().saturating_sub(ts);
                if secs < 60 {
                    "just now".to_string()
                } else if secs < 3600 {
                    format!("{} min ago", secs / 60)
                } else if secs < 86400 {
                    format!("{} hours ago", secs / 3600)
                } else {
                    format!("{} days ago", secs / 86400)
                }
            }
        }
    }

    /// Check if this package was installed by Anna
    pub fn installed_by_anna(&self) -> bool {
        self.install_source == InstallSource::Anna
    }
}

//! Helper package types (v0.0.221).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

    /// Check if this package was installed by Anna
    pub fn installed_by_anna(&self) -> bool {
        self.install_source == InstallSource::Anna
    }
}

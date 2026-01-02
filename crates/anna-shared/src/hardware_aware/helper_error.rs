//! Helper error types (v0.0.434).
//!
//! Error types for helper operations.

/// Helper error.
#[derive(Debug, Clone)]
pub enum HelperError {
    /// No packages defined for this distro.
    NoPackages(String),
    /// Helper not installed.
    NotInstalled(String),
    /// Helper was not installed by Anna.
    NotAnnaInstalled(String),
    /// Installation failed.
    InstallFailed(String),
    /// Uninstallation failed.
    UninstallFailed(String),
    /// Unknown package manager.
    UnknownPackageManager(String),
}

impl std::fmt::Display for HelperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPackages(id) => write!(f, "No packages defined for helper: {}", id),
            Self::NotInstalled(id) => write!(f, "Helper not installed: {}", id),
            Self::NotAnnaInstalled(id) => write!(f, "Helper {} was not installed by Anna", id),
            Self::InstallFailed(msg) => write!(f, "Installation failed: {}", msg),
            Self::UninstallFailed(msg) => write!(f, "Uninstallation failed: {}", msg),
            Self::UnknownPackageManager(distro) => {
                write!(f, "Unknown package manager for distro: {}", distro)
            }
        }
    }
}

impl std::error::Error for HelperError {}

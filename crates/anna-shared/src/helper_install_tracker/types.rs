// v0.0.532: Helper Install Tracker Types (Phase 108)
// Enums for helper installation tracking

use serde::{Deserialize, Serialize};

/// Who installed the helper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum HelperInstaller {
    #[default]
    User,
    Anna,
    System,
}

impl std::fmt::Display for HelperInstaller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Anna => write!(f, "Anna"),
            Self::System => write!(f, "System"),
        }
    }
}

/// Helper category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HelperCategory {
    SystemInfo,
    NetworkDiag,
    DiskUtils,
    HardwareProbe,
    AudioVideo,
    Security,
    DevTools,
    Monitoring,
}

impl Default for HelperCategory {
    fn default() -> Self {
        Self::SystemInfo
    }
}

impl std::fmt::Display for HelperCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SystemInfo => write!(f, "System Info"),
            Self::NetworkDiag => write!(f, "Network Diagnostics"),
            Self::DiskUtils => write!(f, "Disk Utilities"),
            Self::HardwareProbe => write!(f, "Hardware Probe"),
            Self::AudioVideo => write!(f, "Audio/Video"),
            Self::Security => write!(f, "Security"),
            Self::DevTools => write!(f, "Dev Tools"),
            Self::Monitoring => write!(f, "Monitoring"),
        }
    }
}

/// Helper installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HelperStatus {
    #[default]
    NotInstalled,
    Installing,
    Installed,
    Failed,
    Removed,
}

impl std::fmt::Display for HelperStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "Not Installed"),
            Self::Installing => write!(f, "Installing"),
            Self::Installed => write!(f, "Installed"),
            Self::Failed => write!(f, "Failed"),
            Self::Removed => write!(f, "Removed"),
        }
    }
}

//! Package Installation Tracker Types
//!
//! Core types for tracking package installations.

use serde::{Deserialize, Serialize};

/// Who installed the package
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstalledBy {
    Anna,
    User,
    System,
    Unknown,
}

impl InstalledBy {
    pub fn symbol(&self) -> &'static str {
        match self {
            InstalledBy::Anna => "A",
            InstalledBy::User => "U",
            InstalledBy::System => "S",
            InstalledBy::Unknown => "?",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            InstalledBy::Anna => "installed by Anna",
            InstalledBy::User => "installed by user",
            InstalledBy::System => "system package",
            InstalledBy::Unknown => "unknown source",
        }
    }
}

/// Package manager type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManager {
    Pacman,
    Apt,
    Dnf,
    Zypper,
    Flatpak,
    Snap,
    Pip,
    Npm,
    Cargo,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "pacman",
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Zypper => "zypper",
            PackageManager::Flatpak => "flatpak",
            PackageManager::Snap => "snap",
            PackageManager::Pip => "pip",
            PackageManager::Npm => "npm",
            PackageManager::Cargo => "cargo",
        }
    }
}

/// A single package installation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRecord {
    /// Package name
    pub name: String,
    /// Version if known
    pub version: Option<String>,
    /// Who installed it
    pub installed_by: InstalledBy,
    /// Package manager used
    pub manager: PackageManager,
    /// Timestamp when installed
    pub installed_at: u64,
    /// Why it was installed
    pub reason: Option<String>,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// Whether currently installed
    pub is_installed: bool,
    /// Timestamp when removed (if removed)
    pub removed_at: Option<u64>,
}

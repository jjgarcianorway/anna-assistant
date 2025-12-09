//! Fact key enum (v0.0.181).

use serde::{Deserialize, Serialize};

/// Keys for facts that Anna can learn and remember
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FactKey {
    PreferredEditor,
    EditorInstalled(String),
    BinaryAvailable(String),
    NetworkPrimaryInterface,
    NetworkPreference,
    PreferredShell,
    InitSystem,
    PackageManager,
    UnitExists(String),
    MountExists(String),
    // v0.0.41 additions
    WallpaperFolder,
    BootTimeBaseline,
    InstalledPackage(String),
    Desktop,
    GpuPresent,
    Hostname,
    Kernel,
    Custom(String),
}

impl std::fmt::Display for FactKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreferredEditor => write!(f, "preferred_editor"),
            Self::EditorInstalled(e) => write!(f, "editor_installed:{}", e),
            Self::BinaryAvailable(b) => write!(f, "binary_available:{}", b),
            Self::NetworkPrimaryInterface => write!(f, "network_primary_interface"),
            Self::NetworkPreference => write!(f, "network_preference"),
            Self::PreferredShell => write!(f, "preferred_shell"),
            Self::InitSystem => write!(f, "init_system"),
            Self::PackageManager => write!(f, "package_manager"),
            Self::UnitExists(u) => write!(f, "unit_exists:{}", u),
            Self::MountExists(m) => write!(f, "mount_exists:{}", m),
            // v0.0.41 additions
            Self::WallpaperFolder => write!(f, "wallpaper_folder"),
            Self::BootTimeBaseline => write!(f, "boot_time_baseline"),
            Self::InstalledPackage(p) => write!(f, "installed_package:{}", p),
            Self::Desktop => write!(f, "desktop"),
            Self::GpuPresent => write!(f, "gpu_present"),
            Self::Hostname => write!(f, "hostname"),
            Self::Kernel => write!(f, "kernel"),
            Self::Custom(k) => write!(f, "custom:{}", k),
        }
    }
}

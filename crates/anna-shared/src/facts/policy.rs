//! Staleness policy and lifecycle management (v0.0.181).

use serde::{Deserialize, Serialize};

use super::key::FactKey;

/// Staleness policy for facts (v0.0.32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StalenessPolicy {
    Never,
    TTLSeconds(u64),
    SessionOnly,
}

impl Default for StalenessPolicy {
    fn default() -> Self {
        Self::TTLSeconds(30 * 24 * 3600)
    } // 30 days
}

/// Pinned TTL constants for v0.0.41
pub mod ttl {
    /// Installed packages: 7 days (invalidated on pacman hooks later)
    pub const INSTALLED_PACKAGE_SECS: u64 = 7 * 24 * 3600;
    /// Preferred editor: 90 days
    pub const PREFERRED_EDITOR_SECS: u64 = 90 * 24 * 3600;
    /// Boot time baseline: 30 days (keep 14 samples in history)
    pub const BOOT_TIME_SECS: u64 = 30 * 24 * 3600;
    /// Network facts: 1 day
    pub const NETWORK_SECS: u64 = 24 * 3600;
    /// Binary available: 7 days
    pub const BINARY_AVAILABLE_SECS: u64 = 7 * 24 * 3600;
    /// Desktop environment: 30 days
    pub const DESKTOP_SECS: u64 = 30 * 24 * 3600;
}

/// Get default staleness policy for a fact key (v0.0.41 pinned TTLs)
pub fn default_policy(key: &FactKey) -> StalenessPolicy {
    match key {
        FactKey::PreferredEditor => StalenessPolicy::TTLSeconds(ttl::PREFERRED_EDITOR_SECS),
        FactKey::BinaryAvailable(_) => StalenessPolicy::TTLSeconds(ttl::BINARY_AVAILABLE_SECS),
        FactKey::EditorInstalled(_) => StalenessPolicy::TTLSeconds(ttl::BINARY_AVAILABLE_SECS),
        FactKey::NetworkPrimaryInterface => StalenessPolicy::TTLSeconds(ttl::NETWORK_SECS),
        FactKey::NetworkPreference => StalenessPolicy::TTLSeconds(ttl::NETWORK_SECS),
        FactKey::InstalledPackage(_) => StalenessPolicy::TTLSeconds(ttl::INSTALLED_PACKAGE_SECS),
        FactKey::BootTimeBaseline => StalenessPolicy::TTLSeconds(ttl::BOOT_TIME_SECS),
        FactKey::Desktop => StalenessPolicy::TTLSeconds(ttl::DESKTOP_SECS),
        FactKey::InitSystem | FactKey::PackageManager | FactKey::Hostname | FactKey::Kernel => {
            StalenessPolicy::Never // System constants rarely change
        }
        FactKey::GpuPresent => StalenessPolicy::Never, // Hardware doesn't change
        FactKey::UnitExists(_) | FactKey::MountExists(_) => {
            StalenessPolicy::TTLSeconds(ttl::BINARY_AVAILABLE_SECS)
        }
        _ => StalenessPolicy::TTLSeconds(30 * 24 * 3600),
    }
}

/// Lifecycle status for facts (v0.0.32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FactLifecycle {
    #[default]
    Active,
    Stale,
    Archived,
}

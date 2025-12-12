//! Recipe domain classification (v0.0.420).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Domain categorization for recipes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeDomain {
    /// Desktop environment, editors, GUI apps (vim, hyprland, etc.)
    Desktop,
    /// Network configuration, connectivity, wifi, ethernet
    Network,
    /// Disk, filesystems, partitions, mounts
    Storage,
    /// systemd services, daemons
    Services,
    /// CPU, memory, swap, boot time
    Performance,
    /// Firewall, permissions, users
    Security,
    /// General purpose, catch-all
    #[default]
    Generic,
}

impl RecipeDomain {
    /// Get all domains
    pub fn all() -> &'static [RecipeDomain] {
        &[
            RecipeDomain::Desktop,
            RecipeDomain::Network,
            RecipeDomain::Storage,
            RecipeDomain::Services,
            RecipeDomain::Performance,
            RecipeDomain::Security,
            RecipeDomain::Generic,
        ]
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "desktop" | "editor" | "vim" | "gui" | "hyprland" => RecipeDomain::Desktop,
            "network" | "wifi" | "ethernet" | "ip" | "dns" => RecipeDomain::Network,
            "storage" | "disk" | "filesystem" | "mount" | "btrfs" => RecipeDomain::Storage,
            "services" | "systemd" | "daemon" | "service" => RecipeDomain::Services,
            "performance" | "memory" | "cpu" | "ram" | "swap" | "boot" => RecipeDomain::Performance,
            "security" | "firewall" | "permissions" | "user" => RecipeDomain::Security,
            _ => RecipeDomain::Generic,
        }
    }

    /// Get subdirectory name for storage
    pub fn subdir(&self) -> &'static str {
        match self {
            RecipeDomain::Desktop => "desktop",
            RecipeDomain::Network => "network",
            RecipeDomain::Storage => "storage",
            RecipeDomain::Services => "services",
            RecipeDomain::Performance => "performance",
            RecipeDomain::Security => "security",
            RecipeDomain::Generic => "generic",
        }
    }

    /// Get related keywords for this domain
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            RecipeDomain::Desktop => &["vim", "editor", "gui", "hyprland", "waybar", "config"],
            RecipeDomain::Network => &["network", "wifi", "ethernet", "ip", "dns", "interface"],
            RecipeDomain::Storage => &["disk", "storage", "filesystem", "mount", "btrfs", "space"],
            RecipeDomain::Services => &[
                "service", "systemd", "daemon", "enable", "disable", "restart",
            ],
            RecipeDomain::Performance => &["memory", "ram", "cpu", "swap", "boot", "slow", "free"],
            RecipeDomain::Security => &["firewall", "permissions", "user", "sudo", "security"],
            RecipeDomain::Generic => &[],
        }
    }
}

impl fmt::Display for RecipeDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecipeDomain::Desktop => write!(f, "Desktop"),
            RecipeDomain::Network => write!(f, "Network"),
            RecipeDomain::Storage => write!(f, "Storage"),
            RecipeDomain::Services => write!(f, "Services"),
            RecipeDomain::Performance => write!(f, "Performance"),
            RecipeDomain::Security => write!(f, "Security"),
            RecipeDomain::Generic => write!(f, "Generic"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_from_str() {
        assert_eq!(RecipeDomain::from_str("vim"), RecipeDomain::Desktop);
        assert_eq!(RecipeDomain::from_str("memory"), RecipeDomain::Performance);
        assert_eq!(RecipeDomain::from_str("unknown"), RecipeDomain::Generic);
    }
}

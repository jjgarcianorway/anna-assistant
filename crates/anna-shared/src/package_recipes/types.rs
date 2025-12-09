//! Package recipe types (v0.0.230).

use serde::{Deserialize, Serialize};

/// Supported package managers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManager {
    Pacman,
    Apt,
    Dnf,
    Flatpak,
    Snap,
}

impl PackageManager {
    /// Get canonical name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "pacman",
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Flatpak => "Flatpak",
            PackageManager::Snap => "Snap",
        }
    }

    /// Get install command template
    pub fn install_cmd(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "pacman -S --noconfirm",
            PackageManager::Apt => "apt install -y",
            PackageManager::Dnf => "dnf install -y",
            PackageManager::Flatpak => "flatpak install -y",
            PackageManager::Snap => "snap install",
        }
    }

    /// Get remove command template
    pub fn remove_cmd(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "pacman -R --noconfirm",
            PackageManager::Apt => "apt remove -y",
            PackageManager::Dnf => "dnf remove -y",
            PackageManager::Flatpak => "flatpak uninstall -y",
            PackageManager::Snap => "snap remove",
        }
    }

    /// Get search command
    pub fn search_cmd(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "pacman -Ss",
            PackageManager::Apt => "apt search",
            PackageManager::Dnf => "dnf search",
            PackageManager::Flatpak => "flatpak search",
            PackageManager::Snap => "snap find",
        }
    }

    /// Check if package is installed command
    pub fn check_installed_cmd(&self, pkg: &str) -> String {
        match self {
            PackageManager::Pacman => format!("pacman -Q {}", pkg),
            PackageManager::Apt => format!("dpkg -l {}", pkg),
            PackageManager::Dnf => format!("rpm -q {}", pkg),
            PackageManager::Flatpak => format!("flatpak list | grep -i {}", pkg),
            PackageManager::Snap => format!("snap list {}", pkg),
        }
    }

    /// Detect available package manager on system
    pub fn detect() -> Option<Self> {
        // Check for common package managers
        if std::process::Command::new("pacman")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(PackageManager::Pacman);
        }
        if std::process::Command::new("apt")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(PackageManager::Apt);
        }
        if std::process::Command::new("dnf")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(PackageManager::Dnf);
        }
        None
    }
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Package category for common software
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageCategory {
    /// Text editors
    Editor,
    /// Development tools
    Development,
    /// System utilities
    System,
    /// Media players
    Media,
    /// Network tools
    Network,
    /// Compression utilities
    Compression,
}

impl PackageCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            PackageCategory::Editor => "Editor",
            PackageCategory::Development => "Development",
            PackageCategory::System => "System utility",
            PackageCategory::Media => "Media",
            PackageCategory::Network => "Network",
            PackageCategory::Compression => "Compression",
        }
    }
}

/// A package recipe for installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRecipe {
    /// Package name (canonical)
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Category
    pub category: PackageCategory,
    /// Description
    pub description: String,
    /// Package names by manager (key: manager name, value: package name)
    pub packages: std::collections::HashMap<String, String>,
}

impl PackageRecipe {
    /// Create a new package recipe
    pub fn new(name: &str, display_name: &str, category: PackageCategory, desc: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            category,
            description: desc.to_string(),
            packages: std::collections::HashMap::new(),
        }
    }

    /// Add package name for a manager
    pub fn with_package(mut self, manager: PackageManager, pkg: &str) -> Self {
        self.packages
            .insert(manager.display_name().to_string(), pkg.to_string());
        self
    }

    /// Get package name for a manager
    pub fn package_for(&self, manager: &PackageManager) -> Option<&str> {
        self.packages
            .get(manager.display_name())
            .map(|s| s.as_str())
    }

    /// Get install command for a manager
    pub fn install_command(&self, manager: &PackageManager) -> Option<String> {
        self.package_for(manager)
            .map(|pkg| format!("{} {}", manager.install_cmd(), pkg))
    }
}

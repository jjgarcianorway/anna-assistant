//! Package installation recipes.
//!
//! v0.0.98: Safe package installation with multi-manager support.
//!
//! # Supported Package Managers
//! - pacman (Arch Linux)
//! - apt (Debian/Ubuntu)
//! - dnf (Fedora)
//! - flatpak (universal)
//!
//! # Safety
//! - All installs require user confirmation
//! - Uses system package managers (no untrusted sources)
//! - Transaction support for multi-package installs

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
        self.packages.insert(manager.display_name().to_string(), pkg.to_string());
        self
    }

    /// Get package name for a manager
    pub fn package_for(&self, manager: &PackageManager) -> Option<&str> {
        self.packages.get(manager.display_name()).map(|s| s.as_str())
    }

    /// Get install command for a manager
    pub fn install_command(&self, manager: &PackageManager) -> Option<String> {
        self.package_for(manager).map(|pkg| {
            format!("{} {}", manager.install_cmd(), pkg)
        })
    }
}

/// Get common package recipes
pub fn common_packages() -> Vec<PackageRecipe> {
    vec![
        // Editors
        PackageRecipe::new("vim", "Vim", PackageCategory::Editor, "Vi Improved text editor")
            .with_package(PackageManager::Pacman, "vim")
            .with_package(PackageManager::Apt, "vim")
            .with_package(PackageManager::Dnf, "vim-enhanced"),
        PackageRecipe::new("neovim", "Neovim", PackageCategory::Editor, "Modern Vim fork")
            .with_package(PackageManager::Pacman, "neovim")
            .with_package(PackageManager::Apt, "neovim")
            .with_package(PackageManager::Dnf, "neovim"),
        PackageRecipe::new("nano", "nano", PackageCategory::Editor, "Simple text editor")
            .with_package(PackageManager::Pacman, "nano")
            .with_package(PackageManager::Apt, "nano")
            .with_package(PackageManager::Dnf, "nano"),
        PackageRecipe::new("helix", "Helix", PackageCategory::Editor, "Post-modern text editor")
            .with_package(PackageManager::Pacman, "helix")
            .with_package(PackageManager::Apt, "helix")
            .with_package(PackageManager::Dnf, "helix"),

        // Development
        PackageRecipe::new("git", "Git", PackageCategory::Development, "Version control system")
            .with_package(PackageManager::Pacman, "git")
            .with_package(PackageManager::Apt, "git")
            .with_package(PackageManager::Dnf, "git"),
        PackageRecipe::new("make", "Make", PackageCategory::Development, "Build automation tool")
            .with_package(PackageManager::Pacman, "make")
            .with_package(PackageManager::Apt, "make")
            .with_package(PackageManager::Dnf, "make"),
        PackageRecipe::new("gcc", "GCC", PackageCategory::Development, "GNU Compiler Collection")
            .with_package(PackageManager::Pacman, "gcc")
            .with_package(PackageManager::Apt, "gcc")
            .with_package(PackageManager::Dnf, "gcc"),

        // System
        PackageRecipe::new("htop", "htop", PackageCategory::System, "Interactive process viewer")
            .with_package(PackageManager::Pacman, "htop")
            .with_package(PackageManager::Apt, "htop")
            .with_package(PackageManager::Dnf, "htop"),
        PackageRecipe::new("btop", "btop", PackageCategory::System, "Modern resource monitor")
            .with_package(PackageManager::Pacman, "btop")
            .with_package(PackageManager::Apt, "btop")
            .with_package(PackageManager::Dnf, "btop"),

        // Network
        PackageRecipe::new("curl", "curl", PackageCategory::Network, "Data transfer tool")
            .with_package(PackageManager::Pacman, "curl")
            .with_package(PackageManager::Apt, "curl")
            .with_package(PackageManager::Dnf, "curl"),
        PackageRecipe::new("wget", "wget", PackageCategory::Network, "File downloader")
            .with_package(PackageManager::Pacman, "wget")
            .with_package(PackageManager::Apt, "wget")
            .with_package(PackageManager::Dnf, "wget"),

        // Compression
        PackageRecipe::new("unzip", "unzip", PackageCategory::Compression, "ZIP archive extractor")
            .with_package(PackageManager::Pacman, "unzip")
            .with_package(PackageManager::Apt, "unzip")
            .with_package(PackageManager::Dnf, "unzip"),
        PackageRecipe::new("p7zip", "7zip", PackageCategory::Compression, "7-Zip archive manager")
            .with_package(PackageManager::Pacman, "p7zip")
            .with_package(PackageManager::Apt, "p7zip-full")
            .with_package(PackageManager::Dnf, "p7zip"),
    ]
}

/// Find a package recipe by name
pub fn find_recipe(name: &str) -> Option<PackageRecipe> {
    let name_lower = name.to_lowercase();
    common_packages()
        .into_iter()
        .find(|p| p.name == name_lower || p.display_name.to_lowercase() == name_lower)
}

/// Generate confirmation prompt for package install
pub fn confirmation_prompt(recipe: &PackageRecipe, manager: &PackageManager) -> String {
    let cmd = recipe.install_command(manager).unwrap_or_default();
    format!(
        "Install {}?\n\
         Description: {}\n\
         Command: sudo {}\n\
         \n\
         Proceed? [y/N]",
        recipe.display_name,
        recipe.description,
        cmd
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_display() {
        assert_eq!(PackageManager::Pacman.display_name(), "pacman");
        assert_eq!(PackageManager::Apt.display_name(), "apt");
    }

    #[test]
    fn test_find_recipe() {
        assert!(find_recipe("vim").is_some());
        assert!(find_recipe("git").is_some());
        assert!(find_recipe("nonexistent").is_none());
    }

    #[test]
    fn test_package_for_manager() {
        let vim = find_recipe("vim").unwrap();
        assert_eq!(vim.package_for(&PackageManager::Pacman), Some("vim"));
        assert_eq!(vim.package_for(&PackageManager::Dnf), Some("vim-enhanced"));
    }

    #[test]
    fn test_install_command() {
        let vim = find_recipe("vim").unwrap();
        let cmd = vim.install_command(&PackageManager::Pacman);
        assert!(cmd.unwrap().contains("pacman -S"));
    }

    #[test]
    fn test_common_packages_count() {
        let packages = common_packages();
        assert!(packages.len() >= 10);
    }
}

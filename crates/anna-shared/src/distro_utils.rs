//! Distro-aware utilities for package management (v0.0.383).
//!
//! Provides distro-specific command recommendations and advice.
//! Detects package manager from distro name and adapts suggestions.

use serde::{Deserialize, Serialize};

/// Detected package manager family
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageManager {
    /// Arch Linux: pacman, yay, paru
    Pacman,
    /// Debian/Ubuntu: apt, apt-get
    Apt,
    /// Fedora/RHEL: dnf, yum
    Dnf,
    /// openSUSE: zypper
    Zypper,
    /// Alpine: apk
    Apk,
    /// Gentoo: emerge
    Emerge,
    /// NixOS: nix
    Nix,
    /// Unknown distro
    Unknown,
}

impl PackageManager {
    /// Detect package manager from distro name
    pub fn from_distro(distro: &str) -> Self {
        let d = distro.to_lowercase();

        if d.contains("arch") || d.contains("manjaro") || d.contains("endeavour")
            || d.contains("artix") || d.contains("garuda")
        {
            Self::Pacman
        } else if d.contains("ubuntu") || d.contains("debian") || d.contains("mint")
            || d.contains("pop!_os") || d.contains("elementary") || d.contains("zorin")
            || d.contains("kali") || d.contains("raspbian")
        {
            Self::Apt
        } else if d.contains("fedora") || d.contains("rhel") || d.contains("centos")
            || d.contains("rocky") || d.contains("alma") || d.contains("oracle linux")
        {
            Self::Dnf
        } else if d.contains("opensuse") || d.contains("suse") {
            Self::Zypper
        } else if d.contains("alpine") {
            Self::Apk
        } else if d.contains("gentoo") {
            Self::Emerge
        } else if d.contains("nixos") {
            Self::Nix
        } else {
            Self::Unknown
        }
    }

    /// Get the primary package manager command
    pub fn command(&self) -> &'static str {
        match self {
            Self::Pacman => "pacman",
            Self::Apt => "apt",
            Self::Dnf => "dnf",
            Self::Zypper => "zypper",
            Self::Apk => "apk",
            Self::Emerge => "emerge",
            Self::Nix => "nix-env",
            Self::Unknown => "package-manager",
        }
    }

    /// Get command to update package database
    pub fn update_db_command(&self) -> &'static str {
        match self {
            Self::Pacman => "sudo pacman -Sy",
            Self::Apt => "sudo apt update",
            Self::Dnf => "sudo dnf check-update",
            Self::Zypper => "sudo zypper refresh",
            Self::Apk => "sudo apk update",
            Self::Emerge => "sudo emerge --sync",
            Self::Nix => "nix-channel --update",
            Self::Unknown => "# Update package database",
        }
    }

    /// Get command to upgrade all packages
    pub fn upgrade_command(&self) -> &'static str {
        match self {
            Self::Pacman => "sudo pacman -Syu",
            Self::Apt => "sudo apt upgrade",
            Self::Dnf => "sudo dnf upgrade",
            Self::Zypper => "sudo zypper update",
            Self::Apk => "sudo apk upgrade",
            Self::Emerge => "sudo emerge -avuDN @world",
            Self::Nix => "nix-env -u",
            Self::Unknown => "# Upgrade packages",
        }
    }

    /// Get command to install a package
    pub fn install_command(&self, package: &str) -> String {
        match self {
            Self::Pacman => format!("sudo pacman -S {}", package),
            Self::Apt => format!("sudo apt install {}", package),
            Self::Dnf => format!("sudo dnf install {}", package),
            Self::Zypper => format!("sudo zypper install {}", package),
            Self::Apk => format!("sudo apk add {}", package),
            Self::Emerge => format!("sudo emerge {}", package),
            Self::Nix => format!("nix-env -iA nixpkgs.{}", package),
            Self::Unknown => format!("# Install {}", package),
        }
    }

    /// Get command to search for a package
    pub fn search_command(&self, query: &str) -> String {
        match self {
            Self::Pacman => format!("pacman -Ss {}", query),
            Self::Apt => format!("apt search {}", query),
            Self::Dnf => format!("dnf search {}", query),
            Self::Zypper => format!("zypper search {}", query),
            Self::Apk => format!("apk search {}", query),
            Self::Emerge => format!("emerge -s {}", query),
            Self::Nix => format!("nix-env -qaP | grep {}", query),
            Self::Unknown => format!("# Search for {}", query),
        }
    }

    /// Get command to remove a package
    pub fn remove_command(&self, package: &str) -> String {
        match self {
            Self::Pacman => format!("sudo pacman -R {}", package),
            Self::Apt => format!("sudo apt remove {}", package),
            Self::Dnf => format!("sudo dnf remove {}", package),
            Self::Zypper => format!("sudo zypper remove {}", package),
            Self::Apk => format!("sudo apk del {}", package),
            Self::Emerge => format!("sudo emerge -C {}", package),
            Self::Nix => format!("nix-env -e {}", package),
            Self::Unknown => format!("# Remove {}", package),
        }
    }

    /// Get command to list installed packages
    pub fn list_installed_command(&self) -> &'static str {
        match self {
            Self::Pacman => "pacman -Q",
            Self::Apt => "apt list --installed",
            Self::Dnf => "dnf list installed",
            Self::Zypper => "zypper packages --installed-only",
            Self::Apk => "apk info",
            Self::Emerge => "qlist -I",
            Self::Nix => "nix-env -q",
            Self::Unknown => "# List installed packages",
        }
    }

    /// Get command to show package info
    pub fn info_command(&self, package: &str) -> String {
        match self {
            Self::Pacman => format!("pacman -Qi {}", package),
            Self::Apt => format!("apt show {}", package),
            Self::Dnf => format!("dnf info {}", package),
            Self::Zypper => format!("zypper info {}", package),
            Self::Apk => format!("apk info {}", package),
            Self::Emerge => format!("equery meta {}", package),
            Self::Nix => format!("nix-env -qa --description {}", package),
            Self::Unknown => format!("# Info for {}", package),
        }
    }

    /// Get command to clean package cache
    pub fn clean_cache_command(&self) -> &'static str {
        match self {
            Self::Pacman => "sudo pacman -Sc",
            Self::Apt => "sudo apt clean",
            Self::Dnf => "sudo dnf clean all",
            Self::Zypper => "sudo zypper clean",
            Self::Apk => "sudo apk cache clean",
            Self::Emerge => "sudo eclean-dist -d",
            Self::Nix => "nix-collect-garbage -d",
            Self::Unknown => "# Clean package cache",
        }
    }

    /// Get a friendly name for this package manager
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pacman => "pacman",
            Self::Apt => "APT",
            Self::Dnf => "DNF",
            Self::Zypper => "Zypper",
            Self::Apk => "apk",
            Self::Emerge => "Portage",
            Self::Nix => "Nix",
            Self::Unknown => "package manager",
        }
    }
}

/// Distro-aware context for generating advice
#[derive(Debug, Clone, Default)]
pub struct DistroContext {
    /// Raw distro name (e.g., "Arch Linux", "Ubuntu 22.04")
    pub distro: String,
    /// Detected package manager
    pub package_manager: Option<PackageManager>,
}

impl DistroContext {
    /// Create from distro name
    pub fn from_distro(distro: &str) -> Self {
        let pm = if distro.is_empty() {
            None
        } else {
            Some(PackageManager::from_distro(distro))
        };

        Self {
            distro: distro.to_string(),
            package_manager: pm,
        }
    }

    /// Get package manager or default to Unknown
    pub fn pm(&self) -> PackageManager {
        self.package_manager.unwrap_or(PackageManager::Unknown)
    }

    /// Check if we have valid distro info
    pub fn is_known(&self) -> bool {
        self.package_manager.map(|pm| pm != PackageManager::Unknown).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_arch() {
        assert_eq!(PackageManager::from_distro("Arch Linux"), PackageManager::Pacman);
        assert_eq!(PackageManager::from_distro("Manjaro Linux"), PackageManager::Pacman);
        assert_eq!(PackageManager::from_distro("EndeavourOS"), PackageManager::Pacman);
    }

    #[test]
    fn test_detect_debian() {
        assert_eq!(PackageManager::from_distro("Ubuntu 22.04"), PackageManager::Apt);
        assert_eq!(PackageManager::from_distro("Debian GNU/Linux 12"), PackageManager::Apt);
        assert_eq!(PackageManager::from_distro("Linux Mint 21"), PackageManager::Apt);
    }

    #[test]
    fn test_detect_fedora() {
        assert_eq!(PackageManager::from_distro("Fedora Linux 39"), PackageManager::Dnf);
        assert_eq!(PackageManager::from_distro("Rocky Linux 9"), PackageManager::Dnf);
    }

    #[test]
    fn test_install_command() {
        let pm = PackageManager::Pacman;
        assert_eq!(pm.install_command("vim"), "sudo pacman -S vim");

        let pm = PackageManager::Apt;
        assert_eq!(pm.install_command("vim"), "sudo apt install vim");
    }

    #[test]
    fn test_distro_context() {
        let ctx = DistroContext::from_distro("Arch Linux");
        assert!(ctx.is_known());
        assert_eq!(ctx.pm(), PackageManager::Pacman);

        let ctx = DistroContext::from_distro("");
        assert!(!ctx.is_known());
    }
}

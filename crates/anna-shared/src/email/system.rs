//! Email system utilities (v0.0.206).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Path to Anna's inbox file for async queries
/// Users can write questions here and Anna will process them
pub fn inbox_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".anna").join("inbox")
}

/// Legacy constant for backwards compatibility
pub const ANNA_EMAIL: &str = "anna@localhost";

/// Check if email system is available
pub fn is_email_available() -> bool {
    // Check for mail command
    Command::new("which")
        .arg("mail")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the package name for email support on this distro
pub fn email_package_name() -> &'static str {
    // Check for Arch
    if PathBuf::from("/etc/arch-release").exists() {
        return "s-nail"; // Arch Linux
    }
    // Check for Debian/Ubuntu
    if PathBuf::from("/etc/debian_version").exists() {
        return "mailutils";
    }
    // Check for Fedora/RHEL
    if PathBuf::from("/etc/fedora-release").exists()
        || PathBuf::from("/etc/redhat-release").exists()
    {
        return "mailx";
    }
    // Default
    "mailutils"
}

/// Install email package (returns command to run)
pub fn install_email_command() -> String {
    let pkg = email_package_name();

    // Detect package manager
    if PathBuf::from("/usr/bin/pacman").exists() {
        format!("sudo pacman -S --noconfirm {}", pkg)
    } else if PathBuf::from("/usr/bin/apt").exists() {
        format!("sudo apt install -y {}", pkg)
    } else if PathBuf::from("/usr/bin/dnf").exists() {
        format!("sudo dnf install -y {}", pkg)
    } else if PathBuf::from("/usr/bin/yum").exists() {
        format!("sudo yum install -y {}", pkg)
    } else {
        format!("# Install {} using your package manager", pkg)
    }
}

/// Count queries in inbox file (lines starting with "?")
pub fn count_inbox_queries(path: &PathBuf) -> usize {
    fs::read_to_string(path)
        .map(|content| {
            content
                .lines()
                .filter(|line| {
                    line.starts_with('?') || (!line.trim().is_empty() && !line.starts_with('#'))
                })
                .count()
        })
        .unwrap_or(0)
}

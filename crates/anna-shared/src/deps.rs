//! Tool dependency management for Anna.
//!
//! v0.0.909: Anna can install tools she needs and clean them up on uninstall.
//!
//! INVARIANT: Deps tracking is system-wide at /var/lib/anna/installed_deps.txt.

use crate::paths::paths;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the installed deps file (system-wide)
pub fn deps_file_path() -> PathBuf {
    paths().installed_deps_file()
}

/// Check if a command is available in PATH
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the package manager for the current system
pub fn detect_package_manager() -> Option<&'static str> {
    if command_exists("pacman") {
        Some("pacman")
    } else if command_exists("apt") {
        Some("apt")
    } else if command_exists("dnf") {
        Some("dnf")
    } else if command_exists("zypper") {
        Some("zypper")
    } else {
        None
    }
}

/// Install a package using the system package manager
/// Returns Ok(true) if installed, Ok(false) if already installed, Err on failure
pub fn install_package(package: &str) -> anyhow::Result<bool> {
    let pm = detect_package_manager().ok_or_else(|| {
        anyhow::anyhow!("No supported package manager found")
    })?;

    // Check if already installed
    if command_exists(package) {
        return Ok(false);
    }

    let result = match pm {
        "pacman" => Command::new("sudo")
            .args(["pacman", "-S", "--noconfirm", package])
            .output(),
        "apt" => Command::new("sudo")
            .args(["apt", "install", "-y", package])
            .output(),
        "dnf" => Command::new("sudo")
            .args(["dnf", "install", "-y", package])
            .output(),
        "zypper" => Command::new("sudo")
            .args(["zypper", "install", "-y", package])
            .output(),
        _ => return Err(anyhow::anyhow!("Unsupported package manager")),
    };

    match result {
        Ok(output) if output.status.success() => {
            // Record the installation
            record_installed_package(package)?;
            Ok(true)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Package installation failed: {}", stderr))
        }
        Err(e) => Err(anyhow::anyhow!("Failed to run package manager: {}", e)),
    }
}

/// Record a package as installed by Anna
fn record_installed_package(package: &str) -> anyhow::Result<()> {
    let path = deps_file_path();

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read existing packages
    let existing = read_installed_packages().unwrap_or_default();
    if existing.contains(&package.to_string()) {
        return Ok(()); // Already recorded
    }

    // Append the package
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    writeln!(file, "{}", package)?;
    Ok(())
}

/// Read the list of packages installed by Anna
pub fn read_installed_packages() -> anyhow::Result<Vec<String>> {
    let path = deps_file_path();
    if !path.exists() {
        return Ok(vec![]);
    }

    let file = fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let packages: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.trim().is_empty())
        .collect();

    Ok(packages)
}

/// Remove all packages that Anna installed
pub fn remove_installed_packages() -> anyhow::Result<Vec<String>> {
    let packages = read_installed_packages()?;
    if packages.is_empty() {
        return Ok(vec![]);
    }

    let pm = detect_package_manager().ok_or_else(|| {
        anyhow::anyhow!("No supported package manager found")
    })?;

    let mut removed = Vec::new();

    for package in &packages {
        let result = match pm {
            "pacman" => Command::new("sudo")
                .args(["pacman", "-Rs", "--noconfirm", package])
                .output(),
            "apt" => Command::new("sudo")
                .args(["apt", "remove", "-y", package])
                .output(),
            "dnf" => Command::new("sudo")
                .args(["dnf", "remove", "-y", package])
                .output(),
            "zypper" => Command::new("sudo")
                .args(["zypper", "remove", "-y", package])
                .output(),
            _ => continue,
        };

        if let Ok(output) = result {
            if output.status.success() {
                removed.push(package.clone());
            }
        }
    }

    // Remove the deps file
    let path = deps_file_path();
    if path.exists() {
        let _ = fs::remove_file(&path);
    }

    Ok(removed)
}

/// Common tools Anna might need for diagnostics
pub const DIAGNOSTIC_TOOLS: &[(&str, &str)] = &[
    ("bc", "Calculator for shell math"),
    ("jq", "JSON processor"),
    ("htop", "Interactive process viewer"),
    ("iotop", "I/O monitoring"),
    ("nethogs", "Per-process network usage"),
    ("lsof", "List open files"),
    ("strace", "System call tracer"),
    ("sysstat", "System performance tools (iostat, mpstat)"),
    ("smartmontools", "Disk health monitoring (smartctl)"),
    ("net-tools", "Network utilities (netstat)"),
];

/// Check which diagnostic tools are missing
pub fn missing_diagnostic_tools() -> Vec<(&'static str, &'static str)> {
    DIAGNOSTIC_TOOLS
        .iter()
        .filter(|(cmd, _)| !command_exists(cmd))
        .copied()
        .collect()
}

/// v0.0.924: Proactively install missing diagnostic tools
/// Returns the list of tools that were successfully installed
pub fn install_missing_diagnostic_tools() -> Vec<String> {
    let missing = missing_diagnostic_tools();
    if missing.is_empty() {
        return vec![];
    }

    let mut installed = Vec::new();

    for (tool, _desc) in missing {
        // Map tool name to package name (some differ)
        let package = match tool {
            "bc" => "bc",
            "jq" => "jq",
            "htop" => "htop",
            "iotop" => "iotop",
            "nethogs" => "nethogs",
            "lsof" => "lsof",
            "strace" => "strace",
            "sysstat" => "sysstat",
            "smartmontools" => "smartmontools",
            "net-tools" => "net-tools",
            _ => tool,
        };

        match install_package(package) {
            Ok(true) => {
                installed.push(tool.to_string());
            }
            Ok(false) => {
                // Already installed, just tool name differs from command
            }
            Err(_) => {
                // Failed to install, skip
            }
        }
    }

    installed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists() {
        // These should exist on any Linux system
        assert!(command_exists("ls"));
        assert!(command_exists("cat"));
        // This shouldn't exist
        assert!(!command_exists("nonexistent_command_xyz"));
    }

    #[test]
    fn test_detect_package_manager() {
        // Should detect something on Linux
        let pm = detect_package_manager();
        // Can't assert specific value as it depends on distro
        println!("Detected package manager: {:?}", pm);
    }
}

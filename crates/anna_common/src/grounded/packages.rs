//! Package Module v6.0 - Grounded in pacman
//!
//! Source of truth: pacman commands only
//! No invented data. No hallucinations.

use std::process::Command;

/// A package from pacman -Q
#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub install_reason: InstallReason,
    pub source: PackageSource,
    pub installed_size: u64,
    pub install_date: String,
    pub config_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallReason {
    Explicit,    // User installed
    Dependency,  // Auto-installed as dependency
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageSource {
    Official,    // From official repos
    Aur,         // Foreign/AUR package
    Unknown,
}

/// Package counts - all from real pacman queries
#[derive(Debug, Clone, Default)]
pub struct PackageCounts {
    pub total: usize,
    pub explicit: usize,
    pub dependency: usize,
    pub aur: usize,
}

impl PackageCounts {
    /// Get real counts from pacman
    /// Source: pacman -Q, pacman -Qe, pacman -Qd, pacman -Qm
    pub fn query() -> Self {
        Self {
            total: count_packages("pacman -Q"),
            explicit: count_packages("pacman -Qe"),
            dependency: count_packages("pacman -Qd"),
            aur: count_packages("pacman -Qm"),
        }
    }
}

/// Count packages from a pacman command
fn count_packages(cmd: &str) -> usize {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return 0;
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .count()
        }
        _ => 0,
    }
}

/// Get package info from pacman -Qi
/// Returns None if package not installed
pub fn get_package_info(name: &str) -> Option<Package> {
    let output = Command::new("pacman")
        .args(["-Qi", name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_pacman_qi(&stdout, name)
}

/// Parse pacman -Qi output
fn parse_pacman_qi(output: &str, name: &str) -> Option<Package> {
    let mut pkg = Package {
        name: name.to_string(),
        version: String::new(),
        description: String::new(),
        install_reason: InstallReason::Unknown,
        source: PackageSource::Unknown,
        installed_size: 0,
        install_date: String::new(),
        config_files: Vec::new(),
    };

    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => pkg.name = value.to_string(),
                "Version" => pkg.version = value.to_string(),
                "Description" => pkg.description = value.to_string(),
                "Install Reason" => {
                    pkg.install_reason = if value.contains("Explicitly") {
                        InstallReason::Explicit
                    } else if value.contains("dependency") {
                        InstallReason::Dependency
                    } else {
                        InstallReason::Unknown
                    };
                }
                "Installed Size" => {
                    pkg.installed_size = parse_size(value);
                }
                "Install Date" => pkg.install_date = value.to_string(),
                _ => {}
            }
        }
    }

    // Check if AUR/foreign
    pkg.source = if is_foreign_package(&pkg.name) {
        PackageSource::Aur
    } else {
        PackageSource::Official
    };

    // Get config files
    pkg.config_files = get_package_config_files(&pkg.name);

    if pkg.version.is_empty() {
        None
    } else {
        Some(pkg)
    }
}

/// Parse size string like "1.5 MiB" to bytes
fn parse_size(s: &str) -> u64 {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 2 {
        return 0;
    }

    let num: f64 = parts[0].parse().unwrap_or(0.0);
    let unit = parts[1].to_uppercase();

    let multiplier = match unit.as_str() {
        "B" => 1,
        "KIB" | "KB" => 1024,
        "MIB" | "MB" => 1024 * 1024,
        "GIB" | "GB" => 1024 * 1024 * 1024,
        _ => 1,
    };

    (num * multiplier as f64) as u64
}

/// Check if package is foreign (AUR)
/// Source: pacman -Qm
fn is_foreign_package(name: &str) -> bool {
    let output = Command::new("pacman")
        .args(["-Qm", name])
        .output();

    matches!(output, Ok(out) if out.status.success())
}

/// Get config files for a package
/// Source: pacman -Ql filtered to /etc/
pub fn get_package_config_files(name: &str) -> Vec<String> {
    let output = Command::new("pacman")
        .args(["-Ql", name])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        let path = parts[1];
                        // Only return files in /etc/
                        if path.starts_with("/etc/") && !path.ends_with('/') {
                            return Some(path.to_string());
                        }
                    }
                    None
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Get which package owns a file
/// Source: pacman -Qo
pub fn get_owning_package(path: &str) -> Option<String> {
    let output = Command::new("pacman")
        .args(["-Qo", path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Output format: "/usr/bin/vim is owned by vim 9.1.0-1"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.split("is owned by").collect();
    if parts.len() == 2 {
        let pkg_ver = parts[1].trim();
        // Extract just the package name (before version)
        pkg_ver.split_whitespace().next().map(|s| s.to_string())
    } else {
        None
    }
}

/// List all installed packages
/// Source: pacman -Q
pub fn list_installed_packages() -> Vec<(String, String)> {
    let output = Command::new("pacman")
        .args(["-Q"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_counts() {
        let counts = PackageCounts::query();
        // Should have some packages on any Arch system
        assert!(counts.total > 0);
        // Explicit + dependency should roughly equal total
        // (some edge cases exist)
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024 B"), 1024);
        assert_eq!(parse_size("1 KiB"), 1024);
        assert_eq!(parse_size("1 MiB"), 1024 * 1024);
        assert_eq!(parse_size("1.5 MiB"), (1.5 * 1024.0 * 1024.0) as u64);
    }
}

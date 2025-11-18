use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Orphaned packages detection and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedPackages {
    pub total_orphans: u32,
    pub total_size_mb: f64,
    pub orphan_packages: Vec<OrphanPackage>,
    pub removal_safe: bool,
    pub removal_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanPackage {
    pub name: String,
    pub version: String,
    pub install_size_kb: u64,
    pub install_size_mb: f64,
    pub install_date: Option<DateTime<Utc>>,
    pub description: String,
    pub install_reason: InstallReason,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallReason {
    Explicit,   // Explicitly installed
    Dependency, // Installed as dependency
    Orphan,     // Was dependency, no longer required
}

impl OrphanedPackages {
    /// Detect orphaned packages
    pub fn detect() -> Self {
        let orphan_packages = detect_orphaned_packages();
        let total_orphans = orphan_packages.len() as u32;

        let total_size_mb: f64 = orphan_packages.iter().map(|p| p.install_size_mb).sum();

        // Determine if removal is safe
        // Orphaned packages are generally safe to remove, but we'll check if any are critical
        let has_critical_packages = orphan_packages.iter().any(|p| is_critical_package(&p.name));
        let removal_safe = !has_critical_packages;

        // Generate removal recommendations
        let mut removal_recommendations = Vec::new();

        if total_orphans > 0 {
            removal_recommendations.push(format!(
                "Found {} orphaned package(s) using {:.2} MB",
                total_orphans, total_size_mb
            ));

            if removal_safe {
                removal_recommendations.push(
                    "Run 'sudo pacman -Rns $(pacman -Qtdq)' to remove orphaned packages"
                        .to_string(),
                );
            } else {
                removal_recommendations.push(
                    "Some orphaned packages may be critical - review before removal".to_string(),
                );
            }

            // Recommend reviewing packages over 100 MB
            let large_orphans: Vec<&OrphanPackage> = orphan_packages
                .iter()
                .filter(|p| p.install_size_mb > 100.0)
                .collect();

            if !large_orphans.is_empty() {
                removal_recommendations.push(format!(
                    "{} orphaned package(s) over 100 MB - consider removal to free space",
                    large_orphans.len()
                ));
            }
        } else {
            removal_recommendations
                .push("No orphaned packages found - system is clean".to_string());
        }

        Self {
            total_orphans,
            total_size_mb,
            orphan_packages,
            removal_safe,
            removal_recommendations,
        }
    }
}

fn detect_orphaned_packages() -> Vec<OrphanPackage> {
    let mut packages = Vec::new();

    // Run pacman -Qtd to get orphaned packages (dependencies no longer required)
    let output = Command::new("pacman").args(&["-Qtd"]).output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                // Parse "package-name version" format
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let version = parts[1].to_string();

                    // Get package details
                    if let Some(package_info) = get_package_info(&name) {
                        packages.push(OrphanPackage {
                            name,
                            version,
                            install_size_kb: package_info.install_size_kb,
                            install_size_mb: package_info.install_size_kb as f64 / 1024.0,
                            install_date: package_info.install_date,
                            description: package_info.description,
                            install_reason: InstallReason::Orphan,
                        });
                    } else {
                        // Fallback if we can't get detailed info
                        packages.push(OrphanPackage {
                            name,
                            version,
                            install_size_kb: 0,
                            install_size_mb: 0.0,
                            install_date: None,
                            description: String::new(),
                            install_reason: InstallReason::Orphan,
                        });
                    }
                }
            }
        }
    }

    // Sort by size (largest first)
    packages.sort_by(|a, b| b.install_size_kb.cmp(&a.install_size_kb));

    packages
}

struct PackageInfo {
    install_size_kb: u64,
    install_date: Option<DateTime<Utc>>,
    description: String,
}

fn get_package_info(package_name: &str) -> Option<PackageInfo> {
    let output = Command::new("pacman")
        .args(&["-Qi", package_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut install_size_kb = 0u64;
    let mut install_date = None;
    let mut description = String::new();

    for line in stdout.lines() {
        if line.starts_with("Installed Size") {
            // Parse "Installed Size      : 123.45 MiB" or "Installed Size      : 456.78 KiB"
            if let Some(size_part) = line.split(':').nth(1) {
                let size_str = size_part.trim();
                let parts: Vec<&str> = size_str.split_whitespace().collect();

                if parts.len() >= 2 {
                    if let Ok(size) = parts[0].parse::<f64>() {
                        let unit = parts[1];
                        install_size_kb = match unit {
                            "MiB" => (size * 1024.0) as u64,
                            "KiB" => size as u64,
                            "GiB" => (size * 1024.0 * 1024.0) as u64,
                            "B" => (size / 1024.0) as u64,
                            _ => 0,
                        };
                    }
                }
            }
        } else if line.starts_with("Install Date") {
            // Parse "Install Date        : Mon 01 Jan 2024 12:00:00 PM UTC"
            if let Some(date_part) = line.split(':').nth(1) {
                let date_str = date_part.trim();

                // Try to parse with multiple date formats
                // pacman uses format like "Tue 15 Oct 2024 10:30:45 AM PDT"
                if let Ok(naive_dt) =
                    NaiveDateTime::parse_from_str(date_str, "%a %d %b %Y %I:%M:%S %p %Z")
                {
                    install_date = Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
                } else if let Ok(naive_dt) =
                    NaiveDateTime::parse_from_str(date_str, "%a %d %b %Y %H:%M:%S %Z")
                {
                    install_date = Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
                }
            }
        } else if line.starts_with("Description") {
            if let Some(desc_part) = line.split(':').nth(1) {
                description = desc_part.trim().to_string();
            }
        }
    }

    Some(PackageInfo {
        install_size_kb,
        install_date,
        description,
    })
}

fn is_critical_package(package_name: &str) -> bool {
    // List of package name patterns that should not be auto-removed
    let critical_patterns = [
        "linux",
        "kernel",
        "systemd",
        "glibc",
        "gcc",
        "bash",
        "pacman",
        "filesystem",
        "base",
    ];

    critical_patterns
        .iter()
        .any(|pattern| package_name.contains(pattern))
}

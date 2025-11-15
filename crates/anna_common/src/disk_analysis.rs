//! Real Disk Space Analysis
//!
//! This module provides ACTUAL disk space analysis, not token "check if low" probes.
//! It identifies what's consuming space and provides actionable recommendations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents disk usage for a specific directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub size_human: String,
    pub category: DiskCategory,
}

/// Categories of disk consumers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiskCategory {
    Containers,    // Docker, Podman
    Builds,        // Build artifacts, cargo target/, etc
    PackageCache,  // Pacman cache
    Logs,          // System logs
    Downloads,     // ~/Downloads
    VirtualMachines, // VMs, ISOs
    Other,
}

impl DiskCategory {
    pub fn icon(&self) -> &'static str {
        match self {
            DiskCategory::Containers => "🐳",
            DiskCategory::Builds => "🏗️",
            DiskCategory::PackageCache => "📦",
            DiskCategory::Logs => "📝",
            DiskCategory::Downloads => "⬇️",
            DiskCategory::VirtualMachines => "💿",
            DiskCategory::Other => "📁",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DiskCategory::Containers => "Containers",
            DiskCategory::Builds => "Builds",
            DiskCategory::PackageCache => "Packages",
            DiskCategory::Logs => "Logs",
            DiskCategory::Downloads => "Downloads",
            DiskCategory::VirtualMachines => "VMs",
            DiskCategory::Other => "Other",
        }
    }
}

/// Result of disk analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskAnalysis {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
    pub mount_point: PathBuf,
    pub top_consumers: Vec<DiskUsage>,
}

impl DiskAnalysis {
    /// Analyze disk usage for the root partition
    pub fn analyze_root() -> Result<Self> {
        let mount_point = PathBuf::from("/");

        // Get filesystem stats using df
        let (total, used, available) = get_filesystem_stats(&mount_point)?;
        let usage_percent = (used as f64 / total as f64) * 100.0;

        // Analyze top consumers
        let mut consumers = Vec::new();

        // Check common large directories
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());

        // Container data
        if let Ok(size) = get_dir_size(&format!("{}/.local/share/containers", home)) {
            consumers.push(DiskUsage {
                path: format!("{}/.local/share/containers", home).into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::Containers,
            });
        }
        if let Ok(size) = get_dir_size("/var/lib/docker") {
            consumers.push(DiskUsage {
                path: "/var/lib/docker".into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::Containers,
            });
        }

        // Build artifacts
        if let Ok(size) = get_dir_size(&format!("{}/builds", home)) {
            consumers.push(DiskUsage {
                path: format!("{}/builds", home).into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::Builds,
            });
        }
        if let Ok(size) = get_dir_size(&format!("{}/.cargo/registry", home)) {
            consumers.push(DiskUsage {
                path: format!("{}/.cargo/registry", home).into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::Builds,
            });
        }

        // Package cache
        if let Ok(size) = get_dir_size("/var/cache/pacman/pkg") {
            consumers.push(DiskUsage {
                path: "/var/cache/pacman/pkg".into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::PackageCache,
            });
        }

        // Logs
        if let Ok(size) = get_dir_size("/var/log") {
            consumers.push(DiskUsage {
                path: "/var/log".into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::Logs,
            });
        }

        // Downloads
        if let Ok(size) = get_dir_size(&format!("{}/Downloads", home)) {
            consumers.push(DiskUsage {
                path: format!("{}/Downloads", home).into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::Downloads,
            });
        }

        // Virtual machines
        if let Ok(size) = get_dir_size(&format!("{}/.local/share/libvirt", home)) {
            consumers.push(DiskUsage {
                path: format!("{}/.local/share/libvirt", home).into(),
                size_bytes: size,
                size_human: format_size(size),
                category: DiskCategory::VirtualMachines,
            });
        }

        // Sort by size, largest first
        consumers.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

        // Keep only top 5
        consumers.truncate(5);

        Ok(DiskAnalysis {
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percent,
            mount_point,
            top_consumers: consumers,
        })
    }

    /// Get recommendations based on analysis
    pub fn get_recommendations(&self) -> Vec<DiskRecommendation> {
        let mut recommendations = Vec::new();

        // Group consumers by category
        let mut by_category: HashMap<DiskCategory, Vec<&DiskUsage>> = HashMap::new();
        for consumer in &self.top_consumers {
            by_category.entry(consumer.category).or_default().push(consumer);
        }

        // Containers
        if let Some(containers) = by_category.get(&DiskCategory::Containers) {
            let total_size: u64 = containers.iter().map(|c| c.size_bytes).sum();
            if total_size > 10 * 1024 * 1024 * 1024 { // > 10GB
                recommendations.push(DiskRecommendation::clean_containers(total_size));
            }
        }

        // Package cache
        if let Some(packages) = by_category.get(&DiskCategory::PackageCache) {
            let total_size: u64 = packages.iter().map(|c| c.size_bytes).sum();
            if total_size > 5 * 1024 * 1024 * 1024 { // > 5GB
                recommendations.push(DiskRecommendation::clean_package_cache(total_size));
            }
        }

        // Logs
        if let Some(logs) = by_category.get(&DiskCategory::Logs) {
            let total_size: u64 = logs.iter().map(|c| c.size_bytes).sum();
            if total_size > 1024 * 1024 * 1024 { // > 1GB
                recommendations.push(DiskRecommendation::clean_logs(total_size));
            }
        }

        // Builds
        if let Some(builds) = by_category.get(&DiskCategory::Builds) {
            let total_size: u64 = builds.iter().map(|c| c.size_bytes).sum();
            if total_size > 5 * 1024 * 1024 * 1024 { // > 5GB
                recommendations.push(DiskRecommendation::review_builds(total_size, builds));
            }
        }

        recommendations
    }
}

/// A specific recommendation for freeing disk space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskRecommendation {
    pub title: String,
    pub command: Option<String>,
    pub explanation: String,
    pub warning: Option<String>,
    pub estimated_savings_bytes: u64,
    pub estimated_savings_human: String,
    pub risk_level: RecommendationRisk,
    pub wiki_url: String,
    pub wiki_section: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationRisk {
    Safe,      // Can't break anything
    Low,       // Unlikely to cause issues
    Medium,    // Requires understanding what you're doing
    High,      // Could break things
}

impl DiskRecommendation {
    fn clean_containers(estimated_savings: u64) -> Self {
        DiskRecommendation {
            title: "Clean container images".to_string(),
            command: Some("podman system prune -a && docker system prune -a".to_string()),
            explanation: "Removes unused container images and stopped containers from both Podman and Docker. This is safe and will free up significant space.".to_string(),
            warning: Some("This removes ALL unused container data".to_string()),
            estimated_savings_bytes: estimated_savings,
            estimated_savings_human: format_size(estimated_savings),
            risk_level: RecommendationRisk::Low,
            wiki_url: "https://wiki.archlinux.org/title/Podman".to_string(),
            wiki_section: Some("Pruning".to_string()),
        }
    }

    fn clean_package_cache(estimated_savings: u64) -> Self {
        DiskRecommendation {
            title: "Clean package cache".to_string(),
            command: Some("sudo paccache -rk1".to_string()),
            explanation: "Keeps only the latest version of each installed package. This is safe and recommended for regular maintenance. You can always re-download packages if needed.".to_string(),
            warning: None,
            estimated_savings_bytes: estimated_savings,
            estimated_savings_human: format_size(estimated_savings),
            risk_level: RecommendationRisk::Safe,
            wiki_url: "https://wiki.archlinux.org/title/Pacman".to_string(),
            wiki_section: Some("Cleaning_the_package_cache".to_string()),
        }
    }

    fn clean_logs(estimated_savings: u64) -> Self {
        DiskRecommendation {
            title: "Clean system logs".to_string(),
            command: Some("sudo journalctl --vacuum-size=100M".to_string()),
            explanation: "Reduces journal logs to 100MB. Older logs are rarely needed and can be safely removed. The journal will continue working normally after this.".to_string(),
            warning: None,
            estimated_savings_bytes: estimated_savings,
            estimated_savings_human: format_size(estimated_savings),
            risk_level: RecommendationRisk::Safe,
            wiki_url: "https://wiki.archlinux.org/title/Systemd/Journal".to_string(),
            wiki_section: Some("Journal_size_limit".to_string()),
        }
    }

    fn review_builds(estimated_savings: u64, paths: &[&DiskUsage]) -> Self {
        let path_list = paths.iter()
            .map(|p| p.path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        DiskRecommendation {
            title: "Review build artifacts".to_string(),
            command: Some(format!("ncdu {}", paths[0].path.display())),
            explanation: format!("Use ncdu to interactively explore build directories: {}. Navigate with arrow keys, delete with 'd'. You decide what to keep.", path_list),
            warning: Some("Make sure you don't delete active projects".to_string()),
            estimated_savings_bytes: estimated_savings,
            estimated_savings_human: format_size(estimated_savings),
            risk_level: RecommendationRisk::Medium,
            wiki_url: "https://wiki.archlinux.org/title/List_of_applications/Utilities".to_string(),
            wiki_section: Some("Disk_usage_display".to_string()),
        }
    }
}

/// Get filesystem statistics using df
fn get_filesystem_stats(mount_point: &Path) -> Result<(u64, u64, u64)> {
    let output = Command::new("df")
        .args(["--block-size=1", mount_point.to_str().unwrap()])
        .output()
        .context("Failed to run df command")?;

    if !output.status.success() {
        anyhow::bail!("df command failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        anyhow::bail!("Unexpected df output");
    }

    // Parse the second line (first line is header)
    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 4 {
        anyhow::bail!("Could not parse df output");
    }

    let total = parts[1].parse::<u64>().context("Failed to parse total")?;
    let used = parts[2].parse::<u64>().context("Failed to parse used")?;
    let available = parts[3].parse::<u64>().context("Failed to parse available")?;

    Ok((total, used, available))
}

/// Get size of a directory using du
fn get_dir_size(path: &str) -> Result<u64> {
    // Check if path exists first
    if !Path::new(path).exists() {
        return Ok(0);
    }

    let output = Command::new("du")
        .args(["-sb", path])
        .output()
        .context("Failed to run du command")?;

    if !output.status.success() {
        return Ok(0); // Directory might not be readable, return 0
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let size_str = stdout.split_whitespace().next()
        .context("Could not parse du output")?;

    size_str.parse::<u64>().context("Failed to parse size")
}

/// Format bytes as human-readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1}TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024), "1.0KB");
        assert_eq!(format_size(1024 * 1024), "1.0MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0GB");
    }

    #[test]
    fn test_disk_analysis() {
        // This will actually analyze the system, so just verify it doesn't crash
        let result = DiskAnalysis::analyze_root();
        assert!(result.is_ok());
    }
}

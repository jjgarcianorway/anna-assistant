// Anna v0.12.3 - Package Analysis Collector for Arch Linux
// Analyzes installed packages, orphans, AUR packages, and system groups

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Package inventory matching schema v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInventory {
    pub version: String,
    pub generated_at: String,
    pub total_packages: usize,
    pub explicit_packages: usize,
    pub orphans: Vec<String>,
    pub aur_packages: Vec<String>,
    pub groups: PackageGroups,
    pub recent_events: Vec<PackageEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageGroups {
    pub base: Vec<String>,
    pub base_devel: Vec<String>,
    pub xorg: Vec<String>,
    pub multimedia: Vec<String>,
    pub nvidia: Vec<String>,
    pub vulkan: Vec<String>,
    pub cuda: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEvent {
    pub timestamp: String,
    pub action: String, // installed, upgraded, removed
    pub package: String,
    pub version: Option<String>,
}

/// Package analysis collector
pub struct PackageCollector {
    timeout_per_cmd: Duration,
}

impl Default for PackageCollector {
    fn default() -> Self {
        Self {
            timeout_per_cmd: Duration::from_secs(2),
        }
    }
}

impl PackageCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Collect package inventory
    pub async fn collect(&self) -> Result<PackageInventory> {
        let start = std::time::Instant::now();

        // Get all installed packages
        let all_packages = self.get_all_packages().await?;
        let explicit_packages = self.get_explicit_packages().await?;

        // Get orphans
        let orphans = self.get_orphan_packages().await.unwrap_or_else(|e| {
            warn!("Failed to get orphan packages: {}", e);
            Vec::new()
        });

        // Detect AUR packages (packages not in official repos)
        let aur_packages = self.detect_aur_packages(&all_packages).await.unwrap_or_else(|e| {
            warn!("Failed to detect AUR packages: {}", e);
            Vec::new()
        });

        // Analyze package groups
        let groups = self.analyze_groups(&all_packages).await;

        // Get recent package events
        let recent_events = self.parse_pacman_log(30).await.unwrap_or_else(|e| {
            debug!("Failed to parse pacman log: {}", e);
            Vec::new()
        });

        let elapsed = start.elapsed();
        info!("Package inventory collected in {} ms", elapsed.as_millis());

        Ok(PackageInventory {
            version: "1".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            total_packages: all_packages.len(),
            explicit_packages: explicit_packages.len(),
            orphans,
            aur_packages,
            groups,
            recent_events,
        })
    }

    /// Get all installed packages
    async fn get_all_packages(&self) -> Result<Vec<String>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("pacman")
                .args(["-Qq"])
                .output(),
        )
        .await
        .context("timeout")??;

        if !output.status.success() {
            anyhow::bail!("pacman -Qq failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
    }

    /// Get explicitly installed packages
    async fn get_explicit_packages(&self) -> Result<Vec<String>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("pacman")
                .args(["-Qeq"])
                .output(),
        )
        .await
        .context("timeout")??;

        if !output.status.success() {
            anyhow::bail!("pacman -Qeq failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
    }

    /// Get orphan packages (installed as deps but no longer needed)
    async fn get_orphan_packages(&self) -> Result<Vec<String>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("pacman")
                .args(["-Qtdq"])
                .output(),
        )
        .await
        .context("timeout")??;

        // Exit code 1 means no orphans found, which is OK
        if output.status.code() == Some(1) {
            return Ok(Vec::new());
        }

        if !output.status.success() {
            anyhow::bail!("pacman -Qtdq failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.trim().to_string()).collect())
    }

    /// Detect AUR packages by checking if they're in official repos
    async fn detect_aur_packages(&self, all_packages: &[String]) -> Result<Vec<String>> {
        let mut aur_packages = Vec::new();

        // Sample up to 100 packages to check (full check would be too slow)
        let sample_size = all_packages.len().min(100);

        for pkg in all_packages.iter().take(sample_size) {
            // Check if package is in official repos
            if let Ok(output) = timeout(
                Duration::from_millis(500),
                tokio::process::Command::new("pacman")
                    .args(["-Si", pkg])
                    .output(),
            )
            .await
            {
                if let Ok(output) = output {
                    if !output.status.success() {
                        // Not in repos, likely AUR
                        aur_packages.push(pkg.clone());
                    }
                }
            }
        }

        Ok(aur_packages)
    }

    /// Analyze installed packages by common groups
    async fn analyze_groups(&self, all_packages: &[String]) -> PackageGroups {
        let pkg_set: HashSet<&str> = all_packages.iter().map(|s| s.as_str()).collect();

        // Base group
        let base_packages = vec!["filesystem", "gcc-libs", "glibc", "bash", "coreutils"];
        let base: Vec<String> = base_packages
            .iter()
            .filter(|p| pkg_set.contains(*p))
            .map(|s| s.to_string())
            .collect();

        // Base-devel group
        let base_devel_packages = vec!["base-devel", "gcc", "make", "cmake", "git"];
        let base_devel: Vec<String> = base_devel_packages
            .iter()
            .filter(|p| pkg_set.contains(*p))
            .map(|s| s.to_string())
            .collect();

        // Xorg group
        let xorg_packages = vec!["xorg-server", "xorg-xinit", "xf86-video-intel", "xf86-video-amdgpu"];
        let xorg: Vec<String> = xorg_packages
            .iter()
            .filter(|p| pkg_set.contains(*p))
            .map(|s| s.to_string())
            .collect();

        // Multimedia
        let multimedia_packages = vec!["ffmpeg", "gstreamer", "pulseaudio", "pipewire"];
        let multimedia: Vec<String> = multimedia_packages
            .iter()
            .filter(|p| pkg_set.contains(*p))
            .map(|s| s.to_string())
            .collect();

        // NVIDIA
        let nvidia_packages = vec!["nvidia", "nvidia-utils", "nvidia-settings", "nvidia-dkms"];
        let nvidia: Vec<String> = nvidia_packages
            .iter()
            .filter(|p| pkg_set.contains(*p))
            .map(|s| s.to_string())
            .collect();

        // Vulkan
        let vulkan_packages = vec!["vulkan-icd-loader", "lib32-vulkan-icd-loader", "vulkan-tools"];
        let vulkan: Vec<String> = vulkan_packages
            .iter()
            .filter(|p| pkg_set.contains(*p))
            .map(|s| s.to_string())
            .collect();

        // CUDA
        let cuda_packages = vec!["cuda", "cudnn", "tensorrt"];
        let cuda: Vec<String> = cuda_packages
            .iter()
            .filter(|p| pkg_set.contains(*p))
            .map(|s| s.to_string())
            .collect();

        PackageGroups {
            base,
            base_devel,
            xorg,
            multimedia,
            nvidia,
            vulkan,
            cuda,
        }
    }

    /// Parse recent events from /var/log/pacman.log
    async fn parse_pacman_log(&self, days: u32) -> Result<Vec<PackageEvent>> {
        let log_path = "/var/log/pacman.log";
        if !std::path::Path::new(log_path).exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(log_path)?;
        let mut events = Vec::new();

        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::days(days as i64);

        for line in content.lines().rev().take(1000) {
            // Format: [2025-01-15T12:34:56-0700] [ALPM] installed linux (6.17.6-arch1-1)
            if let Some(event) = parse_log_line(line) {
                if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&event.timestamp) {
                    if ts.with_timezone(&chrono::Utc) < cutoff {
                        break; // Past our cutoff
                    }
                    events.push(event);
                }
            }
        }

        events.reverse(); // Chronological order
        Ok(events)
    }
}

/// Parse a single pacman log line
fn parse_log_line(line: &str) -> Option<PackageEvent> {
    // Format: [2025-01-15T12:34:56-0700] [ALPM] installed linux (6.17.6-arch1-1)
    if !line.contains("[ALPM]") {
        return None;
    }

    let parts: Vec<&str> = line.split(']').collect();
    if parts.len() < 3 {
        return None;
    }

    let timestamp = parts[0].trim_start_matches('[').to_string();
    let message = parts[2].trim();

    let action = if message.contains("installed") {
        "installed"
    } else if message.contains("upgraded") {
        "upgraded"
    } else if message.contains("removed") {
        "removed"
    } else {
        return None;
    };

    // Extract package name and version
    let tokens: Vec<&str> = message.split_whitespace().collect();
    if tokens.len() < 2 {
        return None;
    }

    let package = tokens[1].to_string();
    let version = if tokens.len() >= 3 {
        Some(tokens[2].trim_matches(|c| c == '(' || c == ')').to_string())
    } else {
        None
    };

    Some(PackageEvent {
        timestamp,
        action: action.to_string(),
        package,
        version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line_installed() {
        let line = "[2025-01-15T12:34:56-0700] [ALPM] installed linux (6.17.6-arch1-1)";
        let event = parse_log_line(line).unwrap();
        assert_eq!(event.action, "installed");
        assert_eq!(event.package, "linux");
        assert_eq!(event.version, Some("6.17.6-arch1-1".to_string()));
    }

    #[test]
    fn test_parse_log_line_upgraded() {
        let line = "[2025-01-15T12:34:56-0700] [ALPM] upgraded nvidia (545.29.06-1 -> 550.54.14-1)";
        let event = parse_log_line(line).unwrap();
        assert_eq!(event.action, "upgraded");
        assert_eq!(event.package, "nvidia");
    }

    #[test]
    fn test_parse_log_line_invalid() {
        let line = "[2025-01-15T12:34:56-0700] [PACMAN] Running 'pacman -Syu'";
        assert!(parse_log_line(line).is_none());
    }
}

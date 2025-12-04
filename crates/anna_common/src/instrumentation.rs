//! Anna Instrumentation Manifest v7.26.0
//!
//! Tracks tools auto-installed by Anna for system probing.
//! Manifest stored in /var/lib/anna/instrumentation.json
//!
//! Features:
//! - Tool tracking: name, version, install time, reason
//! - Rate limiting: 1 install per 24 hours (configurable)
//! - Install logging: all installs logged to ops_log
//! - AUR gate: AUR packages blocked unless explicitly enabled

use crate::config::{AnnaConfig, DATA_DIR};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Path to the instrumentation manifest
pub const INSTRUMENTATION_FILE: &str = "instrumentation.json";

/// An installed tool tracked by Anna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledTool {
    /// Package name (e.g., "lm_sensors")
    pub package: String,

    /// Installed version
    pub version: String,

    /// When this tool was installed (RFC3339)
    pub installed_at: DateTime<Utc>,

    /// Why Anna installed it (e.g., "hardware monitoring", "disk health")
    pub reason: String,

    /// Which metrics this tool enables (e.g., ["cpu_temp", "fan_speed"])
    pub metrics_enabled: Vec<String>,

    /// Whether this is optional (true) or required for core functionality (false)
    pub optional: bool,

    /// Source: "official" or "aur"
    pub source: String,
}

/// An available tool Anna could install
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableTool {
    /// Package name
    pub package: String,

    /// What metrics it enables
    pub metrics_enabled: Vec<String>,

    /// Why Anna would need it
    pub reason: String,

    /// Whether optional
    pub optional: bool,

    /// Source: "official" or "aur"
    pub source: String,

    /// Whether currently blocked by AUR gate
    pub blocked_by_aur_gate: bool,
}

/// Install attempt record for rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallAttempt {
    /// When the attempt was made
    pub attempted_at: DateTime<Utc>,

    /// Package name
    pub package: String,

    /// Whether it succeeded
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

/// The instrumentation manifest
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstrumentationManifest {
    /// Schema version
    pub version: u32,

    /// Tools currently installed by Anna
    pub installed: HashMap<String, InstalledTool>,

    /// Recent install attempts (for rate limiting)
    pub recent_attempts: Vec<InstallAttempt>,

    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
}

impl InstrumentationManifest {
    const CURRENT_VERSION: u32 = 1;

    /// Load manifest from disk
    pub fn load() -> Self {
        let path = Self::manifest_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(manifest) = serde_json::from_str(&content) {
                    return manifest;
                }
            }
        }
        Self {
            version: Self::CURRENT_VERSION,
            ..Default::default()
        }
    }

    /// Save manifest to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::manifest_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let path_str = path
            .to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(path_str, &content)
    }

    /// Path to the manifest file
    pub fn manifest_path() -> PathBuf {
        PathBuf::from(DATA_DIR).join(INSTRUMENTATION_FILE)
    }

    /// Record a tool installation
    pub fn record_install(&mut self, tool: InstalledTool) {
        self.installed.insert(tool.package.clone(), tool);
        self.last_updated = Some(Utc::now());
    }

    /// Record an install attempt (for rate limiting)
    pub fn record_attempt(&mut self, package: &str, success: bool, error: Option<String>) {
        self.recent_attempts.push(InstallAttempt {
            attempted_at: Utc::now(),
            package: package.to_string(),
            success,
            error,
        });
        // Keep only last 24 hours of attempts
        self.prune_old_attempts();
        self.last_updated = Some(Utc::now());
    }

    /// Prune attempts older than 24 hours
    fn prune_old_attempts(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        self.recent_attempts.retain(|a| a.attempted_at > cutoff);
    }

    /// Check if we're rate limited (max installs per 24h)
    pub fn is_rate_limited(&self, config: &AnnaConfig) -> bool {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        let recent_count = self
            .recent_attempts
            .iter()
            .filter(|a| a.attempted_at > cutoff && a.success)
            .count();
        recent_count >= config.instrumentation.max_installs_per_day as usize
    }

    /// Get time until rate limit resets
    pub fn rate_limit_reset_time(&self) -> Option<DateTime<Utc>> {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        self.recent_attempts
            .iter()
            .filter(|a| a.attempted_at > cutoff && a.success)
            .map(|a| a.attempted_at + chrono::Duration::hours(24))
            .min()
    }

    /// Check if a package is already installed by Anna
    pub fn is_installed(&self, package: &str) -> bool {
        self.installed.contains_key(package)
    }

    /// Get installed tool info
    pub fn get_installed(&self, package: &str) -> Option<&InstalledTool> {
        self.installed.get(package)
    }

    /// Get all installed tools
    pub fn installed_tools(&self) -> impl Iterator<Item = &InstalledTool> {
        self.installed.values()
    }

    /// Get count of installed tools
    pub fn installed_count(&self) -> usize {
        self.installed.len()
    }
}

/// Known tools that Anna can auto-install
pub fn get_known_tools() -> Vec<AvailableTool> {
    vec![
        AvailableTool {
            package: "lm_sensors".to_string(),
            metrics_enabled: vec!["cpu_temp".into(), "fan_speed".into(), "voltage".into()],
            reason: "Hardware temperature and fan monitoring".to_string(),
            optional: false,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "smartmontools".to_string(),
            metrics_enabled: vec!["disk_health".into(), "smart_attrs".into()],
            reason: "SATA/SAS disk health monitoring".to_string(),
            optional: false,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "nvme-cli".to_string(),
            metrics_enabled: vec!["nvme_health".into(), "nvme_temp".into()],
            reason: "NVMe SSD health and temperature".to_string(),
            optional: false,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "ethtool".to_string(),
            metrics_enabled: vec!["nic_stats".into(), "link_speed".into()],
            reason: "Network interface diagnostics".to_string(),
            optional: true,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "iw".to_string(),
            metrics_enabled: vec!["wifi_signal".into(), "wifi_stats".into()],
            reason: "WiFi signal and connection quality".to_string(),
            optional: true,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "wireless_tools".to_string(),
            metrics_enabled: vec!["iwconfig".into()],
            reason: "Legacy WiFi diagnostics".to_string(),
            optional: true,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "usbutils".to_string(),
            metrics_enabled: vec!["usb_devices".into()],
            reason: "USB device enumeration".to_string(),
            optional: false,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "pciutils".to_string(),
            metrics_enabled: vec!["pci_devices".into()],
            reason: "PCI device enumeration".to_string(),
            optional: false,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
        AvailableTool {
            package: "hdparm".to_string(),
            metrics_enabled: vec!["disk_info".into(), "disk_cache".into()],
            reason: "SATA disk parameters and info".to_string(),
            optional: true,
            source: "official".to_string(),
            blocked_by_aur_gate: false,
        },
    ]
}

/// Check which tools are missing and could be installed
pub fn get_missing_tools(config: &AnnaConfig) -> Vec<AvailableTool> {
    let manifest = InstrumentationManifest::load();
    let known = get_known_tools();

    known
        .into_iter()
        .filter(|tool| {
            // Not already installed by Anna
            if manifest.is_installed(&tool.package) {
                return false;
            }
            // Not blocked by AUR gate
            if tool.source == "aur" && !config.instrumentation.allow_aur {
                return false;
            }
            // Check if actually missing from system
            !is_package_installed(&tool.package)
        })
        .collect()
}

/// Check if a package is installed on the system
pub fn is_package_installed(package: &str) -> bool {
    std::process::Command::new("pacman")
        .args(["-Qi", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the current package version
pub fn get_package_version(package: &str) -> Option<String> {
    let output = std::process::Command::new("pacman")
        .args(["-Qi", package])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("Version") {
            return line.split(':').nth(1).map(|v| v.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_default() {
        let manifest = InstrumentationManifest::default();
        assert_eq!(manifest.version, 0);
        assert!(manifest.installed.is_empty());
        assert!(manifest.recent_attempts.is_empty());
    }

    #[test]
    fn test_known_tools() {
        let tools = get_known_tools();
        assert!(!tools.is_empty());
        // All known tools should be from official repos
        for tool in &tools {
            assert_eq!(tool.source, "official");
        }
    }

    #[test]
    fn test_rate_limiting() {
        let mut manifest = InstrumentationManifest::default();
        let config = AnnaConfig::default();

        assert!(!manifest.is_rate_limited(&config));

        manifest.record_attempt("test-pkg", true, None);
        assert!(manifest.is_rate_limited(&config)); // 1 install max per day
    }

    #[test]
    fn test_record_install() {
        let mut manifest = InstrumentationManifest::default();
        let tool = InstalledTool {
            package: "test-pkg".to_string(),
            version: "1.0.0".to_string(),
            installed_at: Utc::now(),
            reason: "testing".to_string(),
            metrics_enabled: vec!["test".to_string()],
            optional: true,
            source: "official".to_string(),
        };

        manifest.record_install(tool);
        assert!(manifest.is_installed("test-pkg"));
        assert_eq!(manifest.installed_count(), 1);
    }
}

//! Anna Instrumentation State v7.37.0
//!
//! Persists what tools Anna has installed, when, and why.
//! State file: /var/lib/anna/internal/instrumentation_state.json
//!
//! v7.37.0 requirements:
//! - Track tool_id, package, install_time, reason, trigger, result, version
//! - Show in annactl status
//! - Log all installs to ops_log

use crate::atomic_write;
use crate::config::DATA_DIR;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// State file path
pub const INSTRUMENTATION_STATE_FILE: &str = "internal/instrumentation_state.json";

/// Tool scopes - what hardware/software category a tool enables
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolScope {
    #[serde(rename = "hw:disk")]
    HwDisk,
    #[serde(rename = "hw:network")]
    HwNetwork,
    #[serde(rename = "hw:usb")]
    HwUsb,
    #[serde(rename = "hw:pci")]
    HwPci,
    #[serde(rename = "hw:sensors")]
    HwSensors,
    #[serde(rename = "hw:system")]
    HwSystem,
    #[serde(rename = "general")]
    General,
}

impl ToolScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolScope::HwDisk => "hw:disk",
            ToolScope::HwNetwork => "hw:network",
            ToolScope::HwUsb => "hw:usb",
            ToolScope::HwPci => "hw:pci",
            ToolScope::HwSensors => "hw:sensors",
            ToolScope::HwSystem => "hw:system",
            ToolScope::General => "general",
        }
    }
}

/// Install trigger - how the install was initiated
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallTrigger {
    #[serde(rename = "on_demand")]
    OnDemand,
    #[serde(rename = "background")]
    Background,
}

impl InstallTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallTrigger::OnDemand => "on_demand",
            InstallTrigger::Background => "background",
        }
    }
}

/// Install result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallOutcome {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
}

/// A tool installed by Anna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledToolRecord {
    /// Tool identifier (e.g., "smartctl", "ethtool")
    pub tool_id: String,
    /// Package name (e.g., "smartmontools", "ethtool")
    pub package: String,
    /// When installed (RFC3339)
    pub installed_at: DateTime<Utc>,
    /// Why it was needed
    pub reason: String,
    /// Which scope(s) it enables
    pub scopes: Vec<ToolScope>,
    /// How it was triggered
    pub trigger: InstallTrigger,
    /// Install outcome
    pub outcome: InstallOutcome,
    /// Error message if failed
    pub error: Option<String>,
    /// Installed version (from pacman)
    pub version: Option<String>,
}

/// Tool registry entry - known tools Anna can install
#[derive(Debug, Clone)]
pub struct ToolRegistryEntry {
    pub tool_id: &'static str,
    pub package: &'static str,
    pub scopes: &'static [ToolScope],
    pub reason: &'static str,
    pub probes_enabled: &'static [&'static str],
}

/// Known tools Anna can auto-install
pub static TOOL_REGISTRY: &[ToolRegistryEntry] = &[
    ToolRegistryEntry {
        tool_id: "smartctl",
        package: "smartmontools",
        scopes: &[ToolScope::HwDisk],
        reason: "SATA/SAS disk SMART health monitoring",
        probes_enabled: &["disk_health", "smart_attrs", "disk_temp"],
    },
    ToolRegistryEntry {
        tool_id: "nvme",
        package: "nvme-cli",
        scopes: &[ToolScope::HwDisk],
        reason: "NVMe SSD health and temperature",
        probes_enabled: &["nvme_health", "nvme_temp", "nvme_smart"],
    },
    ToolRegistryEntry {
        tool_id: "ethtool",
        package: "ethtool",
        scopes: &[ToolScope::HwNetwork],
        reason: "Ethernet interface diagnostics",
        probes_enabled: &["nic_stats", "link_speed", "driver_info"],
    },
    ToolRegistryEntry {
        tool_id: "iw",
        package: "iw",
        scopes: &[ToolScope::HwNetwork],
        reason: "WiFi signal and connection quality",
        probes_enabled: &["wifi_signal", "wifi_config", "wifi_stats"],
    },
    ToolRegistryEntry {
        tool_id: "lsusb",
        package: "usbutils",
        scopes: &[ToolScope::HwUsb],
        reason: "USB device enumeration",
        probes_enabled: &["usb_devices", "usb_tree"],
    },
    ToolRegistryEntry {
        tool_id: "lspci",
        package: "pciutils",
        scopes: &[ToolScope::HwPci],
        reason: "PCI device enumeration",
        probes_enabled: &["pci_devices", "gpu_info"],
    },
    ToolRegistryEntry {
        tool_id: "sensors",
        package: "lm_sensors",
        scopes: &[ToolScope::HwSensors],
        reason: "Hardware temperature and fan monitoring",
        probes_enabled: &["cpu_temp", "fan_speed", "voltage"],
    },
    ToolRegistryEntry {
        tool_id: "dmidecode",
        package: "dmidecode",
        scopes: &[ToolScope::HwSystem],
        reason: "DMI/SMBIOS system information",
        probes_enabled: &["system_info", "memory_info", "bios_info"],
    },
];

/// Get tools needed for a specific scope
pub fn get_tools_for_scope(scope: &ToolScope) -> Vec<&'static ToolRegistryEntry> {
    TOOL_REGISTRY
        .iter()
        .filter(|t| t.scopes.contains(scope))
        .collect()
}

/// The full instrumentation state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstrumentationState {
    /// Schema version
    pub version: u32,
    /// Tools installed by Anna (by tool_id)
    pub installed: HashMap<String, InstalledToolRecord>,
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
}

impl InstrumentationState {
    const CURRENT_VERSION: u32 = 1;

    /// State file path
    pub fn state_path() -> PathBuf {
        PathBuf::from(DATA_DIR).join(INSTRUMENTATION_STATE_FILE)
    }

    /// Load state from disk
    pub fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self {
            version: Self::CURRENT_VERSION,
            ..Default::default()
        }
    }

    /// Save state to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let path_str = path
            .to_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(path_str, &content)
    }

    /// Record a successful install
    pub fn record_install(&mut self, record: InstalledToolRecord) {
        self.installed.insert(record.tool_id.clone(), record);
        self.last_updated = Some(Utc::now());
    }

    /// Check if a tool is installed by Anna
    pub fn is_installed(&self, tool_id: &str) -> bool {
        self.installed
            .get(tool_id)
            .map(|r| r.outcome == InstallOutcome::Success)
            .unwrap_or(false)
    }

    /// Get count of successfully installed tools
    pub fn installed_count(&self) -> usize {
        self.installed
            .values()
            .filter(|r| r.outcome == InstallOutcome::Success)
            .count()
    }

    /// Get all successfully installed tools
    pub fn installed_tools(&self) -> impl Iterator<Item = &InstalledToolRecord> {
        self.installed
            .values()
            .filter(|r| r.outcome == InstallOutcome::Success)
    }

    /// Format for status display
    pub fn format_status(&self) -> String {
        let count = self.installed_count();
        if count == 0 {
            "none (clean)".to_string()
        } else {
            format!("{} tool(s)", count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_not_empty() {
        assert!(!TOOL_REGISTRY.is_empty());
    }

    #[test]
    fn test_get_tools_for_scope() {
        let disk_tools = get_tools_for_scope(&ToolScope::HwDisk);
        assert!(!disk_tools.is_empty());
        assert!(disk_tools.iter().any(|t| t.tool_id == "smartctl"));
    }

    #[test]
    fn test_instrumentation_state_default() {
        let state = InstrumentationState::default();
        assert!(state.installed.is_empty());
        assert_eq!(state.installed_count(), 0);
    }

    #[test]
    fn test_format_status_empty() {
        let state = InstrumentationState::default();
        assert_eq!(state.format_status(), "none (clean)");
    }
}

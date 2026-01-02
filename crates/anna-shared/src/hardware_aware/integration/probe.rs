//! Probe helper integration (v0.0.434).
//!
//! Connects hardware-aware system to probes.

use super::super::helper_entry::{HelperCatalog, HelperEntry};
use super::super::helper_manager::HelperManager;
use serde::{Deserialize, Serialize};

/// Probe helper integration.
#[derive(Debug, Clone)]
pub struct ProbeHelper {
    catalog: HelperCatalog,
}

impl ProbeHelper {
    /// Create with default catalog.
    pub fn new() -> Self {
        Self {
            catalog: HelperCatalog::default_catalog(),
        }
    }

    /// Get the best command for a probe given available helpers.
    pub fn best_command(&self, probe_type: &str, manager: &HelperManager) -> ProbeCommand {
        match probe_type {
            "temperature" | "cpu_temp" => {
                if manager.is_tracked("lm_sensors")
                    && self
                        .catalog
                        .get("lm_sensors")
                        .map(|h| h.is_installed())
                        .unwrap_or(false)
                {
                    ProbeCommand::helper("sensors -j", "lm_sensors")
                } else {
                    ProbeCommand::fallback(
                        "cat /sys/class/thermal/thermal_zone*/temp",
                        "Raw thermal zone data (less accurate without lm_sensors)",
                    )
                }
            }
            "disk_health" | "smart" => {
                if manager.is_tracked("smartmontools")
                    && self
                        .catalog
                        .get("smartmontools")
                        .map(|h| h.is_installed())
                        .unwrap_or(false)
                {
                    ProbeCommand::helper("smartctl -a /dev/sda", "smartmontools")
                } else {
                    ProbeCommand::unavailable("Disk health requires smartmontools")
                }
            }
            "nvme" => {
                if manager.is_tracked("nvme_cli")
                    && self
                        .catalog
                        .get("nvme_cli")
                        .map(|h| h.is_installed())
                        .unwrap_or(false)
                {
                    ProbeCommand::helper("nvme smart-log /dev/nvme0", "nvme_cli")
                } else {
                    ProbeCommand::fallback(
                        "cat /sys/class/nvme/nvme*/model",
                        "Basic NVMe info (detailed stats require nvme-cli)",
                    )
                }
            }
            "network" | "nic" => {
                if manager.is_tracked("ethtool")
                    && self
                        .catalog
                        .get("ethtool")
                        .map(|h| h.is_installed())
                        .unwrap_or(false)
                {
                    ProbeCommand::helper("ethtool eth0", "ethtool")
                } else {
                    ProbeCommand::fallback(
                        "ip link show",
                        "Basic network info (detailed stats require ethtool)",
                    )
                }
            }
            "hardware" | "inventory" => {
                if manager.is_tracked("lshw")
                    && self
                        .catalog
                        .get("lshw")
                        .map(|h| h.is_installed())
                        .unwrap_or(false)
                {
                    ProbeCommand::helper("lshw -short", "lshw")
                } else if manager.is_tracked("dmidecode")
                    && self
                        .catalog
                        .get("dmidecode")
                        .map(|h| h.is_installed())
                        .unwrap_or(false)
                {
                    ProbeCommand::helper("dmidecode -t system", "dmidecode")
                } else {
                    ProbeCommand::fallback(
                        "cat /proc/cpuinfo /proc/meminfo",
                        "Basic hardware info from /proc",
                    )
                }
            }
            // Default probes that don't need helpers
            "memory" | "proc_meminfo" => ProbeCommand::builtin("cat /proc/meminfo"),
            "disk_usage" => ProbeCommand::builtin("df -h"),
            "uptime" => ProbeCommand::builtin("uptime"),
            "loadavg" => ProbeCommand::builtin("cat /proc/loadavg"),
            _ => ProbeCommand::unknown(probe_type),
        }
    }

    /// Get helpers that would improve a probe.
    pub fn suggested_helpers(
        &self,
        probe_type: &str,
        manager: &HelperManager,
    ) -> Vec<&HelperEntry> {
        self.catalog
            .helpers_for_probe(probe_type)
            .into_iter()
            .filter(|h| !manager.is_tracked(&h.id))
            .collect()
    }
}

impl Default for ProbeHelper {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of determining probe command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeCommand {
    /// Command to run.
    pub command: Option<String>,
    /// Helper used (if any).
    pub helper_used: Option<String>,
    /// Whether this is a fallback.
    pub is_fallback: bool,
    /// Note about limitations.
    pub note: Option<String>,
    /// Whether probe is available.
    pub available: bool,
}

impl ProbeCommand {
    /// Create for builtin probe (no helper needed).
    pub fn builtin(command: &str) -> Self {
        Self {
            command: Some(command.to_string()),
            helper_used: None,
            is_fallback: false,
            note: None,
            available: true,
        }
    }

    /// Create for helper-based probe.
    pub fn helper(command: &str, helper_id: &str) -> Self {
        Self {
            command: Some(command.to_string()),
            helper_used: Some(helper_id.to_string()),
            is_fallback: false,
            note: None,
            available: true,
        }
    }

    /// Create for fallback probe.
    pub fn fallback(command: &str, note: &str) -> Self {
        Self {
            command: Some(command.to_string()),
            helper_used: None,
            is_fallback: true,
            note: Some(note.to_string()),
            available: true,
        }
    }

    /// Create for unavailable probe.
    pub fn unavailable(note: &str) -> Self {
        Self {
            command: None,
            helper_used: None,
            is_fallback: false,
            note: Some(note.to_string()),
            available: false,
        }
    }

    /// Create for unknown probe type.
    pub fn unknown(probe_type: &str) -> Self {
        Self {
            command: None,
            helper_used: None,
            is_fallback: false,
            note: Some(format!("Unknown probe type: {}", probe_type)),
            available: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_helper_builtin() {
        let helper = ProbeHelper::new();
        let manager = HelperManager::new();

        let cmd = helper.best_command("memory", &manager);
        assert!(cmd.available);
        assert!(cmd.command.is_some());
        assert!(cmd.helper_used.is_none());
    }

    #[test]
    fn test_probe_helper_fallback() {
        let helper = ProbeHelper::new();
        let manager = HelperManager::new();

        // Without lm_sensors, should get fallback
        let cmd = helper.best_command("temperature", &manager);
        assert!(cmd.available);
        assert!(cmd.is_fallback);
        assert!(cmd.note.is_some());
    }

    #[test]
    fn test_probe_helper_unavailable() {
        let helper = ProbeHelper::new();
        let manager = HelperManager::new();

        // Disk health without smartmontools
        let cmd = helper.best_command("disk_health", &manager);
        assert!(!cmd.available);
    }

    #[test]
    fn test_suggested_helpers() {
        let helper = ProbeHelper::new();
        let manager = HelperManager::new();

        let suggestions = helper.suggested_helpers("temperature", &manager);
        assert!(suggestions.iter().any(|h| h.id == "lm_sensors"));
    }
}

//! Integration with probes and specialists (v0.0.434).
//!
//! Connects hardware-aware system to probes and specialist responses.

use super::helpers::{HelperCatalog, HelperEntry, HelperManager};
use super::model_health::{ModelHealth, ModelStatus};
use super::model_plan::ModelPlan;
use super::catalog::ModelRole;
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
                    && self.catalog.get("lm_sensors").map(|h| h.is_installed()).unwrap_or(false)
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
                    && self.catalog.get("smartmontools").map(|h| h.is_installed()).unwrap_or(false)
                {
                    ProbeCommand::helper("smartctl -a /dev/sda", "smartmontools")
                } else {
                    ProbeCommand::unavailable("Disk health requires smartmontools")
                }
            }
            "nvme" => {
                if manager.is_tracked("nvme_cli")
                    && self.catalog.get("nvme_cli").map(|h| h.is_installed()).unwrap_or(false)
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
                    && self.catalog.get("ethtool").map(|h| h.is_installed()).unwrap_or(false)
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
                    && self.catalog.get("lshw").map(|h| h.is_installed()).unwrap_or(false)
                {
                    ProbeCommand::helper("lshw -short", "lshw")
                } else if manager.is_tracked("dmidecode")
                    && self.catalog.get("dmidecode").map(|h| h.is_installed()).unwrap_or(false)
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
    pub fn suggested_helpers(&self, probe_type: &str, manager: &HelperManager) -> Vec<&HelperEntry> {
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

/// Specialist helper integration.
#[derive(Debug, Clone)]
pub struct SpecialistHelper {
    catalog: HelperCatalog,
}

impl SpecialistHelper {
    /// Create with default catalog.
    pub fn new() -> Self {
        Self {
            catalog: HelperCatalog::default_catalog(),
        }
    }

    /// Suggest a helper installation step for a specialist result.
    pub fn suggest_helper_step(
        &self,
        helper_id: &str,
        reason: &str,
        distro: &str,
    ) -> Option<HelperSuggestion> {
        let helper = self.catalog.get(helper_id)?;
        let packages = helper.packages_for_distro(distro);

        if packages.is_empty() {
            return None;
        }

        let package_manager = if distro.to_lowercase().contains("arch") {
            "pacman -S"
        } else if distro.to_lowercase().contains("debian")
            || distro.to_lowercase().contains("ubuntu")
        {
            "apt install"
        } else if distro.to_lowercase().contains("fedora") {
            "dnf install"
        } else {
            "package-manager install"
        };

        Some(HelperSuggestion {
            helper_id: helper_id.to_string(),
            helper_name: helper.name.clone(),
            purpose: helper.purpose.clone(),
            reason: reason.to_string(),
            install_command: format!("sudo {} {}", package_manager, packages.join(" ")),
            packages: packages.to_vec(),
        })
    }

    /// Check if a model is available for a role.
    pub fn check_model_availability(
        &self,
        role: ModelRole,
        plan: &ModelPlan,
        health: &ModelHealth,
    ) -> ModelAvailability {
        let model_name = match role {
            ModelRole::Translator => &plan.translator_model,
            ModelRole::Junior => &plan.junior_model,
            ModelRole::Senior => &plan.senior_model,
        };

        let status = health.status(model_name);

        match status {
            ModelStatus::Ok | ModelStatus::Unverified => ModelAvailability::Available {
                model: model_name.clone(),
            },
            ModelStatus::Missing => ModelAvailability::Missing {
                model: model_name.clone(),
                fallback: self.find_fallback(role, plan, health),
            },
            ModelStatus::Broken => ModelAvailability::Broken {
                model: model_name.clone(),
                error: health
                    .models
                    .get(model_name)
                    .and_then(|r| r.last_error.clone()),
                fallback: self.find_fallback(role, plan, health),
            },
            ModelStatus::Installing => ModelAvailability::Installing {
                model: model_name.clone(),
            },
        }
    }

    /// Find a fallback model for a role.
    fn find_fallback(
        &self,
        role: ModelRole,
        plan: &ModelPlan,
        health: &ModelHealth,
    ) -> Option<String> {
        // For senior, try junior; for junior, try translator
        let fallback_model = match role {
            ModelRole::Senior => Some(&plan.junior_model),
            ModelRole::Junior => Some(&plan.translator_model),
            ModelRole::Translator => None,
        };

        fallback_model.and_then(|model| {
            if health.status(model).is_usable() {
                Some(model.clone())
            } else {
                None
            }
        })
    }
}

impl Default for SpecialistHelper {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper installation suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperSuggestion {
    /// Helper ID.
    pub helper_id: String,
    /// Helper name.
    pub helper_name: String,
    /// Purpose of the helper.
    pub purpose: String,
    /// Why it's being suggested.
    pub reason: String,
    /// Command to install.
    pub install_command: String,
    /// Packages to install.
    pub packages: Vec<String>,
}

impl HelperSuggestion {
    /// Format for specialist response step.
    pub fn format_step(&self) -> String {
        format!(
            "Install {} to {}. Command: {}",
            self.helper_name, self.purpose.to_lowercase(), self.install_command
        )
    }
}

/// Model availability status.
#[derive(Debug, Clone)]
pub enum ModelAvailability {
    /// Model is available.
    Available { model: String },
    /// Model is missing.
    Missing { model: String, fallback: Option<String> },
    /// Model is broken.
    Broken {
        model: String,
        error: Option<String>,
        fallback: Option<String>,
    },
    /// Model is being installed.
    Installing { model: String },
}

impl ModelAvailability {
    /// Whether the model (or fallback) can be used.
    pub fn can_proceed(&self) -> bool {
        match self {
            Self::Available { .. } => true,
            Self::Missing { fallback, .. } => fallback.is_some(),
            Self::Broken { fallback, .. } => fallback.is_some(),
            Self::Installing { .. } => false,
        }
    }

    /// Get the model to use (original or fallback).
    pub fn usable_model(&self) -> Option<&str> {
        match self {
            Self::Available { model } => Some(model),
            Self::Missing { fallback, .. } => fallback.as_deref(),
            Self::Broken { fallback, .. } => fallback.as_deref(),
            Self::Installing { .. } => None,
        }
    }

    /// Whether using a fallback.
    pub fn is_fallback(&self) -> bool {
        matches!(
            self,
            Self::Missing { fallback: Some(_), .. } | Self::Broken { fallback: Some(_), .. }
        )
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

    #[test]
    fn test_specialist_helper_suggestion() {
        let helper = SpecialistHelper::new();

        let suggestion = helper.suggest_helper_step(
            "lm_sensors",
            "To get accurate CPU temperatures",
            "Arch Linux",
        );

        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert!(s.install_command.contains("pacman"));
        assert!(s.packages.contains(&"lm_sensors".to_string()));
    }

    #[test]
    fn test_model_availability_available() {
        let avail = ModelAvailability::Available {
            model: "qwen3:4b".to_string(),
        };

        assert!(avail.can_proceed());
        assert_eq!(avail.usable_model(), Some("qwen3:4b"));
        assert!(!avail.is_fallback());
    }

    #[test]
    fn test_model_availability_missing_with_fallback() {
        let avail = ModelAvailability::Missing {
            model: "qwen2.5:14b".to_string(),
            fallback: Some("qwen2.5:7b".to_string()),
        };

        assert!(avail.can_proceed());
        assert_eq!(avail.usable_model(), Some("qwen2.5:7b"));
        assert!(avail.is_fallback());
    }

    #[test]
    fn test_model_availability_missing_no_fallback() {
        let avail = ModelAvailability::Missing {
            model: "qwen3:0.6b".to_string(),
            fallback: None,
        };

        assert!(!avail.can_proceed());
        assert!(avail.usable_model().is_none());
    }
}

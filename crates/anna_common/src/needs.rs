//! Anna Needs Model v7.6.0 - Track Missing Tools and Data Sources
//!
//! Deterministic tracking of what Anna needs to do her job properly:
//! - Missing tools (smartctl, nvme, sensors, man, etc.)
//! - Missing doc packages (arch-wiki-docs)
//! - Configuration gaps
//!
//! All status checks are based on real tool invocations:
//! - No guessing or hardcoded assumptions
//! - Failed checks recorded with reason
//! - Self-provisioning respects /etc/anna/config.toml

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Types of needs Anna can have
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeedType {
    /// External tool needed (smartctl, nvme, sensors, etc.)
    Tool,
    /// Documentation package (man pages, arch-wiki-docs)
    Docs,
    /// Kernel module or driver
    Driver,
    /// Permission issue (e.g., need root for smartctl)
    Permission,
}

impl NeedType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NeedType::Tool => "tool",
            NeedType::Docs => "docs",
            NeedType::Driver => "driver",
            NeedType::Permission => "permission",
        }
    }
}

/// Status of a need
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedStatus {
    /// Need is satisfied (tool installed, working)
    Satisfied,
    /// Need is missing
    Missing,
    /// Auto-install blocked in config
    Blocked,
    /// Install was attempted but failed
    Failed,
}

impl NeedStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NeedStatus::Satisfied => "satisfied",
            NeedStatus::Missing => "missing",
            NeedStatus::Blocked => "blocked",
            NeedStatus::Failed => "failed",
        }
    }
}

/// Scope where this need applies
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NeedScope {
    /// Hardware-related (hw:disk, hw:cpu, hw:gpu, hw:network)
    Hardware(String),
    /// Software-related (sw:system, sw:config)
    Software(String),
    /// System-wide
    System,
}

impl NeedScope {
    pub fn as_str(&self) -> String {
        match self {
            NeedScope::Hardware(s) => format!("hw:{}", s),
            NeedScope::Software(s) => format!("sw:{}", s),
            NeedScope::System => "system".to_string(),
        }
    }
}

/// A single need entry
#[derive(Debug, Clone)]
pub struct Need {
    /// Unique identifier (e.g., "tool:smartctl", "docs:arch-wiki")
    pub id: String,
    /// Type of need
    pub need_type: NeedType,
    /// Current status
    pub status: NeedStatus,
    /// Why this is needed (deterministic string)
    pub reason: String,
    /// Where this applies
    pub scope: NeedScope,
    /// Package to install (if applicable)
    pub package: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Collection of Anna's needs
#[derive(Debug, Clone, Default)]
pub struct AnnaNeeds {
    /// All tracked needs
    pub needs: HashMap<String, Need>,
}

impl AnnaNeeds {
    pub fn new() -> Self {
        Self {
            needs: HashMap::new(),
        }
    }

    /// Check all known tool dependencies
    pub fn check_all() -> Self {
        let mut needs = Self::new();

        // Hardware health tools
        needs.check_tool(
            "smartctl",
            "smartmontools",
            "disk SMART health",
            NeedScope::Hardware("disk".to_string()),
        );
        needs.check_tool(
            "nvme",
            "nvme-cli",
            "NVMe disk health",
            NeedScope::Hardware("disk".to_string()),
        );
        needs.check_tool(
            "sensors",
            "lm_sensors",
            "CPU/GPU temperature monitoring",
            NeedScope::Hardware("thermal".to_string()),
        );
        needs.check_tool(
            "nvidia-smi",
            "nvidia-utils",
            "NVIDIA GPU monitoring",
            NeedScope::Hardware("gpu".to_string()),
        );

        // Documentation tools
        needs.check_tool(
            "man",
            "man-db",
            "man page documentation",
            NeedScope::Software("docs".to_string()),
        );

        // Arch Wiki local docs
        needs.check_arch_wiki();

        // Network tools
        needs.check_tool(
            "iw",
            "iw",
            "wireless interface details",
            NeedScope::Hardware("network".to_string()),
        );
        needs.check_tool(
            "ethtool",
            "ethtool",
            "ethernet interface details",
            NeedScope::Hardware("network".to_string()),
        );

        needs
    }

    /// Check if a specific tool is available
    fn check_tool(&mut self, tool: &str, package: &str, reason: &str, scope: NeedScope) {
        let id = format!("tool:{}", tool);

        let status = if command_exists(tool) {
            NeedStatus::Satisfied
        } else {
            NeedStatus::Missing
        };

        self.needs.insert(
            id.clone(),
            Need {
                id,
                need_type: NeedType::Tool,
                status,
                reason: reason.to_string(),
                scope,
                package: Some(package.to_string()),
                error: None,
            },
        );
    }

    /// Check if Arch Wiki local docs are available
    fn check_arch_wiki(&mut self) {
        let id = "docs:arch-wiki".to_string();

        // Check for arch-wiki-docs or arch-wiki-lite
        let wiki_paths = [
            "/usr/share/doc/arch-wiki/html",
            "/usr/share/doc/arch-wiki-lite",
        ];

        let exists = wiki_paths.iter().any(|p| Path::new(p).exists());

        let status = if exists {
            NeedStatus::Satisfied
        } else {
            NeedStatus::Missing
        };

        self.needs.insert(
            id.clone(),
            Need {
                id,
                need_type: NeedType::Docs,
                status,
                reason: "local config documentation (improves config discovery)".to_string(),
                scope: NeedScope::Software("config".to_string()),
                package: Some("arch-wiki-docs".to_string()),
                error: None,
            },
        );
    }

    /// Get needs by status
    pub fn by_status(&self, status: NeedStatus) -> Vec<&Need> {
        self.needs.values().filter(|n| n.status == status).collect()
    }

    /// Get unsatisfied needs (missing, blocked, or failed)
    pub fn unsatisfied(&self) -> Vec<&Need> {
        self.needs
            .values()
            .filter(|n| n.status != NeedStatus::Satisfied)
            .collect()
    }

    /// Get needs relevant to a specific scope prefix
    pub fn for_scope(&self, prefix: &str) -> Vec<&Need> {
        self.needs
            .values()
            .filter(|n| n.scope.as_str().starts_with(prefix))
            .collect()
    }

    /// Check if a specific tool is available
    pub fn is_tool_available(&self, tool: &str) -> bool {
        let id = format!("tool:{}", tool);
        self.needs
            .get(&id)
            .map(|n| n.status == NeedStatus::Satisfied)
            .unwrap_or(false)
    }

    /// Get count summary
    pub fn summary(&self) -> NeedsSummary {
        let mut summary = NeedsSummary::default();
        for need in self.needs.values() {
            match need.status {
                NeedStatus::Satisfied => summary.satisfied += 1,
                NeedStatus::Missing => summary.missing += 1,
                NeedStatus::Blocked => summary.blocked += 1,
                NeedStatus::Failed => summary.failed += 1,
            }
        }
        summary
    }
}

/// Summary counts
#[derive(Debug, Clone, Default)]
pub struct NeedsSummary {
    pub satisfied: usize,
    pub missing: usize,
    pub blocked: usize,
    pub failed: usize,
}

impl NeedsSummary {
    pub fn total_unsatisfied(&self) -> usize {
        self.missing + self.blocked + self.failed
    }
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Quick check for specific tools (for use in hw/sw modules)
pub fn is_smartctl_available() -> bool {
    command_exists("smartctl")
}

pub fn is_nvme_available() -> bool {
    command_exists("nvme")
}

pub fn is_sensors_available() -> bool {
    command_exists("sensors")
}

pub fn is_nvidia_smi_available() -> bool {
    command_exists("nvidia-smi")
}

pub fn is_iw_available() -> bool {
    command_exists("iw")
}

pub fn is_ethtool_available() -> bool {
    command_exists("ethtool")
}

pub fn is_man_available() -> bool {
    command_exists("man")
}

/// Get the missing tool message for display
pub fn get_tool_status(tool: &str) -> (bool, &'static str) {
    let available = command_exists(tool);
    let msg = if available { "installed" } else { "missing" };
    (available, msg)
}

/// Hardware dependencies for display in hw overview
#[derive(Debug, Clone, Default)]
pub struct HardwareDeps {
    pub smartctl: (bool, &'static str),
    pub nvme: (bool, &'static str),
    pub sensors: (bool, &'static str),
    pub nvidia_smi: (bool, &'static str),
    pub iw: (bool, &'static str),
    pub ethtool: (bool, &'static str),
}

impl HardwareDeps {
    pub fn check() -> Self {
        Self {
            smartctl: get_tool_status("smartctl"),
            nvme: get_tool_status("nvme"),
            sensors: get_tool_status("sensors"),
            nvidia_smi: get_tool_status("nvidia-smi"),
            iw: get_tool_status("iw"),
            ethtool: get_tool_status("ethtool"),
        }
    }

    /// Get list of missing tools
    pub fn missing_tools(&self) -> Vec<(&'static str, &'static str)> {
        let mut missing = Vec::new();
        if !self.smartctl.0 {
            missing.push(("smartctl", "smartmontools"));
        }
        if !self.nvme.0 {
            missing.push(("nvme", "nvme-cli"));
        }
        if !self.sensors.0 {
            missing.push(("sensors", "lm_sensors"));
        }
        if !self.nvidia_smi.0 {
            missing.push(("nvidia-smi", "nvidia-utils"));
        }
        if !self.iw.0 {
            missing.push(("iw", "iw"));
        }
        if !self.ethtool.0 {
            missing.push(("ethtool", "ethtool"));
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_all() {
        let needs = AnnaNeeds::check_all();
        // Should have checked multiple tools
        assert!(!needs.needs.is_empty());
    }

    #[test]
    fn test_need_type_as_str() {
        assert_eq!(NeedType::Tool.as_str(), "tool");
        assert_eq!(NeedType::Docs.as_str(), "docs");
    }

    #[test]
    fn test_need_status_as_str() {
        assert_eq!(NeedStatus::Satisfied.as_str(), "satisfied");
        assert_eq!(NeedStatus::Missing.as_str(), "missing");
    }

    #[test]
    fn test_scope_as_str() {
        assert_eq!(NeedScope::Hardware("disk".to_string()).as_str(), "hw:disk");
        assert_eq!(
            NeedScope::Software("config".to_string()).as_str(),
            "sw:config"
        );
        assert_eq!(NeedScope::System.as_str(), "system");
    }

    #[test]
    fn test_summary() {
        let needs = AnnaNeeds::check_all();
        let summary = needs.summary();
        // Total should equal number of checks
        assert_eq!(
            summary.satisfied + summary.missing + summary.blocked + summary.failed,
            needs.needs.len()
        );
    }
}

//! Anna Toolchain Hygiene v7.22.0 - Self-Managed Diagnostic Tools
//!
//! Anna can detect, install, and track her own diagnostic toolchain:
//! - Local Arch Wiki docs (arch-wiki-docs)
//! - Storage tools (smartmontools, nvme-cli)
//! - Network tools (ethtool, iw)
//! - Hardware info tools (pciutils, usbutils, lm_sensors)
//!
//! Rules:
//! - Only diagnostic tools and documentation packages
//! - Never desktop apps, shells, or user-facing software
//! - All operations logged to /var/lib/anna/internal/ops.log
//! - No user config files modified
//!
//! Installation is done via pacman only (no AUR, no scripts).

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ops_log::{OpsAction, OpsEntry, OpsLogReader, OpsLogWriter};

/// Internal ops log path
pub const OPS_LOG_PATH: &str = "/var/lib/anna/internal/ops.log";

/// Tool categories that Anna is allowed to manage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    /// Local documentation (arch-wiki-docs)
    Documentation,
    /// Storage diagnostics (smartmontools, nvme-cli)
    Storage,
    /// Network diagnostics (ethtool, iw)
    Network,
    /// Hardware info (pciutils, usbutils, lm_sensors)
    Hardware,
}

impl ToolCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ToolCategory::Documentation => "documentation",
            ToolCategory::Storage => "storage",
            ToolCategory::Network => "network",
            ToolCategory::Hardware => "hardware",
        }
    }
}

/// A tool that Anna may need
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaTool {
    /// Tool name (command or package)
    pub name: String,
    /// Package to install (may differ from command name)
    pub package: String,
    /// Category
    pub category: ToolCategory,
    /// Why Anna needs this
    pub reason: String,
    /// Command to check availability (if different from name)
    pub check_command: Option<String>,
    /// Path to check for docs/data (alternative to command)
    pub check_path: Option<String>,
}

impl AnnaTool {
    /// Check if this tool is available
    pub fn is_available(&self) -> bool {
        // Check path first if specified
        if let Some(ref path) = self.check_path {
            if Path::new(path).exists() {
                return true;
            }
        }

        // Check command
        let cmd = self.check_command.as_ref().unwrap_or(&self.name);
        command_exists(cmd)
    }

    /// Get version if available
    pub fn get_version(&self) -> Option<String> {
        // Try pacman -Q
        let output = Command::new("pacman").args(["-Q", &self.package]).output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Format: "package version"
                let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(parts[1].to_string());
                }
            }
        }

        None
    }
}

/// Tool status for display
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub tool: AnnaTool,
    pub available: bool,
    pub version: Option<String>,
}

/// Complete toolchain status
#[derive(Debug, Clone)]
pub struct ToolchainStatus {
    pub tools: Vec<ToolStatus>,
    pub recent_ops: Vec<OpsEntry>,
    pub last_check: DateTime<Utc>,
}

impl ToolchainStatus {
    /// Get summary for status display
    pub fn summary(&self) -> ToolchainSummary {
        let mut by_category: HashMap<ToolCategory, (usize, usize)> = HashMap::new();

        for status in &self.tools {
            let entry = by_category.entry(status.tool.category).or_insert((0, 0));
            entry.1 += 1; // Total
            if status.available {
                entry.0 += 1; // Ready
            }
        }

        ToolchainSummary { by_category }
    }

    /// Get missing tools
    pub fn missing(&self) -> Vec<&ToolStatus> {
        self.tools.iter().filter(|t| !t.available).collect()
    }

    /// Check if category is ready
    pub fn is_category_ready(&self, category: ToolCategory) -> bool {
        self.tools
            .iter()
            .filter(|t| t.tool.category == category)
            .all(|t| t.available)
    }
}

/// Summary for compact display
#[derive(Debug, Clone)]
pub struct ToolchainSummary {
    pub by_category: HashMap<ToolCategory, (usize, usize)>, // (ready, total)
}

impl ToolchainSummary {
    /// Format for status line
    pub fn format_status_line(&self, category: ToolCategory) -> String {
        if let Some((ready, total)) = self.by_category.get(&category) {
            if ready == total {
                "ready".to_string()
            } else {
                format!("{}/{} ready", ready, total)
            }
        } else {
            "n/a".to_string()
        }
    }
}

/// Anna's allowed toolchain definition
pub fn get_anna_tools() -> Vec<AnnaTool> {
    vec![
        // Documentation
        AnnaTool {
            name: "local-wiki".to_string(),
            package: "arch-wiki-docs".to_string(),
            category: ToolCategory::Documentation,
            reason: "local Arch Wiki for config discovery".to_string(),
            check_command: None,
            check_path: Some("/usr/share/doc/arch-wiki/html".to_string()),
        },
        // Storage tools
        AnnaTool {
            name: "smartctl".to_string(),
            package: "smartmontools".to_string(),
            category: ToolCategory::Storage,
            reason: "SMART disk health monitoring".to_string(),
            check_command: Some("smartctl".to_string()),
            check_path: None,
        },
        AnnaTool {
            name: "nvme".to_string(),
            package: "nvme-cli".to_string(),
            category: ToolCategory::Storage,
            reason: "NVMe disk health and telemetry".to_string(),
            check_command: Some("nvme".to_string()),
            check_path: None,
        },
        // Network tools
        AnnaTool {
            name: "ethtool".to_string(),
            package: "ethtool".to_string(),
            category: ToolCategory::Network,
            reason: "ethernet interface details and firmware".to_string(),
            check_command: Some("ethtool".to_string()),
            check_path: None,
        },
        AnnaTool {
            name: "iw".to_string(),
            package: "iw".to_string(),
            category: ToolCategory::Network,
            reason: "wireless interface details".to_string(),
            check_command: Some("iw".to_string()),
            check_path: None,
        },
        // Hardware tools
        AnnaTool {
            name: "lspci".to_string(),
            package: "pciutils".to_string(),
            category: ToolCategory::Hardware,
            reason: "PCI device enumeration".to_string(),
            check_command: Some("lspci".to_string()),
            check_path: None,
        },
        AnnaTool {
            name: "lsusb".to_string(),
            package: "usbutils".to_string(),
            category: ToolCategory::Hardware,
            reason: "USB device enumeration".to_string(),
            check_command: Some("lsusb".to_string()),
            check_path: None,
        },
        AnnaTool {
            name: "sensors".to_string(),
            package: "lm_sensors".to_string(),
            category: ToolCategory::Hardware,
            reason: "temperature and fan monitoring".to_string(),
            check_command: Some("sensors".to_string()),
            check_path: None,
        },
    ]
}

/// Check toolchain status
pub fn check_toolchain() -> ToolchainStatus {
    let tools = get_anna_tools();
    let mut statuses = Vec::new();

    for tool in tools {
        let available = tool.is_available();
        let version = if available { tool.get_version() } else { None };

        statuses.push(ToolStatus {
            tool,
            available,
            version,
        });
    }

    // Get recent operations
    let reader = OpsLogReader::new();
    let recent_ops = reader.read_recent(10);

    ToolchainStatus {
        tools: statuses,
        recent_ops,
        last_check: Utc::now(),
    }
}

/// Install result
#[derive(Debug, Clone)]
pub enum InstallResult {
    /// Already installed
    AlreadyInstalled,
    /// Successfully installed
    Installed,
    /// Installation failed
    Failed(String),
    /// Tool not in allowed list
    NotAllowed,
    /// Pacman is locked or unavailable
    Locked,
}

/// Install a tool from the allowed list
pub fn install_tool(tool_name: &str, reason: &str) -> InstallResult {
    let tools = get_anna_tools();

    // Find the tool
    let tool = match tools
        .iter()
        .find(|t| t.name == tool_name || t.package == tool_name)
    {
        Some(t) => t,
        None => return InstallResult::NotAllowed,
    };

    // Check if already available
    if tool.is_available() {
        return InstallResult::AlreadyInstalled;
    }

    // Try to install via pacman
    let output = Command::new("pacman")
        .args(["-S", "--noconfirm", &tool.package])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                // Log the installation
                let writer = OpsLogWriter::new();
                let full_reason = format!("reason=\"{}\"", reason);
                let _ = writer.record(&OpsEntry::with_details(
                    OpsAction::Install,
                    &tool.package,
                    &full_reason,
                    true,
                ));

                InstallResult::Installed
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.contains("lock") {
                    InstallResult::Locked
                } else {
                    InstallResult::Failed(stderr.to_string())
                }
            }
        }
        Err(e) => InstallResult::Failed(e.to_string()),
    }
}

/// Ensure a tool is available, installing if necessary
pub fn ensure_tool(tool_name: &str, reason: &str) -> bool {
    let tools = get_anna_tools();

    // Find the tool
    let tool = match tools
        .iter()
        .find(|t| t.name == tool_name || t.package == tool_name)
    {
        Some(t) => t,
        None => return false,
    };

    // Check if already available
    if tool.is_available() {
        return true;
    }

    // Try to install
    matches!(install_tool(tool_name, reason), InstallResult::Installed)
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Format toolchain for annactl sw anna output
pub fn format_toolchain_section(status: &ToolchainStatus) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[ANNA TOOLCHAIN]".to_string());
    lines.push("  Required tools:".to_string());

    // Group by category
    for category in [
        ToolCategory::Documentation,
        ToolCategory::Storage,
        ToolCategory::Network,
        ToolCategory::Hardware,
    ] {
        let cat_tools: Vec<_> = status
            .tools
            .iter()
            .filter(|t| t.tool.category == category)
            .collect();

        if cat_tools.is_empty() {
            continue;
        }

        for tool in cat_tools {
            let status_str = if tool.available {
                if let Some(ref ver) = tool.version {
                    format!("present ({} {})", tool.tool.package, ver)
                } else {
                    "present".to_string()
                }
            } else {
                "missing".to_string()
            };

            lines.push(format!(
                "    {:<14} {}",
                tool.tool.name.to_string() + ":",
                status_str
            ));
        }
    }

    // Recent operations
    if !status.recent_ops.is_empty() {
        lines.push(String::new());
        lines.push("  Last operations:".to_string());
        for op in status.recent_ops.iter().rev().take(5) {
            lines.push(format!(
                "    {} {} {}",
                op.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                op.action.as_str(),
                op.target
            ));
        }
    }

    lines
}

/// Format toolchain summary for status command
pub fn format_toolchain_status_section(status: &ToolchainStatus) -> Vec<String> {
    let mut lines = Vec::new();
    let summary = status.summary();

    lines.push("[ANNA TOOLCHAIN]".to_string());

    // Documentation
    let doc_status = summary.format_status_line(ToolCategory::Documentation);
    lines.push(format!("  Local wiki:     {}", doc_status));

    // Storage
    let storage_status = summary.format_status_line(ToolCategory::Storage);
    let storage_missing: Vec<_> = status
        .tools
        .iter()
        .filter(|t| t.tool.category == ToolCategory::Storage && !t.available)
        .map(|t| t.tool.name.as_str())
        .collect();
    if storage_missing.is_empty() {
        lines.push(format!("  Storage tools:  {}", storage_status));
    } else {
        lines.push(format!(
            "  Storage tools:  missing {}",
            storage_missing.join(", ")
        ));
    }

    // Network
    let network_missing: Vec<_> = status
        .tools
        .iter()
        .filter(|t| t.tool.category == ToolCategory::Network && !t.available)
        .map(|t| t.tool.name.as_str())
        .collect();
    if network_missing.is_empty() {
        lines.push("  Network tools:  ready".to_string());
    } else {
        lines.push(format!(
            "  Network tools:  missing {}",
            network_missing.join(", ")
        ));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_anna_tools() {
        let tools = get_anna_tools();
        assert!(!tools.is_empty());

        // Should have tools in each category
        assert!(tools
            .iter()
            .any(|t| t.category == ToolCategory::Documentation));
        assert!(tools.iter().any(|t| t.category == ToolCategory::Storage));
        assert!(tools.iter().any(|t| t.category == ToolCategory::Network));
        assert!(tools.iter().any(|t| t.category == ToolCategory::Hardware));
    }

    #[test]
    fn test_check_toolchain() {
        let status = check_toolchain();
        // Should have checked all tools
        assert!(!status.tools.is_empty());
    }

    #[test]
    fn test_tool_category_label() {
        assert_eq!(ToolCategory::Documentation.label(), "documentation");
        assert_eq!(ToolCategory::Storage.label(), "storage");
    }

    #[test]
    fn test_install_not_allowed() {
        let result = install_tool("firefox", "test");
        assert!(matches!(result, InstallResult::NotAllowed));
    }

    #[test]
    fn test_format_toolchain_section() {
        let status = check_toolchain();
        let lines = format_toolchain_section(&status);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("TOOLCHAIN"));
    }
}

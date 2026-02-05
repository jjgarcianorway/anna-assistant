//! Tool Manager - Anna's self-installing diagnostic tools.
//!
//! v0.3.124: Anna can install tools she needs and track them for cleanup.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Tool that Anna might need to install.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticTool {
    /// Package name
    pub package: String,
    /// Command to check if installed
    pub check_command: String,
    /// Why Anna needs this tool
    pub purpose: String,
    /// When it was installed
    pub installed_at: Option<String>,
}

/// Anna's installed tools registry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstalledTools {
    /// Tools Anna has installed
    pub tools: HashSet<String>,
}

impl InstalledTools {
    fn path() -> PathBuf {
        PathBuf::from("/var/lib/anna/installed_tools.json")
    }

    /// Load from disk.
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk.
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)
    }

    /// Mark a tool as installed by Anna.
    pub fn mark_installed(&mut self, package: &str) {
        self.tools.insert(package.to_string());
    }

    /// Remove a tool from the registry.
    pub fn mark_removed(&mut self, package: &str) {
        self.tools.remove(package);
    }

    /// Check if a tool was installed by Anna.
    pub fn was_installed_by_anna(&self, package: &str) -> bool {
        self.tools.contains(package)
    }

    /// Get all Anna-installed tools.
    pub fn all(&self) -> Vec<String> {
        self.tools.iter().cloned().collect()
    }

    /// Count of tools.
    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

/// Common diagnostic tools Anna might need.
pub fn get_recommended_tools() -> Vec<DiagnosticTool> {
    vec![
        DiagnosticTool {
            package: "bc".to_string(),
            check_command: "which bc".to_string(),
            purpose: "Precision calculations for resource analysis".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "jq".to_string(),
            check_command: "which jq".to_string(),
            purpose: "JSON parsing for API responses".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "htop".to_string(),
            check_command: "which htop".to_string(),
            purpose: "Interactive process monitoring".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "lsof".to_string(),
            check_command: "which lsof".to_string(),
            purpose: "List open files and network connections".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "nethogs".to_string(),
            check_command: "which nethogs".to_string(),
            purpose: "Per-process network bandwidth monitoring".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "iotop".to_string(),
            check_command: "which iotop".to_string(),
            purpose: "Disk I/O monitoring per process".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "strace".to_string(),
            check_command: "which strace".to_string(),
            purpose: "System call tracing for debugging".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "tcpdump".to_string(),
            check_command: "which tcpdump".to_string(),
            purpose: "Network packet analysis".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "sysstat".to_string(),
            check_command: "which sar".to_string(),
            purpose: "System performance statistics (sar, iostat)".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "bpftrace".to_string(),
            check_command: "which bpftrace".to_string(),
            purpose: "Advanced kernel tracing with eBPF".to_string(),
            installed_at: None,
        },
        DiagnosticTool {
            package: "perf".to_string(),
            check_command: "which perf".to_string(),
            purpose: "Performance analysis and profiling".to_string(),
            installed_at: None,
        },
    ]
}

/// Check if a tool is installed.
pub fn is_tool_installed(check_command: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(check_command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Find missing diagnostic tools.
pub fn find_missing_tools() -> Vec<DiagnosticTool> {
    get_recommended_tools()
        .into_iter()
        .filter(|tool| !is_tool_installed(&tool.check_command))
        .collect()
}

/// Install a tool and mark it in registry.
pub async fn install_tool(package: &str, purpose: &str) -> anyhow::Result<()> {
    use std::process::Command;

    // Try to install with pacman
    let output = Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg(package)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to install {}: {}", package, String::from_utf8_lossy(&output.stderr));
    }

    // Mark as installed by Anna
    let mut tools = InstalledTools::load();
    tools.mark_installed(package);
    tools.save()?;

    // Record in package history
    use crate::package_history::{PackageHistory, PackageEventType};
    let mut history = PackageHistory::load();
    history.record(package, PackageEventType::Install, "latest", true);
    history.save()?;

    tracing::info!("Anna installed tool: {} ({})", package, purpose);
    Ok(())
}

/// Generate uninstall commands for Anna's tools.
pub fn generate_uninstall_commands() -> Vec<String> {
    let tools = InstalledTools::load();
    if tools.count() == 0 {
        return vec![];
    }

    let packages = tools.all().join(" ");
    vec![
        format!("sudo pacman -Rns {}", packages),
    ]
}

/// Format list of Anna's installed tools.
pub fn format_installed_tools() -> String {
    let tools = InstalledTools::load();
    if tools.count() == 0 {
        return "No diagnostic tools installed by Anna.".to_string();
    }

    let mut lines = vec![
        format!("Anna has installed {} diagnostic tool(s):", tools.count()),
        String::new(),
    ];

    for tool in tools.all() {
        lines.push(format!("  - {}", tool));
    }

    lines.push(String::new());
    lines.push("These will be removed when you uninstall Anna.".to_string());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let mut tools = InstalledTools::default();
        tools.mark_installed("jq");
        tools.mark_installed("bc");

        assert_eq!(tools.count(), 2);
        assert!(tools.was_installed_by_anna("jq"));
        assert!(!tools.was_installed_by_anna("curl"));

        tools.mark_removed("jq");
        assert_eq!(tools.count(), 1);
        assert!(!tools.was_installed_by_anna("jq"));
    }

    #[test]
    fn test_recommended_tools() {
        let tools = get_recommended_tools();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.package == "jq"));
    }
}

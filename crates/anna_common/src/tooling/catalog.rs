//! Tool Catalog - v6.60.0
//!
//! Defines ALL tools Anna can execute. This is the single source of truth.
//! The LLM cannot invent commands - it can only select from this catalog.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};

/// Unique identifier for each allowed tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolId {
    // === Memory ===
    FreeMem,

    // === CPU ===
    LsCpu,
    CpuInfo,

    // === GPU ===
    LsPci,
    LsPciGpu,

    // === Disk ===
    DfHuman,
    DuHomeTop,
    DuVarTop,
    LsBlk,
    FindMnt,

    // === Packages ===
    PacmanQuery,
    PacmanQueryGames,
    PacmanQueryFileManagers,
    PacmanOrphans,
    PacmanUpdates,
    CheckUpdates,

    // === Network ===
    IpAddrShow,
    IpRoute,
    NmcliDeviceStatus,
    NmcliConnectionShow,
    SsListening,
    PingTest,
    ResolvConf,

    // === Services ===
    SystemctlFailed,
    SystemctlListUnits,
    JournalctlErrors,
    JournalctlRecent,

    // === System Info ===
    Uptime,
    UnameAll,
    Hostnamectl,
    OsRelease,

    // === Process ===
    PsAux,
    TopBatch,

    // === Files ===
    LsDir,
    CatFile,

    // === Shell ===
    Echo,
    Whoami,
}

impl ToolId {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::FreeMem => "free_mem",
            Self::LsCpu => "lscpu",
            Self::CpuInfo => "cpu_info",
            Self::LsPci => "lspci",
            Self::LsPciGpu => "lspci_gpu",
            Self::DfHuman => "df_human",
            Self::DuHomeTop => "du_home_top",
            Self::DuVarTop => "du_var_top",
            Self::LsBlk => "lsblk",
            Self::FindMnt => "findmnt",
            Self::PacmanQuery => "pacman_query",
            Self::PacmanQueryGames => "pacman_query_games",
            Self::PacmanQueryFileManagers => "pacman_query_fm",
            Self::PacmanOrphans => "pacman_orphans",
            Self::PacmanUpdates => "pacman_updates",
            Self::CheckUpdates => "checkupdates",
            Self::IpAddrShow => "ip_addr",
            Self::IpRoute => "ip_route",
            Self::NmcliDeviceStatus => "nmcli_device",
            Self::NmcliConnectionShow => "nmcli_connection",
            Self::SsListening => "ss_listening",
            Self::PingTest => "ping_test",
            Self::ResolvConf => "resolv_conf",
            Self::SystemctlFailed => "systemctl_failed",
            Self::SystemctlListUnits => "systemctl_units",
            Self::JournalctlErrors => "journalctl_errors",
            Self::JournalctlRecent => "journalctl_recent",
            Self::Uptime => "uptime",
            Self::UnameAll => "uname",
            Self::Hostnamectl => "hostnamectl",
            Self::OsRelease => "os_release",
            Self::PsAux => "ps_aux",
            Self::TopBatch => "top_batch",
            Self::LsDir => "ls_dir",
            Self::CatFile => "cat_file",
            Self::Echo => "echo",
            Self::Whoami => "whoami",
        }
    }
}

/// Tool classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    /// Fast, read-only, no side effects
    Stateless,
    /// May take longer or use significant resources
    Heavy,
    /// Requires root/sudo
    RootRequired,
    /// Modifies system state (not used for diagnostics)
    Modifying,
}

/// Tool specification
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub id: ToolId,
    pub binary: &'static str,
    pub args: Vec<&'static str>,
    pub kind: ToolKind,
    pub required: bool,
    pub description: &'static str,
}

/// Result from running a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_id: String,
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

impl ToolResult {
    /// Check if the result indicates the tool is not installed
    pub fn is_not_installed(&self) -> bool {
        self.stderr.contains("command not found")
            || self.stderr.contains("No such file or directory")
            || self.exit_code == 127
    }
}

/// Get the complete tool catalog
pub fn tool_catalog() -> Vec<ToolSpec> {
    vec![
        // === Memory ===
        ToolSpec {
            id: ToolId::FreeMem,
            binary: "free",
            args: vec!["-m"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Show memory usage in MB",
        },
        // === CPU ===
        ToolSpec {
            id: ToolId::LsCpu,
            binary: "lscpu",
            args: vec![],
            kind: ToolKind::Stateless,
            required: true,
            description: "Show CPU information",
        },
        ToolSpec {
            id: ToolId::CpuInfo,
            binary: "cat",
            args: vec!["/proc/cpuinfo"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Read CPU info from /proc",
        },
        // === GPU ===
        ToolSpec {
            id: ToolId::LsPci,
            binary: "lspci",
            args: vec![],
            kind: ToolKind::Stateless,
            required: false,
            description: "List PCI devices",
        },
        ToolSpec {
            id: ToolId::LsPciGpu,
            binary: "lspci",
            args: vec!["-v"],
            kind: ToolKind::Stateless,
            required: false,
            description: "List PCI devices with details (for GPU)",
        },
        // === Disk ===
        ToolSpec {
            id: ToolId::DfHuman,
            binary: "df",
            args: vec!["-h"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Show disk usage in human format",
        },
        ToolSpec {
            id: ToolId::DuHomeTop,
            binary: "du",
            args: vec!["-h", "--max-depth=1"],
            kind: ToolKind::Heavy,
            required: false,
            description: "Disk usage of home subdirectories",
        },
        ToolSpec {
            id: ToolId::DuVarTop,
            binary: "du",
            args: vec!["-h", "--max-depth=1", "/var"],
            kind: ToolKind::Heavy,
            required: false,
            description: "Disk usage of /var subdirectories",
        },
        ToolSpec {
            id: ToolId::LsBlk,
            binary: "lsblk",
            args: vec!["-o", "NAME,SIZE,TYPE,MOUNTPOINT"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List block devices",
        },
        ToolSpec {
            id: ToolId::FindMnt,
            binary: "findmnt",
            args: vec!["-l"],
            kind: ToolKind::Stateless,
            required: false,
            description: "List mount points",
        },
        // === Packages ===
        ToolSpec {
            id: ToolId::PacmanQuery,
            binary: "pacman",
            args: vec!["-Q"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List installed packages",
        },
        ToolSpec {
            id: ToolId::PacmanQueryGames,
            binary: "pacman",
            args: vec!["-Qq"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List packages (names only) for game filtering",
        },
        ToolSpec {
            id: ToolId::PacmanQueryFileManagers,
            binary: "pacman",
            args: vec!["-Qq"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List packages (names only) for file manager filtering",
        },
        ToolSpec {
            id: ToolId::PacmanOrphans,
            binary: "pacman",
            args: vec!["-Qdtq"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List orphaned packages",
        },
        ToolSpec {
            id: ToolId::PacmanUpdates,
            binary: "pacman",
            args: vec!["-Qu"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List packages with available updates",
        },
        ToolSpec {
            id: ToolId::CheckUpdates,
            binary: "checkupdates",
            args: vec![],
            kind: ToolKind::Heavy,
            required: false,
            description: "Check for updates (arch-specific)",
        },
        // === Network ===
        ToolSpec {
            id: ToolId::IpAddrShow,
            binary: "ip",
            args: vec!["addr", "show"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Show network interfaces",
        },
        ToolSpec {
            id: ToolId::IpRoute,
            binary: "ip",
            args: vec!["route"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Show routing table",
        },
        ToolSpec {
            id: ToolId::NmcliDeviceStatus,
            binary: "nmcli",
            args: vec!["device", "status"],
            kind: ToolKind::Stateless,
            required: false,
            description: "NetworkManager device status",
        },
        ToolSpec {
            id: ToolId::NmcliConnectionShow,
            binary: "nmcli",
            args: vec!["connection", "show", "--active"],
            kind: ToolKind::Stateless,
            required: false,
            description: "NetworkManager active connections",
        },
        ToolSpec {
            id: ToolId::SsListening,
            binary: "ss",
            args: vec!["-tuln"],
            kind: ToolKind::Stateless,
            required: false,
            description: "List listening ports",
        },
        ToolSpec {
            id: ToolId::PingTest,
            binary: "ping",
            args: vec!["-c", "1", "-W", "2", "8.8.8.8"],
            kind: ToolKind::Stateless,
            required: false,
            description: "Test internet connectivity",
        },
        ToolSpec {
            id: ToolId::ResolvConf,
            binary: "cat",
            args: vec!["/etc/resolv.conf"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Show DNS configuration",
        },
        // === Services ===
        ToolSpec {
            id: ToolId::SystemctlFailed,
            binary: "systemctl",
            args: vec!["--failed", "--no-pager"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List failed systemd units",
        },
        ToolSpec {
            id: ToolId::SystemctlListUnits,
            binary: "systemctl",
            args: vec!["list-units", "--type=service", "--no-pager"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List systemd service units",
        },
        ToolSpec {
            id: ToolId::JournalctlErrors,
            binary: "journalctl",
            args: vec!["-p", "err", "-n", "20", "--no-pager"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Recent error log entries",
        },
        ToolSpec {
            id: ToolId::JournalctlRecent,
            binary: "journalctl",
            args: vec!["-n", "50", "--no-pager"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Recent log entries",
        },
        // === System Info ===
        ToolSpec {
            id: ToolId::Uptime,
            binary: "uptime",
            args: vec![],
            kind: ToolKind::Stateless,
            required: true,
            description: "System uptime and load",
        },
        ToolSpec {
            id: ToolId::UnameAll,
            binary: "uname",
            args: vec!["-a"],
            kind: ToolKind::Stateless,
            required: true,
            description: "System information",
        },
        ToolSpec {
            id: ToolId::Hostnamectl,
            binary: "hostnamectl",
            args: vec![],
            kind: ToolKind::Stateless,
            required: false,
            description: "Hostname and OS info",
        },
        ToolSpec {
            id: ToolId::OsRelease,
            binary: "cat",
            args: vec!["/etc/os-release"],
            kind: ToolKind::Stateless,
            required: true,
            description: "OS release information",
        },
        // === Process ===
        ToolSpec {
            id: ToolId::PsAux,
            binary: "ps",
            args: vec!["aux", "--sort=-pcpu"],
            kind: ToolKind::Stateless,
            required: true,
            description: "Process list sorted by CPU",
        },
        ToolSpec {
            id: ToolId::TopBatch,
            binary: "top",
            args: vec!["-bn1", "-o", "%MEM"],
            kind: ToolKind::Stateless,
            required: false,
            description: "Top processes by memory",
        },
        // === Files ===
        ToolSpec {
            id: ToolId::LsDir,
            binary: "ls",
            args: vec!["-la"],
            kind: ToolKind::Stateless,
            required: true,
            description: "List directory contents",
        },
        ToolSpec {
            id: ToolId::CatFile,
            binary: "cat",
            args: vec![],
            kind: ToolKind::Stateless,
            required: true,
            description: "Read file contents",
        },
        // === Shell ===
        ToolSpec {
            id: ToolId::Echo,
            binary: "echo",
            args: vec![],
            kind: ToolKind::Stateless,
            required: true,
            description: "Echo text",
        },
        ToolSpec {
            id: ToolId::Whoami,
            binary: "whoami",
            args: vec![],
            kind: ToolKind::Stateless,
            required: true,
            description: "Current user",
        },
    ]
}

/// Get a specific tool by ID
pub fn get_tool(id: ToolId) -> Option<ToolSpec> {
    tool_catalog().into_iter().find(|t| t.id == id)
}

/// Build a lookup map from ToolId to ToolSpec
pub fn build_catalog_map() -> HashMap<ToolId, ToolSpec> {
    tool_catalog().into_iter().map(|t| (t.id, t)).collect()
}

/// Run a tool and return structured result
pub fn run_tool(spec: &ToolSpec) -> ToolResult {
    run_tool_with_extra_args(spec, &[])
}

/// Run a tool with additional arguments
pub fn run_tool_with_extra_args(spec: &ToolSpec, extra_args: &[&str]) -> ToolResult {
    let start = Instant::now();

    let mut cmd = Command::new(spec.binary);
    for arg in &spec.args {
        cmd.arg(arg);
    }
    for arg in extra_args {
        cmd.arg(arg);
    }

    let output = cmd.output();
    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let exit_code = out.status.code().unwrap_or(-1);

            ToolResult {
                tool_id: spec.id.name().to_string(),
                success: out.status.success(),
                exit_code,
                stdout,
                stderr,
                duration_ms,
            }
        }
        Err(e) => ToolResult {
            tool_id: spec.id.name().to_string(),
            success: false,
            exit_code: -1,
            stdout: String::new(),
            stderr: format!("Failed to execute {}: {}", spec.binary, e),
            duration_ms,
        },
    }
}

/// Run self-test on all tools in catalog
pub fn self_test_catalog() -> Vec<(ToolSpec, ToolResult)> {
    let catalog = tool_catalog();
    let mut results = Vec::new();

    for spec in catalog {
        let result = run_tool(&spec);
        results.push((spec, result));
    }

    results
}

/// Format self-test results as a table
pub fn format_self_test_results(results: &[(ToolSpec, ToolResult)]) -> String {
    let mut output = String::new();
    output.push_str("🔧  Tool Catalog Self-Test Results\n\n");

    let mut available = 0;
    let mut missing = 0;
    let mut failed = 0;

    for (spec, result) in results {
        let status = if result.success {
            available += 1;
            "✅"
        } else if result.is_not_installed() {
            missing += 1;
            "❌"
        } else {
            failed += 1;
            "⚠️"
        };

        let req = if spec.required { "[req]" } else { "" };
        output.push_str(&format!(
            "  {}  {:20} {:6} {}ms {}\n",
            status,
            spec.binary,
            req,
            result.duration_ms,
            if !result.success && !result.stderr.is_empty() {
                &result.stderr[..result.stderr.len().min(50)]
            } else {
                ""
            }
        ));
    }

    output.push_str(&format!(
        "\n  Summary: {} available, {} missing, {} failed\n",
        available, missing, failed
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_core_tools() {
        let catalog = tool_catalog();
        let ids: Vec<_> = catalog.iter().map(|t| t.id).collect();

        // These MUST be in the catalog
        assert!(ids.contains(&ToolId::FreeMem));
        assert!(ids.contains(&ToolId::LsCpu));
        assert!(ids.contains(&ToolId::LsPci));
        assert!(ids.contains(&ToolId::PacmanQuery));
        assert!(ids.contains(&ToolId::DfHuman));
        assert!(ids.contains(&ToolId::IpAddrShow));
        assert!(ids.contains(&ToolId::SystemctlFailed));
        assert!(ids.contains(&ToolId::JournalctlErrors));
    }

    #[test]
    fn test_get_tool() {
        let spec = get_tool(ToolId::FreeMem);
        assert!(spec.is_some());
        let spec = spec.unwrap();
        assert_eq!(spec.binary, "free");
        assert_eq!(spec.args, vec!["-m"]);
    }

    #[test]
    fn test_run_tool_free() {
        let spec = get_tool(ToolId::FreeMem).unwrap();
        let result = run_tool(&spec);

        // On a real system, free should work
        if result.success {
            assert!(result.stdout.contains("Mem:"));
            assert!(result.exit_code == 0);
        }
    }

    #[test]
    fn test_run_tool_lscpu() {
        let spec = get_tool(ToolId::LsCpu).unwrap();
        let result = run_tool(&spec);

        if result.success {
            assert!(
                result.stdout.contains("CPU") || result.stdout.contains("Architecture")
            );
        }
    }

    #[test]
    fn test_run_tool_df() {
        let spec = get_tool(ToolId::DfHuman).unwrap();
        let result = run_tool(&spec);

        if result.success {
            assert!(result.stdout.contains("Filesystem") || result.stdout.contains("/"));
        }
    }

    #[test]
    fn test_catalog_no_duplicates() {
        let catalog = tool_catalog();
        let mut ids: Vec<_> = catalog.iter().map(|t| t.id).collect();
        let orig_len = ids.len();
        ids.sort_by_key(|id| id.name());
        ids.dedup();
        assert_eq!(ids.len(), orig_len, "Duplicate ToolIds in catalog");
    }
}

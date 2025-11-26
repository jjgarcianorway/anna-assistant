//! Strict Tool Catalog - v6.58.0 Toolchain Reality Lock
//!
//! This module defines the ONLY commands Anna is allowed to execute.
//! The LLM cannot invent commands - it can only select from this catalog.
//!
//! If a command is not in this catalog, it CANNOT be executed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tool definition in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (e.g., "free", "pacman")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// The base command to execute
    pub command: String,
    /// Allowed flags/arguments (if empty, only base command allowed)
    pub allowed_args: Vec<AllowedArg>,
    /// Category for organization
    pub category: ToolCategory,
    /// Whether this tool requires root/sudo
    pub requires_root: bool,
    /// Risk level
    pub risk: RiskLevel,
}

/// Allowed argument patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedArg {
    /// Argument pattern (e.g., "-m", "-h", "--query")
    pub pattern: String,
    /// Description of what this arg does
    pub description: String,
    /// Whether this arg takes a value
    pub takes_value: bool,
}

/// Tool categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCategory {
    /// Memory/RAM inspection
    Memory,
    /// CPU inspection
    Cpu,
    /// Disk/storage inspection
    Disk,
    /// Network inspection
    Network,
    /// Package management
    Packages,
    /// Service/systemd management
    Services,
    /// Process inspection
    Processes,
    /// General system info
    System,
    /// Shell utilities
    Shell,
    /// File operations (read-only)
    Files,
}

/// Risk levels for commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only, safe to run
    ReadOnly,
    /// May write but reversible
    LowRisk,
    /// Potentially destructive
    HighRisk,
}

/// The strict tool catalog
#[derive(Debug, Clone)]
pub struct StrictToolCatalog {
    tools: HashMap<String, ToolDefinition>,
}

impl Default for StrictToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl StrictToolCatalog {
    /// Create the catalog with all allowed tools
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // ========================================
        // MEMORY TOOLS
        // ========================================
        tools.insert("free".to_string(), ToolDefinition {
            name: "free".to_string(),
            description: "Display amount of free and used memory".to_string(),
            command: "free".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-m".to_string(), description: "Display in megabytes".to_string(), takes_value: false },
                AllowedArg { pattern: "-g".to_string(), description: "Display in gigabytes".to_string(), takes_value: false },
                AllowedArg { pattern: "-h".to_string(), description: "Human readable".to_string(), takes_value: false },
                AllowedArg { pattern: "-b".to_string(), description: "Display in bytes".to_string(), takes_value: false },
            ],
            category: ToolCategory::Memory,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("cat_meminfo".to_string(), ToolDefinition {
            name: "cat_meminfo".to_string(),
            description: "Read /proc/meminfo for memory details".to_string(),
            command: "cat /proc/meminfo".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Memory,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // CPU TOOLS
        // ========================================
        tools.insert("lscpu".to_string(), ToolDefinition {
            name: "lscpu".to_string(),
            description: "Display CPU architecture information".to_string(),
            command: "lscpu".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-e".to_string(), description: "Extended format".to_string(), takes_value: false },
                AllowedArg { pattern: "-p".to_string(), description: "Parseable format".to_string(), takes_value: false },
            ],
            category: ToolCategory::Cpu,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("cat_cpuinfo".to_string(), ToolDefinition {
            name: "cat_cpuinfo".to_string(),
            description: "Read /proc/cpuinfo for CPU details and flags".to_string(),
            command: "cat /proc/cpuinfo".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Cpu,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("grep_cpuflags".to_string(), ToolDefinition {
            name: "grep_cpuflags".to_string(),
            description: "Extract CPU flags from /proc/cpuinfo".to_string(),
            command: "grep -i flags /proc/cpuinfo | head -1".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Cpu,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // DISK TOOLS
        // ========================================
        tools.insert("df".to_string(), ToolDefinition {
            name: "df".to_string(),
            description: "Report file system disk space usage".to_string(),
            command: "df".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-h".to_string(), description: "Human readable".to_string(), takes_value: false },
                AllowedArg { pattern: "-T".to_string(), description: "Show filesystem type".to_string(), takes_value: false },
                AllowedArg { pattern: "/".to_string(), description: "Root filesystem".to_string(), takes_value: false },
                AllowedArg { pattern: "/home".to_string(), description: "Home filesystem".to_string(), takes_value: false },
            ],
            category: ToolCategory::Disk,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("du".to_string(), ToolDefinition {
            name: "du".to_string(),
            description: "Estimate file space usage".to_string(),
            command: "du".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-h".to_string(), description: "Human readable".to_string(), takes_value: false },
                AllowedArg { pattern: "-s".to_string(), description: "Summarize".to_string(), takes_value: false },
                AllowedArg { pattern: "-d".to_string(), description: "Max depth".to_string(), takes_value: true },
                AllowedArg { pattern: "--max-depth".to_string(), description: "Max depth".to_string(), takes_value: true },
            ],
            category: ToolCategory::Disk,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("lsblk".to_string(), ToolDefinition {
            name: "lsblk".to_string(),
            description: "List block devices".to_string(),
            command: "lsblk".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-f".to_string(), description: "Show filesystems".to_string(), takes_value: false },
                AllowedArg { pattern: "-o".to_string(), description: "Output columns".to_string(), takes_value: true },
            ],
            category: ToolCategory::Disk,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // PACKAGE MANAGEMENT
        // ========================================
        tools.insert("pacman_query".to_string(), ToolDefinition {
            name: "pacman_query".to_string(),
            description: "Query installed packages".to_string(),
            command: "pacman".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-Q".to_string(), description: "Query local packages".to_string(), takes_value: false },
                AllowedArg { pattern: "-Qq".to_string(), description: "Query quiet (names only)".to_string(), takes_value: false },
                AllowedArg { pattern: "-Qs".to_string(), description: "Search local packages".to_string(), takes_value: true },
                AllowedArg { pattern: "-Qi".to_string(), description: "Query package info".to_string(), takes_value: true },
                AllowedArg { pattern: "-Qe".to_string(), description: "Explicitly installed".to_string(), takes_value: false },
            ],
            category: ToolCategory::Packages,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("pacman_sync".to_string(), ToolDefinition {
            name: "pacman_sync".to_string(),
            description: "Sync/search remote packages".to_string(),
            command: "pacman".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-Ss".to_string(), description: "Search remote packages".to_string(), takes_value: true },
                AllowedArg { pattern: "-Si".to_string(), description: "Remote package info".to_string(), takes_value: true },
            ],
            category: ToolCategory::Packages,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("pacman_checkupdates".to_string(), ToolDefinition {
            name: "pacman_checkupdates".to_string(),
            description: "Check for available updates".to_string(),
            command: "checkupdates".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Packages,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // NETWORK TOOLS
        // ========================================
        tools.insert("ip_addr".to_string(), ToolDefinition {
            name: "ip_addr".to_string(),
            description: "Show IP addresses".to_string(),
            command: "ip addr show".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Network,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("ip_route".to_string(), ToolDefinition {
            name: "ip_route".to_string(),
            description: "Show routing table".to_string(),
            command: "ip route show".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Network,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("nmcli_device".to_string(), ToolDefinition {
            name: "nmcli_device".to_string(),
            description: "Show NetworkManager device status".to_string(),
            command: "nmcli device status".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Network,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("nmcli_connection".to_string(), ToolDefinition {
            name: "nmcli_connection".to_string(),
            description: "Show NetworkManager connections".to_string(),
            command: "nmcli connection show".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Network,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("ping".to_string(), ToolDefinition {
            name: "ping".to_string(),
            description: "Test network connectivity".to_string(),
            command: "ping".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-c".to_string(), description: "Count".to_string(), takes_value: true },
                AllowedArg { pattern: "-W".to_string(), description: "Timeout".to_string(), takes_value: true },
            ],
            category: ToolCategory::Network,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("ss".to_string(), ToolDefinition {
            name: "ss".to_string(),
            description: "Socket statistics".to_string(),
            command: "ss".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-t".to_string(), description: "TCP sockets".to_string(), takes_value: false },
                AllowedArg { pattern: "-u".to_string(), description: "UDP sockets".to_string(), takes_value: false },
                AllowedArg { pattern: "-l".to_string(), description: "Listening".to_string(), takes_value: false },
                AllowedArg { pattern: "-n".to_string(), description: "Numeric".to_string(), takes_value: false },
            ],
            category: ToolCategory::Network,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // SYSTEMD/SERVICES
        // ========================================
        tools.insert("systemctl_status".to_string(), ToolDefinition {
            name: "systemctl_status".to_string(),
            description: "Show service status".to_string(),
            command: "systemctl status".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "--no-pager".to_string(), description: "No pager".to_string(), takes_value: false },
            ],
            category: ToolCategory::Services,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("systemctl_list_units".to_string(), ToolDefinition {
            name: "systemctl_list_units".to_string(),
            description: "List systemd units".to_string(),
            command: "systemctl list-units".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "--type=service".to_string(), description: "Only services".to_string(), takes_value: false },
                AllowedArg { pattern: "--state=failed".to_string(), description: "Only failed".to_string(), takes_value: false },
                AllowedArg { pattern: "--state=running".to_string(), description: "Only running".to_string(), takes_value: false },
                AllowedArg { pattern: "--no-pager".to_string(), description: "No pager".to_string(), takes_value: false },
                AllowedArg { pattern: "--no-legend".to_string(), description: "No legend".to_string(), takes_value: false },
            ],
            category: ToolCategory::Services,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("systemctl_is_active".to_string(), ToolDefinition {
            name: "systemctl_is_active".to_string(),
            description: "Check if service is active".to_string(),
            command: "systemctl is-active".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Services,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("journalctl".to_string(), ToolDefinition {
            name: "journalctl".to_string(),
            description: "Query systemd journal".to_string(),
            command: "journalctl".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-n".to_string(), description: "Number of lines".to_string(), takes_value: true },
                AllowedArg { pattern: "-u".to_string(), description: "Unit".to_string(), takes_value: true },
                AllowedArg { pattern: "-p".to_string(), description: "Priority".to_string(), takes_value: true },
                AllowedArg { pattern: "--no-pager".to_string(), description: "No pager".to_string(), takes_value: false },
                AllowedArg { pattern: "-b".to_string(), description: "Current boot".to_string(), takes_value: false },
            ],
            category: ToolCategory::Services,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // PROCESS TOOLS
        // ========================================
        tools.insert("ps".to_string(), ToolDefinition {
            name: "ps".to_string(),
            description: "Report process status".to_string(),
            command: "ps".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "aux".to_string(), description: "All users, extended".to_string(), takes_value: false },
                AllowedArg { pattern: "-e".to_string(), description: "All processes".to_string(), takes_value: false },
                AllowedArg { pattern: "-f".to_string(), description: "Full format".to_string(), takes_value: false },
            ],
            category: ToolCategory::Processes,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("pgrep".to_string(), ToolDefinition {
            name: "pgrep".to_string(),
            description: "Find processes by name".to_string(),
            command: "pgrep".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-x".to_string(), description: "Exact match".to_string(), takes_value: false },
                AllowedArg { pattern: "-l".to_string(), description: "List name".to_string(), takes_value: false },
            ],
            category: ToolCategory::Processes,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // SYSTEM INFO
        // ========================================
        tools.insert("uname".to_string(), ToolDefinition {
            name: "uname".to_string(),
            description: "Print system information".to_string(),
            command: "uname".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-a".to_string(), description: "All info".to_string(), takes_value: false },
                AllowedArg { pattern: "-r".to_string(), description: "Kernel release".to_string(), takes_value: false },
                AllowedArg { pattern: "-m".to_string(), description: "Machine".to_string(), takes_value: false },
            ],
            category: ToolCategory::System,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("hostnamectl".to_string(), ToolDefinition {
            name: "hostnamectl".to_string(),
            description: "Show hostname and OS info".to_string(),
            command: "hostnamectl".to_string(),
            allowed_args: vec![],
            category: ToolCategory::System,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("lspci".to_string(), ToolDefinition {
            name: "lspci".to_string(),
            description: "List PCI devices".to_string(),
            command: "lspci".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-v".to_string(), description: "Verbose".to_string(), takes_value: false },
                AllowedArg { pattern: "-k".to_string(), description: "Show kernel drivers".to_string(), takes_value: false },
            ],
            category: ToolCategory::System,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("lsusb".to_string(), ToolDefinition {
            name: "lsusb".to_string(),
            description: "List USB devices".to_string(),
            command: "lsusb".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-v".to_string(), description: "Verbose".to_string(), takes_value: false },
            ],
            category: ToolCategory::System,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("sensors".to_string(), ToolDefinition {
            name: "sensors".to_string(),
            description: "Show hardware sensors".to_string(),
            command: "sensors".to_string(),
            allowed_args: vec![],
            category: ToolCategory::System,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        // ========================================
        // SHELL UTILITIES (for pipelines)
        // ========================================
        tools.insert("echo".to_string(), ToolDefinition {
            name: "echo".to_string(),
            description: "Print text".to_string(),
            command: "echo".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Shell,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("cat".to_string(), ToolDefinition {
            name: "cat".to_string(),
            description: "Read file contents".to_string(),
            command: "cat".to_string(),
            allowed_args: vec![],
            category: ToolCategory::Files,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("head".to_string(), ToolDefinition {
            name: "head".to_string(),
            description: "Output first lines".to_string(),
            command: "head".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-n".to_string(), description: "Number of lines".to_string(), takes_value: true },
            ],
            category: ToolCategory::Shell,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("tail".to_string(), ToolDefinition {
            name: "tail".to_string(),
            description: "Output last lines".to_string(),
            command: "tail".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-n".to_string(), description: "Number of lines".to_string(), takes_value: true },
            ],
            category: ToolCategory::Shell,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("grep".to_string(), ToolDefinition {
            name: "grep".to_string(),
            description: "Search for patterns".to_string(),
            command: "grep".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-i".to_string(), description: "Case insensitive".to_string(), takes_value: false },
                AllowedArg { pattern: "-v".to_string(), description: "Invert match".to_string(), takes_value: false },
                AllowedArg { pattern: "-c".to_string(), description: "Count".to_string(), takes_value: false },
                AllowedArg { pattern: "-E".to_string(), description: "Extended regex".to_string(), takes_value: false },
            ],
            category: ToolCategory::Shell,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("wc".to_string(), ToolDefinition {
            name: "wc".to_string(),
            description: "Word/line count".to_string(),
            command: "wc".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-l".to_string(), description: "Lines".to_string(), takes_value: false },
                AllowedArg { pattern: "-w".to_string(), description: "Words".to_string(), takes_value: false },
            ],
            category: ToolCategory::Shell,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("sort".to_string(), ToolDefinition {
            name: "sort".to_string(),
            description: "Sort lines".to_string(),
            command: "sort".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-n".to_string(), description: "Numeric".to_string(), takes_value: false },
                AllowedArg { pattern: "-r".to_string(), description: "Reverse".to_string(), takes_value: false },
                AllowedArg { pattern: "-h".to_string(), description: "Human numeric".to_string(), takes_value: false },
            ],
            category: ToolCategory::Shell,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("ls".to_string(), ToolDefinition {
            name: "ls".to_string(),
            description: "List directory contents".to_string(),
            command: "ls".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-l".to_string(), description: "Long format".to_string(), takes_value: false },
                AllowedArg { pattern: "-a".to_string(), description: "All files".to_string(), takes_value: false },
                AllowedArg { pattern: "-h".to_string(), description: "Human readable".to_string(), takes_value: false },
            ],
            category: ToolCategory::Files,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        tools.insert("find".to_string(), ToolDefinition {
            name: "find".to_string(),
            description: "Find files".to_string(),
            command: "find".to_string(),
            allowed_args: vec![
                AllowedArg { pattern: "-name".to_string(), description: "Name pattern".to_string(), takes_value: true },
                AllowedArg { pattern: "-type".to_string(), description: "File type".to_string(), takes_value: true },
                AllowedArg { pattern: "-maxdepth".to_string(), description: "Max depth".to_string(), takes_value: true },
            ],
            category: ToolCategory::Files,
            requires_root: false,
            risk: RiskLevel::ReadOnly,
        });

        Self { tools }
    }

    /// Get a tool definition by name
    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Check if a tool exists in the catalog
    pub fn exists(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tools in a category
    pub fn by_category(&self, category: ToolCategory) -> Vec<&ToolDefinition> {
        self.tools.values().filter(|t| t.category == category).collect()
    }

    /// Get all tool names
    pub fn all_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get tool descriptions for LLM context (compact format)
    pub fn to_llm_context(&self) -> String {
        let mut lines = vec!["Available tools (ONLY use these, do not invent commands):".to_string()];

        for category in [
            ToolCategory::Memory,
            ToolCategory::Cpu,
            ToolCategory::Disk,
            ToolCategory::Packages,
            ToolCategory::Network,
            ToolCategory::Services,
            ToolCategory::Processes,
            ToolCategory::System,
            ToolCategory::Shell,
            ToolCategory::Files,
        ] {
            let tools = self.by_category(category);
            if !tools.is_empty() {
                lines.push(format!("\n## {:?}", category));
                for tool in tools {
                    lines.push(format!("- {}: {}", tool.name, tool.description));
                }
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_essential_tools() {
        let catalog = StrictToolCatalog::new();

        // Memory tools
        assert!(catalog.exists("free"));
        assert!(catalog.exists("cat_meminfo"));

        // CPU tools
        assert!(catalog.exists("lscpu"));
        assert!(catalog.exists("cat_cpuinfo"));

        // Disk tools
        assert!(catalog.exists("df"));
        assert!(catalog.exists("du"));

        // Package tools
        assert!(catalog.exists("pacman_query"));

        // Network tools
        assert!(catalog.exists("ip_addr"));
        assert!(catalog.exists("ping"));

        // Service tools
        assert!(catalog.exists("systemctl_status"));
        assert!(catalog.exists("journalctl"));
    }

    #[test]
    fn test_forbidden_tools_not_in_catalog() {
        let catalog = StrictToolCatalog::new();

        // These should NOT exist
        assert!(!catalog.exists("rm"));
        assert!(!catalog.exists("dd"));
        assert!(!catalog.exists("mkfs"));
        assert!(!catalog.exists("fdisk"));
        assert!(!catalog.exists("reboot"));
        assert!(!catalog.exists("shutdown"));
        assert!(!catalog.exists("systemd-cryptgen")); // Fabricated tool
    }

    #[test]
    fn test_category_filtering() {
        let catalog = StrictToolCatalog::new();

        let memory_tools = catalog.by_category(ToolCategory::Memory);
        assert!(!memory_tools.is_empty());
        assert!(memory_tools.iter().any(|t| t.name == "free"));
    }

    #[test]
    fn test_llm_context_generation() {
        let catalog = StrictToolCatalog::new();
        let context = catalog.to_llm_context();

        assert!(context.contains("Available tools"));
        assert!(context.contains("free"));
        assert!(context.contains("lscpu"));
        assert!(context.contains("pacman_query"));
    }
}

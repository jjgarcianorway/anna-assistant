//! Sandbox execution environments for safe command testing.

use super::{SandboxType, ExperimentResults, ExperimentRecommendation, PackageChange, ServiceChange};
use serde::{Deserialize, Serialize};

/// Configuration for sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Sandbox type to use
    pub sandbox_type: SandboxType,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Allow network access
    pub network: bool,
    /// Paths to bind read-only
    pub readonly_binds: Vec<String>,
    /// Paths to bind read-write
    pub readwrite_binds: Vec<String>,
    /// Environment variables to set
    pub env: Vec<(String, String)>,
    /// Working directory
    pub workdir: Option<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            sandbox_type: SandboxType::None,
            timeout_secs: 30,
            network: false,
            readonly_binds: vec![
                "/usr".to_string(),
                "/lib".to_string(),
                "/lib64".to_string(),
                "/bin".to_string(),
                "/sbin".to_string(),
                "/etc".to_string(),
            ],
            readwrite_binds: Vec::new(),
            env: Vec::new(),
            workdir: None,
        }
    }
}

/// Select the appropriate sandbox for given commands
pub fn select_sandbox(commands: &[String]) -> SandboxType {
    let mut max_risk = 0;

    for cmd in commands {
        let risk = categorize_command_risk(cmd);
        max_risk = max_risk.max(risk);
    }

    match max_risk {
        0 => SandboxType::None,      // Read-only commands
        1 => SandboxType::DryRun,    // Commands with dry-run support
        2 => SandboxType::FilesystemNamespace, // File modifications
        3 => SandboxType::FullNamespace, // Service/process modifications
        4..=5 => {
            // Package/system modifications - use containers if available
            if SandboxType::Bubblewrap.is_available() {
                SandboxType::Bubblewrap
            } else if SandboxType::Podman.is_available() {
                SandboxType::Podman
            } else if SandboxType::Docker.is_available() {
                SandboxType::Docker
            } else {
                SandboxType::FullNamespace
            }
        }
        _ => SandboxType::VirtualMachine, // Critical system changes
    }
}

/// Categorize command risk level (0-6)
fn categorize_command_risk(cmd: &str) -> u8 {
    let cmd_lower = cmd.to_lowercase();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let base_cmd = parts.first().map(|s| *s).unwrap_or("");

    // Dangerous commands (6) - check first!
    if is_dangerous_command(&cmd_lower) {
        return 6;
    }

    // Critical system commands (5)
    if is_critical_command(base_cmd) {
        return 5;
    }

    // Read-only commands (0)
    if is_readonly_command(base_cmd) {
        return 0;
    }

    // Service/process modifications (3)
    if cmd_lower.contains("systemctl") || cmd_lower.contains("service") {
        if cmd_lower.contains("status") || cmd_lower.contains("show") || cmd_lower.contains("list") {
            return 0;
        }
        return 3;
    }

    // Package modifications (4)
    if is_package_command(base_cmd) {
        if cmd_lower.contains("-q") || cmd_lower.contains("-si") || cmd_lower.contains("-qi") {
            return 0; // Query operations
        }
        return 4;
    }

    // File modifications (2)
    if is_file_modifying_command(base_cmd) {
        return 2;
    }

    // Commands with dry-run support (1)
    if has_dry_run_support(base_cmd) && !cmd_lower.contains(" -s") && !cmd_lower.contains(" --sync") {
        return 1;
    }

    // Default to moderate risk
    2
}

/// Check if command is read-only
fn is_readonly_command(cmd: &str) -> bool {
    let readonly = [
        "ls", "cat", "head", "tail", "grep", "find", "locate", "which", "whereis",
        "file", "stat", "df", "du", "free", "top", "htop", "ps", "pgrep", "lsof",
        "netstat", "ss", "ip", "route", "arp", "ping", "traceroute", "dig", "nslookup",
        "host", "uname", "hostname", "date", "uptime", "who", "w", "id", "groups",
        "env", "printenv", "echo", "pwd", "readlink", "lsblk", "blkid", "fdisk",
        "smartctl", "journalctl", "dmesg", "lspci", "lsusb", "lscpu", "lsmem",
        "timedatectl", "localectl", "hostnamectl",
    ];
    readonly.contains(&cmd)
}

/// Check if command has dry-run support
fn has_dry_run_support(cmd: &str) -> bool {
    let with_dry_run = [
        "pacman", "yay", "paru", "rsync", "cp", "mv", "rm", "mkdir",
        "makepkg", "git", "npm", "pip", "cargo",
    ];
    with_dry_run.contains(&cmd)
}

/// Check if command modifies files
fn is_file_modifying_command(cmd: &str) -> bool {
    let file_modifiers = [
        "cp", "mv", "rm", "mkdir", "rmdir", "touch", "chmod", "chown", "chgrp",
        "ln", "unlink", "truncate", "dd", "tar", "unzip", "gzip", "gunzip",
        "bzip2", "xz", "sed", "awk", "patch", "install",
    ];
    file_modifiers.contains(&cmd)
}

/// Check if command is a package manager
fn is_package_command(cmd: &str) -> bool {
    let pkg_cmds = ["pacman", "yay", "paru", "pikaur", "trizen", "aurman", "makepkg"];
    pkg_cmds.contains(&cmd)
}

/// Check if command is critical
fn is_critical_command(cmd: &str) -> bool {
    let critical = [
        "reboot", "shutdown", "poweroff", "halt", "init",
        "mount", "umount", "mkfs", "fsck", "parted", "fdisk", "gdisk",
        "modprobe", "rmmod", "insmod",
        "iptables", "nft", "firewall-cmd",
    ];
    critical.contains(&cmd)
}

/// Check if command is dangerous
fn is_dangerous_command(cmd: &str) -> bool {
    // rm -rf / patterns
    if cmd.contains("rm ") && cmd.contains("-rf") {
        // Check for root directory deletion
        if cmd.ends_with(" /") || cmd.contains(" / ") || cmd.contains("/*") {
            return true;
        }
    }
    // dd to system disk
    if cmd.contains("dd ") && (cmd.contains("of=/dev/sd") || cmd.contains("of=/dev/nvme")) {
        return true;
    }
    // mkfs on system partitions
    if cmd.contains("mkfs") && (cmd.contains("/dev/sd") || cmd.contains("/dev/nvme")) {
        return true;
    }
    false
}

/// Build command line for sandbox execution
pub fn build_sandbox_command(config: &SandboxConfig, command: &str) -> Vec<String> {
    match config.sandbox_type {
        SandboxType::None => vec!["sh".to_string(), "-c".to_string(), command.to_string()],

        SandboxType::DryRun => {
            // Add dry-run flag if applicable
            let dry_run_cmd = add_dry_run_flag(command);
            vec!["sh".to_string(), "-c".to_string(), dry_run_cmd]
        }

        SandboxType::FilesystemNamespace => {
            let mut args = vec![
                "unshare".to_string(),
                "--mount".to_string(),
                "--map-root-user".to_string(),
            ];
            args.extend(vec!["sh".to_string(), "-c".to_string(), command.to_string()]);
            args
        }

        SandboxType::FullNamespace => {
            let mut args = vec![
                "unshare".to_string(),
                "--mount".to_string(),
                "--pid".to_string(),
                "--fork".to_string(),
                "--map-root-user".to_string(),
            ];
            if !config.network {
                args.push("--net".to_string());
            }
            args.extend(vec!["sh".to_string(), "-c".to_string(), command.to_string()]);
            args
        }

        SandboxType::Bubblewrap => {
            let mut args = vec!["bwrap".to_string()];

            // Add readonly binds
            for path in &config.readonly_binds {
                args.extend(vec!["--ro-bind".to_string(), path.clone(), path.clone()]);
            }

            // Add readwrite binds
            for path in &config.readwrite_binds {
                args.extend(vec!["--bind".to_string(), path.clone(), path.clone()]);
            }

            // Add tmpfs for /tmp
            args.extend(vec!["--tmpfs".to_string(), "/tmp".to_string()]);

            // Add proc and dev
            args.extend(vec!["--proc".to_string(), "/proc".to_string()]);
            args.extend(vec!["--dev".to_string(), "/dev".to_string()]);

            // Network isolation
            if !config.network {
                args.push("--unshare-net".to_string());
            }

            // Working directory
            if let Some(ref wd) = config.workdir {
                args.extend(vec!["--chdir".to_string(), wd.clone()]);
            }

            args.extend(vec!["sh".to_string(), "-c".to_string(), command.to_string()]);
            args
        }

        SandboxType::NspawnContainer | SandboxType::Docker | SandboxType::Podman => {
            // For containers, we'd need a base image - simplified for now
            vec!["sh".to_string(), "-c".to_string(), command.to_string()]
        }

        SandboxType::VirtualMachine => {
            // VM execution would require complex setup
            vec!["sh".to_string(), "-c".to_string(), command.to_string()]
        }
    }
}

/// Add dry-run flag to command if applicable
fn add_dry_run_flag(command: &str) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let base_cmd = parts.first().map(|s| *s).unwrap_or("");

    match base_cmd {
        "pacman" => {
            if command.contains("-S") || command.contains("-R") || command.contains("-U") {
                format!("{} --print", command)
            } else {
                command.to_string()
            }
        }
        "yay" | "paru" => {
            if command.contains("-S") || command.contains("-R") {
                format!("{} --print", command)
            } else {
                command.to_string()
            }
        }
        "rsync" => {
            if !command.contains("-n") && !command.contains("--dry-run") {
                format!("{} --dry-run", command)
            } else {
                command.to_string()
            }
        }
        "rm" | "mv" | "cp" => {
            // These don't have true dry-run, but we can use -i for interactive
            // or just skip the operation in sandbox
            command.to_string()
        }
        _ => command.to_string(),
    }
}

/// Parse package changes from pacman --print output
pub fn parse_package_changes(output: &str) -> Vec<PackageChange> {
    let mut changes = Vec::new();

    for line in output.lines() {
        // pacman --print format: /path/to/package-version.pkg.tar.zst
        if line.contains(".pkg.tar") {
            if let Some(pkg_name) = extract_package_name(line) {
                changes.push(PackageChange {
                    name: pkg_name,
                    action: super::PackageAction::Install,
                    version: None,
                });
            }
        }
    }

    changes
}

/// Extract package name from package path
fn extract_package_name(path: &str) -> Option<String> {
    let filename = path.rsplit('/').next()?;
    // Format: name-version-rel-arch.pkg.tar.zst
    let without_ext = filename.split(".pkg.tar").next()?;
    // Split from end, architecture is last
    let parts: Vec<&str> = without_ext.rsplitn(4, '-').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readonly_detection() {
        assert!(is_readonly_command("ls"));
        assert!(is_readonly_command("cat"));
        assert!(!is_readonly_command("rm"));
        assert!(!is_readonly_command("pacman"));
    }

    #[test]
    fn test_dangerous_detection() {
        assert!(is_dangerous_command("rm -rf /"));
        assert!(is_dangerous_command("rm -rf /*"));
        assert!(is_dangerous_command("dd if=/dev/zero of=/dev/sda"));
        assert!(!is_dangerous_command("rm file.txt"));
    }

    #[test]
    fn test_command_risk() {
        assert_eq!(categorize_command_risk("ls -la"), 0);
        assert_eq!(categorize_command_risk("cat /etc/passwd"), 0);
        assert!(categorize_command_risk("pacman -S vim") > 2);
        assert!(categorize_command_risk("rm -rf /") > 4);
    }

    #[test]
    fn test_sandbox_selection() {
        assert_eq!(select_sandbox(&["ls -la".to_string()]), SandboxType::None);
        assert!(select_sandbox(&["pacman -S vim".to_string()]) != SandboxType::None);
    }

    #[test]
    fn test_dry_run_flag() {
        assert!(add_dry_run_flag("pacman -S vim").contains("--print"));
        assert!(add_dry_run_flag("rsync -av src dst").contains("--dry-run"));
    }
}

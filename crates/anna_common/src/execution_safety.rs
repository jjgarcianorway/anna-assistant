//! Execution Safety - Risk classification and safe execution orchestration
//!
//! v6.50.0: Turn Anna from planner-only into trusted operator

use serde::{Deserialize, Serialize};

/// Risk classification for plans
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Safe,     // User-level configs, low-risk changes
    Moderate, // System configs with backups, service restarts
    High,     // Potentially dangerous, plan-only
}

/// Domain classification for commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CommandDomain {
    UserConfig,      // ~/.config, ~/.vimrc, etc
    SystemConfig,    // /etc configs
    Packages,        // Package installation/removal
    Services,        // Systemd services
    Network,         // Network configuration
    Boot,            // Bootloader, kernel
    Disk,            // Partitioning, filesystem operations
    Crypto,          // Keys, certificates
    ReadOnly,        // Inspection commands
}

/// Execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionMode {
    PlanOnly,        // Show plan, do not execute
    ConfirmRequired, // Show plan, require confirmation
    Automatic,       // Execute without confirmation (future)
}

/// Plan summary for user confirmation
#[derive(Debug, Clone)]
pub struct PlanSummary {
    pub description: String,           // Human-friendly description
    pub risk_level: RiskLevel,
    pub domains: Vec<CommandDomain>,
    pub command_count: usize,
    pub will_create_backups: bool,
    pub execution_mode: ExecutionMode,
}

impl PlanSummary {
    /// Get confirmation prompt text
    pub fn confirmation_prompt(&self) -> String {
        format!(
            "Risk: {:?}. Run this plan? y/N",
            self.risk_level
        )
    }

    /// Get risk description
    pub fn risk_description(&self) -> String {
        match self.risk_level {
            RiskLevel::Safe => "Safe (user-level config)".to_string(),
            RiskLevel::Moderate => {
                if self.will_create_backups {
                    "Moderate (system config with backup)".to_string()
                } else {
                    "Moderate (system change)".to_string()
                }
            }
            RiskLevel::High => "High (potentially dangerous, plan-only)".to_string(),
        }
    }
}

/// Classify command risk based on domain and intent
pub fn classify_command_risk(command: &str, description: &str) -> RiskLevel {
    let cmd_lower = command.to_lowercase();
    let desc_lower = description.to_lowercase();

    // High risk - never auto-execute
    if cmd_lower.contains("fdisk")
        || cmd_lower.contains("mkfs")
        || cmd_lower.contains("dd if=")
        || cmd_lower.contains("parted")
        || cmd_lower.contains("grub-install")
        || cmd_lower.contains("/boot")
        || desc_lower.contains("bootloader")
        || desc_lower.contains("partition")
        || desc_lower.contains("format disk")
        || desc_lower.contains("certificate")
        || desc_lower.contains("private key")
    {
        return RiskLevel::High;
    }

    // Moderate risk - system level changes
    if command.starts_with("/etc/")
        || cmd_lower.contains("systemctl restart")
        || cmd_lower.contains("systemctl enable")
        || cmd_lower.contains("systemctl disable")
        || cmd_lower.contains("pacman -S")
        || cmd_lower.contains("pacman -R")
        || cmd_lower.contains("yay -S")
        || cmd_lower.contains("yay -R")
        || cmd_lower.contains("networkmanager")
        || desc_lower.contains("system config")
        || desc_lower.contains("service")
    {
        return RiskLevel::Moderate;
    }

    // Safe - user-level configs
    if command.starts_with("~/")
        || command.contains("$HOME")
        || cmd_lower.contains(".vimrc")
        || cmd_lower.contains(".config")
        || cmd_lower.contains(".bashrc")
        || cmd_lower.contains(".zshrc")
        || desc_lower.contains("user config")
        || desc_lower.contains("editor config")
    {
        return RiskLevel::Safe;
    }

    // Read-only commands are safe
    if cmd_lower.starts_with("ls ")
        || cmd_lower.starts_with("cat ")
        || cmd_lower.starts_with("grep ")
        || cmd_lower.starts_with("find ")
        || cmd_lower.starts_with("systemctl status")
        || cmd_lower.starts_with("pacman -Q")
    {
        return RiskLevel::Safe;
    }

    // Default to moderate for unknown commands
    RiskLevel::Moderate
}

/// Classify command domain
pub fn classify_command_domain(command: &str) -> CommandDomain {
    let cmd_lower = command.to_lowercase();

    if cmd_lower.contains("fdisk") || cmd_lower.contains("mkfs") || cmd_lower.contains("parted") {
        CommandDomain::Disk
    } else if cmd_lower.contains("/boot") || cmd_lower.contains("grub") {
        CommandDomain::Boot
    } else if cmd_lower.contains("certificate") || cmd_lower.contains("key") {
        CommandDomain::Crypto
    } else if cmd_lower.contains("systemctl") || cmd_lower.contains("service") {
        CommandDomain::Services
    } else if cmd_lower.contains("pacman") || cmd_lower.contains("yay") {
        CommandDomain::Packages
    } else if cmd_lower.contains("networkmanager") || cmd_lower.contains("nmcli") {
        CommandDomain::Network
    } else if command.starts_with("/etc/") {
        CommandDomain::SystemConfig
    } else if command.starts_with("~/") || command.contains("$HOME") || command.contains(".config") {
        CommandDomain::UserConfig
    } else if cmd_lower.starts_with("ls ")
        || cmd_lower.starts_with("cat ")
        || cmd_lower.starts_with("grep ")
        || cmd_lower.starts_with("find ")
    {
        CommandDomain::ReadOnly
    } else {
        CommandDomain::UserConfig
    }
}

/// Determine execution mode based on risk and context
pub fn determine_execution_mode(
    risk_level: &RiskLevel,
    is_interactive: bool,
) -> ExecutionMode {
    match risk_level {
        RiskLevel::High => ExecutionMode::PlanOnly,
        RiskLevel::Moderate | RiskLevel::Safe => {
            if is_interactive {
                ExecutionMode::ConfirmRequired
            } else {
                ExecutionMode::PlanOnly // One-shot stays plan-only for v6.50.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_high_risk() {
        assert_eq!(
            classify_command_risk("fdisk /dev/sda", "partition disk"),
            RiskLevel::High
        );
        assert_eq!(
            classify_command_risk("mkfs.ext4 /dev/sda1", "format partition"),
            RiskLevel::High
        );
        assert_eq!(
            classify_command_risk("grub-install /dev/sda", "install bootloader"),
            RiskLevel::High
        );
    }

    #[test]
    fn test_classify_moderate_risk() {
        assert_eq!(
            classify_command_risk("systemctl restart sshd", "restart service"),
            RiskLevel::Moderate
        );
        assert_eq!(
            classify_command_risk("pacman -S vim", "install package"),
            RiskLevel::Moderate
        );
        assert_eq!(
            classify_command_risk("/etc/ssh/sshd_config", "system config"),
            RiskLevel::Moderate
        );
    }

    #[test]
    fn test_classify_safe() {
        assert_eq!(
            classify_command_risk("~/.vimrc", "user config"),
            RiskLevel::Safe
        );
        assert_eq!(
            classify_command_risk("ls -la", "list files"),
            RiskLevel::Safe
        );
        assert_eq!(
            classify_command_risk("cat ~/.bashrc", "read file"),
            RiskLevel::Safe
        );
    }

    #[test]
    fn test_classify_domains() {
        assert_eq!(
            classify_command_domain("systemctl restart sshd"),
            CommandDomain::Services
        );
        assert_eq!(
            classify_command_domain("pacman -S vim"),
            CommandDomain::Packages
        );
        assert_eq!(
            classify_command_domain("~/.vimrc"),
            CommandDomain::UserConfig
        );
        assert_eq!(
            classify_command_domain("/etc/ssh/sshd_config"),
            CommandDomain::SystemConfig
        );
    }

    #[test]
    fn test_execution_mode_high_risk() {
        let mode = determine_execution_mode(&RiskLevel::High, true);
        assert_eq!(mode, ExecutionMode::PlanOnly);

        let mode = determine_execution_mode(&RiskLevel::High, false);
        assert_eq!(mode, ExecutionMode::PlanOnly);
    }

    #[test]
    fn test_execution_mode_interactive() {
        let mode = determine_execution_mode(&RiskLevel::Safe, true);
        assert_eq!(mode, ExecutionMode::ConfirmRequired);

        let mode = determine_execution_mode(&RiskLevel::Moderate, true);
        assert_eq!(mode, ExecutionMode::ConfirmRequired);
    }

    #[test]
    fn test_execution_mode_one_shot() {
        let mode = determine_execution_mode(&RiskLevel::Safe, false);
        assert_eq!(mode, ExecutionMode::PlanOnly);

        let mode = determine_execution_mode(&RiskLevel::Moderate, false);
        assert_eq!(mode, ExecutionMode::PlanOnly);
    }

    #[test]
    fn test_plan_summary_risk_description() {
        let summary = PlanSummary {
            description: "Test".to_string(),
            risk_level: RiskLevel::Safe,
            domains: vec![],
            command_count: 1,
            will_create_backups: false,
            execution_mode: ExecutionMode::ConfirmRequired,
        };

        assert!(summary.risk_description().contains("Safe"));

        let summary_with_backup = PlanSummary {
            risk_level: RiskLevel::Moderate,
            will_create_backups: true,
            ..summary.clone()
        };

        assert!(summary_with_backup.risk_description().contains("backup"));
    }
}

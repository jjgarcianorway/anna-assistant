//! Action Proposal v0.0.67 - Safe, Rollbackable Actions
//!
//! Provides structured action proposals from departments:
//! - ActionProposal: Complete action specification with rollback
//! - ProposalRisk: Categorized risk levels with confirmation gates
//! - Confirmation phrases match existing policy system
//!
//! All proposals must be confirmed before execution.
//! High-risk actions require explicit confirmation phrases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::case_lifecycle::ActionRisk;

// ============================================================================
// Action Proposal
// ============================================================================

/// A proposed action from a department
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProposal {
    /// Unique ID for this proposal
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Risk level
    pub risk: ActionRisk,
    /// Whether confirmation is required
    pub requires_confirmation: bool,
    /// Confirmation phrase (if required)
    pub confirmation_phrase: Option<String>,
    /// Human-readable rollback plan
    pub rollback_plan_human: String,
    /// Steps in plain English (for human mode)
    pub steps_human: Vec<String>,
    /// Exact commands/operations (for debug mode)
    pub steps_debug: Vec<String>,
    /// Affected packages
    pub affected_packages: Vec<String>,
    /// Affected services
    pub affected_services: Vec<String>,
    /// Affected files
    pub affected_files: Vec<String>,
}

impl ActionProposal {
    /// Create a read-only (informational) action
    pub fn read_only(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            risk: ActionRisk::ReadOnly,
            requires_confirmation: false,
            confirmation_phrase: None,
            rollback_plan_human: "No changes to rollback".to_string(),
            steps_human: Vec::new(),
            steps_debug: Vec::new(),
            affected_packages: Vec::new(),
            affected_services: Vec::new(),
            affected_files: Vec::new(),
        }
    }

    /// Create a low-risk action (simple y/n confirmation)
    pub fn low_risk(id: &str, title: &str, rollback: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            risk: ActionRisk::Low,
            requires_confirmation: true,
            confirmation_phrase: Some("y".to_string()),
            rollback_plan_human: rollback.to_string(),
            steps_human: Vec::new(),
            steps_debug: Vec::new(),
            affected_packages: Vec::new(),
            affected_services: Vec::new(),
            affected_files: Vec::new(),
        }
    }

    /// Create a medium-risk action (explicit confirmation)
    pub fn medium_risk(id: &str, title: &str, rollback: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            risk: ActionRisk::Medium,
            requires_confirmation: true,
            confirmation_phrase: Some("confirm".to_string()),
            rollback_plan_human: rollback.to_string(),
            steps_human: Vec::new(),
            steps_debug: Vec::new(),
            affected_packages: Vec::new(),
            affected_services: Vec::new(),
            affected_files: Vec::new(),
        }
    }

    /// Create a high-risk action (strong confirmation phrase)
    pub fn high_risk(id: &str, title: &str, rollback: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            risk: ActionRisk::High,
            requires_confirmation: true,
            confirmation_phrase: Some("I assume the risk".to_string()),
            rollback_plan_human: rollback.to_string(),
            steps_human: Vec::new(),
            steps_debug: Vec::new(),
            affected_packages: Vec::new(),
            affected_services: Vec::new(),
            affected_files: Vec::new(),
        }
    }

    // Builder methods

    pub fn with_steps_human(mut self, steps: Vec<String>) -> Self {
        self.steps_human = steps;
        self
    }

    pub fn with_steps_debug(mut self, steps: Vec<String>) -> Self {
        self.steps_debug = steps;
        self
    }

    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    pub fn with_services(mut self, services: Vec<String>) -> Self {
        self.affected_services = services;
        self
    }

    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.affected_files = files;
        self
    }

    /// Human-readable description of the action
    pub fn human_description(&self) -> String {
        let mut parts = vec![self.title.clone()];

        if !self.steps_human.is_empty() {
            parts.push(format!("Steps: {}", self.steps_human.join(", ")));
        }

        if self.requires_confirmation {
            let phrase = self.confirmation_phrase.as_deref().unwrap_or("y");
            parts.push(format!("Requires confirmation: type \"{}\"", phrase));
        }

        parts.join("\n")
    }

    /// Debug description with commands
    pub fn debug_description(&self) -> String {
        let mut parts = vec![
            format!("[{}] {} (risk: {:?})", self.id, self.title, self.risk),
        ];

        if !self.steps_debug.is_empty() {
            parts.push(format!("Commands:\n  {}", self.steps_debug.join("\n  ")));
        }

        if !self.affected_services.is_empty() {
            parts.push(format!("Services: {}", self.affected_services.join(", ")));
        }

        if !self.affected_packages.is_empty() {
            parts.push(format!("Packages: {}", self.affected_packages.join(", ")));
        }

        if !self.affected_files.is_empty() {
            parts.push(format!("Files: {}", self.affected_files.join(", ")));
        }

        parts.push(format!("Rollback: {}", self.rollback_plan_human));

        parts.join("\n")
    }

    /// Check if confirmation matches
    pub fn check_confirmation(&self, input: &str) -> bool {
        if !self.requires_confirmation {
            return true;
        }

        match &self.confirmation_phrase {
            Some(phrase) => input.trim().to_lowercase() == phrase.to_lowercase(),
            None => true,
        }
    }
}

// ============================================================================
// Common Action Proposals
// ============================================================================

/// Common networking actions
pub mod networking_actions {
    use super::*;

    pub fn restart_networkmanager() -> ActionProposal {
        ActionProposal::low_risk(
            "net_restart_nm",
            "Restart NetworkManager",
            "NetworkManager will restart automatically; connections will reconnect",
        )
        .with_steps_human(vec![
            "Restart the NetworkManager service".to_string(),
            "Wait for connections to re-establish".to_string(),
        ])
        .with_steps_debug(vec![
            "systemctl restart NetworkManager".to_string(),
        ])
        .with_services(vec!["NetworkManager.service".to_string()])
    }

    pub fn restart_iwd() -> ActionProposal {
        ActionProposal::low_risk(
            "net_restart_iwd",
            "Restart iwd (WiFi daemon)",
            "iwd will restart; WiFi connections will need to reconnect",
        )
        .with_steps_human(vec![
            "Restart the iwd WiFi service".to_string(),
            "Reconnect to WiFi network".to_string(),
        ])
        .with_steps_debug(vec![
            "systemctl restart iwd".to_string(),
        ])
        .with_services(vec!["iwd.service".to_string()])
    }

    pub fn reconnect_wifi(ssid: &str) -> ActionProposal {
        ActionProposal::low_risk(
            "net_reconnect_wifi",
            &format!("Reconnect to WiFi network '{}'", ssid),
            "Previous connection state will be lost",
        )
        .with_steps_human(vec![
            format!("Disconnect from current WiFi"),
            format!("Reconnect to network '{}'", ssid),
        ])
        .with_steps_debug(vec![
            format!("nmcli device wifi rescan"),
            format!("nmcli device wifi connect \"{}\"", ssid),
        ])
    }

    pub fn flush_dns_cache() -> ActionProposal {
        ActionProposal::read_only(
            "net_flush_dns",
            "Flush DNS cache",
        )
        .with_steps_human(vec![
            "Clear cached DNS entries".to_string(),
            "New lookups will query DNS servers directly".to_string(),
        ])
        .with_steps_debug(vec![
            "resolvectl flush-caches".to_string(),
        ])
    }

    pub fn switch_to_iwd() -> ActionProposal {
        ActionProposal::high_risk(
            "net_switch_iwd",
            "Switch WiFi management from NetworkManager to iwd",
            "Revert by disabling iwd and re-enabling NetworkManager WiFi backend",
        )
        .with_steps_human(vec![
            "Install iwd if not present".to_string(),
            "Configure NetworkManager to use iwd backend".to_string(),
            "Restart services".to_string(),
        ])
        .with_steps_debug(vec![
            "pacman -S --noconfirm iwd".to_string(),
            "mkdir -p /etc/NetworkManager/conf.d".to_string(),
            "echo '[device]\nwifi.backend=iwd' > /etc/NetworkManager/conf.d/wifi_backend.conf".to_string(),
            "systemctl enable --now iwd".to_string(),
            "systemctl restart NetworkManager".to_string(),
        ])
        .with_packages(vec!["iwd".to_string()])
        .with_services(vec!["iwd.service".to_string(), "NetworkManager.service".to_string()])
        .with_files(vec!["/etc/NetworkManager/conf.d/wifi_backend.conf".to_string()])
    }

    pub fn install_helper(package: &str, purpose: &str) -> ActionProposal {
        ActionProposal::medium_risk(
            &format!("net_install_{}", package),
            &format!("Install {} ({})", package, purpose),
            &format!("Remove with: pacman -R {}", package),
        )
        .with_steps_human(vec![
            format!("Install the {} package", package),
        ])
        .with_steps_debug(vec![
            format!("pacman -S --noconfirm {}", package),
        ])
        .with_packages(vec![package.to_string()])
    }
}

/// Common storage actions
pub mod storage_actions {
    use super::*;

    pub fn check_filesystem(mount: &str) -> ActionProposal {
        ActionProposal::read_only(
            "storage_check_fs",
            &format!("Check filesystem on {}", mount),
        )
        .with_steps_human(vec![
            format!("Run filesystem check on {}", mount),
        ])
    }

    pub fn btrfs_scrub_start(mount: &str) -> ActionProposal {
        ActionProposal::medium_risk(
            "storage_btrfs_scrub",
            &format!("Start BTRFS scrub on {}", mount),
            "Scrub can be cancelled with: btrfs scrub cancel",
        )
        .with_steps_human(vec![
            format!("Start background scrub to verify data integrity on {}", mount),
            "This may take several hours depending on data size".to_string(),
        ])
        .with_steps_debug(vec![
            format!("btrfs scrub start {}", mount),
        ])
    }

    pub fn btrfs_balance_start(mount: &str) -> ActionProposal {
        ActionProposal::high_risk(
            "storage_btrfs_balance",
            &format!("Start BTRFS balance on {}", mount),
            "Balance can be cancelled with: btrfs balance cancel. May take hours.",
        )
        .with_steps_human(vec![
            format!("Rebalance data across devices on {}", mount),
            "WARNING: This can take many hours and heavily use disk I/O".to_string(),
        ])
        .with_steps_debug(vec![
            format!("btrfs balance start {}", mount),
        ])
    }

    pub fn clean_package_cache() -> ActionProposal {
        ActionProposal::low_risk(
            "storage_clean_cache",
            "Clean package cache",
            "Removed packages can be re-downloaded if needed",
        )
        .with_steps_human(vec![
            "Remove old package versions from cache".to_string(),
            "Keep only the 3 most recent versions".to_string(),
        ])
        .with_steps_debug(vec![
            "paccache -r".to_string(),
        ])
    }

    pub fn create_btrfs_snapshot(subvol: &str, name: &str) -> ActionProposal {
        ActionProposal::low_risk(
            "storage_btrfs_snapshot",
            &format!("Create BTRFS snapshot of {}", subvol),
            &format!("Delete snapshot with: btrfs subvolume delete {}", name),
        )
        .with_steps_human(vec![
            format!("Create read-only snapshot of {}", subvol),
        ])
        .with_steps_debug(vec![
            format!("btrfs subvolume snapshot -r {} {}", subvol, name),
        ])
    }

    pub fn dangerous_btrfs_check_repair(mount: &str) -> ActionProposal {
        ActionProposal::high_risk(
            "storage_btrfs_repair",
            &format!("Run BTRFS repair on {} (DANGEROUS)", mount),
            "CANNOT BE UNDONE. Ensure you have backups before proceeding.",
        )
        .with_steps_human(vec![
            "WARNING: This is a destructive operation".to_string(),
            format!("Run btrfs check --repair on {}", mount),
            "Only use as last resort after data backup".to_string(),
        ])
        .with_steps_debug(vec![
            format!("# WARNING: Requires unmounted filesystem"),
            format!("umount {}", mount),
            format!("btrfs check --repair /dev/XXX"),
        ])
    }

    pub fn do_nothing() -> ActionProposal {
        ActionProposal::read_only(
            "storage_do_nothing",
            "No action needed - storage is healthy",
        )
        .with_steps_human(vec![
            "Continue monitoring system health".to_string(),
        ])
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_proposal_read_only() {
        let action = ActionProposal::read_only("test", "Test action");
        assert_eq!(action.risk, ActionRisk::ReadOnly);
        assert!(!action.requires_confirmation);
    }

    #[test]
    fn test_action_proposal_low_risk() {
        let action = ActionProposal::low_risk("test", "Test", "Undo by...");
        assert_eq!(action.risk, ActionRisk::Low);
        assert!(action.requires_confirmation);
        assert!(action.check_confirmation("y"));
        assert!(action.check_confirmation("Y"));
        assert!(!action.check_confirmation("no"));
    }

    #[test]
    fn test_action_proposal_high_risk() {
        let action = ActionProposal::high_risk("test", "Dangerous", "Cannot undo");
        assert_eq!(action.risk, ActionRisk::High);
        assert!(action.check_confirmation("I assume the risk"));
        assert!(!action.check_confirmation("yes"));
        assert!(!action.check_confirmation("y"));
    }

    #[test]
    fn test_networking_actions() {
        let restart = networking_actions::restart_networkmanager();
        assert!(restart.affected_services.contains(&"NetworkManager.service".to_string()));

        let switch = networking_actions::switch_to_iwd();
        assert_eq!(switch.risk, ActionRisk::High);
        assert!(switch.affected_packages.contains(&"iwd".to_string()));
    }

    #[test]
    fn test_storage_actions() {
        let scrub = storage_actions::btrfs_scrub_start("/");
        assert_eq!(scrub.risk, ActionRisk::Medium);

        let repair = storage_actions::dangerous_btrfs_check_repair("/");
        assert_eq!(repair.risk, ActionRisk::High);
        assert!(repair.title.contains("DANGEROUS"));
    }

    #[test]
    fn test_human_description_no_commands() {
        let action = networking_actions::restart_networkmanager();
        let human = action.human_description();
        assert!(human.contains("Restart"));
        assert!(!human.contains("systemctl"));
    }

    #[test]
    fn test_debug_description_has_commands() {
        let action = networking_actions::restart_networkmanager();
        let debug = action.debug_description();
        assert!(debug.contains("systemctl restart"));
    }
}

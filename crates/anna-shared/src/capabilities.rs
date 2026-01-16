//! Capability Ledger - The single source of truth for Anna's powers (Phase 45)
//!
//! This module defines every capability boundary in the system.
//! It serves as both documentation and enforcement mechanism.
//!
//! # Purpose
//!
//! Anna is powerful enough that trust must be structural, not remembered.
//! This ledger makes every capability explicit, inspectable, and versioned.
//!
//! # Rules
//!
//! 1. Every execution capability MUST have a corresponding entry here
//! 2. Every entry MUST have tests proving both success and rejection
//! 3. Changes to this ledger REQUIRE a version bump
//! 4. The ledger is documentation-first but test-enforced
//!
//! # Categories
//!
//! - Diagnosis: Reading system state (safe, no confirmation)
//! - Proposal: Suggesting actions (no execution)
//! - Execution: Running commands (requires human confirmation)
//! - Filesystem: Reading/writing files (restricted paths)
//! - Network: Network operations (currently none)

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Ledger version - must be bumped when capabilities change.
/// Format: LEDGER_MAJOR.LEDGER_MINOR (independent of crate version)
pub const LEDGER_VERSION: &str = "1.0";

/// Execution level for a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionLevel {
    /// Cannot execute anything
    None,
    /// Commands shown to user for manual copy/paste
    ManualOnly,
    /// Requires human to provide command at runtime
    HumanConfirmed,
    /// Safe commands can auto-execute after confirmation phrase
    HumanConfirmedSafeAutomatic,
}

impl ExecutionLevel {
    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionLevel::None => "No execution capability",
            ExecutionLevel::ManualOnly => "Commands shown for manual execution",
            ExecutionLevel::HumanConfirmed => "Human must provide command at runtime",
            ExecutionLevel::HumanConfirmedSafeAutomatic => {
                "Safe commands auto-execute after exact confirmation"
            }
        }
    }
}

/// Category of capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityCategory {
    /// Reading system state (lspci, lsmod, etc.)
    Diagnosis,
    /// Suggesting fixes without execution
    Proposal,
    /// Running commands on the system
    Execution,
    /// Reading files from filesystem
    FilesystemRead,
    /// Writing files to filesystem
    FilesystemWrite,
    /// Network operations
    Network,
    /// Audio/video media operations
    Media,
    /// Package management
    PackageManagement,
}

/// Persistence behavior for a capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersistenceBehavior {
    /// No persistence
    None,
    /// Audit record only (ExecutionAttempt)
    AuditOnly,
    /// Changes system state
    StateChanging,
}

/// Arguments policy for allowed binaries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgsPolicy {
    /// Any arguments allowed
    Unrestricted,
    /// Only specific arguments allowed
    Restricted(Vec<String>),
    /// No arguments allowed
    NoArgs,
}

/// A single capability entry in the ledger.
/// Note: Only Serialize is derived since this is a static ledger.
/// Deserialization is not needed - the ledger is defined in code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Capability {
    /// Unique identifier for this capability
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Category of capability
    pub category: CapabilityCategory,
    /// Execution level
    pub execution_level: ExecutionLevel,
    /// Allowed binaries (empty if none)
    pub allowed_binaries: &'static [&'static str],
    /// Arguments policy
    pub args_policy: &'static str,
    /// Required confirmation text (None if no confirmation needed)
    pub requires_confirmation: Option<&'static str>,
    /// Persistence behavior
    pub persistence: PersistenceBehavior,
    /// Module where this capability is implemented
    pub implementation_module: &'static str,
}

// =============================================================================
// THE CAPABILITY LEDGER
// =============================================================================
//
// This is the authoritative list of what Anna can do.
// Changes here REQUIRE a version bump and test updates.
//
// =============================================================================

/// All registered capabilities in the system.
pub static CAPABILITIES: &[Capability] = &[
    // -------------------------------------------------------------------------
    // DIAGNOSIS CAPABILITIES (read-only, no confirmation)
    // -------------------------------------------------------------------------
    Capability {
        name: "system_state_diagnosis",
        description: "Read system state via diagnostic commands",
        category: CapabilityCategory::Diagnosis,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &["lspci", "lsusb", "lsmod", "lscpu", "free", "df", "uname", "hostname"],
        args_policy: "Read-only flags only (-m, -mm, -k, -g, -h, -r)",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "anna_shared::profile, anna_shared::monitor",
    },
    Capability {
        name: "wifi_diagnosis",
        description: "Diagnose WiFi issues by reading wireless state",
        category: CapabilityCategory::Diagnosis,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &["iw", "lsmod", "lspci"],
        args_policy: "Read-only: 'iw <dev> link', 'lsmod', 'lspci -k'",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "annad::assisted_ops::detection",
    },
    Capability {
        name: "config_file_read",
        description: "Read configuration files for diagnosis",
        category: CapabilityCategory::FilesystemRead,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &["cat"],
        args_policy: "Specific paths only: /etc/modprobe.d/*.conf",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "annad::assisted_ops::detection",
    },

    // -------------------------------------------------------------------------
    // PROPOSAL CAPABILITIES (no execution)
    // -------------------------------------------------------------------------
    Capability {
        name: "assisted_operation_proposal",
        description: "Propose fixes as AssistedOperation structures",
        category: CapabilityCategory::Proposal,
        execution_level: ExecutionLevel::ManualOnly,
        allowed_binaries: &[],
        args_policy: "No binaries - proposals are data structures",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "annad::assisted_ops::types",
    },
    Capability {
        name: "execution_request_creation",
        description: "Create ExecutionRequest for human review",
        category: CapabilityCategory::Proposal,
        execution_level: ExecutionLevel::ManualOnly,
        allowed_binaries: &[],
        args_policy: "No binaries - requests are data structures",
        requires_confirmation: None,
        persistence: PersistenceBehavior::AuditOnly,
        implementation_module: "anna_shared::execution_request",
    },

    // -------------------------------------------------------------------------
    // EXECUTION CAPABILITIES (requires human confirmation)
    // -------------------------------------------------------------------------
    Capability {
        name: "human_mediated_execution",
        description: "Execute commands via HumanExecutionAdapter",
        category: CapabilityCategory::Execution,
        execution_level: ExecutionLevel::HumanConfirmedSafeAutomatic,
        allowed_binaries: &["iw", "lsmod", "lspci", "cat", "echo"],
        args_policy: "No sudo, no pipes, no redirects, no command substitution",
        requires_confirmation: Some("I understand this will not execute automatically."),
        persistence: PersistenceBehavior::AuditOnly,
        implementation_module: "anna_shared::human_execution",
    },
    Capability {
        name: "automatic_safe_execution",
        description: "Auto-execute safe commands after explicit confirmation",
        category: CapabilityCategory::Execution,
        execution_level: ExecutionLevel::HumanConfirmedSafeAutomatic,
        allowed_binaries: &["iw", "lsmod", "lspci", "cat", "echo"],
        args_policy: "Same restrictions as human_mediated_execution",
        requires_confirmation: Some("I understand this will execute automatically."),
        persistence: PersistenceBehavior::AuditOnly,
        implementation_module: "annactl::repair",
    },

    // -------------------------------------------------------------------------
    // FILESYSTEM CAPABILITIES
    // -------------------------------------------------------------------------
    Capability {
        name: "state_persistence",
        description: "Write Anna's own state files",
        category: CapabilityCategory::FilesystemWrite,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &[],
        args_policy: "Rust std::fs only, paths under /var/lib/anna/",
        requires_confirmation: None,
        persistence: PersistenceBehavior::StateChanging,
        implementation_module: "anna_shared::paths, anna_shared::safe_ops",
    },
    Capability {
        name: "audit_logging",
        description: "Write execution attempt audit records",
        category: CapabilityCategory::FilesystemWrite,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &[],
        args_policy: "Rust std::fs only, paths under /var/lib/anna/execution_attempts/",
        requires_confirmation: None,
        persistence: PersistenceBehavior::AuditOnly,
        implementation_module: "anna_shared::human_execution",
    },

    // -------------------------------------------------------------------------
    // EXPLICIT NON-CAPABILITIES
    // -------------------------------------------------------------------------
    Capability {
        name: "NO_network_requests",
        description: "Anna cannot make arbitrary network requests",
        category: CapabilityCategory::Network,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &[],
        args_policy: "FORBIDDEN - no wget, curl, nc, etc. in execution path",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "FORBIDDEN",
    },
    Capability {
        name: "NO_package_installation",
        description: "Anna cannot install packages automatically",
        category: CapabilityCategory::PackageManagement,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &[],
        args_policy: "FORBIDDEN - pacman, yay, apt etc. not in allowlist",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "FORBIDDEN",
    },
    Capability {
        name: "NO_sudo_execution",
        description: "Anna cannot execute sudo commands automatically",
        category: CapabilityCategory::Execution,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &[],
        args_policy: "FORBIDDEN - sudo, pkexec, doas rejected by HumanExecutionAdapter",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "FORBIDDEN",
    },
    Capability {
        name: "NO_destructive_commands",
        description: "Anna cannot run destructive commands",
        category: CapabilityCategory::Execution,
        execution_level: ExecutionLevel::None,
        allowed_binaries: &[],
        args_policy: "FORBIDDEN - rm, dd, mkfs, fdisk rejected by HumanExecutionAdapter",
        requires_confirmation: None,
        persistence: PersistenceBehavior::None,
        implementation_module: "FORBIDDEN",
    },
];

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Get all capabilities in a specific category.
pub fn capabilities_by_category(category: CapabilityCategory) -> Vec<&'static Capability> {
    CAPABILITIES
        .iter()
        .filter(|c| c.category == category)
        .collect()
}

/// Get all capabilities that allow execution.
pub fn execution_capabilities() -> Vec<&'static Capability> {
    CAPABILITIES
        .iter()
        .filter(|c| {
            matches!(
                c.execution_level,
                ExecutionLevel::HumanConfirmed | ExecutionLevel::HumanConfirmedSafeAutomatic
            )
        })
        .collect()
}

/// Get the complete set of allowed binaries across all capabilities.
pub fn all_allowed_binaries() -> HashSet<&'static str> {
    CAPABILITIES
        .iter()
        .flat_map(|c| c.allowed_binaries.iter().copied())
        .collect()
}

/// Get capabilities that are explicitly forbidden (NO_ prefix).
pub fn forbidden_capabilities() -> Vec<&'static Capability> {
    CAPABILITIES
        .iter()
        .filter(|c| c.name.starts_with("NO_"))
        .collect()
}

/// Verify that the HumanExecutionAdapter allowlist matches the ledger.
/// Returns mismatches if any.
pub fn verify_execution_allowlist_consistency(
    adapter_allowlist: &[&str],
) -> Result<(), Vec<String>> {
    let ledger_binaries: HashSet<&str> = CAPABILITIES
        .iter()
        .filter(|c| c.name == "human_mediated_execution" || c.name == "automatic_safe_execution")
        .flat_map(|c| c.allowed_binaries.iter().copied())
        .collect();

    let adapter_set: HashSet<&str> = adapter_allowlist.iter().copied().collect();

    let mut mismatches = Vec::new();

    // Check for binaries in adapter but not in ledger
    for binary in &adapter_set {
        if !ledger_binaries.contains(binary) {
            mismatches.push(format!(
                "Binary '{}' in adapter allowlist but not in capability ledger",
                binary
            ));
        }
    }

    // Check for binaries in ledger but not in adapter
    for binary in &ledger_binaries {
        if !adapter_set.contains(binary) {
            mismatches.push(format!(
                "Binary '{}' in capability ledger but not in adapter allowlist",
                binary
            ));
        }
    }

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(mismatches)
    }
}

// =============================================================================
// TRUST SURFACE REPORT
// =============================================================================

/// Generate a trust surface report as a formatted string.
pub fn generate_trust_surface_report() -> String {
    let mut report = String::new();

    report.push_str("# Anna Trust Surface Report\n\n");
    report.push_str(&format!("Ledger Version: {}\n", LEDGER_VERSION));
    report.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // What Anna can read
    report.push_str("## What Anna Can Read\n\n");
    for cap in capabilities_by_category(CapabilityCategory::Diagnosis) {
        report.push_str(&format!("- **{}**: {}\n", cap.name, cap.description));
        if !cap.allowed_binaries.is_empty() {
            report.push_str(&format!("  - Binaries: {}\n", cap.allowed_binaries.join(", ")));
        }
        report.push_str(&format!("  - Args policy: {}\n", cap.args_policy));
    }
    for cap in capabilities_by_category(CapabilityCategory::FilesystemRead) {
        report.push_str(&format!("- **{}**: {}\n", cap.name, cap.description));
        report.push_str(&format!("  - Args policy: {}\n", cap.args_policy));
    }
    report.push('\n');

    // What Anna can suggest
    report.push_str("## What Anna Can Suggest\n\n");
    for cap in capabilities_by_category(CapabilityCategory::Proposal) {
        report.push_str(&format!("- **{}**: {}\n", cap.name, cap.description));
        report.push_str(&format!(
            "  - Execution level: {}\n",
            cap.execution_level.description()
        ));
    }
    report.push('\n');

    // What Anna can execute
    report.push_str("## What Anna Can Execute\n\n");
    for cap in execution_capabilities() {
        report.push_str(&format!("- **{}**: {}\n", cap.name, cap.description));
        report.push_str(&format!(
            "  - Execution level: {}\n",
            cap.execution_level.description()
        ));
        if !cap.allowed_binaries.is_empty() {
            report.push_str(&format!("  - Binaries: {}\n", cap.allowed_binaries.join(", ")));
        }
        report.push_str(&format!("  - Args policy: {}\n", cap.args_policy));
        if let Some(conf) = cap.requires_confirmation {
            report.push_str(&format!("  - Confirmation required: \"{}\"\n", conf));
        }
    }
    report.push('\n');

    // What Anna will never do
    report.push_str("## What Anna Will NEVER Do\n\n");
    for cap in forbidden_capabilities() {
        report.push_str(&format!(
            "- **{}**: {}\n",
            cap.name.strip_prefix("NO_").unwrap_or(cap.name),
            cap.description
        ));
        report.push_str(&format!("  - Policy: {}\n", cap.args_policy));
    }
    report.push('\n');

    // Summary statistics
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- Total capabilities defined: {}\n", CAPABILITIES.len()));
    report.push_str(&format!(
        "- Execution capabilities: {}\n",
        execution_capabilities().len()
    ));
    report.push_str(&format!(
        "- Forbidden capabilities: {}\n",
        forbidden_capabilities().len()
    ));
    report.push_str(&format!(
        "- Unique allowed binaries: {}\n",
        all_allowed_binaries().len()
    ));

    report
}

/// Generate a deterministic trust surface (without timestamp for diffing).
pub fn generate_deterministic_trust_surface() -> String {
    let mut report = String::new();

    report.push_str("# Anna Trust Surface (Deterministic)\n\n");
    report.push_str(&format!("Ledger Version: {}\n\n", LEDGER_VERSION));

    // Sorted list of all capabilities
    let mut caps: Vec<_> = CAPABILITIES.iter().collect();
    caps.sort_by_key(|c| c.name);

    for cap in caps {
        report.push_str(&format!("## {}\n", cap.name));
        report.push_str(&format!("- Description: {}\n", cap.description));
        report.push_str(&format!("- Category: {:?}\n", cap.category));
        report.push_str(&format!("- Execution: {:?}\n", cap.execution_level));
        if !cap.allowed_binaries.is_empty() {
            let mut bins: Vec<_> = cap.allowed_binaries.to_vec();
            bins.sort();
            report.push_str(&format!("- Binaries: {}\n", bins.join(", ")));
        }
        report.push_str(&format!("- Args: {}\n", cap.args_policy));
        if let Some(conf) = cap.requires_confirmation {
            report.push_str(&format!("- Confirmation: \"{}\"\n", conf));
        }
        report.push_str(&format!("- Persistence: {:?}\n", cap.persistence));
        report.push_str(&format!("- Module: {}\n", cap.implementation_module));
        report.push('\n');
    }

    report
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // LEDGER INTEGRITY TESTS
    // =========================================================================

    #[test]
    fn test_all_capabilities_have_unique_names() {
        let mut names = HashSet::new();
        for cap in CAPABILITIES {
            assert!(
                names.insert(cap.name),
                "Duplicate capability name: {}",
                cap.name
            );
        }
    }

    #[test]
    fn test_all_capabilities_have_descriptions() {
        for cap in CAPABILITIES {
            assert!(
                !cap.description.is_empty(),
                "Capability {} has empty description",
                cap.name
            );
        }
    }

    #[test]
    fn test_all_capabilities_have_implementation_module() {
        for cap in CAPABILITIES {
            assert!(
                !cap.implementation_module.is_empty(),
                "Capability {} has empty implementation_module",
                cap.name
            );
        }
    }

    #[test]
    fn test_execution_capabilities_have_confirmation() {
        for cap in execution_capabilities() {
            if cap.execution_level == ExecutionLevel::HumanConfirmedSafeAutomatic {
                assert!(
                    cap.requires_confirmation.is_some(),
                    "Execution capability {} must have confirmation string",
                    cap.name
                );
            }
        }
    }

    #[test]
    fn test_forbidden_capabilities_have_no_execution() {
        for cap in forbidden_capabilities() {
            assert_eq!(
                cap.execution_level,
                ExecutionLevel::None,
                "Forbidden capability {} must have ExecutionLevel::None",
                cap.name
            );
            assert!(
                cap.allowed_binaries.is_empty(),
                "Forbidden capability {} must have empty allowed_binaries",
                cap.name
            );
        }
    }

    // =========================================================================
    // ALLOWLIST CONSISTENCY TESTS
    // =========================================================================

    #[test]
    fn test_adapter_allowlist_matches_ledger() {
        // This is the actual allowlist from HumanExecutionAdapter
        const ADAPTER_ALLOWLIST: &[&str] = &["iw", "lsmod", "lspci", "cat", "echo"];

        let result = verify_execution_allowlist_consistency(ADAPTER_ALLOWLIST);
        assert!(
            result.is_ok(),
            "Allowlist mismatch: {:?}",
            result.err().unwrap()
        );
    }

    #[test]
    fn test_allowlist_mismatch_detection() {
        // Test that mismatches are detected
        let wrong_allowlist = &["iw", "lsmod", "wget"]; // wget is not in ledger
        let result = verify_execution_allowlist_consistency(wrong_allowlist);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("wget")));
    }

    // =========================================================================
    // TRUST SURFACE TESTS
    // =========================================================================

    #[test]
    fn test_trust_surface_report_generates() {
        let report = generate_trust_surface_report();
        assert!(!report.is_empty());
        assert!(report.contains("Trust Surface Report"));
        assert!(report.contains("What Anna Can Read"));
        assert!(report.contains("What Anna Can Execute"));
        assert!(report.contains("What Anna Will NEVER Do"));
    }

    #[test]
    fn test_deterministic_surface_is_stable() {
        let report1 = generate_deterministic_trust_surface();
        let report2 = generate_deterministic_trust_surface();
        assert_eq!(report1, report2, "Deterministic surface must be stable");
    }

    // =========================================================================
    // BOUNDARY TESTS
    // =========================================================================

    #[test]
    fn test_no_network_binaries_allowed() {
        let network_binaries = ["wget", "curl", "nc", "netcat", "ssh", "scp"];
        let allowed = all_allowed_binaries();

        for binary in &network_binaries {
            assert!(
                !allowed.contains(binary),
                "Network binary '{}' must not be in allowed list",
                binary
            );
        }
    }

    #[test]
    fn test_no_package_manager_binaries_allowed() {
        let pkg_binaries = ["pacman", "yay", "apt", "apt-get", "dnf", "yum", "zypper"];
        let allowed = all_allowed_binaries();

        for binary in &pkg_binaries {
            assert!(
                !allowed.contains(binary),
                "Package manager '{}' must not be in allowed list",
                binary
            );
        }
    }

    #[test]
    fn test_no_destructive_binaries_allowed() {
        let destructive = ["rm", "dd", "mkfs", "fdisk", "parted", "shred"];
        let allowed = all_allowed_binaries();

        for binary in &destructive {
            assert!(
                !allowed.contains(binary),
                "Destructive binary '{}' must not be in allowed list",
                binary
            );
        }
    }

    #[test]
    fn test_no_privilege_escalation_allowed() {
        let escalation = ["sudo", "pkexec", "doas", "su"];
        let allowed = all_allowed_binaries();

        for binary in &escalation {
            assert!(
                !allowed.contains(binary),
                "Privilege escalation '{}' must not be in allowed list",
                binary
            );
        }
    }

    // =========================================================================
    // CONFIRMATION STRING TESTS
    // =========================================================================

    #[test]
    fn test_confirmation_strings_are_exact() {
        // These are the canonical confirmation strings
        const MANUAL_CONFIRMATION: &str = "I understand this will not execute automatically.";
        const AUTO_CONFIRMATION: &str = "I understand this will execute automatically.";

        let mut found_manual = false;
        let mut found_auto = false;

        for cap in CAPABILITIES {
            if let Some(conf) = cap.requires_confirmation {
                if conf == MANUAL_CONFIRMATION {
                    found_manual = true;
                }
                if conf == AUTO_CONFIRMATION {
                    found_auto = true;
                }
            }
        }

        assert!(found_manual, "Manual confirmation string not found in ledger");
        assert!(found_auto, "Auto confirmation string not found in ledger");
    }

    // =========================================================================
    // CATEGORY COVERAGE TESTS
    // =========================================================================

    #[test]
    fn test_diagnosis_category_exists() {
        let caps = capabilities_by_category(CapabilityCategory::Diagnosis);
        assert!(!caps.is_empty(), "Must have diagnosis capabilities");
    }

    #[test]
    fn test_proposal_category_exists() {
        let caps = capabilities_by_category(CapabilityCategory::Proposal);
        assert!(!caps.is_empty(), "Must have proposal capabilities");
    }

    #[test]
    fn test_execution_category_exists() {
        let caps = capabilities_by_category(CapabilityCategory::Execution);
        assert!(!caps.is_empty(), "Must have execution capabilities");
    }
}

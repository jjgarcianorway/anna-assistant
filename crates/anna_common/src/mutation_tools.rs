//! Mutation Tools for Anna v0.0.14
//!
//! Safe mutation operations with policy-driven enforcement:
//! - Config file edits (line insert/replace/delete)
//! - Systemd service operations (restart, reload, enable, disable)
//! - Package management (install, remove) with helper tracking
//!
//! Security model (v0.0.14 - Policy-Driven):
//! - All allow/deny decisions come from /etc/anna/policy/
//! - Explicit confirmation phrases defined in risk.toml
//! - Automatic rollback support with helper provenance tracking
//! - Path allowlist/blocklist in capabilities.toml and blocked.toml
//! - File size limits from policy
//! - Package blocks from policy (kernel, bootloader, init blocked by default)
//! - All policy checks generate evidence IDs for audit trail

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Maximum file size for edits (1 MiB)
pub const MAX_EDIT_FILE_SIZE: u64 = 1024 * 1024;

/// Confirmation phrase for medium-risk operations
pub const MEDIUM_RISK_CONFIRMATION: &str = "I CONFIRM (medium risk)";

/// Mutation security classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationRisk {
    /// Medium risk: config edits, service restarts (allowed in v0.0.8)
    Medium,
    /// High risk: package ops, filesystem ops (NOT allowed in v0.0.8)
    High,
}

impl std::fmt::Display for MutationRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationRisk::Medium => write!(f, "medium"),
            MutationRisk::High => write!(f, "high"),
        }
    }
}

/// Mutation tool definition
#[derive(Debug, Clone)]
pub struct MutationToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub risk: MutationRisk,
    pub parameters: &'static [(&'static str, &'static str, bool)], // (name, description, required)
    pub rollback_supported: bool,
    pub human_action: &'static str, // Human-readable action description
}

/// File edit operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileEditOp {
    /// Insert line at position (0-indexed, line_number, content)
    InsertLine { line_number: usize, content: String },
    /// Replace line at position (0-indexed)
    ReplaceLine { line_number: usize, content: String },
    /// Delete line at position (0-indexed)
    DeleteLine { line_number: usize },
    /// Append line at end
    AppendLine { content: String },
    /// Replace text matching pattern
    ReplaceText { pattern: String, replacement: String },
}

/// Mutation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRequest {
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub confirmation_token: Option<String>,
    pub evidence_ids: Vec<String>, // Evidence IDs that justify this mutation
    pub request_id: String,
}

/// Mutation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub tool_name: String,
    pub success: bool,
    pub error: Option<String>,
    pub human_summary: String,
    pub rollback_info: Option<RollbackInfo>,
    pub request_id: String,
    pub timestamp: u64,
}

/// Rollback information for a mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub backup_path: Option<PathBuf>,
    pub rollback_command: Option<String>,
    pub rollback_instructions: String,
    pub prior_state: Option<String>,
}

/// Mutation plan (before execution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationPlan {
    pub mutations: Vec<MutationRequest>,
    pub what_will_change: String,
    pub why_required: String,
    pub risk: MutationRisk,
    pub rollback_plan: String,
    pub confirmation_phrase: String,
    pub junior_approved: bool,
    pub junior_reliability: u8,
}

impl MutationPlan {
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
            what_will_change: String::new(),
            why_required: String::new(),
            risk: MutationRisk::Medium,
            rollback_plan: String::new(),
            confirmation_phrase: MEDIUM_RISK_CONFIRMATION.to_string(),
            junior_approved: false,
            junior_reliability: 0,
        }
    }

    /// Check if plan is approved for execution
    pub fn is_approved_for_execution(&self) -> bool {
        self.junior_approved && self.junior_reliability >= 70
    }
}

impl Default for MutationPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutation validation error
#[derive(Debug, Clone)]
pub enum MutationError {
    /// Path not in allowlist
    PathNotAllowed(PathBuf),
    /// File too large
    FileTooLarge { path: PathBuf, size: u64, max: u64 },
    /// Missing confirmation
    MissingConfirmation,
    /// Wrong confirmation phrase
    WrongConfirmation { expected: String, got: String },
    /// Risk level not allowed
    RiskNotAllowed(MutationRisk),
    /// Tool not in allowlist
    ToolNotAllowed(String),
    /// File not found
    FileNotFound(PathBuf),
    /// Permission denied
    PermissionDenied(PathBuf),
    /// Binary file (not text)
    BinaryFile(PathBuf),
    /// Junior reliability too low
    JuniorReliabilityTooLow { score: u8, required: u8 },
    // v0.0.9: Package errors
    /// Package already installed
    PackageAlreadyInstalled(String),
    /// Package not installed
    PackageNotInstalled(String),
    /// Package not removable (not installed by Anna)
    PackageNotRemovable { package: String, reason: String },
    /// Package install failed
    PackageInstallFailed { package: String, reason: String },
    /// Package remove failed
    PackageRemoveFailed { package: String, reason: String },
    // v0.0.14: Policy-driven errors
    /// Blocked by policy (with evidence ID for audit trail)
    PolicyBlocked {
        path: PathBuf,
        reason: String,
        evidence_id: String,
        policy_rule: String,
    },
    /// Other error
    Other(String),
}

impl std::fmt::Display for MutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationError::PathNotAllowed(p) => write!(f, "Path not in allowlist: {}", p.display()),
            MutationError::FileTooLarge { path, size, max } => {
                write!(f, "File too large: {} ({} bytes, max {})", path.display(), size, max)
            }
            MutationError::MissingConfirmation => write!(f, "Missing confirmation phrase"),
            MutationError::WrongConfirmation { expected, got } => {
                write!(f, "Wrong confirmation: expected '{}', got '{}'", expected, got)
            }
            MutationError::RiskNotAllowed(risk) => write!(f, "Risk level '{}' not allowed", risk),
            MutationError::ToolNotAllowed(name) => write!(f, "Tool '{}' not in mutation allowlist", name),
            MutationError::FileNotFound(p) => write!(f, "File not found: {}", p.display()),
            MutationError::PermissionDenied(p) => write!(f, "Permission denied: {}", p.display()),
            MutationError::BinaryFile(p) => write!(f, "Binary file not supported: {}", p.display()),
            MutationError::JuniorReliabilityTooLow { score, required } => {
                write!(f, "Junior reliability {}% too low (requires {}%)", score, required)
            }
            MutationError::PackageAlreadyInstalled(pkg) => write!(f, "Package '{}' is already installed", pkg),
            MutationError::PackageNotInstalled(pkg) => write!(f, "Package '{}' is not installed", pkg),
            MutationError::PackageNotRemovable { package, reason } => {
                write!(f, "Package '{}' cannot be removed: {}", package, reason)
            }
            MutationError::PackageInstallFailed { package, reason } => {
                write!(f, "Failed to install '{}': {}", package, reason)
            }
            MutationError::PackageRemoveFailed { package, reason } => {
                write!(f, "Failed to remove '{}': {}", package, reason)
            }
            MutationError::PolicyBlocked { path, reason, evidence_id, policy_rule } => {
                write!(f, "Blocked by policy [{}]: {} (rule: {}, target: {})",
                    evidence_id, reason, policy_rule, path.display())
            }
            MutationError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MutationError {}

/// Mutation tool catalog with allowlist
pub struct MutationToolCatalog {
    tools: HashMap<&'static str, MutationToolDef>,
}

impl MutationToolCatalog {
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // File edit tool
        tools.insert("edit_file_lines", MutationToolDef {
            name: "edit_file_lines",
            description: "Edit lines in a text config file (insert, replace, delete)",
            risk: MutationRisk::Medium,
            parameters: &[
                ("path", "Absolute path to file", true),
                ("operations", "Array of edit operations", true),
            ],
            rollback_supported: true,
            human_action: "edit config file",
        });

        // Systemd operations
        tools.insert("systemd_restart", MutationToolDef {
            name: "systemd_restart",
            description: "Restart a systemd service",
            risk: MutationRisk::Medium,
            parameters: &[
                ("service", "Service unit name (e.g., nginx.service)", true),
            ],
            rollback_supported: true,
            human_action: "restart service",
        });

        tools.insert("systemd_reload", MutationToolDef {
            name: "systemd_reload",
            description: "Reload a systemd service configuration",
            risk: MutationRisk::Medium,
            parameters: &[
                ("service", "Service unit name (e.g., nginx.service)", true),
            ],
            rollback_supported: true,
            human_action: "reload service",
        });

        tools.insert("systemd_enable_now", MutationToolDef {
            name: "systemd_enable_now",
            description: "Enable and start a systemd service",
            risk: MutationRisk::Medium,
            parameters: &[
                ("service", "Service unit name (e.g., nginx.service)", true),
            ],
            rollback_supported: true,
            human_action: "enable and start service",
        });

        tools.insert("systemd_disable_now", MutationToolDef {
            name: "systemd_disable_now",
            description: "Disable and stop a systemd service",
            risk: MutationRisk::Medium,
            parameters: &[
                ("service", "Service unit name (e.g., nginx.service)", true),
            ],
            rollback_supported: true,
            human_action: "disable and stop service",
        });

        tools.insert("systemd_daemon_reload", MutationToolDef {
            name: "systemd_daemon_reload",
            description: "Reload systemd manager configuration",
            risk: MutationRisk::Medium,
            parameters: &[],
            rollback_supported: false,
            human_action: "reload systemd daemon",
        });

        // v0.0.9: Package management tools
        tools.insert("package_install", MutationToolDef {
            name: "package_install",
            description: "Install a package via pacman (tracks as anna-installed)",
            risk: MutationRisk::Medium,
            parameters: &[
                ("package", "Package name to install", true),
                ("reason", "Why Anna is installing this package", true),
            ],
            rollback_supported: true,
            human_action: "install package",
        });

        tools.insert("package_remove", MutationToolDef {
            name: "package_remove",
            description: "Remove a package via pacman (only anna-installed packages)",
            risk: MutationRisk::Medium,
            parameters: &[
                ("package", "Package name to remove", true),
            ],
            rollback_supported: true,
            human_action: "remove package",
        });

        Self { tools }
    }

    /// Get tool definition by name
    pub fn get(&self, name: &str) -> Option<&MutationToolDef> {
        self.tools.get(name)
    }

    /// Check if tool is allowed
    pub fn is_allowed(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }

    /// Get all tools
    pub fn all_tools(&self) -> impl Iterator<Item = &MutationToolDef> {
        self.tools.values()
    }
}

impl Default for MutationToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Path Validation (v0.0.14 - Policy-Driven)
// =============================================================================

use crate::policy::{get_policy, PolicyCheckResult};

/// Check if a path is in the allowed edit list (policy-driven)
/// Returns (allowed, evidence_id, reason)
pub fn check_path_policy(path: &Path) -> PolicyCheckResult {
    let path_str = path.to_string_lossy();
    let policy = get_policy();

    // Safety invariant: never allow symlink traversal
    if path.is_symlink() {
        if let Ok(target) = std::fs::read_link(path) {
            // Check the target path too
            return check_path_policy(&target);
        }
    }

    // Use policy for all other checks
    policy.is_path_allowed(&path_str)
}

/// Legacy wrapper for backwards compatibility
pub fn is_path_allowed(path: &Path) -> bool {
    check_path_policy(path).allowed
}

/// Validate a path for mutation (v0.0.14 - policy-driven with evidence)
pub fn validate_mutation_path(path: &Path) -> Result<(), MutationError> {
    let policy = get_policy();

    // Check allowlist via policy
    let check = check_path_policy(path);
    if !check.allowed {
        return Err(MutationError::PolicyBlocked {
            path: path.to_path_buf(),
            reason: check.reason.clone(),
            evidence_id: check.evidence_id.clone(),
            policy_rule: check.policy_rule.clone(),
        });
    }

    // Check file exists
    if !path.exists() {
        return Err(MutationError::FileNotFound(path.to_path_buf()));
    }

    // Check file size (from policy)
    let max_size = policy.get_max_file_size();
    match std::fs::metadata(path) {
        Ok(meta) => {
            if meta.len() > max_size {
                return Err(MutationError::FileTooLarge {
                    path: path.to_path_buf(),
                    size: meta.len(),
                    max: max_size,
                });
            }
        }
        Err(_) => {
            return Err(MutationError::PermissionDenied(path.to_path_buf()));
        }
    }

    // Check if binary (only if policy requires text_only)
    if policy.capabilities.mutation_tools.file_edit.text_only {
        if let Ok(content) = std::fs::read(path) {
            // Check for null bytes (common binary indicator)
            if content.contains(&0) {
                return Err(MutationError::BinaryFile(path.to_path_buf()));
            }
        }
    }

    Ok(())
}

/// Validate a package for installation/removal (v0.0.14)
pub fn validate_package_policy(package: &str) -> Result<(), MutationError> {
    let policy = get_policy();

    if !policy.capabilities.mutation_tools.packages.enabled {
        return Err(MutationError::PolicyBlocked {
            path: PathBuf::from(package),
            reason: "Package operations are disabled in policy".to_string(),
            evidence_id: crate::policy::generate_policy_evidence_id(),
            policy_rule: "capabilities.toml:mutation_tools.packages.enabled".to_string(),
        });
    }

    let check = policy.is_package_allowed(package);
    if !check.allowed {
        return Err(MutationError::PolicyBlocked {
            path: PathBuf::from(package),
            reason: check.reason.clone(),
            evidence_id: check.evidence_id.clone(),
            policy_rule: check.policy_rule.clone(),
        });
    }

    Ok(())
}

/// Validate a service for operations (v0.0.14)
pub fn validate_service_policy(service: &str, operation: &str) -> Result<(), MutationError> {
    let policy = get_policy();

    // Check operation is allowed
    let op_check = policy.is_systemd_operation_allowed(operation);
    if !op_check.allowed {
        return Err(MutationError::PolicyBlocked {
            path: PathBuf::from(service),
            reason: op_check.reason.clone(),
            evidence_id: op_check.evidence_id.clone(),
            policy_rule: op_check.policy_rule.clone(),
        });
    }

    // Check service is allowed
    let svc_check = policy.is_service_allowed(service);
    if !svc_check.allowed {
        return Err(MutationError::PolicyBlocked {
            path: PathBuf::from(service),
            reason: svc_check.reason.clone(),
            evidence_id: svc_check.evidence_id.clone(),
            policy_rule: svc_check.policy_rule.clone(),
        });
    }

    Ok(())
}

/// Validate confirmation phrase
pub fn validate_confirmation(token: Option<&str>, required: &str) -> Result<(), MutationError> {
    match token {
        None => Err(MutationError::MissingConfirmation),
        Some(t) if t.trim() == required => Ok(()),
        Some(t) => Err(MutationError::WrongConfirmation {
            expected: required.to_string(),
            got: t.to_string(),
        }),
    }
}

/// Validate mutation request
pub fn validate_mutation_request<'a>(
    request: &'a MutationRequest,
    catalog: &'a MutationToolCatalog,
) -> Result<&'a MutationToolDef, MutationError> {
    // Check tool is allowed
    let tool = catalog.get(&request.tool_name)
        .ok_or_else(|| MutationError::ToolNotAllowed(request.tool_name.clone()))?;

    // Check risk level (only medium allowed in v0.0.8)
    if tool.risk != MutationRisk::Medium {
        return Err(MutationError::RiskNotAllowed(tool.risk));
    }

    // Check confirmation
    validate_confirmation(request.confirmation_token.as_deref(), MEDIUM_RISK_CONFIRMATION)?;

    // For file edits, validate the path
    if request.tool_name == "edit_file_lines" {
        if let Some(path_val) = request.parameters.get("path") {
            if let Some(path_str) = path_val.as_str() {
                validate_mutation_path(Path::new(path_str))?;
            }
        }
    }

    Ok(tool)
}

// =============================================================================
// Service state helpers
// =============================================================================

/// Get the current state of a systemd service
pub fn get_service_state(service: &str) -> Option<ServiceState> {
    // Try to get active state
    let active_output = std::process::Command::new("systemctl")
        .args(["is-active", service])
        .output()
        .ok()?;
    let is_active = String::from_utf8_lossy(&active_output.stdout).trim() == "active";

    // Try to get enabled state
    let enabled_output = std::process::Command::new("systemctl")
        .args(["is-enabled", service])
        .output()
        .ok()?;
    let is_enabled = String::from_utf8_lossy(&enabled_output.stdout).trim() == "enabled";

    Some(ServiceState { is_active, is_enabled })
}

/// Service state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub is_active: bool,
    pub is_enabled: bool,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let active = if self.is_active { "active" } else { "inactive" };
        let enabled = if self.is_enabled { "enabled" } else { "disabled" };
        write!(f, "{}/{}", active, enabled)
    }
}

// =============================================================================
// Mutation plan generation helpers
// =============================================================================

/// Generate rollback instructions for a file edit
pub fn generate_file_rollback_instructions(path: &Path, backup_path: &Path) -> String {
    format!(
        "To rollback: cp '{}' '{}'\n\
         Backup location: {}",
        backup_path.display(),
        path.display(),
        backup_path.display()
    )
}

/// Generate rollback instructions for a systemd operation
pub fn generate_systemd_rollback_instructions(
    operation: &str,
    service: &str,
    prior_state: Option<&ServiceState>,
) -> String {
    let prior = prior_state
        .map(|s| format!(" (was {})", s))
        .unwrap_or_default();

    match operation {
        "restart" => format!(
            "To rollback: systemctl restart {}{}\n\
             Note: Service data may have been affected by the restart.",
            service, prior
        ),
        "reload" => format!(
            "To rollback: systemctl reload {}{}\n\
             Note: Configuration was reloaded, no persistent change.",
            service, prior
        ),
        "enable_now" => format!(
            "To rollback: systemctl disable --now {}{}\n\
             This will disable and stop the service.",
            service, prior
        ),
        "disable_now" => format!(
            "To rollback: systemctl enable --now {}{}\n\
             This will enable and start the service.",
            service, prior
        ),
        "daemon_reload" => "Note: daemon-reload has no direct rollback.".to_string(),
        _ => format!("Manual rollback required for {}", operation),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_allowed_etc() {
        assert!(is_path_allowed(Path::new("/etc/nginx/nginx.conf")));
        assert!(is_path_allowed(Path::new("/etc/hosts")));
        assert!(is_path_allowed(Path::new("/etc/systemd/system/foo.service")));
    }

    #[test]
    fn test_path_allowed_home() {
        assert!(is_path_allowed(Path::new("/home/user/.bashrc")));
        assert!(is_path_allowed(Path::new("/home/user/.config/something")));
        assert!(is_path_allowed(Path::new("/root/.bashrc")));
    }

    #[test]
    fn test_path_not_allowed() {
        assert!(!is_path_allowed(Path::new("/usr/bin/something")));
        assert!(!is_path_allowed(Path::new("/var/log/syslog")));
        assert!(!is_path_allowed(Path::new("/boot/vmlinuz")));
        assert!(!is_path_allowed(Path::new("/proc/sys/kernel")));
    }

    #[test]
    fn test_confirmation_valid() {
        assert!(validate_confirmation(Some("I CONFIRM (medium risk)"), MEDIUM_RISK_CONFIRMATION).is_ok());
        assert!(validate_confirmation(Some("  I CONFIRM (medium risk)  "), MEDIUM_RISK_CONFIRMATION).is_ok());
    }

    #[test]
    fn test_confirmation_missing() {
        let err = validate_confirmation(None, MEDIUM_RISK_CONFIRMATION).unwrap_err();
        assert!(matches!(err, MutationError::MissingConfirmation));
    }

    #[test]
    fn test_confirmation_wrong() {
        let err = validate_confirmation(Some("yes"), MEDIUM_RISK_CONFIRMATION).unwrap_err();
        assert!(matches!(err, MutationError::WrongConfirmation { .. }));
    }

    #[test]
    fn test_mutation_catalog_has_expected_tools() {
        let catalog = MutationToolCatalog::new();
        assert!(catalog.is_allowed("edit_file_lines"));
        assert!(catalog.is_allowed("systemd_restart"));
        assert!(catalog.is_allowed("systemd_reload"));
        assert!(catalog.is_allowed("systemd_enable_now"));
        assert!(catalog.is_allowed("systemd_disable_now"));
        assert!(catalog.is_allowed("systemd_daemon_reload"));
    }

    #[test]
    fn test_mutation_catalog_rejects_unknown() {
        let catalog = MutationToolCatalog::new();
        assert!(!catalog.is_allowed("rm"));
        assert!(!catalog.is_allowed("pacman"));
        assert!(!catalog.is_allowed("chmod"));
    }

    #[test]
    fn test_mutation_plan_approval() {
        let mut plan = MutationPlan::new();

        // Not approved initially
        assert!(!plan.is_approved_for_execution());

        // Junior approved but low reliability
        plan.junior_approved = true;
        plan.junior_reliability = 50;
        assert!(!plan.is_approved_for_execution());

        // Junior approved with sufficient reliability
        plan.junior_reliability = 70;
        assert!(plan.is_approved_for_execution());

        plan.junior_reliability = 85;
        assert!(plan.is_approved_for_execution());
    }

    #[test]
    fn test_risk_display() {
        assert_eq!(format!("{}", MutationRisk::Medium), "medium");
        assert_eq!(format!("{}", MutationRisk::High), "high");
    }
}

//! Mutation Engine v1 for Anna v0.0.80
//!
//! First-class mutation pipeline with these stages:
//! 1. Plan - Generate structured action plan from request
//! 2. Preview - Show diff preview with what will change
//! 3. Confirm - Require explicit confirmation phrase based on risk
//! 4. Execute - Run the mutation with progress messages
//! 5. Verify - Run verification checks
//! 6. Rollback - If verification fails, offer and execute rollback
//!
//! Supported mutations in v0.0.80:
//! - systemd service control (start/stop/restart/enable/disable)
//! - package install/remove (pacman only, no AUR)
//! - config file safe edit (allowlisted files only)

use crate::mutation_safety::{MutationState, PreflightCheck};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// =============================================================================
// Risk Levels and Confirmation
// =============================================================================

/// Risk level for mutations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationRiskLevel {
    /// Low risk - reversible, local changes
    Low,
    /// Medium risk - config edits, service restarts
    Medium,
    /// High risk - destructive, may affect system stability
    High,
}

impl MutationRiskLevel {
    /// Get the required confirmation phrase for this risk level
    pub fn confirmation_phrase(&self) -> &'static str {
        match self {
            MutationRiskLevel::Low => "I CONFIRM (low risk)",
            MutationRiskLevel::Medium => "I CONFIRM (medium risk)",
            MutationRiskLevel::High => "I CONFIRM (high risk)",
        }
    }

    /// Get the rollback confirmation phrase
    pub fn rollback_phrase() -> &'static str {
        "I CONFIRM ROLLBACK"
    }
}

impl std::fmt::Display for MutationRiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationRiskLevel::Low => write!(f, "low"),
            MutationRiskLevel::Medium => write!(f, "medium"),
            MutationRiskLevel::High => write!(f, "high"),
        }
    }
}

// =============================================================================
// Mutation Types (v0.0.80 scope)
// =============================================================================

/// Supported mutation categories in v0.0.80
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationCategory {
    /// systemd service control
    ServiceControl,
    /// Package install/remove
    PackageManagement,
    /// Config file safe edit
    ConfigEdit,
}

/// Systemd service action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

impl std::fmt::Display for ServiceAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceAction::Start => write!(f, "start"),
            ServiceAction::Stop => write!(f, "stop"),
            ServiceAction::Restart => write!(f, "restart"),
            ServiceAction::Enable => write!(f, "enable"),
            ServiceAction::Disable => write!(f, "disable"),
        }
    }
}

/// Package action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageAction {
    Install,
    Remove,
}

/// Config edit operation type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigEditOp {
    /// Add a line at end of file
    AddLine { line: String },
    /// Replace exact line with new content
    ReplaceLine { old: String, new: String },
    /// Comment out a line (prepend #)
    CommentLine { pattern: String },
    /// Uncomment a line (remove leading #)
    UncommentLine { pattern: String },
}

// =============================================================================
// Whitelists for v0.0.80
// =============================================================================

/// Allowed services for v0.0.80 (deliberately small scope)
pub const ALLOWED_SERVICES: &[&str] = &[
    "NetworkManager",
    "sshd",
    "docker",
    "bluetooth",
];

/// Allowed config files for v0.0.80 (safe allowlist)
pub const ALLOWED_CONFIG_FILES: &[&str] = &[
    "/etc/pacman.conf",
    "/etc/ssh/sshd_config",
    "/etc/NetworkManager/NetworkManager.conf",
];

/// Path to mutation stats file
pub const MUTATION_STATS_FILE: &str = "/var/lib/anna/internal/mutation_stats.json";

/// Check if a service is in the allowed whitelist
pub fn is_service_allowed(service: &str) -> bool {
    ALLOWED_SERVICES.contains(&service)
}

/// Check if a config file is in the allowed whitelist
pub fn is_config_allowed(path: &str) -> bool {
    ALLOWED_CONFIG_FILES.contains(&path)
}

// =============================================================================
// Mutation Plan
// =============================================================================

/// A single step in a mutation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationStep {
    /// Step ID for tracking
    pub step_id: String,
    /// Human-readable description of what this step does
    pub description: String,
    /// Category of mutation
    pub category: MutationCategory,
    /// Specific mutation details
    pub mutation: MutationDetail,
    /// Risk level for this step
    pub risk: MutationRiskLevel,
    /// Affected resources (files, services, packages)
    pub affected_resources: Vec<String>,
}

/// Details of a specific mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationDetail {
    /// Service control
    ServiceControl {
        service: String,
        action: ServiceAction,
    },
    /// Package install
    PackageInstall {
        packages: Vec<String>,
    },
    /// Package remove
    PackageRemove {
        packages: Vec<String>,
    },
    /// Config file edit
    ConfigEdit {
        path: String,
        operation: ConfigEditOp,
    },
}

/// Verification check to run after mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    /// Check description
    pub description: String,
    /// Command to run (empty for probe-based checks)
    pub command: Option<String>,
    /// Expected outcome description
    pub expected: String,
}

/// Rollback step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Step description
    pub description: String,
    /// How to undo the change
    pub undo_action: String,
    /// Backup location if applicable
    pub backup_path: Option<PathBuf>,
}

/// A complete mutation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationPlanV1 {
    /// Unique plan ID
    pub plan_id: String,
    /// Case ID this plan belongs to
    pub case_id: Option<String>,
    /// Steps to execute
    pub steps: Vec<MutationStep>,
    /// Overall risk level (highest of all steps)
    pub risk: MutationRiskLevel,
    /// Affected resources summary
    pub affected_resources: Vec<String>,
    /// Verification checks to run after execution
    pub verification_checks: Vec<VerificationCheck>,
    /// Rollback steps if verification fails
    pub rollback_steps: Vec<RollbackStep>,
    /// Current state of the plan
    pub state: MutationPlanState,
    /// Created timestamp
    pub created_at: u64,
}

/// State of a mutation plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationPlanState {
    /// Plan created, not yet previewed
    Created,
    /// Preview generated
    Previewed,
    /// Waiting for user confirmation
    AwaitingConfirmation,
    /// Privilege check failed
    BlockedPrivilege,
    /// User confirmed, ready to execute
    Confirmed,
    /// Execution in progress
    Executing,
    /// Execution complete, verification pending
    ExecutionComplete,
    /// Verification in progress
    Verifying,
    /// Verification passed
    VerifiedSuccess,
    /// Verification failed, rollback offered
    VerificationFailed,
    /// Rollback in progress
    RollingBack,
    /// Rollback complete
    RolledBack,
    /// Cancelled by user
    Cancelled,
}

impl std::fmt::Display for MutationPlanState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationPlanState::Created => write!(f, "created"),
            MutationPlanState::Previewed => write!(f, "previewed"),
            MutationPlanState::AwaitingConfirmation => write!(f, "awaiting_confirmation"),
            MutationPlanState::BlockedPrivilege => write!(f, "blocked_privilege"),
            MutationPlanState::Confirmed => write!(f, "confirmed"),
            MutationPlanState::Executing => write!(f, "executing"),
            MutationPlanState::ExecutionComplete => write!(f, "execution_complete"),
            MutationPlanState::Verifying => write!(f, "verifying"),
            MutationPlanState::VerifiedSuccess => write!(f, "verified_success"),
            MutationPlanState::VerificationFailed => write!(f, "verification_failed"),
            MutationPlanState::RollingBack => write!(f, "rolling_back"),
            MutationPlanState::RolledBack => write!(f, "rolled_back"),
            MutationPlanState::Cancelled => write!(f, "cancelled"),
        }
    }
}

// =============================================================================
// Preview Output
// =============================================================================

/// Preview of what a mutation will change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationPreview {
    /// Plan this preview is for
    pub plan_id: String,
    /// Human-readable summary
    pub summary: String,
    /// Detailed changes by step
    pub step_previews: Vec<StepPreview>,
    /// Risk level
    pub risk: MutationRiskLevel,
    /// Required confirmation phrase
    pub confirmation_phrase: String,
    /// Whether privilege is available to execute
    pub privilege_available: bool,
    /// If privilege not available, manual commands to run
    pub manual_commands: Option<Vec<String>>,
}

/// Preview for a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepPreview {
    /// Step ID
    pub step_id: String,
    /// Step description
    pub description: String,
    /// Current state (before mutation)
    pub current_state: String,
    /// Intended state (after mutation)
    pub intended_state: String,
    /// Diff preview for file edits (unified diff format)
    pub diff: Option<String>,
}

// =============================================================================
// Execution Result
// =============================================================================

/// Result of executing a mutation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationExecutionResult {
    /// Plan ID
    pub plan_id: String,
    /// Overall success
    pub success: bool,
    /// Step results
    pub step_results: Vec<StepExecutionResult>,
    /// Verification results
    pub verification_results: Vec<VerificationResult>,
    /// Whether rollback was performed
    pub rolled_back: bool,
    /// Rollback result if applicable
    pub rollback_result: Option<RollbackResult>,
    /// Final state
    pub final_state: MutationPlanState,
    /// Timestamp
    pub completed_at: u64,
}

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    /// Step ID
    pub step_id: String,
    /// Success
    pub success: bool,
    /// Output message
    pub message: String,
    /// Command stdout if applicable
    pub stdout: Option<String>,
    /// Command stderr if applicable
    pub stderr: Option<String>,
    /// Exit code if command was run
    pub exit_code: Option<i32>,
}

/// Result of a verification check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Check description
    pub description: String,
    /// Passed
    pub passed: bool,
    /// Actual result
    pub actual: String,
}

/// Result of rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    /// Success
    pub success: bool,
    /// Steps rolled back
    pub steps_rolled_back: Vec<String>,
    /// Message
    pub message: String,
}

// =============================================================================
// Mutation Stats (for annactl status)
// =============================================================================

/// Statistics about mutations for status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MutationStats {
    /// Whether privilege is available for mutations
    pub privilege_available: bool,
    /// Total successful mutations
    pub successful_count: u32,
    /// Total rollback count
    pub rollback_count: u32,
    /// Total blocked by privilege
    pub blocked_privilege_count: u32,
    /// Last mutation outcome
    pub last_mutation_outcome: Option<String>,
    /// Last mutation timestamp
    pub last_mutation_timestamp: Option<u64>,
}

impl MutationStats {
    /// Load stats from file
    pub fn load() -> Self {
        let path = std::path::Path::new("/var/lib/anna/internal/mutation_stats.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(stats) = serde_json::from_str(&content) {
                    return stats;
                }
            }
        }
        Self::default()
    }

    /// Save stats to file
    pub fn save(&self) -> Result<(), String> {
        let path = std::path::Path::new("/var/lib/anna/internal/mutation_stats.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        let content =
            serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    /// Record a successful mutation
    pub fn record_success(&mut self) {
        self.successful_count += 1;
        self.last_mutation_outcome = Some("success".to_string());
        self.last_mutation_timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
    }

    /// Record a rollback
    pub fn record_rollback(&mut self) {
        self.rollback_count += 1;
        self.last_mutation_outcome = Some("rolled_back".to_string());
        self.last_mutation_timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
    }

    /// Record a privilege block
    pub fn record_privilege_block(&mut self) {
        self.blocked_privilege_count += 1;
        self.last_mutation_outcome = Some("blocked_privilege".to_string());
        self.last_mutation_timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
    }

    /// Format for status display
    pub fn format_status(&self) -> String {
        let mut lines = Vec::new();

        // Privilege status
        let priv_status = if self.privilege_available {
            "available"
        } else {
            "blocked (needs privilege)"
        };
        lines.push(format!("  Capability:  {}", priv_status));

        // Counts
        lines.push(format!("  Successful:  {}", self.successful_count));
        lines.push(format!("  Rollbacks:   {}", self.rollback_count));
        lines.push(format!("  Blocked:     {}", self.blocked_privilege_count));

        // Last mutation
        if let Some(ref outcome) = self.last_mutation_outcome {
            let timestamp = self
                .last_mutation_timestamp
                .map(format_timestamp)
                .unwrap_or_else(|| "unknown".to_string());
            lines.push(format!("  Last:        {} ({})", outcome, timestamp));
        }

        lines.join("\n")
    }
}

/// Format timestamp as relative time
fn format_timestamp(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let diff = now.saturating_sub(ts);
    if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_confirmation_phrases() {
        assert_eq!(
            MutationRiskLevel::Low.confirmation_phrase(),
            "I CONFIRM (low risk)"
        );
        assert_eq!(
            MutationRiskLevel::Medium.confirmation_phrase(),
            "I CONFIRM (medium risk)"
        );
        assert_eq!(
            MutationRiskLevel::High.confirmation_phrase(),
            "I CONFIRM (high risk)"
        );
    }

    #[test]
    fn test_service_whitelist() {
        assert!(is_service_allowed("NetworkManager"));
        assert!(is_service_allowed("sshd"));
        assert!(is_service_allowed("docker"));
        assert!(is_service_allowed("bluetooth"));
        assert!(!is_service_allowed("random-service"));
        assert!(!is_service_allowed("httpd"));
    }

    #[test]
    fn test_config_whitelist() {
        assert!(is_config_allowed("/etc/pacman.conf"));
        assert!(is_config_allowed("/etc/ssh/sshd_config"));
        assert!(is_config_allowed("/etc/NetworkManager/NetworkManager.conf"));
        assert!(!is_config_allowed("/etc/passwd"));
        assert!(!is_config_allowed("/etc/shadow"));
    }

    #[test]
    fn test_mutation_stats_default() {
        let stats = MutationStats::default();
        assert_eq!(stats.successful_count, 0);
        assert_eq!(stats.rollback_count, 0);
        assert_eq!(stats.blocked_privilege_count, 0);
        assert!(stats.last_mutation_outcome.is_none());
    }

    #[test]
    fn test_mutation_stats_record() {
        let mut stats = MutationStats::default();
        stats.record_success();
        assert_eq!(stats.successful_count, 1);
        assert_eq!(stats.last_mutation_outcome, Some("success".to_string()));

        stats.record_rollback();
        assert_eq!(stats.rollback_count, 1);
        assert_eq!(
            stats.last_mutation_outcome,
            Some("rolled_back".to_string())
        );
    }
}

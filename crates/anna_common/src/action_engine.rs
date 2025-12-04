//! Action Engine v0.0.54 - Unified safe mutation execution
//!
//! Executes mutation steps only after user confirmation with:
//! - Unified ActionPlan with risk-based confirmation
//! - Diff previews for file edits
//! - Service actions with verification
//! - Package management with provenance tracking
//! - Rollback records in case file directories

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// =============================================================================
// Risk Levels and Confirmation Phrases
// =============================================================================

/// Unified risk level for all action types (v0.0.54 Action Engine)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MutationRiskLevel {
    /// Low risk: reversible, minimal impact (confirmation: "yes")
    Low,
    /// Medium risk: config edits, service restarts (confirmation: "I CONFIRM (medium risk)")
    Medium,
    /// High risk: critical services, package removal (confirmation: "I CONFIRM (high risk)")
    High,
    /// Destructive: data loss possible (confirmation: "I ACCEPT DATA LOSS RISK")
    Destructive,
    /// Denied: operation not allowed
    Denied,
}

impl MutationRiskLevel {
    pub fn confirmation_phrase(&self) -> Option<&'static str> {
        match self {
            MutationRiskLevel::Low => Some(CONFIRM_LOW),
            MutationRiskLevel::Medium => Some(CONFIRM_MEDIUM),
            MutationRiskLevel::High => Some(CONFIRM_HIGH),
            MutationRiskLevel::Destructive => Some(CONFIRM_DESTRUCTIVE),
            MutationRiskLevel::Denied => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MutationRiskLevel::Low => "low",
            MutationRiskLevel::Medium => "medium",
            MutationRiskLevel::High => "high",
            MutationRiskLevel::Destructive => "destructive",
            MutationRiskLevel::Denied => "denied",
        }
    }
}

impl std::fmt::Display for MutationRiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Confirmation phrases (hardcoded safety contract)
pub const CONFIRM_LOW: &str = "yes";
pub const CONFIRM_MEDIUM: &str = "I CONFIRM (medium risk)";
pub const CONFIRM_HIGH: &str = "I CONFIRM (high risk)";
pub const CONFIRM_DESTRUCTIVE: &str = "I ACCEPT DATA LOSS RISK";

// =============================================================================
// Action Step Types
// =============================================================================

/// A single action step in an action plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    /// Unique step ID within the plan
    pub step_id: String,
    /// The action to perform
    pub action: ActionType,
    /// Description for the user
    pub description: String,
    /// Risk level for this step
    pub risk: MutationRiskLevel,
    /// Precheck probes (read-only evidence collection before execution)
    pub precheck_probes: Vec<String>,
    /// Verify probes (read-only evidence collection after execution)
    pub verify_probes: Vec<String>,
    /// Rollback hint (human + machine readable)
    pub rollback_hint: RollbackHint,
    /// Evidence IDs emitted by this step
    pub evidence_ids: Vec<String>,
    /// Step status
    pub status: StepStatus,
}

/// Status of an action step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Previewed,
    Confirmed,
    Executing,
    Succeeded,
    Failed,
    Skipped,
    RolledBack,
}

/// Types of actions the engine can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionType {
    /// Edit an existing file
    EditFile(EditFileAction),
    /// Write a new file
    WriteFile(WriteFileAction),
    /// Delete a file (high/destructive risk)
    DeleteFile(DeleteFileAction),
    /// Systemd service operation
    Systemd(SystemdAction),
    /// Package management via pacman
    Pacman(PacmanAction),
}

// =============================================================================
// File Actions
// =============================================================================

/// Edit an existing file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFileAction {
    /// Target file path
    pub path: PathBuf,
    /// Edit intent
    pub intent: EditIntent,
    /// Unified diff patch (if applicable)
    pub patch: Option<String>,
    /// Owner for the file (username or "root")
    pub owner: String,
    /// Optional file mode (e.g., "0644")
    pub mode: Option<String>,
}

/// Intent of file edit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditIntent {
    /// Append lines to end of file
    Append { lines: Vec<String> },
    /// Replace entire file content
    Replace { content: String },
    /// Apply unified diff patch
    Patch,
    /// Set key=value in config file
    SetKeyValue {
        key: String,
        value: String,
        separator: String,
    },
    /// Insert line at position
    InsertLine { line_number: usize, content: String },
    /// Delete line at position
    DeleteLine { line_number: usize },
}

/// Write a new file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileAction {
    /// Target file path
    pub path: PathBuf,
    /// File content
    pub content: String,
    /// Owner for the file
    pub owner: String,
    /// File mode (e.g., "0644")
    pub mode: String,
}

/// Delete a file (high/destructive risk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileAction {
    /// Target file path
    pub path: PathBuf,
    /// Require file hash match for safety
    pub require_hash: Option<String>,
}

// =============================================================================
// Systemd Actions
// =============================================================================

/// Systemd service operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdAction {
    /// Service unit name (e.g., "nginx.service")
    pub unit: String,
    /// Operation to perform
    pub operation: SystemdOperation,
}

/// Systemd operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemdOperation {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
    EnableNow,
    DisableNow,
}

impl SystemdOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemdOperation::Start => "start",
            SystemdOperation::Stop => "stop",
            SystemdOperation::Restart => "restart",
            SystemdOperation::Reload => "reload",
            SystemdOperation::Enable => "enable",
            SystemdOperation::Disable => "disable",
            SystemdOperation::EnableNow => "enable --now",
            SystemdOperation::DisableNow => "disable --now",
        }
    }

    pub fn inverse(&self) -> Option<Self> {
        match self {
            SystemdOperation::Start => Some(SystemdOperation::Stop),
            SystemdOperation::Stop => Some(SystemdOperation::Start),
            SystemdOperation::Enable => Some(SystemdOperation::Disable),
            SystemdOperation::Disable => Some(SystemdOperation::Enable),
            SystemdOperation::EnableNow => Some(SystemdOperation::DisableNow),
            SystemdOperation::DisableNow => Some(SystemdOperation::EnableNow),
            SystemdOperation::Restart | SystemdOperation::Reload => None,
        }
    }
}

// =============================================================================
// Package Actions
// =============================================================================

/// Package management via pacman
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacmanAction {
    /// Operation to perform
    pub operation: PacmanOperation,
    /// Package names
    pub packages: Vec<String>,
    /// Reason for this operation
    pub reason: PackageReason,
}

/// Pacman operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacmanOperation {
    Install,
    Remove,
}

/// Why a package is being installed/removed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageReason {
    /// Helper package for Anna functionality
    Helper,
    /// User explicitly requested
    UserRequest,
    /// Dependency of another operation
    Dependency,
}

// =============================================================================
// Rollback Hint
// =============================================================================

/// Rollback information for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackHint {
    /// Human-readable instructions
    pub instructions: String,
    /// Machine-readable rollback command (if applicable)
    pub command: Option<String>,
    /// Path to backup file (if applicable)
    pub backup_path: Option<PathBuf>,
    /// Prior state snapshot (if applicable)
    pub prior_state: Option<String>,
}

impl Default for RollbackHint {
    fn default() -> Self {
        Self {
            instructions: "No rollback available".to_string(),
            command: None,
            backup_path: None,
            prior_state: None,
        }
    }
}

// =============================================================================
// Action Plan
// =============================================================================

/// A complete action plan ready for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    /// Unique plan ID
    pub plan_id: String,
    /// Case ID this plan belongs to
    pub case_id: String,
    /// Overall risk level (highest of all steps)
    pub risk: MutationRiskLevel,
    /// Human-readable summary
    pub summary: String,
    /// Steps to execute
    pub steps: Vec<ActionStep>,
    /// Required confirmation phrase
    pub confirmation_phrase: String,
    /// Timestamp when plan was created
    pub created_at: DateTime<Utc>,
    /// Plan status
    pub status: PlanStatus,
}

/// Status of an action plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    Previewed,
    AwaitingConfirmation,
    Confirmed,
    Executing,
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
}

impl ActionPlan {
    /// Create a new action plan
    pub fn new(case_id: &str, summary: &str) -> Self {
        Self {
            plan_id: crate::generate_request_id(),
            case_id: case_id.to_string(),
            risk: MutationRiskLevel::Low,
            summary: summary.to_string(),
            steps: Vec::new(),
            confirmation_phrase: CONFIRM_LOW.to_string(),
            created_at: Utc::now(),
            status: PlanStatus::Draft,
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: ActionStep) {
        // Update overall risk if step risk is higher
        if self.risk_level(step.risk) > self.risk_level(self.risk) {
            self.risk = step.risk;
            self.confirmation_phrase = step
                .risk
                .confirmation_phrase()
                .unwrap_or(CONFIRM_LOW)
                .to_string();
        }
        self.steps.push(step);
    }

    fn risk_level(&self, risk: MutationRiskLevel) -> u8 {
        match risk {
            MutationRiskLevel::Low => 1,
            MutationRiskLevel::Medium => 2,
            MutationRiskLevel::High => 3,
            MutationRiskLevel::Destructive => 4,
            MutationRiskLevel::Denied => 5,
        }
    }

    /// Check if plan can be executed (no denied steps)
    pub fn is_executable(&self) -> bool {
        self.risk != MutationRiskLevel::Denied
            && !self
                .steps
                .iter()
                .any(|s| s.risk == MutationRiskLevel::Denied)
    }

    /// Get count of steps by status
    pub fn step_counts(&self) -> HashMap<StepStatus, usize> {
        let mut counts = HashMap::new();
        for step in &self.steps {
            *counts.entry(step.status).or_insert(0) += 1;
        }
        counts
    }
}

// =============================================================================
// Diff Preview
// =============================================================================

/// Diff preview for file edits (Action Engine v0.0.54)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDiffPreview {
    /// Target file path
    pub path: PathBuf,
    /// Whether file exists
    pub file_exists: bool,
    /// Current file hash (if exists)
    pub current_hash: Option<String>,
    /// Lines of the unified diff
    pub diff_lines: Vec<ActionDiffLine>,
    /// Number of additions
    pub additions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Whether diff was truncated
    pub truncated: bool,
    /// Backup path that will be created
    pub backup_path: PathBuf,
}

/// A line in a diff (Action Engine v0.0.54)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDiffLine {
    /// Line type
    pub line_type: DiffLineType,
    /// Line content
    pub content: String,
    /// Line number in original (if applicable)
    pub old_line: Option<usize>,
    /// Line number in new (if applicable)
    pub new_line: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
}

impl ActionDiffPreview {
    /// Render as human-readable string
    pub fn render(&self, max_lines: usize) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Target: {}", self.path.display()));

        if self.file_exists {
            if let Some(ref hash) = self.current_hash {
                lines.push(format!("Current hash: {}...", &hash[..12.min(hash.len())]));
            }
        } else {
            lines.push("(new file)".to_string());
        }

        lines.push(format!("Backup: {}", self.backup_path.display()));
        lines.push(format!("+{} -{}", self.additions, self.deletions));
        lines.push(String::new());

        for (i, line) in self.diff_lines.iter().enumerate() {
            if i >= max_lines {
                lines.push("(truncated)".to_string());
                break;
            }
            let prefix = match line.line_type {
                DiffLineType::Context => " ",
                DiffLineType::Addition => "+",
                DiffLineType::Deletion => "-",
                DiffLineType::Header => "@",
            };
            lines.push(format!("{}{}", prefix, line.content));
        }

        lines.join("\n")
    }
}

// =============================================================================
// Execution Result
// =============================================================================

/// Result of executing an action plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Plan ID
    pub plan_id: String,
    /// Case ID
    pub case_id: String,
    /// Overall success
    pub success: bool,
    /// Step results
    pub step_results: Vec<StepResult>,
    /// Rollback record created
    pub rollback_record: Option<RollbackRecord>,
    /// Execution started at
    pub started_at: DateTime<Utc>,
    /// Execution completed at
    pub completed_at: DateTime<Utc>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step ID
    pub step_id: String,
    /// Success
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Human-readable summary
    pub summary: String,
    /// Evidence IDs generated
    pub evidence_ids: Vec<String>,
    /// Rollback info for this step
    pub rollback_hint: RollbackHint,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

// =============================================================================
// Rollback Record
// =============================================================================

/// Rollback record stored in case file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRecord {
    /// Case ID
    pub case_id: String,
    /// Plan ID
    pub plan_id: String,
    /// Steps with rollback info
    pub steps: Vec<RollbackStepRecord>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Backups created
    pub backups: Vec<BackupRecord>,
    /// Human-readable restore instructions
    pub restore_instructions: Vec<String>,
    /// Verification results
    pub verifications: Vec<VerificationRecord>,
}

/// Rollback info for a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStepRecord {
    pub step_id: String,
    pub action_type: String,
    pub target: String,
    pub rollback_command: Option<String>,
    pub backup_path: Option<PathBuf>,
    pub prior_state: Option<String>,
}

/// Backup file record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}

/// Verification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRecord {
    pub step_id: String,
    pub check_type: String,
    pub passed: bool,
    pub details: String,
}

impl RollbackRecord {
    /// Save to case directory
    pub fn save(&self, cases_dir: &std::path::Path) -> std::io::Result<PathBuf> {
        let case_path = cases_dir.join(&self.case_id);
        std::fs::create_dir_all(&case_path)?;

        let file_path = case_path.join("rollback.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&file_path, json)?;

        Ok(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_risk_confirmation() {
        assert_eq!(MutationRiskLevel::Low.confirmation_phrase(), Some("yes"));
        assert_eq!(
            MutationRiskLevel::Medium.confirmation_phrase(),
            Some("I CONFIRM (medium risk)")
        );
        assert_eq!(
            MutationRiskLevel::High.confirmation_phrase(),
            Some("I CONFIRM (high risk)")
        );
        assert_eq!(
            MutationRiskLevel::Destructive.confirmation_phrase(),
            Some("I ACCEPT DATA LOSS RISK")
        );
        assert_eq!(MutationRiskLevel::Denied.confirmation_phrase(), None);
    }

    #[test]
    fn test_action_plan_risk_escalation() {
        let mut plan = ActionPlan::new("test-case", "Test plan");
        assert_eq!(plan.risk, MutationRiskLevel::Low);

        plan.add_step(ActionStep {
            step_id: "s1".to_string(),
            action: ActionType::Systemd(SystemdAction {
                unit: "test.service".to_string(),
                operation: SystemdOperation::Restart,
            }),
            description: "Restart test".to_string(),
            risk: MutationRiskLevel::Medium,
            precheck_probes: vec![],
            verify_probes: vec![],
            rollback_hint: RollbackHint::default(),
            evidence_ids: vec![],
            status: StepStatus::Pending,
        });

        assert_eq!(plan.risk, MutationRiskLevel::Medium);
        assert_eq!(plan.confirmation_phrase, CONFIRM_MEDIUM);
    }

    #[test]
    fn test_systemd_operation_inverse() {
        assert_eq!(
            SystemdOperation::Start.inverse(),
            Some(SystemdOperation::Stop)
        );
        assert_eq!(
            SystemdOperation::Enable.inverse(),
            Some(SystemdOperation::Disable)
        );
        assert_eq!(SystemdOperation::Restart.inverse(), None);
    }
}

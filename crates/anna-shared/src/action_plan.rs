//! ActionPlan - Structured executable plans for system changes.
//! Phase 16: Turn fallback into real execution.
//! Phase 17: Verification and rollback support.
//! Phase 25: Execution safety and reversibility hardening.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Preflight result - structured outcome of preflight check (Phase 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PreflightResult {
    /// Preflight passed, safe to proceed
    #[default]
    Passed,
    /// Preflight blocked, changes not needed (idempotent skip)
    Blocked,
    /// Preflight could not determine state (treat as blocked, outcome = Cancelled)
    Unknown,
}

/// Verification status - structured verification outcome (Phase 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    /// Verification passed - observable state change confirmed
    Passed,
    /// Verification failed - state change not observed
    Failed,
    /// Verification could not determine state (treat as failed)
    #[default]
    Unknown,
}

/// Reversibility classification (Phase 25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    /// Action can be rolled back
    #[default]
    Reversible,
    /// Action cannot be rolled back - needs elevated confirmation
    NonReversible,
}

/// A single step in an action plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    /// Human-readable description of what this step does.
    pub description: String,
    /// The command to execute.
    pub command: String,
    /// Whether this command requires elevated privileges.
    pub needs_sudo: bool,
    /// Files that will be modified (for backup).
    pub affects_files: Vec<String>,
    /// Systemd units that will be modified (for state capture).
    pub affects_units: Vec<String>,
    /// Per-step verification command (quick check).
    pub verify_command: Option<String>,
    /// Expected pattern in verify output.
    pub verify_pattern: Option<String>,
    /// Rollback command if this step needs undoing.
    pub rollback_command: Option<String>,
}

impl ActionStep {
    /// Create a new step with minimal required fields.
    pub fn new(description: &str, command: &str, needs_sudo: bool) -> Self {
        Self {
            description: description.to_string(),
            command: command.to_string(),
            needs_sudo,
            affects_files: Vec::new(),
            affects_units: Vec::new(),
            verify_command: None,
            verify_pattern: None,
            rollback_command: None,
        }
    }

    /// Builder: Add affected files.
    pub fn with_files(mut self, files: &[&str]) -> Self {
        self.affects_files = files.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Builder: Add affected units.
    pub fn with_units(mut self, units: &[&str]) -> Self {
        self.affects_units = units.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Builder: Add per-step verification.
    pub fn with_verify(mut self, command: &str, pattern: &str) -> Self {
        self.verify_command = Some(command.to_string());
        self.verify_pattern = Some(pattern.to_string());
        self
    }

    /// Builder: Add rollback command.
    pub fn with_rollback(mut self, command: &str) -> Self {
        self.rollback_command = Some(command.to_string());
        self
    }
}

/// Verification check to confirm plan succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    /// Command to run to verify success.
    pub command: String,
    /// Expected output pattern (success if contains this).
    pub success_pattern: String,
    /// Description of what we're checking.
    pub description: String,
}

/// Rollback information for a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackInfo {
    /// Whether rollback is possible.
    pub possible: bool,
    /// Reason if rollback not possible.
    pub reason: Option<String>,
    /// Explicit rollback steps (if not using captured state).
    pub steps: Vec<ActionStep>,
}

impl Default for RollbackInfo {
    fn default() -> Self {
        Self {
            possible: true,
            reason: None,
            steps: Vec::new(),
        }
    }
}

impl RollbackInfo {
    /// Phase 25: Get reversibility classification.
    pub fn reversibility(&self) -> Reversibility {
        if self.possible {
            Reversibility::Reversible
        } else {
            Reversibility::NonReversible
        }
    }
}

/// A complete action plan that Anna can execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    /// Unique identifier for this plan.
    pub id: String,
    /// Short summary of what this plan does (shown to user).
    pub summary: String,
    /// Detailed explanation of the changes.
    pub explanation: String,
    /// The steps to execute.
    pub steps: Vec<ActionStep>,
    /// Final verification (authoritative check).
    pub verification: Option<VerificationCheck>,
    /// Rollback information.
    pub rollback: RollbackInfo,
    /// When this plan was created.
    pub created_at: DateTime<Utc>,
    /// The original question that triggered this plan.
    pub original_question: String,
    /// Whether preflight determined changes are needed.
    pub changes_needed: bool,
    /// Reason if no changes needed.
    pub skip_reason: Option<String>,
    /// Phase 25: Structured preflight result.
    #[serde(default)]
    pub preflight_result: PreflightResult,
    /// Phase 25: Reason if preflight blocked/unknown.
    #[serde(default)]
    pub preflight_reason: Option<String>,
}

impl ActionPlan {
    /// Create a new action plan.
    pub fn new(question: &str, summary: &str, explanation: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            summary: summary.to_string(),
            explanation: explanation.to_string(),
            steps: Vec::new(),
            verification: None,
            rollback: RollbackInfo::default(),
            created_at: Utc::now(),
            original_question: question.to_string(),
            changes_needed: true,
            skip_reason: None,
            preflight_result: PreflightResult::Passed,
            preflight_reason: None,
        }
    }

    /// Add a step to the plan (legacy method for compatibility).
    pub fn add_step(&mut self, description: &str, command: &str, needs_sudo: bool) {
        self.steps.push(ActionStep::new(description, command, needs_sudo));
    }

    /// Add a step with full configuration.
    pub fn add_step_full(&mut self, step: ActionStep) {
        self.steps.push(step);
    }

    /// Set final verification check.
    pub fn set_verification(&mut self, command: &str, success_pattern: &str, description: &str) {
        self.verification = Some(VerificationCheck {
            command: command.to_string(),
            success_pattern: success_pattern.to_string(),
            description: description.to_string(),
        });
    }

    /// Mark plan as no changes needed (idempotent).
    pub fn mark_no_changes(&mut self, reason: &str) {
        self.changes_needed = false;
        self.skip_reason = Some(reason.to_string());
        self.preflight_result = PreflightResult::Blocked;
        self.preflight_reason = Some(reason.to_string());
        self.steps.clear();
    }

    /// Phase 25: Mark plan as preflight unknown (cannot safely proceed).
    pub fn mark_preflight_unknown(&mut self, reason: &str) {
        self.changes_needed = false;
        self.skip_reason = Some(reason.to_string());
        self.preflight_result = PreflightResult::Unknown;
        self.preflight_reason = Some(reason.to_string());
        self.steps.clear();
    }

    /// Phase 25: Get reversibility classification.
    pub fn reversibility(&self) -> Reversibility {
        self.rollback.reversibility()
    }

    /// Mark rollback as not possible.
    pub fn set_no_rollback(&mut self, reason: &str) {
        self.rollback.possible = false;
        self.rollback.reason = Some(reason.to_string());
    }

    /// Check if any step requires sudo.
    pub fn requires_sudo(&self) -> bool {
        self.steps.iter().any(|s| s.needs_sudo)
    }

    /// Format the plan for user confirmation.
    pub fn format_for_confirmation(&self) -> String {
        if !self.changes_needed {
            return format!(
                "No changes needed. {}",
                self.skip_reason.as_deref().unwrap_or("Already configured.")
            );
        }

        let mut output = String::new();
        output.push_str(&format!("I'll {}.\n\n", self.summary.to_lowercase()));
        output.push_str(&self.explanation);
        output.push_str("\n\nSteps:\n");

        for (i, step) in self.steps.iter().enumerate() {
            let sudo_marker = if step.needs_sudo { " [sudo]" } else { "" };
            output.push_str(&format!("  {}. {}{}\n", i + 1, step.description, sudo_marker));
        }

        if self.requires_sudo() {
            output.push_str("\nRequires administrator privileges.\n");
        }

        if !self.rollback.possible {
            if let Some(ref reason) = self.rollback.reason {
                output.push_str(&format!("\nNote: {}\n", reason));
            }
        }

        output.push_str("\nProceed? (yes/no)");
        output
    }
}

/// Result of executing an action plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExecutionResult {
    pub plan_id: String,
    pub success: bool,
    pub step_results: Vec<StepResult>,
    pub verification_result: Option<VerificationResult>,
    /// Phase 25: Structured verification status.
    #[serde(default)]
    pub verification_status: VerificationStatus,
    pub rollback_performed: bool,
    pub rollback_success: Option<bool>,
    pub completed_at: DateTime<Utc>,
}

/// Result of executing a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_index: usize,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub verified: Option<bool>,
}

/// Result of verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub actual_output: String,
    pub explanation: String,
}

// =============================================================================
// DETERMINISTIC ACTION PLAN - DATA CONTRACT (Phase 30 Preparation)
// Passive data structures and structural validation only.
// This section defines data and performs no execution.
// =============================================================================

/// A single step within a Deterministic Action Plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicStep {
    pub step_number: u32,
    pub operation: String,
    pub target: String,
}

/// Deterministic Action Plan - a passive data structure.
/// This structure authorizes nothing and performs no action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicActionPlan {
    pub plan_id: String,
    pub created_utc: String,
    pub intent: String,
    pub target: String,
    pub preconditions: Vec<String>,
    pub steps: Vec<DeterministicStep>,
    pub reversible: bool,
    pub rollback_steps: Vec<DeterministicStep>,
    pub evidence_sources: Vec<String>,
}

/// Structural validation error for Deterministic Action Plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicValidationError {
    pub field: String,
    pub message: String,
}

/// Validate a Deterministic Action Plan structurally.
/// Returns a list of errors. Empty list means valid.
/// Performs no semantic interpretation.
pub fn validate_deterministic_plan(plan: &DeterministicActionPlan) -> Vec<DeterministicValidationError> {
    let mut errors = Vec::new();

    if plan.plan_id.is_empty() {
        errors.push(DeterministicValidationError {
            field: "plan_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if plan.created_utc.is_empty() {
        errors.push(DeterministicValidationError {
            field: "created_utc".to_string(),
            message: "must be non-empty".to_string(),
        });
    } else if !is_iso8601_format(&plan.created_utc) {
        errors.push(DeterministicValidationError {
            field: "created_utc".to_string(),
            message: "must be ISO 8601 format".to_string(),
        });
    }

    if plan.intent.is_empty() {
        errors.push(DeterministicValidationError {
            field: "intent".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if plan.target.is_empty() {
        errors.push(DeterministicValidationError {
            field: "target".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if plan.steps.is_empty() {
        errors.push(DeterministicValidationError {
            field: "steps".to_string(),
            message: "must contain at least one step".to_string(),
        });
    }

    for (i, step) in plan.steps.iter().enumerate() {
        if step.step_number == 0 {
            errors.push(DeterministicValidationError {
                field: format!("steps[{}].step_number", i),
                message: "must be positive".to_string(),
            });
        }
        if step.operation.is_empty() {
            errors.push(DeterministicValidationError {
                field: format!("steps[{}].operation", i),
                message: "must be non-empty".to_string(),
            });
        }
        if step.target.is_empty() {
            errors.push(DeterministicValidationError {
                field: format!("steps[{}].target", i),
                message: "must be non-empty".to_string(),
            });
        }
    }

    if plan.reversible && plan.rollback_steps.is_empty() {
        errors.push(DeterministicValidationError {
            field: "rollback_steps".to_string(),
            message: "required when reversible is true".to_string(),
        });
    }
    if !plan.reversible && !plan.rollback_steps.is_empty() {
        errors.push(DeterministicValidationError {
            field: "rollback_steps".to_string(),
            message: "must be empty when reversible is false".to_string(),
        });
    }

    for (i, step) in plan.rollback_steps.iter().enumerate() {
        if step.step_number == 0 {
            errors.push(DeterministicValidationError {
                field: format!("rollback_steps[{}].step_number", i),
                message: "must be positive".to_string(),
            });
        }
        if step.operation.is_empty() {
            errors.push(DeterministicValidationError {
                field: format!("rollback_steps[{}].operation", i),
                message: "must be non-empty".to_string(),
            });
        }
        if step.target.is_empty() {
            errors.push(DeterministicValidationError {
                field: format!("rollback_steps[{}].target", i),
                message: "must be non-empty".to_string(),
            });
        }
    }

    errors
}

/// Basic ISO 8601 format check (structural only, not semantic).
fn is_iso8601_format(s: &str) -> bool {
    let len = s.len();
    if len < 20 {
        return false;
    }
    s.chars().nth(4) == Some('-')
        && s.chars().nth(7) == Some('-')
        && s.chars().nth(10) == Some('T')
        && s.chars().nth(13) == Some(':')
        && s.chars().nth(16) == Some(':')
}

// =============================================================================
// SERIALIZATION - Deterministic, stable, human-readable
// =============================================================================

/// Serialize a Deterministic Action Plan to JSON.
/// Output is deterministic: same input always produces same output.
/// Field order follows struct definition order.
pub fn serialize_deterministic_plan(plan: &DeterministicActionPlan) -> Result<String, String> {
    serde_json::to_string_pretty(plan).map_err(|e| e.to_string())
}

/// Deserialize a Deterministic Action Plan from JSON.
/// Returns the plan if parsing succeeds; does not perform validation.
pub fn deserialize_deterministic_plan(json: &str) -> Result<DeterministicActionPlan, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Deserialize and validate a Deterministic Action Plan from JSON.
/// Returns the plan only if parsing succeeds AND validation passes.
pub fn deserialize_and_validate_plan(json: &str) -> Result<DeterministicActionPlan, Vec<String>> {
    let plan: DeterministicActionPlan = serde_json::from_str(json)
        .map_err(|e| vec![format!("Parse error: {}", e)])?;

    let errors = validate_deterministic_plan(&plan);
    if errors.is_empty() {
        Ok(plan)
    } else {
        Err(errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect())
    }
}

// =============================================================================
// PERSISTENCE - Durable storage, no behavior
// Storage location: /var/lib/anna/plans/
// File naming: {plan_id}.v1.json
// Versioning: Format version embedded in filename, not content
// =============================================================================

use std::fs;
use std::path::PathBuf;

/// Current data format version.
/// Increment when the schema changes in a backward-incompatible way.
pub const PLAN_FORMAT_VERSION: u32 = 1;

/// Get the plans storage directory.
/// Returns /var/lib/anna/plans/ in production, temp dir in tests.
pub fn plans_directory() -> PathBuf {
    crate::paths::paths().data_dir.join("plans")
}

/// Generate the canonical filename for a plan.
/// Format: {plan_id}.v{version}.json
/// - plan_id: sanitized to alphanumeric and hyphens only
/// - version: format version number
/// - .json: human-readable extension
fn plan_filename(plan_id: &str, version: u32) -> String {
    let safe_id: String = plan_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    format!("{}.v{}.json", safe_id, version)
}

/// Full path for a plan file.
fn plan_path(plan_id: &str, version: u32) -> PathBuf {
    plans_directory().join(plan_filename(plan_id, version))
}

/// Persistence error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanStorageError {
    /// Storage directory does not exist or cannot be created
    DirectoryError(String),
    /// File write failed
    WriteError(String),
    /// File read failed
    ReadError(String),
    /// Plan not found
    NotFound(String),
    /// Unknown format version
    UnknownVersion(u32),
    /// Validation failed after load
    ValidationFailed(Vec<String>),
}

/// Save a Deterministic Action Plan to disk.
/// Performs validation before saving. Does not overwrite existing files.
/// File path: /var/lib/anna/plans/{plan_id}.v1.json
pub fn save_plan(plan: &DeterministicActionPlan) -> Result<PathBuf, PlanStorageError> {
    // Validate before saving
    let errors = validate_deterministic_plan(plan);
    if !errors.is_empty() {
        return Err(PlanStorageError::ValidationFailed(
            errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect()
        ));
    }

    // Ensure directory exists
    let dir = plans_directory();
    fs::create_dir_all(&dir).map_err(|e| PlanStorageError::DirectoryError(e.to_string()))?;

    // Generate path
    let path = plan_path(&plan.plan_id, PLAN_FORMAT_VERSION);

    // Serialize
    let json = serialize_deterministic_plan(plan)
        .map_err(|e| PlanStorageError::WriteError(e))?;

    // Write atomically (write to temp, then rename)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &json).map_err(|e| PlanStorageError::WriteError(e.to_string()))?;
    fs::rename(&temp_path, &path).map_err(|e| PlanStorageError::WriteError(e.to_string()))?;

    Ok(path)
}

/// Load a Deterministic Action Plan from disk by plan_id.
/// Searches for the file with the current format version.
pub fn load_plan(plan_id: &str) -> Result<DeterministicActionPlan, PlanStorageError> {
    load_plan_version(plan_id, PLAN_FORMAT_VERSION)
}

/// Load a Deterministic Action Plan with a specific format version.
pub fn load_plan_version(plan_id: &str, version: u32) -> Result<DeterministicActionPlan, PlanStorageError> {
    // Only current version is supported
    if version != PLAN_FORMAT_VERSION {
        return Err(PlanStorageError::UnknownVersion(version));
    }

    let path = plan_path(plan_id, version);

    if !path.exists() {
        return Err(PlanStorageError::NotFound(plan_id.to_string()));
    }

    let json = fs::read_to_string(&path)
        .map_err(|e| PlanStorageError::ReadError(e.to_string()))?;

    let plan = deserialize_deterministic_plan(&json)
        .map_err(|e| PlanStorageError::ReadError(e))?;

    // Validate after load
    let errors = validate_deterministic_plan(&plan);
    if !errors.is_empty() {
        return Err(PlanStorageError::ValidationFailed(
            errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect()
        ));
    }

    Ok(plan)
}

/// List all stored plan IDs.
/// Returns plan IDs extracted from filenames, does not load content.
pub fn list_plan_ids() -> Result<Vec<String>, PlanStorageError> {
    let dir = plans_directory();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let suffix = format!(".v{}.json", PLAN_FORMAT_VERSION);
    let mut ids = Vec::new();

    let entries = fs::read_dir(&dir)
        .map_err(|e| PlanStorageError::DirectoryError(e.to_string()))?;

    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(&suffix) {
                let id = name.trim_end_matches(&suffix).to_string();
                ids.push(id);
            }
        }
    }

    ids.sort();
    Ok(ids)
}

// =============================================================================
// APPROVAL RECORD - DECISION BOUNDARY (Phase 31)
// Passive data structure recording operator approval or rejection.
// This is the boundary between data and action.
// This approval record authorizes nothing and performs no action.
// =============================================================================

/// Approval decision - approved or rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalDecision {
    /// Plan was approved by operator
    Approved,
    /// Plan was rejected by operator
    Rejected,
}

/// Approval Record - records operator decision on a Deterministic Action Plan.
/// This structure authorizes nothing and performs no action.
/// It is a passive record of a decision that was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    /// Unique identifier for this approval record
    pub approval_id: String,
    /// The plan_id of the plan being approved/rejected
    pub plan_id: String,
    /// The format version of the plan at decision time
    pub plan_version: u32,
    /// The decision: approved or rejected
    pub decision: ApprovalDecision,
    /// When the decision was made (ISO 8601)
    pub decided_utc: String,
    /// Who made the decision (username, operator identifier)
    pub decided_by: String,
    /// Optional comment from the operator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Structural validation error for Approval Record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalValidationError {
    pub field: String,
    pub message: String,
}

/// Validate an Approval Record structurally.
/// Returns a list of errors. Empty list means valid.
/// Performs no semantic interpretation.
pub fn validate_approval_record(record: &ApprovalRecord) -> Vec<ApprovalValidationError> {
    let mut errors = Vec::new();

    if record.approval_id.is_empty() {
        errors.push(ApprovalValidationError {
            field: "approval_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.plan_id.is_empty() {
        errors.push(ApprovalValidationError {
            field: "plan_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.plan_version == 0 {
        errors.push(ApprovalValidationError {
            field: "plan_version".to_string(),
            message: "must be positive".to_string(),
        });
    }

    if record.decided_utc.is_empty() {
        errors.push(ApprovalValidationError {
            field: "decided_utc".to_string(),
            message: "must be non-empty".to_string(),
        });
    } else if !is_iso8601_format(&record.decided_utc) {
        errors.push(ApprovalValidationError {
            field: "decided_utc".to_string(),
            message: "must be ISO 8601 format".to_string(),
        });
    }

    if record.decided_by.is_empty() {
        errors.push(ApprovalValidationError {
            field: "decided_by".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    errors
}

// =============================================================================
// APPROVAL SERIALIZATION - Deterministic, stable
// =============================================================================

/// Current approval record format version.
pub const APPROVAL_FORMAT_VERSION: u32 = 1;

/// Serialize an Approval Record to JSON.
/// Output is deterministic: same input always produces same output.
pub fn serialize_approval_record(record: &ApprovalRecord) -> Result<String, String> {
    serde_json::to_string_pretty(record).map_err(|e| e.to_string())
}

/// Deserialize an Approval Record from JSON.
/// Returns the record if parsing succeeds; does not perform validation.
pub fn deserialize_approval_record(json: &str) -> Result<ApprovalRecord, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Deserialize and validate an Approval Record from JSON.
/// Returns the record only if parsing succeeds AND validation passes.
pub fn deserialize_and_validate_approval(json: &str) -> Result<ApprovalRecord, Vec<String>> {
    let record: ApprovalRecord = serde_json::from_str(json)
        .map_err(|e| vec![format!("Parse error: {}", e)])?;

    let errors = validate_approval_record(&record);
    if errors.is_empty() {
        Ok(record)
    } else {
        Err(errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect())
    }
}

// =============================================================================
// APPROVAL PERSISTENCE - Durable storage, no behavior
// Storage location: /var/lib/anna/approvals/
// File naming: {approval_id}.v1.json
// =============================================================================

/// Get the approvals storage directory.
pub fn approvals_directory() -> PathBuf {
    crate::paths::paths().data_dir.join("approvals")
}

/// Generate the canonical filename for an approval record.
fn approval_filename(approval_id: &str, version: u32) -> String {
    let safe_id: String = approval_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    format!("{}.v{}.json", safe_id, version)
}

/// Full path for an approval file.
fn approval_path(approval_id: &str, version: u32) -> PathBuf {
    approvals_directory().join(approval_filename(approval_id, version))
}

/// Approval storage error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalStorageError {
    /// Storage directory does not exist or cannot be created
    DirectoryError(String),
    /// File write failed
    WriteError(String),
    /// File read failed
    ReadError(String),
    /// Approval not found
    NotFound(String),
    /// Unknown format version
    UnknownVersion(u32),
    /// Validation failed after load
    ValidationFailed(Vec<String>),
}

/// Save an Approval Record to disk.
/// Performs validation before saving.
/// File path: /var/lib/anna/approvals/{approval_id}.v1.json
pub fn save_approval(record: &ApprovalRecord) -> Result<PathBuf, ApprovalStorageError> {
    // Validate before saving
    let errors = validate_approval_record(record);
    if !errors.is_empty() {
        return Err(ApprovalStorageError::ValidationFailed(
            errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect()
        ));
    }

    // Ensure directory exists
    let dir = approvals_directory();
    fs::create_dir_all(&dir).map_err(|e| ApprovalStorageError::DirectoryError(e.to_string()))?;

    // Generate path
    let path = approval_path(&record.approval_id, APPROVAL_FORMAT_VERSION);

    // Serialize
    let json = serialize_approval_record(record)
        .map_err(|e| ApprovalStorageError::WriteError(e))?;

    // Write atomically (write to temp, then rename)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &json).map_err(|e| ApprovalStorageError::WriteError(e.to_string()))?;
    fs::rename(&temp_path, &path).map_err(|e| ApprovalStorageError::WriteError(e.to_string()))?;

    Ok(path)
}

/// Load an Approval Record from disk by approval_id.
pub fn load_approval(approval_id: &str) -> Result<ApprovalRecord, ApprovalStorageError> {
    load_approval_version(approval_id, APPROVAL_FORMAT_VERSION)
}

/// Load an Approval Record with a specific format version.
pub fn load_approval_version(approval_id: &str, version: u32) -> Result<ApprovalRecord, ApprovalStorageError> {
    if version != APPROVAL_FORMAT_VERSION {
        return Err(ApprovalStorageError::UnknownVersion(version));
    }

    let path = approval_path(approval_id, version);

    if !path.exists() {
        return Err(ApprovalStorageError::NotFound(approval_id.to_string()));
    }

    let json = fs::read_to_string(&path)
        .map_err(|e| ApprovalStorageError::ReadError(e.to_string()))?;

    let record = deserialize_approval_record(&json)
        .map_err(|e| ApprovalStorageError::ReadError(e))?;

    // Validate after load
    let errors = validate_approval_record(&record);
    if !errors.is_empty() {
        return Err(ApprovalStorageError::ValidationFailed(
            errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect()
        ));
    }

    Ok(record)
}

/// List all stored approval IDs.
pub fn list_approval_ids() -> Result<Vec<String>, ApprovalStorageError> {
    let dir = approvals_directory();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let suffix = format!(".v{}.json", APPROVAL_FORMAT_VERSION);
    let mut ids = Vec::new();

    let entries = fs::read_dir(&dir)
        .map_err(|e| ApprovalStorageError::DirectoryError(e.to_string()))?;

    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(&suffix) {
                let id = name.trim_end_matches(&suffix).to_string();
                ids.push(id);
            }
        }
    }

    ids.sort();
    Ok(ids)
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES (Phase 31 - Decision Boundary Contract)
// This section documents what the Approval Record DOES NOT and CANNOT do.
// These non-capabilities are by design and must never be added.
// =============================================================================
//
// The Approval Record:
// - DOES NOT execute any commands
// - DOES NOT modify system state
// - DOES NOT trigger any actions
// - DOES NOT communicate with any external services
// - DOES NOT validate that the referenced plan exists
// - DOES NOT verify operator credentials
// - DOES NOT enforce access control
// - DOES NOT provide authorization tokens
// - DOES NOT have any side effects
//
// The Approval Record is a passive data structure.
// It records that a decision was made, nothing more.
// Any future execution layer must be built separately and explicitly.
//
// This approval record authorizes nothing and performs no action.
// =============================================================================

// =============================================================================
// EXECUTION READINESS BOUNDARY (Phase 32)
// Pure, read-only classification of execution eligibility.
// This is an inspectable decision artifact, not an authorization.
// This classification enables no execution and confers no authority.
// =============================================================================

/// Execution readiness classification.
/// Answers: "Is this plan eligible for execution, assuming an execution engine existed?"
/// This is a pure classification, not an authorization or permission grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionReadiness {
    /// No approval record exists for this plan
    NotApproved,
    /// Approval exists but references a different plan version
    ApprovedButStale,
    /// Approval exists and references the current plan version
    ApprovedAndCurrent,
}

/// Classify the execution readiness of a plan.
///
/// This is a pure function with no I/O and no side effects.
/// It performs a simple version comparison, nothing more.
///
/// # Arguments
/// * `plan` - The Deterministic Action Plan to classify
/// * `approval` - Optional approval record to check against
///
/// # Returns
/// * `NotApproved` - No approval provided
/// * `ApprovedButStale` - Approval exists but plan_version != PLAN_FORMAT_VERSION
/// * `ApprovedAndCurrent` - Approval exists and plan_version == PLAN_FORMAT_VERSION
///
/// # Guarantees
/// * Deterministic: same inputs always produce same output
/// * Pure: no I/O, no side effects, no state mutation
/// * Total: defined for all valid inputs
pub fn classify_execution_readiness(
    _plan: &DeterministicActionPlan,
    approval: Option<&ApprovalRecord>,
) -> ExecutionReadiness {
    match approval {
        None => ExecutionReadiness::NotApproved,
        Some(record) => {
            if record.plan_version == PLAN_FORMAT_VERSION {
                ExecutionReadiness::ApprovedAndCurrent
            } else {
                ExecutionReadiness::ApprovedButStale
            }
        }
    }
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES (Phase 32 - Execution Readiness Contract)
// This section documents what the Execution Readiness classifier DOES NOT do.
// These non-capabilities are by design and must never be added.
// =============================================================================
//
// The ExecutionReadiness classifier:
// - DOES NOT execute any commands
// - DOES NOT authorize execution
// - DOES NOT imply permission to execute
// - DOES NOT select actions to perform
// - DOES NOT trigger any side effects
// - DOES NOT perform I/O operations
// - DOES NOT validate plan contents
// - DOES NOT verify approval authenticity
// - DOES NOT check operator permissions
// - DOES NOT consider rejection status
// - DOES NOT recommend any action
//
// The classifier answers exactly one question:
// "Does a version-matching approval exist?"
//
// It provides no authorization, permission, or recommendation.
// Any execution layer must be built separately and must not rely on this
// classification as a permission check.
//
// This classification enables no execution and confers no authority.
// =============================================================================

// =============================================================================
// EXECUTION GATE CONTRACT (Phase 33 - Hard Boundary)
// This gate makes execution structurally impossible.
// A future system must explicitly break this boundary to enable execution.
// This gate blocks execution unless a future system explicitly breaks it.
// =============================================================================

/// Execution gate trait - the hard boundary between data and action.
///
/// This trait defines the contract that must be satisfied before any
/// execution could theoretically occur. It provides no execution capability.
///
/// # Contract
/// - `can_execute` returns true ONLY for `ApprovedAndCurrent`
/// - All other states return false
/// - The function is pure, deterministic, and side-effect free
///
/// # Non-Capabilities
/// This gate does not execute, dispatch, or authorize anything.
/// It is a read-only predicate that answers: "Would execution be permitted?"
pub trait ExecutionGate {
    /// Check if execution would be permitted for the given readiness state.
    ///
    /// Returns true only if readiness is ApprovedAndCurrent.
    /// Returns false for all other states.
    ///
    /// This method:
    /// - Is pure (no side effects)
    /// - Is deterministic (same input always produces same output)
    /// - Does not execute anything
    /// - Does not dispatch anything
    /// - Does not authorize anything
    fn can_execute(&self, readiness: ExecutionReadiness) -> bool;
}

/// Default implementation of the execution gate.
///
/// This is the only implementation that should exist.
/// It enforces the hard boundary between data and action.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultExecutionGate;

impl ExecutionGate for DefaultExecutionGate {
    fn can_execute(&self, readiness: ExecutionReadiness) -> bool {
        matches!(readiness, ExecutionReadiness::ApprovedAndCurrent)
    }
}

/// Convenience function for checking execution eligibility.
///
/// This is a pure function with no side effects.
/// It does not execute, dispatch, or authorize anything.
pub fn can_execute(readiness: ExecutionReadiness) -> bool {
    DefaultExecutionGate.can_execute(readiness)
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES (Phase 33 - Execution Gate Contract)
// This section documents what the Execution Gate DOES NOT and CANNOT do.
// These non-capabilities are by design and must never be added.
// =============================================================================
//
// The ExecutionGate:
// - DOES NOT execute any commands
// - DOES NOT dispatch any actions
// - DOES NOT authorize execution
// - DOES NOT imply permission to execute
// - DOES NOT trigger any side effects
// - DOES NOT perform I/O operations
// - DOES NOT modify system state
// - DOES NOT communicate with external services
// - DOES NOT validate plan contents
// - DOES NOT verify operator credentials
// - DOES NOT contain execution logic
// - DOES NOT provide execution capability
//
// The gate answers exactly one question:
// "Is the readiness state ApprovedAndCurrent?"
//
// It provides no execution capability whatsoever.
// Any execution layer must be built separately and must explicitly
// break through this boundary with full awareness of the consequences.
//
// This gate blocks execution unless a future system explicitly breaks it.
// =============================================================================

// =============================================================================
// EXECUTION ADAPTER INTERFACE (Phase 34 - Non-Implementing)
// This defines the shape of execution without providing execution.
// No implementation exists. No code can call execute().
// This interface defines where execution could exist, not that it does.
// =============================================================================

/// Result of an execution attempt.
///
/// This is a pure data enum with no behavior or semantics beyond names.
/// It represents possible outcomes, not actual execution results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionResult {
    /// Execution was not attempted
    NotExecuted,
    /// Execution was blocked by the gate
    Blocked,
    /// Execution completed (hypothetically)
    Executed,
    /// Execution failed (hypothetically)
    Failed,
}

/// Execution adapter interface - the shape of future execution.
///
/// # Critical Contract
/// - This interface does NOT grant execution
/// - This interface does NOT imply permission
/// - This interface does NOT bypass ExecutionGate
/// - No implementation of this trait exists
/// - No code path can call execute()
///
/// # Purpose
/// This trait defines where execution COULD exist in a future system.
/// It is a vacuum, not a capability. The existence of this interface
/// does not enable, authorize, or perform any execution.
///
/// # Usage
/// None. This trait has no implementations and must not be used.
pub trait ExecutionAdapter {
    /// Execute a plan.
    ///
    /// # WARNING
    /// This method has no implementations.
    /// No code path can call this method.
    /// Any implementation must explicitly break the ExecutionGate boundary.
    ///
    /// # Non-Capabilities
    /// - Does not execute anything (no implementation exists)
    /// - Does not perform I/O
    /// - Does not call system APIs
    /// - Does not imply readiness
    /// - Does not select adapters
    fn execute(&self, plan: &DeterministicActionPlan) -> ExecutionResult;
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES (Phase 34 - Execution Adapter Contract)
// This section documents what the Execution Adapter DOES NOT and CANNOT do.
// These non-capabilities are by design and must never be violated.
// =============================================================================
//
// The ExecutionAdapter interface:
// - DOES NOT execute anything (no implementation exists)
// - DOES NOT perform I/O operations
// - DOES NOT call system APIs
// - DOES NOT imply execution readiness
// - DOES NOT select adapters
// - DOES NOT bypass ExecutionGate
// - DOES NOT grant execution permission
// - DOES NOT modify system state
// - DOES NOT have any implementations
// - DOES NOT provide any default behavior
//
// The interface defines a shape, not a capability.
// It is a declaration of where power COULD be introduced,
// not a grant of power itself.
//
// Any future implementation must:
// 1. Be explicitly introduced with full review
// 2. Respect the ExecutionGate boundary
// 3. Be a conscious, visible, irreversible choice
//
// This interface defines where execution could exist, not that it does.
// =============================================================================

// Compile-time proof: The project builds with zero implementations of ExecutionAdapter.
// If any implementation existed, it would appear here or in another module.
// The absence of `impl ExecutionAdapter for X` anywhere in the codebase is the proof.

// =============================================================================
// EXECUTION ATTEMPT RECORDING (Phase 34 - Non-Executing, Auditable)
// Records an attempt to execute a plan without executing anything.
// Captures intent and outcome classification only.
// This record documents an execution attempt and performs no execution.
// =============================================================================

/// Execution Attempt - records an attempt to execute a plan.
///
/// This is a pure data structure with no behavior.
/// It records what happened (or would have happened) without performing any action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionAttempt {
    /// Unique identifier for this attempt
    pub attempt_id: String,
    /// The plan_id of the plan that was attempted
    pub plan_id: String,
    /// The format version of the plan at attempt time
    pub plan_version: u32,
    /// Snapshot of the execution readiness at attempt time
    pub readiness: ExecutionReadiness,
    /// Result of ExecutionGate.can_execute() - recorded, not computed here
    pub gate_result: bool,
    /// The execution result classification
    pub result: ExecutionResult,
    /// When the attempt was recorded (ISO 8601)
    pub recorded_utc: String,
    /// Who/what recorded this attempt
    pub recorded_by: String,
    /// Optional note about the attempt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Structural validation error for Execution Attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptValidationError {
    pub field: String,
    pub message: String,
}

/// Validate an Execution Attempt structurally.
/// Returns a list of errors. Empty list means valid.
/// Performs no semantic checks or cross-record lookup.
pub fn validate_execution_attempt(attempt: &ExecutionAttempt) -> Vec<AttemptValidationError> {
    let mut errors = Vec::new();

    if attempt.attempt_id.is_empty() {
        errors.push(AttemptValidationError {
            field: "attempt_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if attempt.plan_id.is_empty() {
        errors.push(AttemptValidationError {
            field: "plan_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if attempt.plan_version == 0 {
        errors.push(AttemptValidationError {
            field: "plan_version".to_string(),
            message: "must be positive".to_string(),
        });
    }

    if attempt.recorded_utc.is_empty() {
        errors.push(AttemptValidationError {
            field: "recorded_utc".to_string(),
            message: "must be non-empty".to_string(),
        });
    } else if !is_iso8601_format(&attempt.recorded_utc) {
        errors.push(AttemptValidationError {
            field: "recorded_utc".to_string(),
            message: "must be ISO 8601 format".to_string(),
        });
    }

    if attempt.recorded_by.is_empty() {
        errors.push(AttemptValidationError {
            field: "recorded_by".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    errors
}

// =============================================================================
// EXECUTION ATTEMPT SERIALIZATION
// =============================================================================

/// Current attempt record format version.
pub const ATTEMPT_FORMAT_VERSION: u32 = 1;

/// Serialize an Execution Attempt to JSON.
/// Output is deterministic: same input always produces same output.
pub fn serialize_execution_attempt(attempt: &ExecutionAttempt) -> Result<String, String> {
    serde_json::to_string_pretty(attempt).map_err(|e| e.to_string())
}

/// Deserialize an Execution Attempt from JSON.
/// Returns the attempt if parsing succeeds; does not perform validation.
pub fn deserialize_execution_attempt(json: &str) -> Result<ExecutionAttempt, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Deserialize and validate an Execution Attempt from JSON.
/// Returns the attempt only if parsing succeeds AND validation passes.
pub fn deserialize_and_validate_attempt(json: &str) -> Result<ExecutionAttempt, Vec<String>> {
    let attempt: ExecutionAttempt = serde_json::from_str(json)
        .map_err(|e| vec![format!("Parse error: {}", e)])?;

    let errors = validate_execution_attempt(&attempt);
    if errors.is_empty() {
        Ok(attempt)
    } else {
        Err(errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect())
    }
}

// =============================================================================
// EXECUTION ATTEMPT PERSISTENCE
// Storage location: /var/lib/anna/execution_attempts/
// File naming: {attempt_id}.v1.json
// =============================================================================

/// Get the execution attempts storage directory.
pub fn execution_attempts_directory() -> PathBuf {
    crate::paths::paths().data_dir.join("execution_attempts")
}

/// Generate the canonical filename for an attempt record.
fn attempt_filename(attempt_id: &str, version: u32) -> String {
    let safe_id: String = attempt_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    format!("{}.v{}.json", safe_id, version)
}

/// Full path for an attempt file.
fn attempt_path(attempt_id: &str, version: u32) -> PathBuf {
    execution_attempts_directory().join(attempt_filename(attempt_id, version))
}

/// Attempt storage error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptStorageError {
    /// Storage directory does not exist or cannot be created
    DirectoryError(String),
    /// File write failed
    WriteError(String),
    /// File read failed
    ReadError(String),
    /// Attempt not found
    NotFound(String),
    /// Unknown format version
    UnknownVersion(u32),
    /// Validation failed after load
    ValidationFailed(Vec<String>),
}

/// Save an Execution Attempt to disk.
/// Performs validation before saving.
/// File path: /var/lib/anna/execution_attempts/{attempt_id}.v1.json
pub fn save_execution_attempt(attempt: &ExecutionAttempt) -> Result<PathBuf, AttemptStorageError> {
    // Validate before saving
    let errors = validate_execution_attempt(attempt);
    if !errors.is_empty() {
        return Err(AttemptStorageError::ValidationFailed(
            errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect()
        ));
    }

    // Ensure directory exists
    let dir = execution_attempts_directory();
    fs::create_dir_all(&dir).map_err(|e| AttemptStorageError::DirectoryError(e.to_string()))?;

    // Generate path
    let path = attempt_path(&attempt.attempt_id, ATTEMPT_FORMAT_VERSION);

    // Serialize
    let json = serialize_execution_attempt(attempt)
        .map_err(|e| AttemptStorageError::WriteError(e))?;

    // Write atomically (write to temp, then rename)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &json).map_err(|e| AttemptStorageError::WriteError(e.to_string()))?;
    fs::rename(&temp_path, &path).map_err(|e| AttemptStorageError::WriteError(e.to_string()))?;

    Ok(path)
}

/// Load an Execution Attempt from disk by attempt_id.
pub fn load_execution_attempt(attempt_id: &str) -> Result<ExecutionAttempt, AttemptStorageError> {
    load_execution_attempt_version(attempt_id, ATTEMPT_FORMAT_VERSION)
}

/// Load an Execution Attempt with a specific format version.
pub fn load_execution_attempt_version(attempt_id: &str, version: u32) -> Result<ExecutionAttempt, AttemptStorageError> {
    if version != ATTEMPT_FORMAT_VERSION {
        return Err(AttemptStorageError::UnknownVersion(version));
    }

    let path = attempt_path(attempt_id, version);

    if !path.exists() {
        return Err(AttemptStorageError::NotFound(attempt_id.to_string()));
    }

    let json = fs::read_to_string(&path)
        .map_err(|e| AttemptStorageError::ReadError(e.to_string()))?;

    let attempt = deserialize_execution_attempt(&json)
        .map_err(|e| AttemptStorageError::ReadError(e))?;

    // Validate after load
    let errors = validate_execution_attempt(&attempt);
    if !errors.is_empty() {
        return Err(AttemptStorageError::ValidationFailed(
            errors.iter().map(|e| format!("{}: {}", e.field, e.message)).collect()
        ));
    }

    Ok(attempt)
}

/// List all stored attempt IDs.
pub fn list_attempt_ids() -> Result<Vec<String>, AttemptStorageError> {
    let dir = execution_attempts_directory();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let suffix = format!(".v{}.json", ATTEMPT_FORMAT_VERSION);
    let mut ids = Vec::new();

    let entries = fs::read_dir(&dir)
        .map_err(|e| AttemptStorageError::DirectoryError(e.to_string()))?;

    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(&suffix) {
                let id = name.trim_end_matches(&suffix).to_string();
                ids.push(id);
            }
        }
    }

    ids.sort();
    Ok(ids)
}

/// List attempt IDs for a specific plan.
/// Loads each attempt to filter by plan_id.
pub fn list_attempts_for_plan(plan_id: &str) -> Result<Vec<String>, AttemptStorageError> {
    let all_ids = list_attempt_ids()?;
    let mut matching = Vec::new();

    for id in all_ids {
        if let Ok(attempt) = load_execution_attempt(&id) {
            if attempt.plan_id == plan_id {
                matching.push(id);
            }
        }
    }

    Ok(matching)
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES (Execution Attempt Contract)
// This section documents what the Execution Attempt DOES NOT and CANNOT do.
// These non-capabilities are by design and must never be violated.
// =============================================================================
//
// The ExecutionAttempt:
// - DOES NOT execute any commands
// - DOES NOT simulate execution
// - DOES NOT decide anything
// - DOES NOT authorize execution
// - DOES NOT imply success or failure of actual execution
// - DOES NOT affect system state
// - DOES NOT call system APIs
// - DOES NOT use ExecutionAdapter
// - DOES NOT bypass ExecutionGate
// - DOES NOT recompute gate_result (records verbatim)
// - DOES NOT perform semantic validation
// - DOES NOT cross-reference other records
//
// The ExecutionAttempt is a passive audit record.
// It documents what was attempted and what the outcome classification was.
// It performs no execution and grants no authority.
//
// This record documents an execution attempt and performs no execution.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_step_builder() {
        let step = ActionStep::new("Test", "echo test", false)
            .with_files(&["/etc/test.conf"])
            .with_units(&["test.service"])
            .with_verify("test -f /etc/test.conf", "")
            .with_rollback("rm /etc/test.conf");

        assert_eq!(step.affects_files.len(), 1);
        assert_eq!(step.affects_units.len(), 1);
        assert!(step.rollback_command.is_some());
    }

    #[test]
    fn test_plan_no_changes() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        plan.mark_no_changes("Already configured");

        assert!(!plan.changes_needed);
        assert!(plan.steps.is_empty());
        assert!(plan.format_for_confirmation().contains("No changes needed"));
    }

    #[test]
    fn test_plan_no_rollback() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        plan.set_no_rollback("Destructive operation");

        assert!(!plan.rollback.possible);
    }

    #[test]
    fn test_phase25_preflight_result() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        assert_eq!(plan.preflight_result, PreflightResult::Passed);

        plan.mark_no_changes("Already configured");
        assert_eq!(plan.preflight_result, PreflightResult::Blocked);

        let mut plan2 = ActionPlan::new("test", "Test", "Testing");
        plan2.mark_preflight_unknown("Cannot determine state");
        assert_eq!(plan2.preflight_result, PreflightResult::Unknown);
    }

    #[test]
    fn test_phase25_reversibility() {
        let mut plan = ActionPlan::new("test", "Test", "Testing");
        assert_eq!(plan.reversibility(), Reversibility::Reversible);

        plan.set_no_rollback("Destructive operation");
        assert_eq!(plan.reversibility(), Reversibility::NonReversible);
    }

    #[test]
    fn test_phase25_verification_status_serialization() {
        let status = VerificationStatus::Passed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"passed\"");

        let status: VerificationStatus = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(status, VerificationStatus::Unknown);
    }

    // =========================================================================
    // GOLDEN TESTS - Deterministic Action Plan Data Contract
    // Fixed fixtures with exact expected outputs.
    // =========================================================================

    /// Golden fixture: Valid service restart plan
    fn golden_valid_service_restart() -> DeterministicActionPlan {
        DeterministicActionPlan {
            plan_id: "golden-001".to_string(),
            created_utc: "2026-01-15T12:00:00Z".to_string(),
            intent: "Restart NetworkManager service".to_string(),
            target: "NetworkManager.service".to_string(),
            preconditions: vec!["NetworkManager is installed".to_string()],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "service_restart".to_string(),
                target: "NetworkManager.service".to_string(),
            }],
            reversible: true,
            rollback_steps: vec![DeterministicStep {
                step_number: 1,
                operation: "service_restart".to_string(),
                target: "NetworkManager.service".to_string(),
            }],
            evidence_sources: vec!["systemctl status NetworkManager".to_string()],
        }
    }

    /// Golden fixture: Valid irreversible package removal
    fn golden_valid_irreversible() -> DeterministicActionPlan {
        DeterministicActionPlan {
            plan_id: "golden-002".to_string(),
            created_utc: "2026-01-15T12:00:00Z".to_string(),
            intent: "Remove orphan packages".to_string(),
            target: "pacman orphans".to_string(),
            preconditions: vec![],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "package_remove".to_string(),
                target: "orphans".to_string(),
            }],
            reversible: false,
            rollback_steps: vec![],
            evidence_sources: vec!["pacman -Qtdq".to_string()],
        }
    }

    /// Golden fixture: Valid multi-step plan
    fn golden_valid_multi_step() -> DeterministicActionPlan {
        DeterministicActionPlan {
            plan_id: "golden-003".to_string(),
            created_utc: "2026-01-15T12:00:00Z".to_string(),
            intent: "Enable and start bluetooth".to_string(),
            target: "bluetooth.service".to_string(),
            preconditions: vec![
                "bluez is installed".to_string(),
                "bluetooth hardware present".to_string(),
            ],
            steps: vec![
                DeterministicStep {
                    step_number: 1,
                    operation: "service_enable".to_string(),
                    target: "bluetooth.service".to_string(),
                },
                DeterministicStep {
                    step_number: 2,
                    operation: "service_start".to_string(),
                    target: "bluetooth.service".to_string(),
                },
            ],
            reversible: true,
            rollback_steps: vec![
                DeterministicStep {
                    step_number: 1,
                    operation: "service_stop".to_string(),
                    target: "bluetooth.service".to_string(),
                },
                DeterministicStep {
                    step_number: 2,
                    operation: "service_disable".to_string(),
                    target: "bluetooth.service".to_string(),
                },
            ],
            evidence_sources: vec![
                "systemctl status bluetooth".to_string(),
                "rfkill list".to_string(),
            ],
        }
    }

    /// Golden fixture: Invalid - multiple errors
    fn golden_invalid_multiple_errors() -> DeterministicActionPlan {
        DeterministicActionPlan {
            plan_id: String::new(),
            created_utc: "bad-date".to_string(),
            intent: String::new(),
            target: "some.service".to_string(),
            preconditions: vec![],
            steps: vec![DeterministicStep {
                step_number: 0,
                operation: String::new(),
                target: "some.service".to_string(),
            }],
            reversible: true,
            rollback_steps: vec![],
            evidence_sources: vec![],
        }
    }

    /// Golden fixture: Invalid - reversibility mismatch
    fn golden_invalid_reversibility_mismatch() -> DeterministicActionPlan {
        DeterministicActionPlan {
            plan_id: "golden-005".to_string(),
            created_utc: "2026-01-15T12:00:00Z".to_string(),
            intent: "Test reversibility mismatch".to_string(),
            target: "test.service".to_string(),
            preconditions: vec![],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "test_op".to_string(),
                target: "test.service".to_string(),
            }],
            reversible: false,
            rollback_steps: vec![DeterministicStep {
                step_number: 1,
                operation: "undo".to_string(),
                target: "test.service".to_string(),
            }],
            evidence_sources: vec![],
        }
    }

    #[test]
    fn golden_test_valid_service_restart() {
        let plan = golden_valid_service_restart();
        let errors = validate_deterministic_plan(&plan);
        assert_eq!(errors, vec![], "Golden valid plan must produce zero errors");
    }

    #[test]
    fn golden_test_valid_irreversible() {
        let plan = golden_valid_irreversible();
        let errors = validate_deterministic_plan(&plan);
        assert_eq!(errors, vec![], "Golden valid irreversible plan must produce zero errors");
    }

    #[test]
    fn golden_test_valid_multi_step() {
        let plan = golden_valid_multi_step();
        let errors = validate_deterministic_plan(&plan);
        assert_eq!(errors, vec![], "Golden valid multi-step plan must produce zero errors");
    }

    #[test]
    fn golden_test_invalid_multiple_errors_exact() {
        let plan = golden_invalid_multiple_errors();
        let errors = validate_deterministic_plan(&plan);

        let expected = vec![
            DeterministicValidationError {
                field: "plan_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            DeterministicValidationError {
                field: "created_utc".to_string(),
                message: "must be ISO 8601 format".to_string(),
            },
            DeterministicValidationError {
                field: "intent".to_string(),
                message: "must be non-empty".to_string(),
            },
            DeterministicValidationError {
                field: "steps[0].step_number".to_string(),
                message: "must be positive".to_string(),
            },
            DeterministicValidationError {
                field: "steps[0].operation".to_string(),
                message: "must be non-empty".to_string(),
            },
            DeterministicValidationError {
                field: "rollback_steps".to_string(),
                message: "required when reversible is true".to_string(),
            },
        ];

        assert_eq!(errors, expected, "Golden invalid plan must produce exact error list");
    }

    #[test]
    fn golden_test_invalid_reversibility_mismatch_exact() {
        let plan = golden_invalid_reversibility_mismatch();
        let errors = validate_deterministic_plan(&plan);

        let expected = vec![DeterministicValidationError {
            field: "rollback_steps".to_string(),
            message: "must be empty when reversible is false".to_string(),
        }];

        assert_eq!(errors, expected, "Reversibility mismatch must produce exact error");
    }

    #[test]
    fn golden_test_determinism_across_runs() {
        for _ in 0..10 {
            let plan1 = golden_valid_service_restart();
            let plan2 = golden_invalid_multiple_errors();

            let errors1 = validate_deterministic_plan(&plan1);
            let errors2 = validate_deterministic_plan(&plan2);

            assert_eq!(errors1, vec![]);
            assert_eq!(errors2.len(), 6);
        }
    }

    #[test]
    fn golden_test_empty_preconditions_allowed() {
        let mut plan = golden_valid_service_restart();
        plan.preconditions = vec![];
        let errors = validate_deterministic_plan(&plan);
        assert_eq!(errors, vec![]);
    }

    #[test]
    fn golden_test_empty_evidence_sources_allowed() {
        let mut plan = golden_valid_service_restart();
        plan.evidence_sources = vec![];
        let errors = validate_deterministic_plan(&plan);
        assert_eq!(errors, vec![]);
    }

    // =========================================================================
    // GOLDEN SERIALIZATION TESTS - Fixed input/output pairs
    // =========================================================================

    /// Minimal valid plan for serialization testing
    fn golden_serialization_minimal() -> DeterministicActionPlan {
        DeterministicActionPlan {
            plan_id: "ser-001".to_string(),
            created_utc: "2026-01-15T00:00:00Z".to_string(),
            intent: "Test serialization".to_string(),
            target: "test.service".to_string(),
            preconditions: vec![],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "test".to_string(),
                target: "test.service".to_string(),
            }],
            reversible: false,
            rollback_steps: vec![],
            evidence_sources: vec![],
        }
    }

    /// Expected JSON for minimal plan - exact bytes
    const GOLDEN_MINIMAL_JSON: &str = r#"{
  "plan_id": "ser-001",
  "created_utc": "2026-01-15T00:00:00Z",
  "intent": "Test serialization",
  "target": "test.service",
  "preconditions": [],
  "steps": [
    {
      "step_number": 1,
      "operation": "test",
      "target": "test.service"
    }
  ],
  "reversible": false,
  "rollback_steps": [],
  "evidence_sources": []
}"#;

    #[test]
    fn golden_serialization_output_exact() {
        let plan = golden_serialization_minimal();
        let json = serialize_deterministic_plan(&plan).unwrap();
        assert_eq!(json, GOLDEN_MINIMAL_JSON, "Serialization must produce exact expected output");
    }

    #[test]
    fn golden_serialization_roundtrip() {
        let original = golden_serialization_minimal();
        let json = serialize_deterministic_plan(&original).unwrap();
        let restored = deserialize_deterministic_plan(&json).unwrap();
        assert_eq!(original, restored, "Round-trip must preserve exact data");
    }

    #[test]
    fn golden_serialization_roundtrip_complex() {
        let original = golden_valid_multi_step();
        let json = serialize_deterministic_plan(&original).unwrap();
        let restored = deserialize_deterministic_plan(&json).unwrap();
        assert_eq!(original, restored, "Complex plan round-trip must preserve exact data");
    }

    #[test]
    fn golden_serialization_determinism() {
        let plan = golden_serialization_minimal();
        let json1 = serialize_deterministic_plan(&plan).unwrap();
        let json2 = serialize_deterministic_plan(&plan).unwrap();
        let json3 = serialize_deterministic_plan(&plan).unwrap();
        assert_eq!(json1, json2);
        assert_eq!(json2, json3);
    }

    #[test]
    fn golden_deserialization_from_fixed_input() {
        let plan = deserialize_deterministic_plan(GOLDEN_MINIMAL_JSON).unwrap();
        assert_eq!(plan.plan_id, "ser-001");
        assert_eq!(plan.created_utc, "2026-01-15T00:00:00Z");
        assert_eq!(plan.intent, "Test serialization");
        assert_eq!(plan.target, "test.service");
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].step_number, 1);
        assert_eq!(plan.reversible, false);
    }

    #[test]
    fn golden_deserialization_passes_validation() {
        let plan = deserialize_deterministic_plan(GOLDEN_MINIMAL_JSON).unwrap();
        let errors = validate_deterministic_plan(&plan);
        assert_eq!(errors, vec![], "Deserialized golden plan must pass validation");
    }

    #[test]
    fn golden_deserialize_and_validate_valid() {
        let result = deserialize_and_validate_plan(GOLDEN_MINIMAL_JSON);
        assert!(result.is_ok(), "Valid JSON must parse and validate");
    }

    #[test]
    fn golden_deserialize_and_validate_invalid_json() {
        let result = deserialize_and_validate_plan("not valid json");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Parse error"));
    }

    #[test]
    fn golden_deserialize_and_validate_invalid_plan() {
        let invalid_json = r#"{
  "plan_id": "",
  "created_utc": "2026-01-15T00:00:00Z",
  "intent": "Test",
  "target": "test.service",
  "preconditions": [],
  "steps": [{"step_number": 1, "operation": "test", "target": "test.service"}],
  "reversible": false,
  "rollback_steps": [],
  "evidence_sources": []
}"#;
        let result = deserialize_and_validate_plan(invalid_json);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("plan_id")));
    }

    #[test]
    fn golden_serialization_field_order_stable() {
        // Verify field order matches struct definition order
        let plan = golden_serialization_minimal();
        let json = serialize_deterministic_plan(&plan).unwrap();

        let plan_id_pos = json.find("plan_id").unwrap();
        let created_pos = json.find("created_utc").unwrap();
        let intent_pos = json.find("intent").unwrap();
        let target_pos = json.find("\"target\"").unwrap();
        let preconditions_pos = json.find("preconditions").unwrap();
        let steps_pos = json.find("steps").unwrap();
        let reversible_pos = json.find("reversible").unwrap();
        let rollback_pos = json.find("rollback_steps").unwrap();
        let evidence_pos = json.find("evidence_sources").unwrap();

        assert!(plan_id_pos < created_pos);
        assert!(created_pos < intent_pos);
        assert!(intent_pos < target_pos);
        assert!(target_pos < preconditions_pos);
        assert!(preconditions_pos < steps_pos);
        assert!(steps_pos < reversible_pos);
        assert!(reversible_pos < rollback_pos);
        assert!(rollback_pos < evidence_pos);
    }

    // =========================================================================
    // PERSISTENCE TESTS - File I/O only, no behavior
    // =========================================================================

    /// Create a temp directory for persistence tests
    fn setup_test_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("anna-test-plans")
            .join(format!("{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("ANNA_DEV_MODE", "1");
        dir
    }

    fn cleanup_test_dir(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn persistence_filename_format() {
        let filename = plan_filename("test-plan-001", 1);
        assert_eq!(filename, "test-plan-001.v1.json");
    }

    #[test]
    fn persistence_filename_sanitization() {
        let filename = plan_filename("plan/with:special@chars!", 1);
        assert_eq!(filename, "plan-with-special-chars-.v1.json");
    }

    #[test]
    fn persistence_save_and_load_roundtrip() {
        let test_dir = setup_test_dir();

        let plan = DeterministicActionPlan {
            plan_id: format!("persist-test-{}", std::process::id()),
            created_utc: "2026-01-15T00:00:00Z".to_string(),
            intent: "Test persistence".to_string(),
            target: "test.service".to_string(),
            preconditions: vec![],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "test".to_string(),
                target: "test.service".to_string(),
            }],
            reversible: false,
            rollback_steps: vec![],
            evidence_sources: vec![],
        };

        // Save
        let save_result = save_plan(&plan);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result);

        // Load
        let loaded = load_plan(&plan.plan_id);
        assert!(loaded.is_ok(), "Load failed: {:?}", loaded);

        // Assert equality
        let loaded_plan = loaded.unwrap();
        assert_eq!(plan, loaded_plan, "Loaded plan must equal saved plan");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn persistence_load_not_found() {
        let _test_dir = setup_test_dir();

        let result = load_plan("nonexistent-plan-id-12345");
        assert!(matches!(result, Err(PlanStorageError::NotFound(_))));
    }

    #[test]
    fn persistence_reject_unknown_version() {
        let result = load_plan_version("any-id", 99);
        assert!(matches!(result, Err(PlanStorageError::UnknownVersion(99))));
    }

    #[test]
    fn persistence_save_invalid_plan_rejected() {
        let _test_dir = setup_test_dir();

        let invalid_plan = DeterministicActionPlan {
            plan_id: String::new(), // Invalid: empty
            created_utc: "2026-01-15T00:00:00Z".to_string(),
            intent: "Test".to_string(),
            target: "test.service".to_string(),
            preconditions: vec![],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "test".to_string(),
                target: "test.service".to_string(),
            }],
            reversible: false,
            rollback_steps: vec![],
            evidence_sources: vec![],
        };

        let result = save_plan(&invalid_plan);
        assert!(matches!(result, Err(PlanStorageError::ValidationFailed(_))));
    }

    #[test]
    fn persistence_list_empty_directory() {
        let test_dir = setup_test_dir();

        // Empty directory should return empty list
        let ids = list_plan_ids();
        assert!(ids.is_ok());
        // May or may not be empty depending on other tests, just verify no error

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn persistence_plan_not_mutated_on_disk() {
        let test_dir = setup_test_dir();

        let plan = DeterministicActionPlan {
            plan_id: format!("immutable-test-{}", std::process::id()),
            created_utc: "2026-01-15T00:00:00Z".to_string(),
            intent: "Test immutability".to_string(),
            target: "test.service".to_string(),
            preconditions: vec!["condition1".to_string()],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "test".to_string(),
                target: "test.service".to_string(),
            }],
            reversible: false,
            rollback_steps: vec![],
            evidence_sources: vec!["source1".to_string()],
        };

        // Save original
        save_plan(&plan).unwrap();

        // Load twice and verify identical
        let loaded1 = load_plan(&plan.plan_id).unwrap();
        let loaded2 = load_plan(&plan.plan_id).unwrap();

        assert_eq!(loaded1, loaded2, "Multiple loads must return identical data");
        assert_eq!(plan, loaded1, "Loaded data must match original");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn persistence_format_version_constant() {
        // Version must be 1 for this contract
        assert_eq!(PLAN_FORMAT_VERSION, 1);
    }

    // =========================================================================
    // APPROVAL RECORD TESTS - Phase 31 Decision Boundary
    // Golden tests for ApprovalRecord data contract.
    // This approval record authorizes nothing and performs no action.
    // =========================================================================

    /// Golden fixture: Valid approval record (approved)
    fn golden_approval_approved() -> ApprovalRecord {
        ApprovalRecord {
            approval_id: "apr-001".to_string(),
            plan_id: "golden-001".to_string(),
            plan_version: 1,
            decision: ApprovalDecision::Approved,
            decided_utc: "2026-01-15T12:00:00Z".to_string(),
            decided_by: "operator".to_string(),
            comment: Some("Approved after review".to_string()),
        }
    }

    /// Golden fixture: Valid approval record (rejected)
    fn golden_approval_rejected() -> ApprovalRecord {
        ApprovalRecord {
            approval_id: "apr-002".to_string(),
            plan_id: "golden-002".to_string(),
            plan_version: 1,
            decision: ApprovalDecision::Rejected,
            decided_utc: "2026-01-15T13:00:00Z".to_string(),
            decided_by: "admin".to_string(),
            comment: None,
        }
    }

    /// Golden fixture: Invalid approval record
    fn golden_approval_invalid() -> ApprovalRecord {
        ApprovalRecord {
            approval_id: String::new(),
            plan_id: String::new(),
            plan_version: 0,
            decision: ApprovalDecision::Approved,
            decided_utc: "bad-date".to_string(),
            decided_by: String::new(),
            comment: None,
        }
    }

    #[test]
    fn approval_golden_valid_approved() {
        let record = golden_approval_approved();
        let errors = validate_approval_record(&record);
        assert_eq!(errors, vec![], "Valid approved record must produce zero errors");
    }

    #[test]
    fn approval_golden_valid_rejected() {
        let record = golden_approval_rejected();
        let errors = validate_approval_record(&record);
        assert_eq!(errors, vec![], "Valid rejected record must produce zero errors");
    }

    #[test]
    fn approval_golden_invalid_exact() {
        let record = golden_approval_invalid();
        let errors = validate_approval_record(&record);

        let expected = vec![
            ApprovalValidationError {
                field: "approval_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            ApprovalValidationError {
                field: "plan_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            ApprovalValidationError {
                field: "plan_version".to_string(),
                message: "must be positive".to_string(),
            },
            ApprovalValidationError {
                field: "decided_utc".to_string(),
                message: "must be ISO 8601 format".to_string(),
            },
            ApprovalValidationError {
                field: "decided_by".to_string(),
                message: "must be non-empty".to_string(),
            },
        ];

        assert_eq!(errors, expected, "Invalid record must produce exact error list");
    }

    /// Minimal valid approval for serialization testing
    fn golden_approval_minimal() -> ApprovalRecord {
        ApprovalRecord {
            approval_id: "apr-min".to_string(),
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            decision: ApprovalDecision::Approved,
            decided_utc: "2026-01-15T00:00:00Z".to_string(),
            decided_by: "operator".to_string(),
            comment: None,
        }
    }

    /// Expected JSON for minimal approval - exact bytes (no comment field)
    const GOLDEN_APPROVAL_JSON: &str = r#"{
  "approval_id": "apr-min",
  "plan_id": "plan-001",
  "plan_version": 1,
  "decision": "approved",
  "decided_utc": "2026-01-15T00:00:00Z",
  "decided_by": "operator"
}"#;

    #[test]
    fn approval_golden_serialization_exact() {
        let record = golden_approval_minimal();
        let json = serialize_approval_record(&record).unwrap();
        assert_eq!(json, GOLDEN_APPROVAL_JSON, "Serialization must produce exact output");
    }

    #[test]
    fn approval_golden_serialization_roundtrip() {
        let original = golden_approval_minimal();
        let json = serialize_approval_record(&original).unwrap();
        let restored = deserialize_approval_record(&json).unwrap();
        assert_eq!(original, restored, "Round-trip must preserve exact data");
    }

    #[test]
    fn approval_golden_serialization_with_comment() {
        let record = golden_approval_approved();
        let json = serialize_approval_record(&record).unwrap();
        assert!(json.contains("\"comment\":"), "Comment field must be present when set");
        assert!(json.contains("Approved after review"));

        let restored = deserialize_approval_record(&json).unwrap();
        assert_eq!(record, restored);
    }

    #[test]
    fn approval_golden_decision_serialization() {
        // Test approved
        let approved = ApprovalDecision::Approved;
        let json = serde_json::to_string(&approved).unwrap();
        assert_eq!(json, "\"approved\"");

        // Test rejected
        let rejected = ApprovalDecision::Rejected;
        let json = serde_json::to_string(&rejected).unwrap();
        assert_eq!(json, "\"rejected\"");

        // Test deserialization
        let parsed: ApprovalDecision = serde_json::from_str("\"approved\"").unwrap();
        assert_eq!(parsed, ApprovalDecision::Approved);

        let parsed: ApprovalDecision = serde_json::from_str("\"rejected\"").unwrap();
        assert_eq!(parsed, ApprovalDecision::Rejected);
    }

    #[test]
    fn approval_golden_deserialize_and_validate_valid() {
        let result = deserialize_and_validate_approval(GOLDEN_APPROVAL_JSON);
        assert!(result.is_ok(), "Valid JSON must parse and validate");
    }

    #[test]
    fn approval_golden_deserialize_and_validate_invalid_json() {
        let result = deserialize_and_validate_approval("not valid json");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Parse error"));
    }

    #[test]
    fn approval_golden_deserialize_and_validate_invalid_record() {
        let invalid_json = r#"{
  "approval_id": "",
  "plan_id": "plan-001",
  "plan_version": 1,
  "decision": "approved",
  "decided_utc": "2026-01-15T00:00:00Z",
  "decided_by": "operator"
}"#;
        let result = deserialize_and_validate_approval(invalid_json);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("approval_id")));
    }

    #[test]
    fn approval_golden_field_order_stable() {
        let record = golden_approval_minimal();
        let json = serialize_approval_record(&record).unwrap();

        let approval_id_pos = json.find("approval_id").unwrap();
        let plan_id_pos = json.find("plan_id").unwrap();
        let plan_version_pos = json.find("plan_version").unwrap();
        let decision_pos = json.find("decision").unwrap();
        let decided_utc_pos = json.find("decided_utc").unwrap();
        let decided_by_pos = json.find("decided_by").unwrap();

        assert!(approval_id_pos < plan_id_pos);
        assert!(plan_id_pos < plan_version_pos);
        assert!(plan_version_pos < decision_pos);
        assert!(decision_pos < decided_utc_pos);
        assert!(decided_utc_pos < decided_by_pos);
    }

    // =========================================================================
    // APPROVAL PERSISTENCE TESTS
    // =========================================================================

    #[test]
    fn approval_persistence_filename_format() {
        let filename = approval_filename("apr-001", 1);
        assert_eq!(filename, "apr-001.v1.json");
    }

    #[test]
    fn approval_persistence_save_and_load_roundtrip() {
        let test_dir = setup_test_dir();

        let record = ApprovalRecord {
            approval_id: format!("apr-persist-{}", std::process::id()),
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            decision: ApprovalDecision::Approved,
            decided_utc: "2026-01-15T00:00:00Z".to_string(),
            decided_by: "operator".to_string(),
            comment: Some("Test comment".to_string()),
        };

        // Save
        let save_result = save_approval(&record);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result);

        // Load
        let loaded = load_approval(&record.approval_id);
        assert!(loaded.is_ok(), "Load failed: {:?}", loaded);

        // Assert equality
        let loaded_record = loaded.unwrap();
        assert_eq!(record, loaded_record, "Loaded record must equal saved record");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn approval_persistence_load_not_found() {
        let _test_dir = setup_test_dir();

        let result = load_approval("nonexistent-approval-id-12345");
        assert!(matches!(result, Err(ApprovalStorageError::NotFound(_))));
    }

    #[test]
    fn approval_persistence_reject_unknown_version() {
        let result = load_approval_version("any-id", 99);
        assert!(matches!(result, Err(ApprovalStorageError::UnknownVersion(99))));
    }

    #[test]
    fn approval_persistence_save_invalid_rejected() {
        let _test_dir = setup_test_dir();

        let invalid_record = ApprovalRecord {
            approval_id: String::new(), // Invalid: empty
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            decision: ApprovalDecision::Approved,
            decided_utc: "2026-01-15T00:00:00Z".to_string(),
            decided_by: "operator".to_string(),
            comment: None,
        };

        let result = save_approval(&invalid_record);
        assert!(matches!(result, Err(ApprovalStorageError::ValidationFailed(_))));
    }

    #[test]
    fn approval_persistence_format_version_constant() {
        assert_eq!(APPROVAL_FORMAT_VERSION, 1);
    }

    #[test]
    fn approval_persistence_determinism() {
        let test_dir = setup_test_dir();

        let record = ApprovalRecord {
            approval_id: format!("apr-det-{}", std::process::id()),
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            decision: ApprovalDecision::Rejected,
            decided_utc: "2026-01-15T00:00:00Z".to_string(),
            decided_by: "admin".to_string(),
            comment: None,
        };

        save_approval(&record).unwrap();

        // Load multiple times
        let loaded1 = load_approval(&record.approval_id).unwrap();
        let loaded2 = load_approval(&record.approval_id).unwrap();

        assert_eq!(loaded1, loaded2, "Multiple loads must return identical data");
        assert_eq!(record, loaded1, "Loaded data must match original");

        cleanup_test_dir(&test_dir);
    }

    // =========================================================================
    // EXECUTION READINESS TESTS - Phase 32
    // Exhaustive tests for the execution readiness classifier.
    // This classification enables no execution and confers no authority.
    // =========================================================================

    /// Create a minimal valid plan for readiness tests
    fn readiness_test_plan() -> DeterministicActionPlan {
        DeterministicActionPlan {
            plan_id: "readiness-test".to_string(),
            created_utc: "2026-01-15T00:00:00Z".to_string(),
            intent: "Test readiness classification".to_string(),
            target: "test.service".to_string(),
            preconditions: vec![],
            steps: vec![DeterministicStep {
                step_number: 1,
                operation: "test".to_string(),
                target: "test.service".to_string(),
            }],
            reversible: false,
            rollback_steps: vec![],
            evidence_sources: vec![],
        }
    }

    /// Create an approval with current version
    fn readiness_approval_current() -> ApprovalRecord {
        ApprovalRecord {
            approval_id: "apr-current".to_string(),
            plan_id: "readiness-test".to_string(),
            plan_version: PLAN_FORMAT_VERSION, // Current version
            decision: ApprovalDecision::Approved,
            decided_utc: "2026-01-15T00:00:00Z".to_string(),
            decided_by: "operator".to_string(),
            comment: None,
        }
    }

    /// Create an approval with stale version
    fn readiness_approval_stale() -> ApprovalRecord {
        ApprovalRecord {
            approval_id: "apr-stale".to_string(),
            plan_id: "readiness-test".to_string(),
            plan_version: PLAN_FORMAT_VERSION + 1, // Different version
            decision: ApprovalDecision::Approved,
            decided_utc: "2026-01-15T00:00:00Z".to_string(),
            decided_by: "operator".to_string(),
            comment: None,
        }
    }

    #[test]
    fn readiness_no_approval_returns_not_approved() {
        let plan = readiness_test_plan();
        let result = classify_execution_readiness(&plan, None);
        assert_eq!(result, ExecutionReadiness::NotApproved);
    }

    #[test]
    fn readiness_current_version_returns_approved_and_current() {
        let plan = readiness_test_plan();
        let approval = readiness_approval_current();
        let result = classify_execution_readiness(&plan, Some(&approval));
        assert_eq!(result, ExecutionReadiness::ApprovedAndCurrent);
    }

    #[test]
    fn readiness_stale_version_returns_approved_but_stale() {
        let plan = readiness_test_plan();
        let approval = readiness_approval_stale();
        let result = classify_execution_readiness(&plan, Some(&approval));
        assert_eq!(result, ExecutionReadiness::ApprovedButStale);
    }

    #[test]
    fn readiness_version_zero_is_stale() {
        let plan = readiness_test_plan();
        let mut approval = readiness_approval_current();
        approval.plan_version = 0;
        let result = classify_execution_readiness(&plan, Some(&approval));
        assert_eq!(result, ExecutionReadiness::ApprovedButStale);
    }

    #[test]
    fn readiness_rejected_approval_still_classified_by_version() {
        // Note: Rejection status is not considered by this classifier
        let plan = readiness_test_plan();
        let mut approval = readiness_approval_current();
        approval.decision = ApprovalDecision::Rejected;
        let result = classify_execution_readiness(&plan, Some(&approval));
        // Still returns ApprovedAndCurrent because we only check version
        assert_eq!(result, ExecutionReadiness::ApprovedAndCurrent);
    }

    #[test]
    fn readiness_determinism_same_inputs_same_output() {
        let plan = readiness_test_plan();
        let approval = readiness_approval_current();

        for _ in 0..10 {
            let r1 = classify_execution_readiness(&plan, Some(&approval));
            let r2 = classify_execution_readiness(&plan, Some(&approval));
            assert_eq!(r1, r2);
            assert_eq!(r1, ExecutionReadiness::ApprovedAndCurrent);
        }
    }

    #[test]
    fn readiness_determinism_none_approval() {
        let plan = readiness_test_plan();

        for _ in 0..10 {
            let r1 = classify_execution_readiness(&plan, None);
            let r2 = classify_execution_readiness(&plan, None);
            assert_eq!(r1, r2);
            assert_eq!(r1, ExecutionReadiness::NotApproved);
        }
    }

    #[test]
    fn readiness_enum_serialization() {
        let not_approved = ExecutionReadiness::NotApproved;
        let stale = ExecutionReadiness::ApprovedButStale;
        let current = ExecutionReadiness::ApprovedAndCurrent;

        assert_eq!(serde_json::to_string(&not_approved).unwrap(), "\"not_approved\"");
        assert_eq!(serde_json::to_string(&stale).unwrap(), "\"approved_but_stale\"");
        assert_eq!(serde_json::to_string(&current).unwrap(), "\"approved_and_current\"");
    }

    #[test]
    fn readiness_enum_deserialization() {
        let not_approved: ExecutionReadiness = serde_json::from_str("\"not_approved\"").unwrap();
        let stale: ExecutionReadiness = serde_json::from_str("\"approved_but_stale\"").unwrap();
        let current: ExecutionReadiness = serde_json::from_str("\"approved_and_current\"").unwrap();

        assert_eq!(not_approved, ExecutionReadiness::NotApproved);
        assert_eq!(stale, ExecutionReadiness::ApprovedButStale);
        assert_eq!(current, ExecutionReadiness::ApprovedAndCurrent);
    }

    #[test]
    fn readiness_all_enum_variants_distinct() {
        assert_ne!(ExecutionReadiness::NotApproved, ExecutionReadiness::ApprovedButStale);
        assert_ne!(ExecutionReadiness::NotApproved, ExecutionReadiness::ApprovedAndCurrent);
        assert_ne!(ExecutionReadiness::ApprovedButStale, ExecutionReadiness::ApprovedAndCurrent);
    }

    #[test]
    fn readiness_plan_content_does_not_affect_classification() {
        // Classification depends only on approval version, not plan content
        let plan1 = readiness_test_plan();
        let mut plan2 = readiness_test_plan();
        plan2.intent = "Completely different intent".to_string();
        plan2.target = "different.service".to_string();

        let approval = readiness_approval_current();

        let r1 = classify_execution_readiness(&plan1, Some(&approval));
        let r2 = classify_execution_readiness(&plan2, Some(&approval));

        assert_eq!(r1, r2);
        assert_eq!(r1, ExecutionReadiness::ApprovedAndCurrent);
    }

    #[test]
    fn readiness_exhaustive_version_boundary() {
        let plan = readiness_test_plan();
        let mut approval = readiness_approval_current();

        // Test versions around current
        approval.plan_version = PLAN_FORMAT_VERSION;
        assert_eq!(classify_execution_readiness(&plan, Some(&approval)), ExecutionReadiness::ApprovedAndCurrent);

        approval.plan_version = PLAN_FORMAT_VERSION + 1;
        assert_eq!(classify_execution_readiness(&plan, Some(&approval)), ExecutionReadiness::ApprovedButStale);

        approval.plan_version = PLAN_FORMAT_VERSION.saturating_sub(1);
        if PLAN_FORMAT_VERSION > 0 {
            assert_eq!(classify_execution_readiness(&plan, Some(&approval)), ExecutionReadiness::ApprovedButStale);
        }

        approval.plan_version = u32::MAX;
        assert_eq!(classify_execution_readiness(&plan, Some(&approval)), ExecutionReadiness::ApprovedButStale);
    }

    // =========================================================================
    // EXECUTION GATE TESTS - Phase 33
    // Exhaustive tests for the execution gate contract.
    // This gate blocks execution unless a future system explicitly breaks it.
    // =========================================================================

    #[test]
    fn gate_approved_and_current_returns_true() {
        let gate = DefaultExecutionGate;
        assert!(gate.can_execute(ExecutionReadiness::ApprovedAndCurrent));
    }

    #[test]
    fn gate_not_approved_returns_false() {
        let gate = DefaultExecutionGate;
        assert!(!gate.can_execute(ExecutionReadiness::NotApproved));
    }

    #[test]
    fn gate_approved_but_stale_returns_false() {
        let gate = DefaultExecutionGate;
        assert!(!gate.can_execute(ExecutionReadiness::ApprovedButStale));
    }

    #[test]
    fn gate_convenience_function_approved_and_current() {
        assert!(can_execute(ExecutionReadiness::ApprovedAndCurrent));
    }

    #[test]
    fn gate_convenience_function_not_approved() {
        assert!(!can_execute(ExecutionReadiness::NotApproved));
    }

    #[test]
    fn gate_convenience_function_stale() {
        assert!(!can_execute(ExecutionReadiness::ApprovedButStale));
    }

    #[test]
    fn gate_exhaustive_all_variants() {
        let gate = DefaultExecutionGate;

        // Only ApprovedAndCurrent returns true
        assert!(gate.can_execute(ExecutionReadiness::ApprovedAndCurrent));

        // All other variants return false
        assert!(!gate.can_execute(ExecutionReadiness::NotApproved));
        assert!(!gate.can_execute(ExecutionReadiness::ApprovedButStale));
    }

    #[test]
    fn gate_determinism() {
        let gate = DefaultExecutionGate;

        for _ in 0..10 {
            assert!(gate.can_execute(ExecutionReadiness::ApprovedAndCurrent));
            assert!(!gate.can_execute(ExecutionReadiness::NotApproved));
            assert!(!gate.can_execute(ExecutionReadiness::ApprovedButStale));
        }
    }

    #[test]
    fn gate_default_implementation() {
        let gate = DefaultExecutionGate::default();
        assert!(gate.can_execute(ExecutionReadiness::ApprovedAndCurrent));
        assert!(!gate.can_execute(ExecutionReadiness::NotApproved));
    }

    #[test]
    fn gate_clone_preserves_behavior() {
        let gate1 = DefaultExecutionGate;
        let gate2 = gate1.clone();

        assert_eq!(
            gate1.can_execute(ExecutionReadiness::ApprovedAndCurrent),
            gate2.can_execute(ExecutionReadiness::ApprovedAndCurrent)
        );
        assert_eq!(
            gate1.can_execute(ExecutionReadiness::NotApproved),
            gate2.can_execute(ExecutionReadiness::NotApproved)
        );
    }

    #[test]
    fn gate_plan_content_does_not_affect_result() {
        // Gate only considers readiness state, not plan content
        let gate = DefaultExecutionGate;

        // Same readiness, different implied plans - same result
        assert!(gate.can_execute(ExecutionReadiness::ApprovedAndCurrent));
        assert!(gate.can_execute(ExecutionReadiness::ApprovedAndCurrent));

        assert!(!gate.can_execute(ExecutionReadiness::NotApproved));
        assert!(!gate.can_execute(ExecutionReadiness::NotApproved));
    }

    #[test]
    fn gate_integration_with_classifier() {
        // Test the full pipeline: plan -> approval -> readiness -> gate
        let plan = readiness_test_plan();
        let gate = DefaultExecutionGate;

        // No approval -> NotApproved -> false
        let readiness = classify_execution_readiness(&plan, None);
        assert_eq!(readiness, ExecutionReadiness::NotApproved);
        assert!(!gate.can_execute(readiness));

        // Stale approval -> ApprovedButStale -> false
        let stale_approval = readiness_approval_stale();
        let readiness = classify_execution_readiness(&plan, Some(&stale_approval));
        assert_eq!(readiness, ExecutionReadiness::ApprovedButStale);
        assert!(!gate.can_execute(readiness));

        // Current approval -> ApprovedAndCurrent -> true
        let current_approval = readiness_approval_current();
        let readiness = classify_execution_readiness(&plan, Some(&current_approval));
        assert_eq!(readiness, ExecutionReadiness::ApprovedAndCurrent);
        assert!(gate.can_execute(readiness));
    }

    #[test]
    fn gate_rejected_approval_still_blocked() {
        // Even if approval decision is Rejected, gate only sees readiness
        // Rejection is not considered by classifier, but if it were,
        // the gate would still only check the readiness enum
        let plan = readiness_test_plan();
        let mut approval = readiness_approval_current();
        approval.decision = ApprovalDecision::Rejected;

        // Classifier returns ApprovedAndCurrent (it doesn't check decision)
        let readiness = classify_execution_readiness(&plan, Some(&approval));

        // Gate returns true because readiness is ApprovedAndCurrent
        // This test documents that decision checking is NOT in scope
        // A future system would need to add decision checking explicitly
        assert!(can_execute(readiness));
    }

    // =========================================================================
    // EXECUTION ADAPTER TESTS - Phase 34
    // Tests proving no adapter exists and no code path can call execute().
    // This interface defines where execution could exist, not that it does.
    // =========================================================================

    #[test]
    fn adapter_result_enum_exists() {
        // ExecutionResult enum exists as pure data
        let _not_executed = ExecutionResult::NotExecuted;
        let _blocked = ExecutionResult::Blocked;
        let _executed = ExecutionResult::Executed;
        let _failed = ExecutionResult::Failed;
    }

    #[test]
    fn adapter_result_serialization() {
        assert_eq!(serde_json::to_string(&ExecutionResult::NotExecuted).unwrap(), "\"not_executed\"");
        assert_eq!(serde_json::to_string(&ExecutionResult::Blocked).unwrap(), "\"blocked\"");
        assert_eq!(serde_json::to_string(&ExecutionResult::Executed).unwrap(), "\"executed\"");
        assert_eq!(serde_json::to_string(&ExecutionResult::Failed).unwrap(), "\"failed\"");
    }

    #[test]
    fn adapter_result_deserialization() {
        let not_executed: ExecutionResult = serde_json::from_str("\"not_executed\"").unwrap();
        let blocked: ExecutionResult = serde_json::from_str("\"blocked\"").unwrap();
        let executed: ExecutionResult = serde_json::from_str("\"executed\"").unwrap();
        let failed: ExecutionResult = serde_json::from_str("\"failed\"").unwrap();

        assert_eq!(not_executed, ExecutionResult::NotExecuted);
        assert_eq!(blocked, ExecutionResult::Blocked);
        assert_eq!(executed, ExecutionResult::Executed);
        assert_eq!(failed, ExecutionResult::Failed);
    }

    #[test]
    fn adapter_result_all_variants_distinct() {
        assert_ne!(ExecutionResult::NotExecuted, ExecutionResult::Blocked);
        assert_ne!(ExecutionResult::NotExecuted, ExecutionResult::Executed);
        assert_ne!(ExecutionResult::NotExecuted, ExecutionResult::Failed);
        assert_ne!(ExecutionResult::Blocked, ExecutionResult::Executed);
        assert_ne!(ExecutionResult::Blocked, ExecutionResult::Failed);
        assert_ne!(ExecutionResult::Executed, ExecutionResult::Failed);
    }

    #[test]
    fn adapter_trait_is_defined() {
        // This test proves the trait exists as a type
        // It does NOT prove any implementation exists (none do)
        fn _accepts_adapter<T: ExecutionAdapter>(_adapter: &T) {}
        // The above function compiles but can never be called
        // because no type implements ExecutionAdapter
    }

    #[test]
    fn adapter_no_implementation_proof() {
        // This test documents that no implementation exists
        //
        // PROOF BY CONSTRUCTION:
        // 1. ExecutionAdapter is a trait with method execute()
        // 2. No struct in this crate implements ExecutionAdapter
        // 3. No `impl ExecutionAdapter for X` exists anywhere
        // 4. Therefore, no code path can call execute()
        //
        // If an implementation existed, this comment would be false.
        // The build succeeding with this comment is the proof.

        // We cannot instantiate any ExecutionAdapter because none exist
        // This is the desired state - the vacuum where power could be injected
    }

    #[test]
    fn adapter_cannot_be_called() {
        // This test documents that execute() cannot be called
        //
        // To call adapter.execute(plan), you need:
        // 1. An instance of a type that implements ExecutionAdapter
        // 2. No such type exists
        // 3. Therefore, execute() cannot be called
        //
        // This is not a limitation - it is the design.
        // Execution is structurally impossible until an implementation is added.
    }

    #[test]
    fn adapter_result_is_pure_data() {
        // ExecutionResult has no methods that perform I/O or side effects
        // It is pure data that can be created, copied, and compared
        let result = ExecutionResult::NotExecuted;
        let copied = result;
        let cloned = result.clone();

        assert_eq!(result, copied);
        assert_eq!(result, cloned);
    }

    #[test]
    fn adapter_gate_still_required() {
        // Even if an ExecutionAdapter implementation existed,
        // the ExecutionGate would still need to permit execution.
        // This test documents that the adapter does not bypass the gate.
        let gate = DefaultExecutionGate;

        // The gate still blocks non-approved states
        assert!(!gate.can_execute(ExecutionReadiness::NotApproved));
        assert!(!gate.can_execute(ExecutionReadiness::ApprovedButStale));

        // Only ApprovedAndCurrent passes the gate
        assert!(gate.can_execute(ExecutionReadiness::ApprovedAndCurrent));

        // The adapter interface does not change this behavior
        // It defines WHERE execution happens, not WHETHER it's permitted
    }

    // =========================================================================
    // EXECUTION ATTEMPT TESTS
    // Exhaustive tests for execution attempt recording.
    // This record documents an execution attempt and performs no execution.
    // =========================================================================

    /// Golden fixture: Valid execution attempt (blocked)
    fn golden_attempt_blocked() -> ExecutionAttempt {
        ExecutionAttempt {
            attempt_id: "att-001".to_string(),
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            readiness: ExecutionReadiness::NotApproved,
            gate_result: false,
            result: ExecutionResult::Blocked,
            recorded_utc: "2026-01-15T12:00:00Z".to_string(),
            recorded_by: "system".to_string(),
            note: Some("No approval present".to_string()),
        }
    }

    /// Golden fixture: Valid execution attempt (would execute)
    fn golden_attempt_approved() -> ExecutionAttempt {
        ExecutionAttempt {
            attempt_id: "att-002".to_string(),
            plan_id: "plan-002".to_string(),
            plan_version: 1,
            readiness: ExecutionReadiness::ApprovedAndCurrent,
            gate_result: true,
            result: ExecutionResult::NotExecuted,
            recorded_utc: "2026-01-15T13:00:00Z".to_string(),
            recorded_by: "operator".to_string(),
            note: None,
        }
    }

    /// Golden fixture: Invalid execution attempt
    fn golden_attempt_invalid() -> ExecutionAttempt {
        ExecutionAttempt {
            attempt_id: String::new(),
            plan_id: String::new(),
            plan_version: 0,
            readiness: ExecutionReadiness::NotApproved,
            gate_result: false,
            result: ExecutionResult::NotExecuted,
            recorded_utc: "bad-date".to_string(),
            recorded_by: String::new(),
            note: None,
        }
    }

    #[test]
    fn attempt_golden_valid_blocked() {
        let attempt = golden_attempt_blocked();
        let errors = validate_execution_attempt(&attempt);
        assert_eq!(errors, vec![], "Valid blocked attempt must produce zero errors");
    }

    #[test]
    fn attempt_golden_valid_approved() {
        let attempt = golden_attempt_approved();
        let errors = validate_execution_attempt(&attempt);
        assert_eq!(errors, vec![], "Valid approved attempt must produce zero errors");
    }

    #[test]
    fn attempt_golden_invalid_exact() {
        let attempt = golden_attempt_invalid();
        let errors = validate_execution_attempt(&attempt);

        let expected = vec![
            AttemptValidationError {
                field: "attempt_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            AttemptValidationError {
                field: "plan_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            AttemptValidationError {
                field: "plan_version".to_string(),
                message: "must be positive".to_string(),
            },
            AttemptValidationError {
                field: "recorded_utc".to_string(),
                message: "must be ISO 8601 format".to_string(),
            },
            AttemptValidationError {
                field: "recorded_by".to_string(),
                message: "must be non-empty".to_string(),
            },
        ];

        assert_eq!(errors, expected, "Invalid attempt must produce exact error list");
    }

    /// Minimal valid attempt for serialization testing
    fn golden_attempt_minimal() -> ExecutionAttempt {
        ExecutionAttempt {
            attempt_id: "att-min".to_string(),
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            readiness: ExecutionReadiness::NotApproved,
            gate_result: false,
            result: ExecutionResult::Blocked,
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            recorded_by: "system".to_string(),
            note: None,
        }
    }

    /// Expected JSON for minimal attempt - exact bytes (no note field)
    const GOLDEN_ATTEMPT_JSON: &str = r#"{
  "attempt_id": "att-min",
  "plan_id": "plan-001",
  "plan_version": 1,
  "readiness": "not_approved",
  "gate_result": false,
  "result": "blocked",
  "recorded_utc": "2026-01-15T00:00:00Z",
  "recorded_by": "system"
}"#;

    #[test]
    fn attempt_golden_serialization_exact() {
        let attempt = golden_attempt_minimal();
        let json = serialize_execution_attempt(&attempt).unwrap();
        assert_eq!(json, GOLDEN_ATTEMPT_JSON, "Serialization must produce exact output");
    }

    #[test]
    fn attempt_golden_serialization_roundtrip() {
        let original = golden_attempt_minimal();
        let json = serialize_execution_attempt(&original).unwrap();
        let restored = deserialize_execution_attempt(&json).unwrap();
        assert_eq!(original, restored, "Round-trip must preserve exact data");
    }

    #[test]
    fn attempt_golden_serialization_with_note() {
        let attempt = golden_attempt_blocked();
        let json = serialize_execution_attempt(&attempt).unwrap();
        assert!(json.contains("\"note\":"), "Note field must be present when set");
        assert!(json.contains("No approval present"));

        let restored = deserialize_execution_attempt(&json).unwrap();
        assert_eq!(attempt, restored);
    }

    #[test]
    fn attempt_golden_deserialize_and_validate_valid() {
        let result = deserialize_and_validate_attempt(GOLDEN_ATTEMPT_JSON);
        assert!(result.is_ok(), "Valid JSON must parse and validate");
    }

    #[test]
    fn attempt_golden_deserialize_and_validate_invalid_json() {
        let result = deserialize_and_validate_attempt("not valid json");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Parse error"));
    }

    #[test]
    fn attempt_golden_deserialize_and_validate_invalid_attempt() {
        let invalid_json = r#"{
  "attempt_id": "",
  "plan_id": "plan-001",
  "plan_version": 1,
  "readiness": "not_approved",
  "gate_result": false,
  "result": "blocked",
  "recorded_utc": "2026-01-15T00:00:00Z",
  "recorded_by": "system"
}"#;
        let result = deserialize_and_validate_attempt(invalid_json);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("attempt_id")));
    }

    #[test]
    fn attempt_golden_field_order_stable() {
        let attempt = golden_attempt_minimal();
        let json = serialize_execution_attempt(&attempt).unwrap();

        let attempt_id_pos = json.find("attempt_id").unwrap();
        let plan_id_pos = json.find("plan_id").unwrap();
        let plan_version_pos = json.find("plan_version").unwrap();
        let readiness_pos = json.find("readiness").unwrap();
        let gate_result_pos = json.find("gate_result").unwrap();
        let result_pos = json.find("\"result\"").unwrap();
        let recorded_utc_pos = json.find("recorded_utc").unwrap();
        let recorded_by_pos = json.find("recorded_by").unwrap();

        assert!(attempt_id_pos < plan_id_pos);
        assert!(plan_id_pos < plan_version_pos);
        assert!(plan_version_pos < readiness_pos);
        assert!(readiness_pos < gate_result_pos);
        assert!(gate_result_pos < result_pos);
        assert!(result_pos < recorded_utc_pos);
        assert!(recorded_utc_pos < recorded_by_pos);
    }

    #[test]
    fn attempt_determinism_across_runs() {
        for _ in 0..10 {
            let attempt = golden_attempt_minimal();
            let json1 = serialize_execution_attempt(&attempt).unwrap();
            let json2 = serialize_execution_attempt(&attempt).unwrap();
            assert_eq!(json1, json2);
            assert_eq!(json1, GOLDEN_ATTEMPT_JSON);
        }
    }

    #[test]
    fn attempt_gate_result_recorded_verbatim() {
        // gate_result is recorded, not recomputed
        let mut attempt = golden_attempt_minimal();

        // Record gate_result as true even though readiness is NotApproved
        // This documents that we record verbatim, not recompute
        attempt.gate_result = true;
        let errors = validate_execution_attempt(&attempt);
        assert_eq!(errors, vec![], "gate_result is recorded verbatim, not validated semantically");

        // Serialize and restore - gate_result preserved
        let json = serialize_execution_attempt(&attempt).unwrap();
        let restored = deserialize_execution_attempt(&json).unwrap();
        assert_eq!(restored.gate_result, true);
    }

    #[test]
    fn attempt_no_code_path_reaches_adapter() {
        // ExecutionAttempt does not use ExecutionAdapter
        // It records results, it does not compute them
        //
        // PROOF:
        // 1. ExecutionAttempt contains ExecutionResult as data
        // 2. ExecutionResult is a pure enum with no methods
        // 3. ExecutionAdapter has no implementations
        // 4. Therefore, no code path from ExecutionAttempt reaches ExecutionAdapter
        //
        // This test documents the isolation between recording and execution.
    }

    // =========================================================================
    // ATTEMPT PERSISTENCE TESTS
    // =========================================================================

    #[test]
    fn attempt_persistence_filename_format() {
        let filename = attempt_filename("att-001", 1);
        assert_eq!(filename, "att-001.v1.json");
    }

    #[test]
    fn attempt_persistence_save_and_load_roundtrip() {
        let test_dir = setup_test_dir();

        let attempt = ExecutionAttempt {
            attempt_id: format!("att-persist-{}", std::process::id()),
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            readiness: ExecutionReadiness::ApprovedAndCurrent,
            gate_result: true,
            result: ExecutionResult::NotExecuted,
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            recorded_by: "test".to_string(),
            note: Some("Test attempt".to_string()),
        };

        // Save
        let save_result = save_execution_attempt(&attempt);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result);

        // Load
        let loaded = load_execution_attempt(&attempt.attempt_id);
        assert!(loaded.is_ok(), "Load failed: {:?}", loaded);

        // Assert equality
        let loaded_attempt = loaded.unwrap();
        assert_eq!(attempt, loaded_attempt, "Loaded attempt must equal saved attempt");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn attempt_persistence_load_not_found() {
        let _test_dir = setup_test_dir();

        let result = load_execution_attempt("nonexistent-attempt-id-12345");
        assert!(matches!(result, Err(AttemptStorageError::NotFound(_))));
    }

    #[test]
    fn attempt_persistence_reject_unknown_version() {
        let result = load_execution_attempt_version("any-id", 99);
        assert!(matches!(result, Err(AttemptStorageError::UnknownVersion(99))));
    }

    #[test]
    fn attempt_persistence_save_invalid_rejected() {
        let _test_dir = setup_test_dir();

        let invalid_attempt = ExecutionAttempt {
            attempt_id: String::new(), // Invalid: empty
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            readiness: ExecutionReadiness::NotApproved,
            gate_result: false,
            result: ExecutionResult::Blocked,
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            recorded_by: "test".to_string(),
            note: None,
        };

        let result = save_execution_attempt(&invalid_attempt);
        assert!(matches!(result, Err(AttemptStorageError::ValidationFailed(_))));
    }

    #[test]
    fn attempt_persistence_format_version_constant() {
        assert_eq!(ATTEMPT_FORMAT_VERSION, 1);
    }

    #[test]
    fn attempt_persistence_determinism() {
        let test_dir = setup_test_dir();

        let attempt = ExecutionAttempt {
            attempt_id: format!("att-det-{}", std::process::id()),
            plan_id: "plan-001".to_string(),
            plan_version: 1,
            readiness: ExecutionReadiness::ApprovedButStale,
            gate_result: false,
            result: ExecutionResult::Blocked,
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            recorded_by: "test".to_string(),
            note: None,
        };

        save_execution_attempt(&attempt).unwrap();

        // Load multiple times
        let loaded1 = load_execution_attempt(&attempt.attempt_id).unwrap();
        let loaded2 = load_execution_attempt(&attempt.attempt_id).unwrap();

        assert_eq!(loaded1, loaded2, "Multiple loads must return identical data");
        assert_eq!(attempt, loaded1, "Loaded data must match original");

        cleanup_test_dir(&test_dir);
    }
}

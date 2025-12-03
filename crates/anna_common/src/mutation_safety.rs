//! Mutation Safety System v0.0.16
//!
//! Provides senior-engineer-level safety for mutations:
//! - Preflight checks before any mutation
//! - Dry-run diff previews for file edits
//! - Post-change verification checks
//! - Automatic rollback on failure
//!
//! Mutation execution state machine:
//! planned -> preflight_ok -> confirmed -> applied -> verified_ok | rolled_back

use crate::mutation_tools::{
    MutationError, MutationPlan, MutationRequest, MutationResult,
    MutationRisk, FileEditOp, RollbackInfo, MEDIUM_RISK_CONFIRMATION,
};
use crate::policy::{get_policy, PolicyCheckResult};
use crate::rollback::RollbackManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum file size for preflight (from policy, default 1 MiB)
const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Maximum diff lines to show in preview
const MAX_DIFF_PREVIEW_LINES: usize = 20;

// =============================================================================
// Mutation State Machine
// =============================================================================

/// Mutation execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationState {
    /// Plan created, not yet checked
    Planned,
    /// Preflight checks passed
    PreflightOk,
    /// User confirmed execution
    Confirmed,
    /// Mutation applied to system
    Applied,
    /// Post-checks verified success
    VerifiedOk,
    /// Post-checks failed, rolled back
    RolledBack,
    /// Preflight or execution failed
    Failed,
}

impl std::fmt::Display for MutationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MutationState::Planned => write!(f, "planned"),
            MutationState::PreflightOk => write!(f, "preflight_ok"),
            MutationState::Confirmed => write!(f, "confirmed"),
            MutationState::Applied => write!(f, "applied"),
            MutationState::VerifiedOk => write!(f, "verified_ok"),
            MutationState::RolledBack => write!(f, "rolled_back"),
            MutationState::Failed => write!(f, "failed"),
        }
    }
}

// =============================================================================
// Preflight Check Results
// =============================================================================

/// Individual preflight check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub evidence_id: Option<String>,
}

impl PreflightCheck {
    pub fn pass(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            message: message.to_string(),
            evidence_id: None,
        }
    }

    pub fn pass_with_evidence(name: &str, message: &str, evidence_id: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            message: message.to_string(),
            evidence_id: Some(evidence_id.to_string()),
        }
    }

    pub fn fail(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            message: message.to_string(),
            evidence_id: None,
        }
    }

    pub fn fail_with_evidence(name: &str, message: &str, evidence_id: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            message: message.to_string(),
            evidence_id: Some(evidence_id.to_string()),
        }
    }
}

/// Complete preflight result for a mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub tool_name: String,
    pub checks: Vec<PreflightCheck>,
    pub all_passed: bool,
    pub file_hash: Option<String>,
    pub file_mtime: Option<u64>,
    pub backup_path: Option<PathBuf>,
    pub prior_state: Option<String>,
}

impl PreflightResult {
    pub fn new(tool_name: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            checks: Vec::new(),
            all_passed: true,
            file_hash: None,
            file_mtime: None,
            backup_path: None,
            prior_state: None,
        }
    }

    pub fn add_check(&mut self, check: PreflightCheck) {
        if !check.passed {
            self.all_passed = false;
        }
        self.checks.push(check);
    }

    /// Format preflight results as human-readable text
    pub fn format_human(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Preflight checks for {}:\n", self.tool_name));

        for check in &self.checks {
            let status = if check.passed { "[OK]" } else { "[FAIL]" };
            let evidence = check.evidence_id.as_ref()
                .map(|id| format!(" [{}]", id))
                .unwrap_or_default();
            output.push_str(&format!("  {} {}: {}{}\n", status, check.name, check.message, evidence));
        }

        if let Some(hash) = &self.file_hash {
            output.push_str(&format!("  File hash: {}\n", &hash[..16]));
        }

        if let Some(path) = &self.backup_path {
            output.push_str(&format!("  Backup: {}\n", path.display()));
        }

        output
    }
}

// =============================================================================
// Dry-Run Diff Preview
// =============================================================================

/// Line-level diff entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLine {
    /// Line unchanged (context)
    Context { line_num: usize, content: String },
    /// Line added
    Added { line_num: usize, content: String },
    /// Line removed
    Removed { line_num: usize, content: String },
    /// Line modified
    Modified { line_num: usize, old: String, new: String },
}

/// Diff preview for a file edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPreview {
    pub path: PathBuf,
    pub lines: Vec<DiffLine>,
    pub additions: usize,
    pub deletions: usize,
    pub modifications: usize,
    pub truncated: bool,
    pub backup_path: PathBuf,
    pub rollback_command: String,
}

impl DiffPreview {
    /// Format diff preview as human-readable text (minimal excerpt)
    pub fn format_human(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Changes to {}:\n", self.path.display()));
        output.push_str(&format!("  +{} added, -{} removed, ~{} modified\n\n",
            self.additions, self.deletions, self.modifications));

        // Show diff excerpt
        for line in self.lines.iter().take(MAX_DIFF_PREVIEW_LINES) {
            match line {
                DiffLine::Context { line_num, content } => {
                    output.push_str(&format!("  {:4} | {}\n", line_num, content));
                }
                DiffLine::Added { line_num, content } => {
                    output.push_str(&format!("+ {:4} | {}\n", line_num, content));
                }
                DiffLine::Removed { line_num, content } => {
                    output.push_str(&format!("- {:4} | {}\n", line_num, content));
                }
                DiffLine::Modified { line_num, old, new } => {
                    output.push_str(&format!("- {:4} | {}\n", line_num, old));
                    output.push_str(&format!("+ {:4} | {}\n", line_num, new));
                }
            }
        }

        if self.truncated {
            output.push_str(&format!("  ... ({} more changes)\n",
                self.lines.len() - MAX_DIFF_PREVIEW_LINES));
        }

        output.push_str(&format!("\nBackup: {}\n", self.backup_path.display()));
        output.push_str(&format!("Rollback: {}\n", self.rollback_command));

        output
    }
}

// =============================================================================
// Post-Check Verification
// =============================================================================

/// Post-check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCheck {
    pub name: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
    pub evidence_id: Option<String>,
}

impl PostCheck {
    pub fn pass(name: &str, expected: &str, actual: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            expected: expected.to_string(),
            actual: actual.to_string(),
            evidence_id: None,
        }
    }

    pub fn fail(name: &str, expected: &str, actual: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            expected: expected.to_string(),
            actual: actual.to_string(),
            evidence_id: None,
        }
    }
}

/// Complete post-check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCheckResult {
    pub tool_name: String,
    pub checks: Vec<PostCheck>,
    pub all_passed: bool,
    pub post_state: Option<String>,
    pub evidence_id: Option<String>,
}

impl PostCheckResult {
    pub fn new(tool_name: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            checks: Vec::new(),
            all_passed: true,
            post_state: None,
            evidence_id: None,
        }
    }

    pub fn add_check(&mut self, check: PostCheck) {
        if !check.passed {
            self.all_passed = false;
        }
        self.checks.push(check);
    }

    pub fn format_human(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Post-checks for {}:\n", self.tool_name));

        for check in &self.checks {
            let status = if check.passed { "[OK]" } else { "[FAIL]" };
            output.push_str(&format!("  {} {}: expected '{}', got '{}'\n",
                status, check.name, check.expected, check.actual));
        }

        output
    }
}

// =============================================================================
// Rollback Result
// =============================================================================

/// Rollback execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub message: String,
    pub evidence_id: String,
    pub restored_state: Option<String>,
}

// =============================================================================
// Safe Mutation Executor
// =============================================================================

/// Safe mutation executor with preflight, dry-run, and post-checks
pub struct SafeMutationExecutor {
    rollback_manager: RollbackManager,
    evidence_counter: u32,
}

impl SafeMutationExecutor {
    pub fn new(rollback_manager: RollbackManager) -> Self {
        Self {
            rollback_manager,
            evidence_counter: 0,
        }
    }

    /// Generate unique evidence ID
    fn next_evidence_id(&mut self, prefix: &str) -> String {
        self.evidence_counter += 1;
        format!("{}{:05}", prefix, self.evidence_counter)
    }

    // =========================================================================
    // Preflight Checks
    // =========================================================================

    /// Run preflight checks for a file edit
    pub fn preflight_file_edit(
        &mut self,
        path: &Path,
        operations: &[FileEditOp],
    ) -> PreflightResult {
        let mut result = PreflightResult::new("edit_file_lines");

        // 1. Check path allowed by policy
        let policy_check = check_path_policy(path);
        if policy_check.allowed {
            result.add_check(PreflightCheck::pass_with_evidence(
                "path_allowed",
                &format!("Path {} is allowed", path.display()),
                &policy_check.evidence_id,
            ));
        } else {
            result.add_check(PreflightCheck::fail_with_evidence(
                "path_allowed",
                &format!("Path blocked: {}", policy_check.reason),
                &policy_check.evidence_id,
            ));
            return result;
        }

        // 2. Check file exists (or parent exists for creation)
        if path.exists() {
            result.add_check(PreflightCheck::pass(
                "file_exists",
                &format!("File {} exists", path.display()),
            ));
        } else {
            // Check if parent directory exists
            if let Some(parent) = path.parent() {
                if parent.exists() && parent.is_dir() {
                    result.add_check(PreflightCheck::pass(
                        "file_creatable",
                        &format!("Parent directory {} exists, file can be created", parent.display()),
                    ));
                } else {
                    result.add_check(PreflightCheck::fail(
                        "file_exists",
                        &format!("Parent directory {} does not exist", parent.display()),
                    ));
                    return result;
                }
            }
        }

        // 3. Check file is text (not binary) if it exists
        if path.exists() {
            match fs::read(path) {
                Ok(content) => {
                    // Check for binary content (null bytes)
                    if content.contains(&0u8) {
                        result.add_check(PreflightCheck::fail(
                            "is_text_file",
                            "File contains binary content",
                        ));
                        return result;
                    }
                    result.add_check(PreflightCheck::pass(
                        "is_text_file",
                        "File is text (no binary content)",
                    ));
                }
                Err(e) => {
                    result.add_check(PreflightCheck::fail(
                        "is_text_file",
                        &format!("Cannot read file: {}", e),
                    ));
                    return result;
                }
            }
        }

        // 4. Check file size under policy limit
        if path.exists() {
            match fs::metadata(path) {
                Ok(meta) => {
                    let size = meta.len();
                    let max_size = get_max_file_size();
                    if size <= max_size {
                        result.add_check(PreflightCheck::pass(
                            "file_size",
                            &format!("File size {} bytes (limit: {} bytes)", size, max_size),
                        ));
                        result.file_mtime = Some(meta.mtime() as u64);
                    } else {
                        result.add_check(PreflightCheck::fail(
                            "file_size",
                            &format!("File too large: {} bytes (limit: {} bytes)", size, max_size),
                        ));
                        return result;
                    }
                }
                Err(e) => {
                    result.add_check(PreflightCheck::fail(
                        "file_size",
                        &format!("Cannot stat file: {}", e),
                    ));
                    return result;
                }
            }
        }

        // 5. Check permissions suitable
        if path.exists() {
            match fs::OpenOptions::new().read(true).write(true).open(path) {
                Ok(_) => {
                    result.add_check(PreflightCheck::pass(
                        "permissions",
                        "File is readable and writable",
                    ));
                }
                Err(e) => {
                    result.add_check(PreflightCheck::fail(
                        "permissions",
                        &format!("Insufficient permissions: {}", e),
                    ));
                    return result;
                }
            }
        }

        // 6. Record current file hash and mtime
        if path.exists() {
            match RollbackManager::hash_file(path) {
                Ok(hash) => {
                    result.file_hash = Some(hash.clone());
                    result.add_check(PreflightCheck::pass(
                        "hash_recorded",
                        &format!("Current hash: {}...", &hash[..16]),
                    ));
                }
                Err(e) => {
                    result.add_check(PreflightCheck::fail(
                        "hash_recorded",
                        &format!("Cannot hash file: {}", e),
                    ));
                    return result;
                }
            }
        }

        // 7. Check backup destination available
        let backup_dir = self.rollback_manager.files_dir();
        if backup_dir.exists() && backup_dir.is_dir() {
            // Check writable
            let test_file = backup_dir.join(".write_test");
            match fs::write(&test_file, "test") {
                Ok(_) => {
                    let _ = fs::remove_file(&test_file);
                    result.add_check(PreflightCheck::pass(
                        "backup_available",
                        &format!("Backup directory {} is writable", backup_dir.display()),
                    ));

                    // Generate backup path
                    let request_id = generate_request_id();
                    result.backup_path = Some(backup_dir.join(format!(
                        "{}_{}.bak",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        request_id
                    )));
                }
                Err(e) => {
                    result.add_check(PreflightCheck::fail(
                        "backup_available",
                        &format!("Backup directory not writable: {}", e),
                    ));
                    return result;
                }
            }
        } else {
            // Try to create backup directory
            match fs::create_dir_all(&backup_dir) {
                Ok(_) => {
                    result.add_check(PreflightCheck::pass(
                        "backup_available",
                        &format!("Created backup directory {}", backup_dir.display()),
                    ));

                    let request_id = generate_request_id();
                    result.backup_path = Some(backup_dir.join(format!(
                        "{}_{}.bak",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        request_id
                    )));
                }
                Err(e) => {
                    result.add_check(PreflightCheck::fail(
                        "backup_available",
                        &format!("Cannot create backup directory: {}", e),
                    ));
                    return result;
                }
            }
        }

        // 8. Validate operations
        let valid_ops = operations.iter().all(|op| match op {
            FileEditOp::InsertLine { line_number, .. } => *line_number < 100000,
            FileEditOp::ReplaceLine { line_number, .. } => *line_number < 100000,
            FileEditOp::DeleteLine { line_number } => *line_number < 100000,
            FileEditOp::AppendLine { .. } => true,
            FileEditOp::ReplaceText { pattern, .. } => !pattern.is_empty(),
        });

        if valid_ops {
            result.add_check(PreflightCheck::pass(
                "operations_valid",
                &format!("{} edit operation(s) validated", operations.len()),
            ));
        } else {
            result.add_check(PreflightCheck::fail(
                "operations_valid",
                "Invalid edit operations",
            ));
        }

        result
    }

    /// Run preflight checks for systemd operation
    pub fn preflight_systemd(
        &mut self,
        service: &str,
        operation: &str,
    ) -> PreflightResult {
        let mut result = PreflightResult::new(&format!("systemd_{}", operation));

        // 1. Check unit exists
        let status = Command::new("systemctl")
            .args(["status", service])
            .output();

        match status {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("could not be found") || stdout.contains("not-found") {
                    result.add_check(PreflightCheck::fail(
                        "unit_exists",
                        &format!("Service {} not found", service),
                    ));
                    return result;
                }
                result.add_check(PreflightCheck::pass(
                    "unit_exists",
                    &format!("Service {} exists", service),
                ));
            }
            Err(e) => {
                result.add_check(PreflightCheck::fail(
                    "unit_exists",
                    &format!("Cannot check service: {}", e),
                ));
                return result;
            }
        }

        // 2. Capture current state
        let is_active = Command::new("systemctl")
            .args(["is-active", service])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let is_enabled = Command::new("systemctl")
            .args(["is-enabled", service])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled")
            .unwrap_or(false);

        let state = format!("active={}, enabled={}", is_active, is_enabled);
        result.prior_state = Some(state.clone());
        result.add_check(PreflightCheck::pass(
            "state_captured",
            &format!("Current state: {}", state),
        ));

        // 3. Check operation allowed by policy
        let policy_check = check_systemd_policy(service, operation);
        if policy_check.allowed {
            result.add_check(PreflightCheck::pass_with_evidence(
                "operation_allowed",
                &format!("Operation {} on {} is allowed", operation, service),
                &policy_check.evidence_id,
            ));
        } else {
            result.add_check(PreflightCheck::fail_with_evidence(
                "operation_allowed",
                &format!("Operation blocked: {}", policy_check.reason),
                &policy_check.evidence_id,
            ));
        }

        result
    }

    /// Run preflight checks for package operation
    pub fn preflight_package(
        &mut self,
        packages: &[&str],
        operation: &str, // "install" or "remove"
    ) -> PreflightResult {
        let mut result = PreflightResult::new(&format!("package_{}", operation));

        // 1. Check distro is Arch
        if !is_arch_linux() {
            result.add_check(PreflightCheck::fail(
                "distro_check",
                "Package operations only supported on Arch Linux",
            ));
            return result;
        }
        result.add_check(PreflightCheck::pass(
            "distro_check",
            "Running on Arch Linux",
        ));

        // 2. Check packages exist and not blocked
        for pkg in packages {
            let policy_check = check_package_policy(pkg, operation);
            if !policy_check.allowed {
                result.add_check(PreflightCheck::fail_with_evidence(
                    "package_allowed",
                    &format!("Package {} blocked: {}", pkg, policy_check.reason),
                    &policy_check.evidence_id,
                ));
                return result;
            }
            result.add_check(PreflightCheck::pass_with_evidence(
                "package_allowed",
                &format!("Package {} is allowed for {}", pkg, operation),
                &policy_check.evidence_id,
            ));
        }

        // 3. Check transaction size (simulate)
        if operation == "install" {
            // Check if packages exist in repos
            for pkg in packages {
                let exists = Command::new("pacman")
                    .args(["-Si", pkg])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !exists {
                    result.add_check(PreflightCheck::fail(
                        "package_exists",
                        &format!("Package {} not found in repositories", pkg),
                    ));
                    return result;
                }
            }
            result.add_check(PreflightCheck::pass(
                "packages_available",
                &format!("{} package(s) found in repositories", packages.len()),
            ));
        }

        // 4. Disk free check
        let disk_free = get_disk_free_bytes("/");
        let min_free = 500 * 1024 * 1024; // 500 MB minimum
        if disk_free >= min_free {
            result.add_check(PreflightCheck::pass(
                "disk_space",
                &format!("{} MB free (minimum: {} MB)", disk_free / 1024 / 1024, min_free / 1024 / 1024),
            ));
        } else {
            result.add_check(PreflightCheck::fail(
                "disk_space",
                &format!("Insufficient disk space: {} MB (need {} MB)",
                    disk_free / 1024 / 1024, min_free / 1024 / 1024),
            ));
        }

        result
    }

    // =========================================================================
    // Dry-Run Diff Preview
    // =========================================================================

    /// Generate diff preview for file edit (dry-run)
    pub fn dry_run_file_edit(
        &self,
        path: &Path,
        operations: &[FileEditOp],
        backup_path: &Path,
    ) -> Result<DiffPreview, MutationError> {
        // Read current content
        let content = if path.exists() {
            fs::read_to_string(path)
                .map_err(|e| MutationError::Other(format!("Cannot read file: {}", e)))?
        } else {
            String::new()
        };

        let original_lines: Vec<String> = content.lines().map(String::from).collect();
        let mut new_lines = original_lines.clone();

        // Apply operations to get new content
        for op in operations {
            apply_edit_op(&mut new_lines, op)?;
        }

        // Generate diff
        let diff_lines = generate_diff(&original_lines, &new_lines);

        let additions = diff_lines.iter().filter(|l| matches!(l, DiffLine::Added { .. })).count();
        let deletions = diff_lines.iter().filter(|l| matches!(l, DiffLine::Removed { .. })).count();
        let modifications = diff_lines.iter().filter(|l| matches!(l, DiffLine::Modified { .. })).count();

        let truncated = diff_lines.len() > MAX_DIFF_PREVIEW_LINES;

        let rollback_command = format!(
            "cp {} {}",
            backup_path.display(),
            path.display()
        );

        Ok(DiffPreview {
            path: path.to_path_buf(),
            lines: diff_lines,
            additions,
            deletions,
            modifications,
            truncated,
            backup_path: backup_path.to_path_buf(),
            rollback_command,
        })
    }

    // =========================================================================
    // Post-Check Verification
    // =========================================================================

    /// Run post-checks for file edit
    pub fn postcheck_file_edit(
        &mut self,
        path: &Path,
        expected_content_patterns: &[&str],
        expected_hash: Option<&str>,
    ) -> PostCheckResult {
        let mut result = PostCheckResult::new("edit_file_lines");
        let evidence_id = self.next_evidence_id("POST");
        result.evidence_id = Some(evidence_id.clone());

        // 1. Check file exists
        if !path.exists() {
            result.add_check(PostCheck::fail(
                "file_exists",
                "file should exist",
                "file not found",
            ));
            return result;
        }
        result.add_check(PostCheck::pass(
            "file_exists",
            "exists",
            "exists",
        ));

        // 2. Check file readable
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                result.add_check(PostCheck::fail(
                    "file_readable",
                    "readable",
                    &format!("error: {}", e),
                ));
                return result;
            }
        };
        result.add_check(PostCheck::pass(
            "file_readable",
            "readable",
            "readable",
        ));
        result.post_state = Some(format!("{} bytes", content.len()));

        // 3. Check expected content patterns
        for pattern in expected_content_patterns {
            if content.contains(pattern) {
                result.add_check(PostCheck::pass(
                    "content_check",
                    &format!("contains '{}'", pattern),
                    "found",
                ));
            } else {
                result.add_check(PostCheck::fail(
                    "content_check",
                    &format!("contains '{}'", pattern),
                    "not found",
                ));
            }
        }

        // 4. Verify hash changed (if provided)
        if let Some(old_hash) = expected_hash {
            match RollbackManager::hash_file(path) {
                Ok(new_hash) => {
                    if new_hash != old_hash {
                        result.add_check(PostCheck::pass(
                            "hash_changed",
                            "file modified",
                            &format!("hash changed to {}...", &new_hash[..16]),
                        ));
                    } else {
                        result.add_check(PostCheck::fail(
                            "hash_changed",
                            "file modified",
                            "hash unchanged",
                        ));
                    }
                }
                Err(e) => {
                    result.add_check(PostCheck::fail(
                        "hash_changed",
                        "hash computed",
                        &format!("error: {}", e),
                    ));
                }
            }
        }

        result
    }

    /// Run post-checks for systemd operation
    pub fn postcheck_systemd(
        &mut self,
        service: &str,
        operation: &str,
    ) -> PostCheckResult {
        let mut result = PostCheckResult::new(&format!("systemd_{}", operation));
        let evidence_id = self.next_evidence_id("POST");
        result.evidence_id = Some(evidence_id.clone());

        // Check service state after operation
        let is_active = Command::new("systemctl")
            .args(["is-active", service])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let is_enabled = Command::new("systemctl")
            .args(["is-enabled", service])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled")
            .unwrap_or(false);

        result.post_state = Some(format!("active={}, enabled={}", is_active, is_enabled));

        match operation {
            "restart" | "reload" => {
                if is_active {
                    result.add_check(PostCheck::pass(
                        "service_active",
                        "active",
                        "active",
                    ));

                    // Check for immediate failure (service crashed right after restart)
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let still_active = Command::new("systemctl")
                        .args(["is-active", service])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);

                    if still_active {
                        result.add_check(PostCheck::pass(
                            "no_immediate_failure",
                            "stable after 500ms",
                            "still running",
                        ));
                    } else {
                        result.add_check(PostCheck::fail(
                            "no_immediate_failure",
                            "stable after 500ms",
                            "service died",
                        ));
                    }
                } else {
                    result.add_check(PostCheck::fail(
                        "service_active",
                        "active",
                        "inactive",
                    ));
                }
            }
            "enable_now" => {
                result.add_check(if is_enabled {
                    PostCheck::pass("service_enabled", "enabled", "enabled")
                } else {
                    PostCheck::fail("service_enabled", "enabled", "disabled")
                });
                result.add_check(if is_active {
                    PostCheck::pass("service_active", "active", "active")
                } else {
                    PostCheck::fail("service_active", "active", "inactive")
                });
            }
            "disable_now" => {
                result.add_check(if !is_enabled {
                    PostCheck::pass("service_disabled", "disabled", "disabled")
                } else {
                    PostCheck::fail("service_disabled", "disabled", "still enabled")
                });
                result.add_check(if !is_active {
                    PostCheck::pass("service_inactive", "inactive", "inactive")
                } else {
                    PostCheck::fail("service_inactive", "inactive", "still active")
                });
            }
            _ => {}
        }

        result
    }

    /// Run post-checks for package operation
    pub fn postcheck_package(
        &mut self,
        packages: &[&str],
        operation: &str,
    ) -> PostCheckResult {
        let mut result = PostCheckResult::new(&format!("package_{}", operation));
        let evidence_id = self.next_evidence_id("POST");
        result.evidence_id = Some(evidence_id.clone());

        for pkg in packages {
            let is_installed = Command::new("pacman")
                .args(["-Q", pkg])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            match operation {
                "install" => {
                    if is_installed {
                        // Get version
                        let version = Command::new("pacman")
                            .args(["-Q", pkg])
                            .output()
                            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                            .unwrap_or_else(|_| "unknown".to_string());

                        result.add_check(PostCheck::pass(
                            &format!("{}_installed", pkg),
                            "installed",
                            &version,
                        ));
                    } else {
                        result.add_check(PostCheck::fail(
                            &format!("{}_installed", pkg),
                            "installed",
                            "not installed",
                        ));
                    }
                }
                "remove" => {
                    if !is_installed {
                        result.add_check(PostCheck::pass(
                            &format!("{}_removed", pkg),
                            "removed",
                            "not present",
                        ));
                    } else {
                        result.add_check(PostCheck::fail(
                            &format!("{}_removed", pkg),
                            "removed",
                            "still installed",
                        ));
                    }
                }
                _ => {}
            }
        }

        result
    }

    // =========================================================================
    // Automatic Rollback
    // =========================================================================

    /// Execute rollback for a file edit
    pub fn rollback_file_edit(
        &mut self,
        path: &Path,
        backup_path: &Path,
        reason: &str,
    ) -> RollbackResult {
        let evidence_id = self.next_evidence_id("RB");

        if !backup_path.exists() {
            return RollbackResult {
                success: false,
                message: format!("Backup file not found: {}", backup_path.display()),
                evidence_id,
                restored_state: None,
            };
        }

        match fs::copy(backup_path, path) {
            Ok(_) => {
                let restored_hash = RollbackManager::hash_file(path)
                    .map(|h| format!("{}...", &h[..16]))
                    .unwrap_or_else(|_| "unknown".to_string());

                RollbackResult {
                    success: true,
                    message: format!(
                        "Rolled back {} to backup {} (reason: {})",
                        path.display(),
                        backup_path.display(),
                        reason
                    ),
                    evidence_id,
                    restored_state: Some(restored_hash),
                }
            }
            Err(e) => {
                RollbackResult {
                    success: false,
                    message: format!("Rollback failed: {}", e),
                    evidence_id,
                    restored_state: None,
                }
            }
        }
    }

    /// Execute rollback for systemd operation
    pub fn rollback_systemd(
        &mut self,
        service: &str,
        prior_state: &str,
        reason: &str,
    ) -> RollbackResult {
        let evidence_id = self.next_evidence_id("RB");

        // Parse prior state
        let was_active = prior_state.contains("active=true");
        let was_enabled = prior_state.contains("enabled=true");

        let mut success = true;
        let mut messages = Vec::new();

        // Restore active state
        if was_active {
            let result = Command::new("systemctl")
                .args(["start", service])
                .output();
            if result.is_err() || !result.unwrap().status.success() {
                success = false;
                messages.push(format!("Failed to restart {}", service));
            }
        } else {
            let result = Command::new("systemctl")
                .args(["stop", service])
                .output();
            if result.is_err() || !result.unwrap().status.success() {
                success = false;
                messages.push(format!("Failed to stop {}", service));
            }
        }

        // Restore enabled state
        if was_enabled {
            let _ = Command::new("systemctl")
                .args(["enable", service])
                .output();
        } else {
            let _ = Command::new("systemctl")
                .args(["disable", service])
                .output();
        }

        RollbackResult {
            success,
            message: if success {
                format!("Rolled back {} to {} (reason: {})", service, prior_state, reason)
            } else {
                messages.join("; ")
            },
            evidence_id,
            restored_state: Some(prior_state.to_string()),
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn get_max_file_size() -> u64 {
    // Try to get from policy, fall back to default
    get_policy().get_max_file_size()
}

fn check_path_policy(path: &Path) -> PolicyCheckResult {
    get_policy().is_path_allowed(&path.to_string_lossy())
}

fn check_systemd_policy(service: &str, _operation: &str) -> PolicyCheckResult {
    get_policy().is_service_allowed(service)
}

fn check_package_policy(package: &str, _operation: &str) -> PolicyCheckResult {
    get_policy().is_package_allowed(package)
}

fn is_arch_linux() -> bool {
    Path::new("/etc/arch-release").exists()
}

fn get_disk_free_bytes(path: &str) -> u64 {
    Command::new("df")
        .args(["--output=avail", "-B1", path])
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .nth(1)
                .and_then(|l| l.trim().parse().ok())
        })
        .unwrap_or(0)
}

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("REQ{:013X}", ts)
}

/// Apply a single edit operation (for dry-run)
fn apply_edit_op(lines: &mut Vec<String>, op: &FileEditOp) -> Result<(), MutationError> {
    match op {
        FileEditOp::InsertLine { line_number, content } => {
            if *line_number > lines.len() {
                return Err(MutationError::Other(format!(
                    "Line {} out of range (file has {} lines)",
                    line_number, lines.len()
                )));
            }
            lines.insert(*line_number, content.clone());
        }
        FileEditOp::ReplaceLine { line_number, content } => {
            if *line_number >= lines.len() {
                return Err(MutationError::Other(format!(
                    "Line {} out of range",
                    line_number
                )));
            }
            lines[*line_number] = content.clone();
        }
        FileEditOp::DeleteLine { line_number } => {
            if *line_number >= lines.len() {
                return Err(MutationError::Other(format!(
                    "Line {} out of range",
                    line_number
                )));
            }
            lines.remove(*line_number);
        }
        FileEditOp::AppendLine { content } => {
            lines.push(content.clone());
        }
        FileEditOp::ReplaceText { pattern, replacement } => {
            for line in lines.iter_mut() {
                *line = line.replace(pattern, replacement);
            }
        }
    }
    Ok(())
}

/// Generate a simple line-based diff
fn generate_diff(original: &[String], modified: &[String]) -> Vec<DiffLine> {
    let mut diff = Vec::new();
    let max_len = original.len().max(modified.len());

    // Use simple line-by-line comparison
    let mut i = 0;
    let mut j = 0;

    while i < original.len() || j < modified.len() {
        if i >= original.len() {
            // Only additions left
            diff.push(DiffLine::Added {
                line_num: j + 1,
                content: modified[j].clone(),
            });
            j += 1;
        } else if j >= modified.len() {
            // Only deletions left
            diff.push(DiffLine::Removed {
                line_num: i + 1,
                content: original[i].clone(),
            });
            i += 1;
        } else if original[i] == modified[j] {
            // Lines match - context
            diff.push(DiffLine::Context {
                line_num: i + 1,
                content: original[i].clone(),
            });
            i += 1;
            j += 1;
        } else {
            // Lines differ - check if it's a modification or add/delete
            // Simple heuristic: if next original matches current modified, it's a deletion
            // If next modified matches current original, it's an addition
            let next_orig_matches = i + 1 < original.len() && original[i + 1] == modified[j];
            let next_mod_matches = j + 1 < modified.len() && original[i] == modified[j + 1];

            if next_orig_matches {
                diff.push(DiffLine::Removed {
                    line_num: i + 1,
                    content: original[i].clone(),
                });
                i += 1;
            } else if next_mod_matches {
                diff.push(DiffLine::Added {
                    line_num: j + 1,
                    content: modified[j].clone(),
                });
                j += 1;
            } else {
                // Treat as modification
                diff.push(DiffLine::Modified {
                    line_num: i + 1,
                    old: original[i].clone(),
                    new: modified[j].clone(),
                });
                i += 1;
                j += 1;
            }
        }
    }

    diff
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mutation_state_display() {
        assert_eq!(MutationState::Planned.to_string(), "planned");
        assert_eq!(MutationState::PreflightOk.to_string(), "preflight_ok");
        assert_eq!(MutationState::VerifiedOk.to_string(), "verified_ok");
        assert_eq!(MutationState::RolledBack.to_string(), "rolled_back");
    }

    #[test]
    fn test_preflight_check_pass() {
        let check = PreflightCheck::pass("test", "passed");
        assert!(check.passed);
        assert_eq!(check.name, "test");
        assert!(check.evidence_id.is_none());
    }

    #[test]
    fn test_preflight_check_fail_with_evidence() {
        let check = PreflightCheck::fail_with_evidence("test", "failed", "POL00001");
        assert!(!check.passed);
        assert_eq!(check.evidence_id, Some("POL00001".to_string()));
    }

    #[test]
    fn test_preflight_result_all_passed() {
        let mut result = PreflightResult::new("test");
        result.add_check(PreflightCheck::pass("a", "ok"));
        result.add_check(PreflightCheck::pass("b", "ok"));
        assert!(result.all_passed);
    }

    #[test]
    fn test_preflight_result_one_failed() {
        let mut result = PreflightResult::new("test");
        result.add_check(PreflightCheck::pass("a", "ok"));
        result.add_check(PreflightCheck::fail("b", "not ok"));
        assert!(!result.all_passed);
    }

    #[test]
    fn test_diff_line_types() {
        let context = DiffLine::Context { line_num: 1, content: "hello".to_string() };
        let added = DiffLine::Added { line_num: 2, content: "world".to_string() };
        let removed = DiffLine::Removed { line_num: 3, content: "old".to_string() };

        // Just check they can be created
        assert!(matches!(context, DiffLine::Context { .. }));
        assert!(matches!(added, DiffLine::Added { .. }));
        assert!(matches!(removed, DiffLine::Removed { .. }));
    }

    #[test]
    fn test_generate_diff_simple() {
        let original = vec!["line1".to_string(), "line2".to_string(), "line3".to_string()];
        let modified = vec!["line1".to_string(), "changed".to_string(), "line3".to_string()];

        let diff = generate_diff(&original, &modified);

        // Should have context, modified, context
        assert!(!diff.is_empty());
    }

    #[test]
    fn test_generate_diff_addition() {
        let original = vec!["line1".to_string()];
        let modified = vec!["line1".to_string(), "line2".to_string()];

        let diff = generate_diff(&original, &modified);

        // Should have context + added
        assert_eq!(diff.len(), 2);
    }

    #[test]
    fn test_generate_diff_deletion() {
        let original = vec!["line1".to_string(), "line2".to_string()];
        let modified = vec!["line1".to_string()];

        let diff = generate_diff(&original, &modified);

        // Should have context + removed
        assert_eq!(diff.len(), 2);
    }

    #[test]
    fn test_diff_preview_format() {
        let preview = DiffPreview {
            path: PathBuf::from("/tmp/test.conf"),
            lines: vec![
                DiffLine::Context { line_num: 1, content: "# comment".to_string() },
                DiffLine::Added { line_num: 2, content: "new_setting=true".to_string() },
            ],
            additions: 1,
            deletions: 0,
            modifications: 0,
            truncated: false,
            backup_path: PathBuf::from("/var/lib/anna/rollback/test.bak"),
            rollback_command: "cp /var/lib/anna/rollback/test.bak /tmp/test.conf".to_string(),
        };

        let formatted = preview.format_human();
        assert!(formatted.contains("/tmp/test.conf"));
        assert!(formatted.contains("+1 added"));
        assert!(formatted.contains("new_setting"));
    }

    #[test]
    fn test_post_check_pass() {
        let check = PostCheck::pass("test", "expected", "actual");
        assert!(check.passed);
    }

    #[test]
    fn test_post_check_fail() {
        let check = PostCheck::fail("test", "expected", "actual");
        assert!(!check.passed);
    }

    #[test]
    fn test_rollback_result() {
        let result = RollbackResult {
            success: true,
            message: "Rolled back successfully".to_string(),
            evidence_id: "RB00001".to_string(),
            restored_state: Some("original".to_string()),
        };
        assert!(result.success);
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();

        assert!(id1.starts_with("REQ"));
        assert!(id2.starts_with("REQ"));
        // IDs should be unique (or at least different due to time)
        // Note: might be same if called in same millisecond
    }

    #[test]
    fn test_apply_edit_op_append() {
        let mut lines = vec!["line1".to_string()];
        let op = FileEditOp::AppendLine { content: "line2".to_string() };

        apply_edit_op(&mut lines, &op).unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], "line2");
    }

    #[test]
    fn test_apply_edit_op_replace() {
        let mut lines = vec!["old".to_string()];
        let op = FileEditOp::ReplaceLine { line_number: 0, content: "new".to_string() };

        apply_edit_op(&mut lines, &op).unwrap();

        assert_eq!(lines[0], "new");
    }

    #[test]
    fn test_apply_edit_op_delete() {
        let mut lines = vec!["line1".to_string(), "line2".to_string()];
        let op = FileEditOp::DeleteLine { line_number: 0 };

        apply_edit_op(&mut lines, &op).unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "line2");
    }

    #[test]
    fn test_apply_edit_op_insert() {
        let mut lines = vec!["line1".to_string(), "line3".to_string()];
        let op = FileEditOp::InsertLine { line_number: 1, content: "line2".to_string() };

        apply_edit_op(&mut lines, &op).unwrap();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1], "line2");
    }

    #[test]
    fn test_apply_edit_op_replace_text() {
        let mut lines = vec!["hello world".to_string()];
        let op = FileEditOp::ReplaceText {
            pattern: "world".to_string(),
            replacement: "rust".to_string()
        };

        apply_edit_op(&mut lines, &op).unwrap();

        assert_eq!(lines[0], "hello rust");
    }

    #[test]
    fn test_is_arch_linux() {
        // This test result depends on the actual system
        let result = is_arch_linux();
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_get_disk_free_bytes() {
        let free = get_disk_free_bytes("/");
        // Should return some value (> 0 on most systems)
        // Just ensure it doesn't panic
        let _ = free;
    }
}

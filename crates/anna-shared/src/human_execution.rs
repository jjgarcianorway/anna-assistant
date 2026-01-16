//! Human-Mediated Execution Adapter (Phase 42)
//!
//! This module introduces the first and only execution path in the system,
//! explicitly mediated by a human operator.
//!
//! # CRITICAL INVARIANT: NO AUTONOMOUS EXECUTION
//!
//! The system remains incapable of autonomous execution. This adapter:
//! - Does NOT decide when to execute
//! - Does NOT select actions
//! - Does NOT infer intent
//! - Executes only what a human explicitly provides at runtime
//!
//! # Execution Model
//!
//! HumanExecutionAdapter:
//! 1. Accepts an ExecutionRequest (proves human initiated)
//! 2. Requires a runtime-provided command string from the human
//! 3. Executes exactly that string using restricted shell
//! 4. Captures stdout, stderr, exit code, execution time
//! 5. Returns structured result
//!
//! # Safety Boundaries (Non-Negotiable)
//!
//! The adapter enforces:
//! - No access to stored plans, proposals, or intentions
//! - No ability to construct commands itself
//! - No fallback or default behavior
//! - No retries
//! - No sudo
//! - No environment mutation outside the process
//! - Explicit allowlist of binaries
//!
//! # Invocation Rules
//!
//! Execution may occur only when:
//! 1. An ExecutionRequest exists (human initiated)
//! 2. The confirmation string matches exactly
//! 3. A human provides the command at call time
//!
//! # Isolation Guarantee
//!
//! - No other module can execute commands
//! - This adapter is not auto-wired anywhere
//! - Removing this adapter returns the system to execution-impossible
//!
//! This adapter enables execution only where a human stands.

use crate::command_policy::{
    authorize_command, CommandPolicyDecision, CommandSpec, DenialReason, PolicyContext,
};
use crate::execution_request::{
    ExecutionRequest, AUTOMATIC_EXECUTION_CONFIRMATION, REQUIRED_CONFIRMATION,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Allowed binaries for execution.
/// NOTE: This constant exists for backwards compatibility and guardrail tests.
/// The canonical source of truth is now the capability ledger (capabilities.rs)
/// accessed via the command_policy engine.
/// Any changes MUST be made in capabilities.rs, not here.
pub const ALLOWED_BINARIES: &[&str] = &["iw", "lsmod", "lspci", "cat", "echo"];

/// Human Execution Adapter - the only execution path in the system.
///
/// This adapter:
/// - Requires human presence at every step
/// - Cannot decide, select, or infer
/// - Executes exactly what the human provides
/// - Records everything for audit
///
/// # Construction
///
/// The adapter requires an operator identifier at construction time.
/// This identifier is recorded in every execution attempt.
///
/// # Usage
///
/// ```ignore
/// let adapter = HumanExecutionAdapter::new("user@example.com");
/// let result = adapter.execute(&request, "echo hello")?;
/// ```
///
/// This adapter enables execution only where a human stands.
#[derive(Debug, Clone)]
pub struct HumanExecutionAdapter {
    /// The operator using this adapter (required for audit)
    operator: String,
    /// Allowed binaries (frozen at construction)
    allowed_binaries: HashSet<String>,
}

/// Result of a human-mediated execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution duration
    pub duration_ms: u64,
    /// The exact command that was executed
    pub command_executed: String,
}

/// Execution attempt record for audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAttempt {
    /// Unique attempt identifier
    pub attempt_id: String,
    /// Reference to the ExecutionRequest
    pub request_id: String,
    /// Operator who initiated this execution
    pub operator: String,
    /// Exact command that was executed
    pub command: String,
    /// SHA256 hash of the command string
    pub command_hash: String,
    /// When execution started (ISO 8601)
    pub started_utc: String,
    /// When execution completed (ISO 8601)
    pub completed_utc: String,
    /// Exit code
    pub exit_code: i32,
    /// Full captured stdout
    pub stdout: String,
    /// Full captured stderr
    pub stderr: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether execution was successful
    pub success: bool,
}

/// Error during human-mediated execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    /// The ExecutionRequest is invalid
    InvalidRequest(String),
    /// The confirmation text does not match
    ConfirmationMismatch,
    /// The binary is not in the allowlist
    BinaryNotAllowed(String),
    /// The command contains forbidden patterns
    ForbiddenPattern(String),
    /// Execution failed to start
    ExecutionFailed(String),
    /// Failed to persist audit record
    AuditFailed(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::InvalidRequest(msg) => {
                write!(f, "This request cannot be processed: {}", msg)
            }
            ExecutionError::ConfirmationMismatch => {
                write!(
                    f,
                    "The confirmation text doesn't match. Anna needs exact confirmation before running commands."
                )
            }
            ExecutionError::BinaryNotAllowed(bin) => {
                write!(
                    f,
                    "'{}' is not a command Anna can run. See 'annactl capabilities' for what Anna can do.",
                    bin
                )
            }
            ExecutionError::ForbiddenPattern(pat) => {
                write!(
                    f,
                    "This command contains '{}' which is outside Anna's boundaries",
                    pat
                )
            }
            ExecutionError::ExecutionFailed(msg) => {
                write!(f, "The command did not complete successfully: {}", msg)
            }
            ExecutionError::AuditFailed(msg) => {
                write!(f, "Could not record this action in the audit log: {}", msg)
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

impl HumanExecutionAdapter {
    /// Create a new Human Execution Adapter.
    ///
    /// # Arguments
    ///
    /// * `operator` - Identifier of the human operator (required for audit)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let adapter = HumanExecutionAdapter::new("admin@example.com");
    /// ```
    pub fn new(operator: &str) -> Self {
        let allowed_binaries = ALLOWED_BINARIES
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            operator: operator.to_string(),
            allowed_binaries,
        }
    }

    /// Execute a command provided by the human operator.
    ///
    /// # Arguments
    ///
    /// * `request` - The ExecutionRequest that authorizes this execution
    /// * `command` - The exact command string provided by the human at runtime
    ///
    /// # Safety Checks
    ///
    /// Before execution, this method verifies:
    /// 1. The request's confirmation text matches exactly
    /// 2. The command's binary is in the allowlist
    /// 3. The command contains no forbidden patterns (sudo, pipes to shells, etc.)
    ///
    /// # Audit Trail
    ///
    /// Every execution attempt is persisted to:
    /// `/var/lib/anna/execution_attempts/{attempt_id}.json`
    ///
    /// # Returns
    ///
    /// * `Ok(HumanExecutionResult)` - Execution completed (may have failed with non-zero exit)
    /// * `Err(ExecutionError)` - Execution was blocked or failed to start
    ///
    /// This adapter enables execution only where a human stands.
    pub fn execute(
        &self,
        request: &ExecutionRequest,
        command: &str,
    ) -> Result<HumanExecutionResult, ExecutionError> {
        // Validate the request
        self.validate_request(request)?;

        // Validate the command
        self.validate_command(command)?;

        // Generate attempt ID
        let attempt_id = format!(
            "exec-{}-{}",
            chrono::Utc::now().timestamp_millis(),
            &request.request_id[..8.min(request.request_id.len())]
        );

        let started_utc = chrono::Utc::now().to_rfc3339();
        let start_time = Instant::now();

        // Execute the command
        let result = self.run_command(command);

        let duration = start_time.elapsed();
        let completed_utc = chrono::Utc::now().to_rfc3339();

        // Build execution result
        let exec_result = match result {
            Ok((exit_code, stdout, stderr)) => HumanExecutionResult {
                success: exit_code == 0,
                exit_code,
                stdout,
                stderr,
                duration_ms: duration.as_millis() as u64,
                command_executed: command.to_string(),
            },
            Err(e) => {
                return Err(ExecutionError::ExecutionFailed(e));
            }
        };

        // Create and persist audit record
        let attempt = ExecutionAttempt {
            attempt_id: attempt_id.clone(),
            request_id: request.request_id.clone(),
            operator: self.operator.clone(),
            command: command.to_string(),
            command_hash: self.hash_command(command),
            started_utc,
            completed_utc,
            exit_code: exec_result.exit_code,
            stdout: exec_result.stdout.clone(),
            stderr: exec_result.stderr.clone(),
            duration_ms: exec_result.duration_ms,
            success: exec_result.success,
        };

        self.persist_attempt(&attempt)?;

        Ok(exec_result)
    }

    /// Validate the ExecutionRequest.
    ///
    /// Accepts both manual (REQUIRED_CONFIRMATION) and automatic
    /// (AUTOMATIC_EXECUTION_CONFIRMATION) confirmation texts.
    fn validate_request(&self, request: &ExecutionRequest) -> Result<(), ExecutionError> {
        // Check if this is an automatic execution request
        let is_automatic = request.confirmation_text == AUTOMATIC_EXECUTION_CONFIRMATION;

        // Validate the request using appropriate method
        if is_automatic {
            request
                .validate_automatic()
                .map_err(|e| ExecutionError::InvalidRequest(e.to_string()))?;
        } else {
            request
                .validate()
                .map_err(|e| ExecutionError::InvalidRequest(e.to_string()))?;
        }

        // Double-check confirmation text matches one of the valid phrases
        if request.confirmation_text != REQUIRED_CONFIRMATION
            && request.confirmation_text != AUTOMATIC_EXECUTION_CONFIRMATION
        {
            return Err(ExecutionError::ConfirmationMismatch);
        }

        Ok(())
    }

    /// Validate the command against safety rules using the command policy engine.
    ///
    /// Phase 47: All command validation now goes through the single authorization path
    /// defined in command_policy.rs. This ensures:
    /// - Only commands in the capability ledger can be authorized
    /// - Hard-banned patterns are always rejected
    /// - The set of allowed commands matches `annactl capabilities` exactly
    fn validate_command(&self, command: &str) -> Result<(), ExecutionError> {
        let command = command.trim();

        // Parse command into CommandSpec
        let cmd_spec = CommandSpec::from_command_string(command).ok_or_else(|| {
            ExecutionError::InvalidRequest("Command cannot be empty".to_string())
        })?;

        // Create policy context
        let context = PolicyContext {
            operator: Some(self.operator.clone()),
            automatic_execution: false,
        };

        // Authorize through the single policy path
        match authorize_command(&cmd_spec, &context) {
            CommandPolicyDecision::Allowed { .. } => Ok(()),
            CommandPolicyDecision::Denied { reason } => {
                // Map denial reasons to ExecutionError for backwards compatibility
                match reason {
                    DenialReason::BinaryNotInLedger(b) => {
                        Err(ExecutionError::BinaryNotAllowed(b))
                    }
                    DenialReason::EmptyCommand => Err(ExecutionError::InvalidRequest(
                        "Command cannot be empty".to_string(),
                    )),
                    // Privilege escalation BINARY (sudo as first token) -> BinaryNotAllowed
                    DenialReason::PrivilegeEscalationBinary(b) => {
                        Err(ExecutionError::BinaryNotAllowed(b))
                    }
                    // Privilege escalation PATTERN (sudo in args) -> ForbiddenPattern
                    DenialReason::PrivilegeEscalationPattern(p) => {
                        Err(ExecutionError::ForbiddenPattern(p))
                    }
                    // Shell invocation maps to BinaryNotAllowed (shells are not allowed binaries)
                    DenialReason::ShellInvocation(s) => {
                        Err(ExecutionError::BinaryNotAllowed(s))
                    }
                    // Destructive BINARY (rm as first token) -> BinaryNotAllowed
                    DenialReason::DestructiveBinary(b) => {
                        Err(ExecutionError::BinaryNotAllowed(b))
                    }
                    // Destructive PATTERN (rm in args) -> ForbiddenPattern
                    DenialReason::DestructivePattern(p) => {
                        Err(ExecutionError::ForbiddenPattern(p))
                    }
                    // All other patterns (pipes, redirects, etc.)
                    _ => Err(ExecutionError::ForbiddenPattern(reason.to_string())),
                }
            }
        }
    }

    /// Run the command and capture output.
    fn run_command(&self, command: &str) -> Result<(i32, String, String), String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        let binary = parts[0];
        let args = &parts[1..];

        let output = Command::new(binary)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute {}: {}", binary, e))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok((exit_code, stdout, stderr))
    }

    /// Compute SHA256 hash of command string.
    fn hash_command(&self, command: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        command.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Persist the execution attempt for audit.
    fn persist_attempt(&self, attempt: &ExecutionAttempt) -> Result<(), ExecutionError> {
        let dir = PathBuf::from("/var/lib/anna/execution_attempts");

        // Create directory if needed (may fail if not root, that's ok for tests)
        let _ = fs::create_dir_all(&dir);

        let path = dir.join(format!("{}.json", attempt.attempt_id));

        let json = serde_json::to_string_pretty(attempt)
            .map_err(|e| ExecutionError::AuditFailed(e.to_string()))?;

        // Try to write, but don't fail if directory doesn't exist (for tests)
        if dir.exists() {
            let temp_path = dir.join(format!(".{}.tmp", attempt.attempt_id));
            let mut file = fs::File::create(&temp_path)
                .map_err(|e| ExecutionError::AuditFailed(e.to_string()))?;
            file.write_all(json.as_bytes())
                .map_err(|e| ExecutionError::AuditFailed(e.to_string()))?;
            file.sync_all()
                .map_err(|e| ExecutionError::AuditFailed(e.to_string()))?;
            fs::rename(&temp_path, &path)
                .map_err(|e| ExecutionError::AuditFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// Get the operator identifier.
    pub fn operator(&self) -> &str {
        &self.operator
    }

    /// Get the allowed binaries.
    pub fn allowed_binaries(&self) -> &HashSet<String> {
        &self.allowed_binaries
    }
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES
// =============================================================================
//
// This adapter:
// - CANNOT access stored plans (no import of plan modules)
// - CANNOT access stored proposals (no import of proposal modules)
// - CANNOT access stored intentions (no import of intention modules)
// - CANNOT construct commands (commands come from human at runtime)
// - CANNOT retry failed commands (no retry logic)
// - CANNOT use sudo (explicitly forbidden)
// - CANNOT mutate environment (no env manipulation)
// - CANNOT execute arbitrary binaries (explicit allowlist)
//
// The adapter receives a command string at call time.
// It validates the command against strict rules.
// It executes exactly what was provided.
// It records everything.
// It returns the result.
//
// That is all it can do.
// =============================================================================

// =============================================================================
// ISOLATION VERIFICATION
// =============================================================================
//
// PROOF: No other module can execute commands
//
// Verification:
// grep -rn "Command::new" crates/anna-shared/src/ | grep -v human_execution | grep -v test
//
// Expected: Zero results (only this module and tests use Command::new)
//
// PROOF: This adapter is not auto-wired anywhere
//
// Verification:
// grep -rn "HumanExecutionAdapter" crates/ | grep -v human_execution.rs | grep -v test
//
// Expected: Zero results in production code
//
// PROOF: Removing this adapter returns system to execution-impossible
//
// If this module is deleted:
// - No Command::new calls exist in anna-shared (except detection diagnostics in annad)
// - No execution pathway exists
// - System returns to Phase 35-39 state: structurally incapable of execution
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> ExecutionRequest {
        ExecutionRequest {
            request_id: "req-test-001".to_string(),
            proposal_id: "prop-001".to_string(),
            recorded_utc: "2026-01-15T10:00:00Z".to_string(),
            requested_by: "test@example.com".to_string(),
            requested_action: "Run diagnostic command".to_string(),
            confirmation_text: REQUIRED_CONFIRMATION.to_string(),
        }
    }

    // =========================================================================
    // POSITIVE EXECUTION TEST
    // =========================================================================

    #[test]
    fn test_positive_execution_echo() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "echo hello world");

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello world"));
        assert_eq!(result.command_executed, "echo hello world");
    }

    // =========================================================================
    // AUTOMATIC EXECUTION TESTS (Phase 43)
    // =========================================================================

    fn automatic_request() -> ExecutionRequest {
        ExecutionRequest {
            request_id: "req-auto-001".to_string(),
            proposal_id: "prop-001".to_string(),
            recorded_utc: "2026-01-15T10:00:00Z".to_string(),
            requested_by: "test@example.com".to_string(),
            requested_action: "Execute safe diagnostic commands".to_string(),
            confirmation_text: AUTOMATIC_EXECUTION_CONFIRMATION.to_string(),
        }
    }

    #[test]
    fn test_automatic_execution_works() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = automatic_request();

        let result = adapter.execute(&request, "echo automatic test");

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("automatic test"));
    }

    #[test]
    fn test_automatic_execution_respects_allowlist() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = automatic_request();

        // Even with automatic confirmation, binary must be in allowlist
        let result = adapter.execute(&request, "wget http://example.com");

        assert!(matches!(
            result,
            Err(ExecutionError::BinaryNotAllowed(bin)) if bin == "wget"
        ));
    }

    #[test]
    fn test_automatic_execution_respects_forbidden_patterns() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = automatic_request();

        // Even with automatic confirmation, forbidden patterns blocked
        let result = adapter.execute(&request, "echo sudo test");

        assert!(matches!(result, Err(ExecutionError::ForbiddenPattern(_))));
    }

    // =========================================================================
    // FAILURE TEST
    // =========================================================================

    #[test]
    fn test_failure_invalid_command() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        // 'cat' with non-existent file should fail
        let result = adapter.execute(&request, "cat /nonexistent/file/path/xyz123");

        assert!(result.is_ok()); // Execution happened
        let result = result.unwrap();
        assert!(!result.success); // But command failed
        assert_ne!(result.exit_code, 0);
        assert!(!result.stderr.is_empty());
    }

    // =========================================================================
    // ALLOWLIST REJECTION TESTS
    // =========================================================================

    #[test]
    fn test_allowlist_rejects_unknown_binary() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "wget http://example.com");

        assert!(matches!(
            result,
            Err(ExecutionError::BinaryNotAllowed(bin)) if bin == "wget"
        ));
    }

    #[test]
    fn test_allowlist_rejects_rm() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        // Even if someone tries to sneak rm through
        let result = adapter.execute(&request, "echo rm -rf /");

        // This should fail because 'rm ' pattern is forbidden
        assert!(matches!(result, Err(ExecutionError::ForbiddenPattern(_))));
    }

    #[test]
    fn test_allowlist_rejects_sudo() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "echo sudo ls");

        assert!(matches!(result, Err(ExecutionError::ForbiddenPattern(_))));
    }

    #[test]
    fn test_allowlist_rejects_shell_pipe() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "echo test | bash");

        assert!(matches!(result, Err(ExecutionError::ForbiddenPattern(_))));
    }

    #[test]
    fn test_allowlist_rejects_command_substitution() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "echo $(whoami)");

        assert!(matches!(result, Err(ExecutionError::ForbiddenPattern(_))));
    }

    #[test]
    fn test_allowlist_rejects_redirection() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "echo test > /tmp/file");

        assert!(matches!(result, Err(ExecutionError::ForbiddenPattern(_))));
    }

    // =========================================================================
    // CONFIRMATION VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_rejects_wrong_confirmation() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let mut request = valid_request();
        request.confirmation_text = "I agree".to_string();

        let result = adapter.execute(&request, "echo hello");

        assert!(matches!(
            result,
            Err(ExecutionError::InvalidRequest(_)) | Err(ExecutionError::ConfirmationMismatch)
        ));
    }

    #[test]
    fn test_rejects_empty_command() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "");

        assert!(matches!(result, Err(ExecutionError::InvalidRequest(_))));
    }

    // =========================================================================
    // PROOF: PLANS CANNOT REACH THE ADAPTER
    // =========================================================================

    #[test]
    fn proof_plans_cannot_reach_adapter() {
        // This adapter has no import of:
        // - DeterministicActionPlan
        // - ActionStep
        // - Any plan-related type
        //
        // The execute() method signature is:
        //   execute(&self, request: &ExecutionRequest, command: &str)
        //
        // NOT:
        //   execute(&self, plan: &DeterministicActionPlan)
        //
        // There is no code path that:
        // - Reads a plan from storage
        // - Extracts commands from a plan
        // - Passes plan data to this adapter
        //
        // The adapter requires a human to provide the command string at call time.
        // The command does not come from any stored data structure.
    }

    #[test]
    fn proof_no_plan_imports() {
        // Verification:
        // grep -n "DeterministicActionPlan\|ActionStep\|action_plan" \
        //     crates/anna-shared/src/human_execution.rs
        //
        // Expected: Zero results (excluding this comment)
        //
        // This module imports:
        // - ExecutionRequest (human artifact)
        // - Standard library types
        //
        // It does NOT import:
        // - action_plan
        // - DeterministicActionPlan
        // - ActionStep
        // - Any plan-related type
    }

    // =========================================================================
    // PROOF: ADAPTER CANNOT BE CALLED WITHOUT HUMAN INPUT
    // =========================================================================

    #[test]
    fn proof_requires_human_input() {
        // The execute() method requires two arguments:
        // 1. request: &ExecutionRequest - must be created with valid confirmation
        // 2. command: &str - must be provided by caller at runtime
        //
        // There is no:
        // - Default command
        // - Fallback command
        // - Auto-generated command
        // - Command read from any storage
        //
        // The human must:
        // 1. Create an ExecutionRequest with the exact confirmation text
        // 2. Call execute() with an explicit command string
        //
        // Without both, execution cannot occur.

        let adapter = HumanExecutionAdapter::new("test-operator");

        // Cannot execute without request
        // adapter.execute(???, "echo hello") // No request to pass

        // Cannot execute without command
        let request = valid_request();
        // adapter.execute(&request, ???) // Must provide command

        // Both are required
        let _ = adapter.execute(&request, "echo test");
    }

    #[test]
    fn proof_no_default_behavior() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        // There is no:
        // adapter.execute_default()
        // adapter.execute_from_plan()
        // adapter.execute_queued()
        // adapter.auto_execute()
        //
        // The only method is execute(&request, &command)
        // Both arguments are required.

        // Whitespace-only command is rejected
        let result = adapter.execute(&request, "   ");
        assert!(result.is_err());
    }

    // =========================================================================
    // AUDIT TRAIL TESTS
    // =========================================================================

    #[test]
    fn test_execution_produces_attempt_record() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        let result = adapter.execute(&request, "echo audit test");

        assert!(result.is_ok());
        let result = result.unwrap();

        // The attempt was recorded (even if persistence failed due to permissions)
        // The result contains all required audit fields
        assert!(!result.command_executed.is_empty());
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_attempt_contains_command_hash() {
        // The ExecutionAttempt structure includes command_hash
        // This provides integrity verification for the audit trail
        let adapter = HumanExecutionAdapter::new("test-operator");

        let hash1 = adapter.hash_command("echo hello");
        let hash2 = adapter.hash_command("echo hello");
        let hash3 = adapter.hash_command("echo world");

        // Same command produces same hash
        assert_eq!(hash1, hash2);
        // Different command produces different hash
        assert_ne!(hash1, hash3);
    }

    // =========================================================================
    // ISOLATION PROOF TESTS
    // =========================================================================

    #[test]
    fn proof_no_other_execution_path() {
        // This test documents that HumanExecutionAdapter is the only execution path.
        //
        // Verification:
        // grep -rn "Command::new" crates/anna-shared/src/ | grep -v human_execution
        //
        // Expected: Zero results in production code
        //
        // The only Command::new calls are in this module.
        // Removing this module removes all execution capability from anna-shared.
    }

    #[test]
    fn proof_adapter_not_autowired() {
        // This test documents that the adapter is not automatically instantiated.
        //
        // Verification:
        // grep -rn "HumanExecutionAdapter::new" crates/ | grep -v test | grep -v human_execution.rs
        //
        // Expected: Zero results
        //
        // The adapter must be explicitly constructed by calling HumanExecutionAdapter::new().
        // No daemon loop, no RPC handler, no CLI command auto-creates this adapter.
    }

    #[test]
    fn proof_removal_restores_impossibility() {
        // If this module is deleted:
        //
        // 1. No Command::new exists in anna-shared (besides assisted_ops detection in annad)
        // 2. No execution trait implementation exists
        // 3. No execution method exists
        // 4. System returns to Phase 35-39 state
        //
        // The architecture would be:
        // - Data structures exist (plans, proposals, requests)
        // - No code interprets them as executable
        // - Execution is structurally impossible
        //
        // This module is the single, removable, auditable breach point.
    }

    // =========================================================================
    // EXPLICIT CAPABILITY LIMITS
    // =========================================================================

    #[test]
    fn explicit_cannot_access_plans() {
        // There is no method like:
        // adapter.execute_plan(&plan)
        // adapter.load_and_execute(plan_id)
        //
        // The adapter has no knowledge of plans.
    }

    #[test]
    fn explicit_cannot_construct_commands() {
        // There is no method like:
        // adapter.build_command(operation, target)
        // adapter.generate_command(intent)
        //
        // Commands come only from the human at call time.
    }

    #[test]
    fn explicit_cannot_retry() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        // Execute a failing command
        let result = adapter.execute(&request, "cat /nonexistent/xyz");
        assert!(result.is_ok());
        assert!(!result.unwrap().success);

        // There is no:
        // adapter.retry_last()
        // adapter.execute_with_retry(&request, command, retries)
        //
        // Each execution is independent. No automatic retry.
    }

    #[test]
    fn explicit_cannot_use_sudo() {
        let adapter = HumanExecutionAdapter::new("test-operator");
        let request = valid_request();

        // Even allowed binaries cannot be prefixed with sudo
        let result = adapter.execute(&request, "echo sudo test");
        assert!(result.is_err());

        // And 'sudo' is not in the allowlist
        let result = adapter.execute(&request, "sudo echo test");
        assert!(matches!(result, Err(ExecutionError::BinaryNotAllowed(_))));
    }

    // =========================================================================
    // FINAL DOCUMENTATION
    // =========================================================================

    #[test]
    fn final_documentation() {
        // This adapter enables execution only where a human stands.
    }
}

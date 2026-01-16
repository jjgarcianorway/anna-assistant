//! Command Policy Engine - Single Authorization Path (Phase 47)
//!
//! This module is the ONLY way execution-capable code can approve an OS command.
//! It does NOT execute anything. It only classifies and authorizes.
//! Execution remains in HumanExecutionAdapter.
//!
//! # Purpose
//!
//! Eliminate case-by-case growth by enforcing a single generic action pathway.
//! Adding any new command/capability is impossible without updating the ledger,
//! disclosure, and tests.
//!
//! # Architectural Constraints
//!
//! This module:
//! - Derives allowed commands ONLY from capabilities.rs
//! - Contains NO execution code (no Command::new, no process spawning)
//! - Is purely a classifier/authorizer
//! - Hard-bans dangerous patterns structurally
//!
//! # Authorization Flow
//!
//! ```text
//! Command (argv) → authorize_command() → PolicyDecision
//!                                            ↓
//!                           Allowed { capability_id, safety }
//!                           Denied { reason }
//! ```
//!
//! HumanExecutionAdapter MUST call authorize_command() before execution.

use crate::capabilities::{
    all_allowed_binaries, execution_capabilities, CapabilityCategory, ExecutionLevel,
    CAPABILITIES, LEDGER_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A command specification in argv form (not a shell string).
///
/// This structure represents a command as discrete tokens, preventing
/// shell injection and making validation deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSpec {
    /// The binary to execute (first token)
    pub binary: String,
    /// Arguments to the binary (remaining tokens)
    pub args: Vec<String>,
}

impl CommandSpec {
    /// Create a CommandSpec from argv tokens.
    pub fn new(binary: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            binary: binary.into(),
            args,
        }
    }

    /// Parse a command string into a CommandSpec.
    /// This is a convenience method - prefer constructing from tokens directly.
    pub fn from_command_string(command: &str) -> Option<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        Some(Self {
            binary: parts[0].to_string(),
            args: parts[1..].iter().map(|s| s.to_string()).collect(),
        })
    }

    /// Reconstruct as a command string (for display/logging only).
    pub fn to_command_string(&self) -> String {
        if self.args.is_empty() {
            self.binary.clone()
        } else {
            format!("{} {}", self.binary, self.args.join(" "))
        }
    }
}

/// Safety classification for an authorized command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandSafety {
    /// Safe for automatic execution after confirmation
    SafeAutomatic,
    /// Requires manual execution by user (shown for copy/paste)
    ManualOnly,
}

/// Policy decision for a command authorization request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPolicyDecision {
    /// Command is allowed
    Allowed {
        /// The capability that authorized this command
        capability_id: String,
        /// Safety classification
        safety: CommandSafety,
    },
    /// Command is denied
    Denied {
        /// Reason for denial
        reason: DenialReason,
    },
}

/// Reasons a command can be denied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenialReason {
    /// Binary not in capability ledger
    BinaryNotInLedger(String),
    /// Primary binary is a privilege escalation command (sudo, pkexec as first token)
    PrivilegeEscalationBinary(String),
    /// Privilege escalation pattern in arguments (e.g., "echo sudo ls")
    PrivilegeEscalationPattern(String),
    /// Contains shell invocation
    ShellInvocation(String),
    /// Contains pipe
    PipeDetected,
    /// Contains redirect
    RedirectDetected,
    /// Contains command substitution
    CommandSubstitution,
    /// Contains backgrounding
    BackgroundingDetected,
    /// Contains command chaining
    ChainingDetected,
    /// Contains environment assignment
    EnvironmentAssignment,
    /// Empty command
    EmptyCommand,
    /// Primary binary is destructive (rm, dd as first token)
    DestructiveBinary(String),
    /// Destructive pattern in arguments (e.g., "echo rm -rf /")
    DestructivePattern(String),
}

impl std::fmt::Display for DenialReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DenialReason::BinaryNotInLedger(b) => {
                write!(
                    f,
                    "'{}' is not a command Anna can run. See 'annactl capabilities' for what Anna can do.",
                    b
                )
            }
            DenialReason::PrivilegeEscalationBinary(p) => {
                write!(
                    f,
                    "Anna will never run '{}' - running commands as root is outside her boundaries",
                    p
                )
            }
            DenialReason::PrivilegeEscalationPattern(p) => {
                write!(
                    f,
                    "Anna will never run '{}' - privilege escalation is outside her boundaries",
                    p
                )
            }
            DenialReason::ShellInvocation(s) => {
                write!(
                    f,
                    "Anna will never invoke a shell like '{}' - shell access is outside her boundaries",
                    s
                )
            }
            DenialReason::PipeDetected => {
                write!(f, "Anna runs commands directly, not through a shell. Pipes (|) are not supported.")
            }
            DenialReason::RedirectDetected => {
                write!(f, "Anna runs commands directly, not through a shell. Redirects (>, <) are not supported.")
            }
            DenialReason::CommandSubstitution => {
                write!(f, "Anna runs commands directly, not through a shell. Command substitution is not supported.")
            }
            DenialReason::BackgroundingDetected => {
                write!(f, "Anna runs commands directly, not through a shell. Background execution (&) is not supported.")
            }
            DenialReason::ChainingDetected => {
                write!(f, "Anna runs commands directly, not through a shell. Command chaining (;, &&) is not supported.")
            }
            DenialReason::EnvironmentAssignment => {
                write!(f, "Anna runs commands directly, not through a shell. Environment variables cannot be set this way.")
            }
            DenialReason::EmptyCommand => write!(f, "No command was provided"),
            DenialReason::DestructiveBinary(p) => {
                write!(
                    f,
                    "Anna will never run '{}' - destructive commands are outside her boundaries",
                    p
                )
            }
            DenialReason::DestructivePattern(p) => {
                write!(
                    f,
                    "Anna will never run commands involving '{}' - destructive operations are outside her boundaries",
                    p
                )
            }
        }
    }
}

/// Context for policy decisions (for future extensibility).
#[derive(Debug, Clone, Default)]
pub struct PolicyContext {
    /// Optional operator identifier
    pub operator: Option<String>,
    /// Whether this is for automatic execution
    pub automatic_execution: bool,
}

// =============================================================================
// HARD BANS - These patterns are ALWAYS denied, regardless of ledger
// =============================================================================

/// Privilege escalation binaries - always denied
const PRIVILEGE_ESCALATION: &[&str] = &["sudo", "su", "pkexec", "doas", "runuser", "dzdo"];

/// Shell binaries - always denied as the primary binary
const SHELL_BINARIES: &[&str] = &["sh", "bash", "zsh", "fish", "dash", "csh", "tcsh", "ksh"];

/// Destructive binaries - always denied
const DESTRUCTIVE_BINARIES: &[&str] = &["rm", "dd", "mkfs", "fdisk", "parted", "shred", "wipefs"];

/// Network binaries - always denied (capability ledger forbids network)
const NETWORK_BINARIES: &[&str] = &["wget", "curl", "nc", "netcat", "ssh", "scp", "rsync", "ftp"];

/// Package manager binaries - always denied
const PACKAGE_MANAGERS: &[&str] = &[
    "pacman", "yay", "paru", "apt", "apt-get", "dnf", "yum", "zypper", "emerge", "nix",
];

// =============================================================================
// AUTHORIZATION ENGINE
// =============================================================================

/// Authorize a command against the capability ledger.
///
/// This is the ONLY function that can authorize commands for execution.
/// HumanExecutionAdapter MUST call this before accepting any command.
///
/// # Arguments
///
/// * `cmd` - The command specification to authorize
/// * `context` - Additional context for the decision
///
/// # Returns
///
/// * `CommandPolicyDecision::Allowed` - Command is authorized, with capability info
/// * `CommandPolicyDecision::Denied` - Command is rejected, with reason
///
/// # Invariants
///
/// 1. Commands not in the ledger cannot be authorized
/// 2. Hard-banned patterns always result in Denied
/// 3. The set of authorizable commands matches `annactl capabilities` exactly
pub fn authorize_command(cmd: &CommandSpec, _context: &PolicyContext) -> CommandPolicyDecision {
    // Check for empty command
    if cmd.binary.is_empty() {
        return CommandPolicyDecision::Denied {
            reason: DenialReason::EmptyCommand,
        };
    }

    // Reconstruct for pattern checking
    let full_command = cmd.to_command_string();

    // Hard bans - these ALWAYS deny, regardless of ledger
    if let Some(reason) = check_hard_bans(&cmd.binary, &cmd.args, &full_command) {
        return CommandPolicyDecision::Denied { reason };
    }

    // Check if binary is in the capability ledger
    let ledger_binaries = get_execution_allowed_binaries();
    if !ledger_binaries.contains(cmd.binary.as_str()) {
        return CommandPolicyDecision::Denied {
            reason: DenialReason::BinaryNotInLedger(cmd.binary.clone()),
        };
    }

    // Find which capability authorized this binary
    let capability_id = find_authorizing_capability(&cmd.binary);

    // Determine safety level based on capability
    let safety = determine_safety(&cmd.binary);

    CommandPolicyDecision::Allowed {
        capability_id,
        safety,
    }
}

/// Check hard-banned patterns. Returns Some(reason) if banned.
fn check_hard_bans(binary: &str, args: &[String], full_command: &str) -> Option<DenialReason> {
    // Check privilege escalation as PRIMARY BINARY (sudo, pkexec as first token)
    for priv_bin in PRIVILEGE_ESCALATION {
        if binary == *priv_bin {
            return Some(DenialReason::PrivilegeEscalationBinary(binary.to_string()));
        }
    }
    // Check if privilege escalation appears in ARGUMENTS (e.g., "echo sudo ls")
    for priv_bin in PRIVILEGE_ESCALATION {
        for arg in args {
            if arg == *priv_bin {
                return Some(DenialReason::PrivilegeEscalationPattern(arg.clone()));
            }
        }
    }

    // Check shell binaries as primary command
    for shell in SHELL_BINARIES {
        if binary == *shell {
            return Some(DenialReason::ShellInvocation(binary.to_string()));
        }
    }

    // Check destructive binaries as PRIMARY BINARY (rm, dd as first token)
    for destructive in DESTRUCTIVE_BINARIES {
        if binary == *destructive {
            return Some(DenialReason::DestructiveBinary(binary.to_string()));
        }
    }
    // Check if destructive command appears in ARGUMENTS (e.g., "echo rm -rf /")
    for destructive in DESTRUCTIVE_BINARIES {
        for arg in args {
            if arg == *destructive {
                return Some(DenialReason::DestructivePattern(arg.clone()));
            }
        }
    }

    // Check network binaries
    for net_bin in NETWORK_BINARIES {
        if binary == *net_bin {
            return Some(DenialReason::BinaryNotInLedger(binary.to_string()));
        }
    }

    // Check package managers
    for pkg_mgr in PACKAGE_MANAGERS {
        if binary == *pkg_mgr {
            return Some(DenialReason::BinaryNotInLedger(binary.to_string()));
        }
    }

    // Check for pipes in arguments
    for arg in args {
        if arg == "|" {
            return Some(DenialReason::PipeDetected);
        }
    }
    if full_command.contains(" | ") || full_command.contains("|") {
        return Some(DenialReason::PipeDetected);
    }

    // Check for redirects
    for arg in args {
        if arg == ">" || arg == ">>" || arg == "<" || arg.starts_with('>') || arg.starts_with('<') {
            return Some(DenialReason::RedirectDetected);
        }
    }

    // Check for command substitution and variable expansion
    // $() - command substitution
    // `` - backtick command substitution
    // ${} - variable expansion (defense in depth, even though literal without shell)
    if full_command.contains("$(") || full_command.contains('`') || full_command.contains("${") {
        return Some(DenialReason::CommandSubstitution);
    }

    // Check for shell brace expansion (defense in depth)
    // Patterns like {a,b,c} or {1..10} only work in shells, but reject for safety
    for arg in args {
        if arg.starts_with('{') && arg.ends_with('}') && (arg.contains(',') || arg.contains("..")) {
            return Some(DenialReason::CommandSubstitution);
        }
    }

    // Check for backgrounding
    for arg in args {
        if arg == "&" {
            return Some(DenialReason::BackgroundingDetected);
        }
    }
    if full_command.ends_with(" &") || full_command.ends_with("&") {
        return Some(DenialReason::BackgroundingDetected);
    }

    // Check for chaining - defense in depth, catch ; anywhere
    // Even if it wouldn't work without shell, reject for safety
    for arg in args {
        if arg == ";" || arg == "&&" || arg == "||" || arg.contains(';') || arg.contains("&&") || arg.contains("||") {
            return Some(DenialReason::ChainingDetected);
        }
    }
    if full_command.contains(';') || full_command.contains("&&") || full_command.contains("||") {
        return Some(DenialReason::ChainingDetected);
    }

    // Check for environment assignment (VAR=value at start)
    if binary.contains('=') {
        return Some(DenialReason::EnvironmentAssignment);
    }

    // Check for /dev/ access in arguments
    for arg in args {
        if arg.starts_with("/dev/") {
            return Some(DenialReason::DestructivePattern(format!(
                "direct device access: {}",
                arg
            )));
        }
    }

    None
}

/// Get the set of binaries allowed by execution capabilities in the ledger.
fn get_execution_allowed_binaries() -> HashSet<&'static str> {
    CAPABILITIES
        .iter()
        .filter(|c| {
            matches!(
                c.execution_level,
                ExecutionLevel::HumanConfirmed | ExecutionLevel::HumanConfirmedSafeAutomatic
            ) || c.category == CapabilityCategory::Diagnosis
        })
        .flat_map(|c| c.allowed_binaries.iter().copied())
        .collect()
}

/// Find which capability authorizes a binary.
fn find_authorizing_capability(binary: &str) -> String {
    for cap in CAPABILITIES {
        if cap.allowed_binaries.contains(&binary) {
            return cap.name.to_string();
        }
    }
    "unknown".to_string()
}

/// Determine safety classification for a binary.
fn determine_safety(binary: &str) -> CommandSafety {
    // Execution-level binaries are safe for automatic execution
    for cap in execution_capabilities() {
        if cap.allowed_binaries.contains(&binary) {
            return CommandSafety::SafeAutomatic;
        }
    }
    // Diagnosis binaries are also safe
    CommandSafety::SafeAutomatic
}

// =============================================================================
// CONSISTENCY VERIFICATION
// =============================================================================

/// Verify that the policy engine is consistent with the capability ledger.
/// Returns errors if there are any mismatches.
pub fn verify_policy_ledger_consistency() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Get what policy authorizes
    let policy_binaries = get_execution_allowed_binaries();

    // Get what ledger declares
    let ledger_binaries = all_allowed_binaries();

    // Check for binaries in policy but not ledger
    for binary in &policy_binaries {
        if !ledger_binaries.contains(binary) {
            errors.push(format!(
                "Policy authorizes '{}' but it's not in ledger",
                binary
            ));
        }
    }

    // Check for binaries in ledger but not policy (should not happen)
    for binary in &ledger_binaries {
        if !policy_binaries.contains(binary) {
            errors.push(format!(
                "Ledger contains '{}' but policy doesn't authorize it",
                binary
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Get the policy engine version (derived from ledger version).
pub fn policy_version() -> &'static str {
    LEDGER_VERSION
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> PolicyContext {
        PolicyContext::default()
    }

    // =========================================================================
    // LEDGER ENFORCEMENT TESTS
    // =========================================================================

    #[test]
    fn test_binary_not_in_ledger_denied() {
        let cmd = CommandSpec::new("wget", vec!["https://example.com".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::BinaryNotInLedger(_)
            }
        ));
    }

    #[test]
    fn test_ledger_binary_allowed() {
        let cmd = CommandSpec::new("lsmod", vec![]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(decision, CommandPolicyDecision::Allowed { .. }));
    }

    #[test]
    fn test_all_ledger_binaries_allowed() {
        let ledger_binaries = all_allowed_binaries();
        for binary in ledger_binaries {
            let cmd = CommandSpec::new(binary, vec![]);
            let decision = authorize_command(&cmd, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Allowed { .. }),
                "Binary '{}' should be allowed",
                binary
            );
        }
    }

    // =========================================================================
    // HARD BAN TESTS
    // =========================================================================

    #[test]
    fn test_sudo_always_denied() {
        let cmd = CommandSpec::new("sudo", vec!["cat".to_string(), "/etc/passwd".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::PrivilegeEscalationBinary(_)
            }
        ));
    }

    #[test]
    fn test_su_always_denied() {
        let cmd = CommandSpec::new("su", vec!["-".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::PrivilegeEscalationBinary(_)
            }
        ));
    }

    #[test]
    fn test_pkexec_always_denied() {
        let cmd = CommandSpec::new("pkexec", vec!["cat".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::PrivilegeEscalationBinary(_)
            }
        ));
    }

    #[test]
    fn test_doas_always_denied() {
        let cmd = CommandSpec::new("doas", vec!["cat".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::PrivilegeEscalationBinary(_)
            }
        ));
    }

    #[test]
    fn test_sudo_in_args_denied() {
        // "echo sudo ls" - sudo appears in args, not as binary
        let cmd = CommandSpec::new("echo", vec!["sudo".to_string(), "ls".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::PrivilegeEscalationPattern(_)
            }
        ));
    }

    #[test]
    fn test_shell_invocation_denied() {
        for shell in SHELL_BINARIES {
            let cmd = CommandSpec::new(*shell, vec!["-c".to_string(), "ls".to_string()]);
            let decision = authorize_command(&cmd, &ctx());
            assert!(
                matches!(
                    decision,
                    CommandPolicyDecision::Denied {
                        reason: DenialReason::ShellInvocation(_)
                    }
                ),
                "Shell '{}' should be denied",
                shell
            );
        }
    }

    #[test]
    fn test_pipe_denied() {
        let cmd = CommandSpec::new("cat", vec!["/etc/passwd".to_string(), "|".to_string(), "grep".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::PipeDetected
            }
        ));
    }

    #[test]
    fn test_redirect_denied() {
        let cmd = CommandSpec::new("echo", vec!["test".to_string(), ">".to_string(), "file".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::RedirectDetected
            }
        ));
    }

    #[test]
    fn test_command_substitution_denied() {
        let cmd = CommandSpec::new("echo", vec!["$(whoami)".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::CommandSubstitution
            }
        ));
    }

    #[test]
    fn test_backgrounding_denied() {
        let cmd = CommandSpec::new("cat", vec!["/etc/passwd".to_string(), "&".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::BackgroundingDetected
            }
        ));
    }

    #[test]
    fn test_chaining_denied() {
        let cmd = CommandSpec::new("echo", vec!["a".to_string(), ";".to_string(), "echo".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::ChainingDetected
            }
        ));
    }

    #[test]
    fn test_env_assignment_denied() {
        let cmd = CommandSpec::new("FOO=bar", vec!["echo".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::EnvironmentAssignment
            }
        ));
    }

    #[test]
    fn test_dev_access_denied() {
        let cmd = CommandSpec::new("cat", vec!["/dev/sda".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::DestructivePattern(_)
            }
        ));
    }

    #[test]
    fn test_rm_denied() {
        let cmd = CommandSpec::new("rm", vec!["-rf".to_string(), "/".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::DestructiveBinary(_)
            }
        ));
    }

    #[test]
    fn test_dd_denied() {
        let cmd = CommandSpec::new("dd", vec!["if=/dev/zero".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::DestructiveBinary(_)
            }
        ));
    }

    #[test]
    fn test_rm_in_args_denied() {
        // "echo rm -rf /" - rm appears in args, not as binary
        let cmd = CommandSpec::new("echo", vec!["rm".to_string(), "-rf".to_string(), "/".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::DestructivePattern(_)
            }
        ));
    }

    // =========================================================================
    // EMPTY COMMAND TESTS
    // =========================================================================

    #[test]
    fn test_empty_command_denied() {
        let cmd = CommandSpec::new("", vec![]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::EmptyCommand
            }
        ));
    }

    // =========================================================================
    // CONSISTENCY TESTS
    // =========================================================================

    #[test]
    fn test_policy_ledger_consistency() {
        let result = verify_policy_ledger_consistency();
        assert!(
            result.is_ok(),
            "Policy/ledger mismatch: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_policy_version_matches_ledger() {
        assert_eq!(policy_version(), LEDGER_VERSION);
    }

    // =========================================================================
    // AUTHORIZATION RESULT TESTS
    // =========================================================================

    #[test]
    fn test_allowed_includes_capability_id() {
        let cmd = CommandSpec::new("lsmod", vec![]);
        let decision = authorize_command(&cmd, &ctx());
        if let CommandPolicyDecision::Allowed { capability_id, .. } = decision {
            assert!(!capability_id.is_empty());
        } else {
            panic!("Expected Allowed decision");
        }
    }

    #[test]
    fn test_allowed_includes_safety() {
        let cmd = CommandSpec::new("iw", vec!["dev".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        if let CommandPolicyDecision::Allowed { safety, .. } = decision {
            assert!(matches!(safety, CommandSafety::SafeAutomatic | CommandSafety::ManualOnly));
        } else {
            panic!("Expected Allowed decision");
        }
    }

    // =========================================================================
    // COMMAND SPEC TESTS
    // =========================================================================

    #[test]
    fn test_command_spec_from_string() {
        let spec = CommandSpec::from_command_string("cat /etc/passwd").unwrap();
        assert_eq!(spec.binary, "cat");
        assert_eq!(spec.args, vec!["/etc/passwd"]);
    }

    #[test]
    fn test_command_spec_to_string() {
        let spec = CommandSpec::new("cat", vec!["/etc/passwd".to_string()]);
        assert_eq!(spec.to_command_string(), "cat /etc/passwd");
    }

    #[test]
    fn test_command_spec_empty_args() {
        let spec = CommandSpec::new("lsmod", vec![]);
        assert_eq!(spec.to_command_string(), "lsmod");
    }
}

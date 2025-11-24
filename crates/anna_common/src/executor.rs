//! Executor - Safe command execution with file-writing detection (6.5.0)
//!
//! Classification rules (conservative):
//! - Harmless: status checks, logs, queries (can be auto-executed after confirmation)
//! - FileWriting: any command that writes/edits files (NEVER executed in 6.5.0)

use crate::orchestrator::Plan;
use std::process::Command;

/// Classification of command execution safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepExecutionKind {
    /// Harmless commands: status checks, logs, queries
    /// Can be executed after user confirmation
    Harmless,

    /// File-writing commands: edits configs, writes files
    /// NEVER executed in 6.5.0
    FileWriting,
}

/// Result of executing a single plan step
#[derive(Debug, Clone)]
pub struct StepExecutionResult {
    pub command: String,
    pub kind: StepExecutionKind,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub skipped_reason: Option<String>,
}

/// Report of plan execution
#[derive(Debug, Clone)]
pub struct ExecutionReport {
    pub results: Vec<StepExecutionResult>,
}

impl ExecutionReport {
    /// Check if all executed steps succeeded
    pub fn all_succeeded(&self) -> bool {
        self.results
            .iter()
            .filter(|r| r.kind == StepExecutionKind::Harmless && r.skipped_reason.is_none())
            .all(|r| r.success)
    }

    /// Count of harmless steps that were executed
    pub fn executed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.kind == StepExecutionKind::Harmless && r.skipped_reason.is_none())
            .count()
    }

    /// Count of steps skipped due to file-writing classification
    pub fn skipped_count(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.skipped_reason.is_some())
            .count()
    }
}

/// Detect if a command writes or edits files
///
/// Conservative rules - returns true if command:
/// - Contains > or >> redirection
/// - Contains sed -i, perl -pi, or similar in-place editing
/// - Contains tee with absolute path
/// - Contains cp, mv, rm targeting system/config paths
/// - Contains pacman package mutations
pub fn is_file_writing_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();

    // Redirection operators
    if cmd.contains('>') && (cmd.contains(">>") || cmd.contains("> ")) {
        return true;
    }

    // In-place editing flags
    if cmd_lower.contains("sed -i")
        || cmd_lower.contains("perl -pi")
        || cmd_lower.contains("python -i")
    {
        return true;
    }

    // tee with absolute paths
    if cmd_lower.contains("tee") && cmd.contains('/') {
        return true;
    }

    // File operations targeting sensitive paths
    let dangerous_ops = ["cp ", "mv ", "rm "];
    let sensitive_paths = ["/etc/", "/usr/", "/var/", "$HOME", "~/"];

    for op in &dangerous_ops {
        if cmd_lower.contains(op) {
            for path in &sensitive_paths {
                if cmd.contains(path) {
                    return true;
                }
            }
        }
    }

    // Package mutations
    if cmd_lower.contains("pacman") {
        if cmd_lower.contains(" -s") || cmd_lower.contains(" -r") || cmd_lower.contains(" -u") {
            return true;
        }
    }

    false
}

/// Classify a command for execution safety
pub fn classify_command(cmd: &str) -> StepExecutionKind {
    if is_file_writing_command(cmd) {
        StepExecutionKind::FileWriting
    } else {
        StepExecutionKind::Harmless
    }
}

/// Execute a plan's commands (only harmless ones)
///
/// File-writing commands are recorded as skipped.
/// All harmless commands are executed in sequence.
pub fn execute_plan(plan: &Plan) -> ExecutionReport {
    let mut results = Vec::new();

    for step in &plan.steps {
        let kind = classify_command(&step.command);

        match kind {
            StepExecutionKind::FileWriting => {
                // Never execute file-writing commands in 6.5.0
                results.push(StepExecutionResult {
                    command: step.command.clone(),
                    kind,
                    success: false,
                    exit_code: None,
                    skipped_reason: Some("File-writing command (not allowed)".to_string()),
                });
            }
            StepExecutionKind::Harmless => {
                // Execute harmless command
                let result = execute_harmless_command(&step.command);
                results.push(result);
            }
        }
    }

    ExecutionReport { results }
}

/// Execute a single harmless command
fn execute_harmless_command(cmd: &str) -> StepExecutionResult {
    // Parse command into program and args
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return StepExecutionResult {
            command: cmd.to_string(),
            kind: StepExecutionKind::Harmless,
            success: false,
            exit_code: None,
            skipped_reason: Some("Empty command".to_string()),
        };
    }

    let program = parts[0];
    let args = &parts[1..];

    // Execute via std::process::Command
    match Command::new(program).args(args).output() {
        Ok(output) => StepExecutionResult {
            command: cmd.to_string(),
            kind: StepExecutionKind::Harmless,
            success: output.status.success(),
            exit_code: output.status.code(),
            skipped_reason: None,
        },
        Err(e) => StepExecutionResult {
            command: cmd.to_string(),
            kind: StepExecutionKind::Harmless,
            success: false,
            exit_code: None,
            skipped_reason: Some(format!("Execution error: {}", e)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harmless_commands() {
        // Status checks
        assert_eq!(
            classify_command("systemctl status nginx.service"),
            StepExecutionKind::Harmless
        );
        assert_eq!(
            classify_command("journalctl -u systemd-resolved.service"),
            StepExecutionKind::Harmless
        );
        assert_eq!(
            classify_command("resolvectl query archlinux.org"),
            StepExecutionKind::Harmless
        );

        // Reading files
        assert_eq!(
            classify_command("cat /etc/resolv.conf"),
            StepExecutionKind::Harmless
        );
        assert_eq!(
            classify_command("ls -la /var/log"),
            StepExecutionKind::Harmless
        );

        // Network queries
        assert_eq!(
            classify_command("ping -c 4 8.8.8.8"),
            StepExecutionKind::Harmless
        );

        // Service operations (not file-writing)
        assert_eq!(
            classify_command("sudo systemctl restart nginx.service"),
            StepExecutionKind::Harmless
        );
    }

    #[test]
    fn test_file_writing_commands() {
        // Redirection
        assert_eq!(
            classify_command("echo foo > /etc/resolv.conf"),
            StepExecutionKind::FileWriting
        );
        assert_eq!(
            classify_command("cat file >> /var/log/test.log"),
            StepExecutionKind::FileWriting
        );

        // In-place editing
        assert_eq!(
            classify_command("sed -i 's/a/b/' /etc/thing.conf"),
            StepExecutionKind::FileWriting
        );
        assert_eq!(
            classify_command("perl -pi -e 's/x/y/' /etc/config"),
            StepExecutionKind::FileWriting
        );

        // tee to file
        assert_eq!(
            classify_command("echo test | tee /etc/hosts"),
            StepExecutionKind::FileWriting
        );

        // File operations on system paths
        assert_eq!(
            classify_command("cp file /etc/nginx/nginx.conf"),
            StepExecutionKind::FileWriting
        );
        assert_eq!(
            classify_command("mv old /usr/share/config"),
            StepExecutionKind::FileWriting
        );
        assert_eq!(
            classify_command("rm /var/tmp/test"),
            StepExecutionKind::FileWriting
        );
        assert_eq!(
            classify_command("cp backup ~/myconfig"),
            StepExecutionKind::FileWriting
        );

        // Package mutations
        assert_eq!(
            classify_command("sudo pacman -Syu"),
            StepExecutionKind::FileWriting
        );
        assert_eq!(
            classify_command("pacman -S nginx"),
            StepExecutionKind::FileWriting
        );
        assert_eq!(
            classify_command("pacman -R oldpackage"),
            StepExecutionKind::FileWriting
        );
    }

    #[test]
    fn test_execution_report_metrics() {
        let report = ExecutionReport {
            results: vec![
                StepExecutionResult {
                    command: "ls".to_string(),
                    kind: StepExecutionKind::Harmless,
                    success: true,
                    exit_code: Some(0),
                    skipped_reason: None,
                },
                StepExecutionResult {
                    command: "echo > file".to_string(),
                    kind: StepExecutionKind::FileWriting,
                    success: false,
                    exit_code: None,
                    skipped_reason: Some("File-writing".to_string()),
                },
            ],
        };

        assert_eq!(report.executed_count(), 1);
        assert_eq!(report.skipped_count(), 1);
        assert!(report.all_succeeded());
    }
}

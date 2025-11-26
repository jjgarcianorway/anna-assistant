//! CommandExec - v6.58.0 Toolchain Reality Lock
//!
//! Single command execution layer that:
//! - Takes a tool definition and arguments
//! - Executes on the real system
//! - Captures real exit code, stdout, stderr, duration
//! - Returns structured results WITHOUT interpretation
//!
//! This layer does NOT reinterpret errors. It passes them exactly as received.

use crate::strict_tool_catalog::{StrictToolCatalog, ToolDefinition, RiskLevel};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{Duration, Instant};

/// Maximum output length to capture (prevent memory issues)
const MAX_OUTPUT_BYTES: usize = 64 * 1024; // 64KB

/// Default timeout for commands
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// The tool name from catalog
    pub tool_name: String,
    /// Full command that was executed
    pub full_command: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Stdout (truncated if too long)
    pub stdout: String,
    /// Whether stdout was truncated
    pub stdout_truncated: bool,
    /// Stderr (truncated if too long)
    pub stderr: String,
    /// Whether stderr was truncated
    pub stderr_truncated: bool,
    /// Execution duration
    pub duration_ms: u64,
    /// Execution status
    pub status: ExecutionStatus,
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Command ran successfully (exit code 0)
    Success,
    /// Command ran but returned non-zero exit code
    NonZeroExit,
    /// Command not found on system
    CommandNotFound,
    /// Permission denied
    PermissionDenied,
    /// Command timed out
    Timeout,
    /// Tool not in catalog (refused to execute)
    NotInCatalog,
    /// Other OS error
    OsError,
}

impl ExecutionStatus {
    /// Human-readable description
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::NonZeroExit => "non-zero exit",
            Self::CommandNotFound => "command not found",
            Self::PermissionDenied => "permission denied",
            Self::Timeout => "timeout",
            Self::NotInCatalog => "not in catalog",
            Self::OsError => "OS error",
        }
    }
}

/// Tool health status from self-test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolHealth {
    /// Tool name
    pub name: String,
    /// Whether tool is available
    pub available: bool,
    /// Brief status message
    pub status_message: String,
    /// Last test exit code (if tested)
    pub last_exit_code: Option<i32>,
}

/// Overall toolchain health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainHealth {
    /// Overall status
    pub status: ToolchainStatus,
    /// Individual tool health
    pub tools: Vec<ToolHealth>,
    /// Timestamp of last test
    pub tested_at: String,
}

/// Toolchain status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolchainStatus {
    /// All essential tools work
    Healthy,
    /// Some tools missing but core works
    Degraded,
    /// Critical tools missing
    Critical,
}

impl ToolchainStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Critical => "critical",
        }
    }
}

/// Command executor with strict catalog enforcement
pub struct CommandExec {
    catalog: StrictToolCatalog,
}

impl Default for CommandExec {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandExec {
    /// Create new executor with default catalog
    pub fn new() -> Self {
        Self {
            catalog: StrictToolCatalog::new(),
        }
    }

    /// Execute a command from the catalog by tool name
    ///
    /// Arguments are appended to the base command.
    /// Returns error if tool is not in catalog.
    pub fn execute(&self, tool_name: &str, args: &[&str]) -> CommandResult {
        let start = Instant::now();

        // Check if tool exists in catalog
        let tool = match self.catalog.get(tool_name) {
            Some(t) => t,
            None => {
                return CommandResult {
                    tool_name: tool_name.to_string(),
                    full_command: format!("{} (NOT IN CATALOG)", tool_name),
                    exit_code: -1,
                    stdout: String::new(),
                    stdout_truncated: false,
                    stderr: format!("Tool '{}' is not in the allowed catalog", tool_name),
                    stderr_truncated: false,
                    duration_ms: start.elapsed().as_millis() as u64,
                    status: ExecutionStatus::NotInCatalog,
                };
            }
        };

        // Build the full command
        let full_command = if args.is_empty() {
            tool.command.clone()
        } else {
            format!("{} {}", tool.command, args.join(" "))
        };

        // Execute via sh -c for consistency
        self.execute_raw(&full_command, tool_name, start)
    }

    /// Execute a raw command string (internal use only)
    ///
    /// This should only be called after catalog validation.
    fn execute_raw(&self, command: &str, tool_name: &str, start: Instant) -> CommandResult {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output();

        let duration_ms = start.elapsed().as_millis() as u64;

        match output {
            Ok(output) => {
                let (stdout, stdout_truncated) = truncate_output(&output.stdout);
                let (stderr, stderr_truncated) = truncate_output(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let status = if output.status.success() {
                    ExecutionStatus::Success
                } else if stderr.contains("command not found") || stderr.contains("No such file or directory") && stderr.contains("/usr/bin") {
                    ExecutionStatus::CommandNotFound
                } else if stderr.contains("Permission denied") {
                    ExecutionStatus::PermissionDenied
                } else {
                    ExecutionStatus::NonZeroExit
                };

                CommandResult {
                    tool_name: tool_name.to_string(),
                    full_command: command.to_string(),
                    exit_code,
                    stdout,
                    stdout_truncated,
                    stderr,
                    stderr_truncated,
                    duration_ms,
                    status,
                }
            }
            Err(e) => {
                let status = if e.kind() == std::io::ErrorKind::NotFound {
                    ExecutionStatus::CommandNotFound
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    ExecutionStatus::PermissionDenied
                } else {
                    ExecutionStatus::OsError
                };

                CommandResult {
                    tool_name: tool_name.to_string(),
                    full_command: command.to_string(),
                    exit_code: -1,
                    stdout: String::new(),
                    stdout_truncated: false,
                    stderr: format!("OS error: {}", e),
                    stderr_truncated: false,
                    duration_ms,
                    status,
                }
            }
        }
    }

    /// Run self-test on essential tools
    pub fn self_test(&self) -> ToolchainHealth {
        let mut tools = Vec::new();
        let mut critical_failures = 0;
        let mut degraded = false;

        // Essential tools that MUST work
        let essential_tests = vec![
            ("echo", "sh -c 'echo anna-ok'", true),
            ("ls", "ls /", true),
            ("cat_meminfo", "cat /proc/meminfo | head -5", true),
            ("free", "free -m", true),
            ("lscpu", "lscpu | head -10", true),
            ("df", "df -h /", true),
            ("pacman", "pacman -Q pacman", true),
            ("ip", "ip addr show lo", true),
            ("journalctl", "journalctl -n 1 --no-pager", false), // May fail without systemd
            ("nmcli", "nmcli device status", false), // Optional
            ("sensors", "sensors", false), // Optional
        ];

        for (name, command, is_critical) in essential_tests {
            let start = Instant::now();
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output();

            let duration_ms = start.elapsed().as_millis() as u64;

            let health = match output {
                Ok(output) if output.status.success() => {
                    ToolHealth {
                        name: name.to_string(),
                        available: true,
                        status_message: format!("OK ({}ms)", duration_ms),
                        last_exit_code: Some(output.status.code().unwrap_or(0)),
                    }
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let msg = if stderr.contains("command not found") {
                        "not installed".to_string()
                    } else {
                        format!("exit {}", output.status.code().unwrap_or(-1))
                    };

                    if is_critical {
                        critical_failures += 1;
                    } else {
                        degraded = true;
                    }

                    ToolHealth {
                        name: name.to_string(),
                        available: false,
                        status_message: msg,
                        last_exit_code: output.status.code(),
                    }
                }
                Err(e) => {
                    if is_critical {
                        critical_failures += 1;
                    } else {
                        degraded = true;
                    }

                    ToolHealth {
                        name: name.to_string(),
                        available: false,
                        status_message: format!("error: {}", e),
                        last_exit_code: None,
                    }
                }
            };

            tools.push(health);
        }

        let status = if critical_failures > 0 {
            ToolchainStatus::Critical
        } else if degraded {
            ToolchainStatus::Degraded
        } else {
            ToolchainStatus::Healthy
        };

        ToolchainHealth {
            status,
            tools,
            tested_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Get the catalog
    pub fn catalog(&self) -> &StrictToolCatalog {
        &self.catalog
    }

    /// Check if a tool is available (exists in catalog)
    pub fn is_tool_available(&self, name: &str) -> bool {
        self.catalog.exists(name)
    }
}

/// Truncate output to max bytes, converting to string
fn truncate_output(bytes: &[u8]) -> (String, bool) {
    let truncated = bytes.len() > MAX_OUTPUT_BYTES;
    let slice = if truncated {
        &bytes[..MAX_OUTPUT_BYTES]
    } else {
        bytes
    };

    let output = String::from_utf8_lossy(slice).to_string();
    (output, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_catalog_tool() {
        let exec = CommandExec::new();

        // echo is in catalog
        let result = exec.execute("echo", &["test"]);
        assert_eq!(result.tool_name, "echo");
        // Note: echo is just "echo" in catalog, this tests the execution
    }

    #[test]
    fn test_execute_non_catalog_tool_fails() {
        let exec = CommandExec::new();

        // rm is NOT in catalog
        let result = exec.execute("rm", &["-rf", "/"]);
        assert_eq!(result.status, ExecutionStatus::NotInCatalog);
        assert!(result.stderr.contains("not in the allowed catalog"));
    }

    #[test]
    fn test_execute_fabricated_tool_fails() {
        let exec = CommandExec::new();

        // systemd-cryptgen is a fabricated tool that should NOT exist
        let result = exec.execute("systemd-cryptgen", &["--version"]);
        assert_eq!(result.status, ExecutionStatus::NotInCatalog);
    }

    #[test]
    fn test_self_test() {
        let exec = CommandExec::new();
        let health = exec.self_test();

        // At minimum, ls and echo should work on any system
        let echo_health = health.tools.iter().find(|t| t.name == "echo");
        assert!(echo_health.is_some());

        let ls_health = health.tools.iter().find(|t| t.name == "ls");
        assert!(ls_health.is_some());
    }

    #[test]
    fn test_real_free_command() {
        let exec = CommandExec::new();

        let result = exec.execute("free", &["-m"]);

        // On a real system, free should succeed
        if result.status == ExecutionStatus::Success {
            assert!(result.stdout.contains("Mem:") || result.stdout.contains("total"));
            assert!(result.exit_code == 0);
        }
        // If it fails, it should be with a real error, not fabricated
    }

    #[test]
    fn test_real_lscpu_command() {
        let exec = CommandExec::new();

        let result = exec.execute("lscpu", &[]);

        if result.status == ExecutionStatus::Success {
            // lscpu output should have real CPU info
            assert!(result.stdout.contains("CPU") || result.stdout.contains("Architecture"));
        }
    }

    #[test]
    fn test_pacman_query() {
        let exec = CommandExec::new();

        let result = exec.execute("pacman_query", &["-Q", "pacman"]);

        // On Arch, this should succeed
        if result.status == ExecutionStatus::Success {
            assert!(result.stdout.contains("pacman"));
        }
    }

    // ===== v6.58.0 HONESTY TESTS =====
    // These tests verify that Anna never fabricates command output

    #[test]
    fn test_honesty_free_output_is_real() {
        let exec = CommandExec::new();
        let result = exec.execute("free", &["-m"]);

        if result.status == ExecutionStatus::Success {
            // Real free output MUST have "Mem:" line
            assert!(result.stdout.contains("Mem:"), "free output missing Mem: line");
            // Real output has numeric values
            assert!(result.stdout.chars().any(|c| c.is_ascii_digit()),
                    "free output has no numbers - fabricated?");
        }
    }

    #[test]
    fn test_honesty_df_output_is_real() {
        let exec = CommandExec::new();
        let result = exec.execute("df", &["-h"]);

        if result.status == ExecutionStatus::Success {
            // Real df output has headers
            assert!(result.stdout.contains("Filesystem") || result.stdout.contains("Use%"),
                    "df output missing headers");
            // Real df has mount points starting with /
            assert!(result.stdout.contains("/"), "df output has no mount points");
        }
    }

    #[test]
    fn test_honesty_uptime_output_is_real() {
        let exec = CommandExec::new();
        let result = exec.execute("uptime", &[]);

        if result.status == ExecutionStatus::Success {
            // Real uptime shows "up" or "load average"
            assert!(result.stdout.contains("up") || result.stdout.contains("load"),
                    "uptime output doesn't look real");
        }
    }

    #[test]
    fn test_honesty_uname_output_is_real() {
        let exec = CommandExec::new();
        let result = exec.execute("uname", &["-a"]);

        if result.status == ExecutionStatus::Success {
            // Real uname shows Linux
            assert!(result.stdout.contains("Linux") || result.stdout.contains("linux"),
                    "uname output doesn't show Linux");
        }
    }

    // ===== v6.58.0 FORBIDDEN STRING TESTS =====
    // These strings MUST NEVER appear in Anna's output - they indicate fabrication

    const FORBIDDEN_FABRICATIONS: &[&str] = &[
        "systemd-cryptgen",           // Invented command
        "pacman -Qqe --needupgrades", // Invented flag
        "--type=imaginary",           // Invented systemctl option
        "free -m -m",                 // Double flag nonsense
        "/usr/bin does not exist",    // Impossible claim
        "command not found: pacman",  // Wrong format (should be "pacman: command not found")
        "Error: unable to contact",   // Vague fabricated error
        "WARNING: Critical error",    // Nonsensical combination
    ];

    #[test]
    fn test_no_fabricated_strings_in_free() {
        let exec = CommandExec::new();
        let result = exec.execute("free", &["-m"]);

        for forbidden in FORBIDDEN_FABRICATIONS {
            assert!(!result.stdout.contains(forbidden),
                    "Fabricated string '{}' found in free stdout", forbidden);
            assert!(!result.stderr.contains(forbidden),
                    "Fabricated string '{}' found in free stderr", forbidden);
        }
    }

    #[test]
    fn test_no_fabricated_strings_in_lscpu() {
        let exec = CommandExec::new();
        let result = exec.execute("lscpu", &[]);

        for forbidden in FORBIDDEN_FABRICATIONS {
            assert!(!result.stdout.contains(forbidden),
                    "Fabricated string '{}' found in lscpu stdout", forbidden);
            assert!(!result.stderr.contains(forbidden),
                    "Fabricated string '{}' found in lscpu stderr", forbidden);
        }
    }

    #[test]
    fn test_catalog_rejects_fabricated_commands() {
        let exec = CommandExec::new();

        // All these should be rejected as NotInCatalog
        let fabricated_commands = vec![
            "systemd-cryptgen",
            "pacman-update",
            "arch-install-wizard",
            "linux-config-tool",
            "fake-system-tool",
            "rm",   // Dangerous and not in catalog
            "dd",   // Dangerous and not in catalog
            "wget", // Not in diagnostic catalog
        ];

        for cmd in fabricated_commands {
            let result = exec.execute(cmd, &[]);
            assert_eq!(result.status, ExecutionStatus::NotInCatalog,
                       "Fabricated command '{}' was not rejected", cmd);
        }
    }

    #[test]
    fn test_real_error_format() {
        let exec = CommandExec::new();

        // Try to run a catalog command with bad args
        // Real errors should have consistent format
        let result = exec.execute("ls", &["/nonexistent/path/that/does/not/exist"]);

        if result.status != ExecutionStatus::Success {
            // Real "not found" errors have specific format
            let is_real_error = result.stderr.contains("No such file") ||
                               result.stderr.contains("cannot access") ||
                               result.stderr.is_empty(); // ls might not error on missing path

            assert!(is_real_error || result.status == ExecutionStatus::Success,
                    "Error doesn't match real format: {}", result.stderr);
        }
    }
}

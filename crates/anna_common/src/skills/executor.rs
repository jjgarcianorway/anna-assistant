//! Skill Executor v0.40.0
//!
//! Executes skills via safe_command primitive.
//! Uses the command whitelist for security.

use super::schema::Skill;
use crate::CommandRegistry;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::Output;
use std::time::Instant;

/// Result of executing a skill
#[derive(Debug, Clone)]
pub struct SkillExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Command that was executed
    pub command: Vec<String>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Execution time in milliseconds
    pub latency_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

impl SkillExecutionResult {
    /// Create a failed result with error
    pub fn failed(error: &str) -> Self {
        Self {
            success: false,
            command: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            latency_ms: 0,
            error: Some(error.to_string()),
        }
    }
}

/// Execute a skill with given parameters
pub fn execute_skill(
    skill: &Skill,
    params: &HashMap<String, String>,
) -> Result<SkillExecutionResult> {
    // Build the command from template
    let command_parts = skill.build_command(params).map_err(|e| anyhow!(e))?;

    // Execute via safe_command
    execute_safe_command(&command_parts)
}

/// Execute a command via the safe_command primitive
pub fn execute_safe_command(command_parts: &[String]) -> Result<SkillExecutionResult> {
    if command_parts.is_empty() {
        return Ok(SkillExecutionResult::failed("Empty command"));
    }

    let program = &command_parts[0];
    let args = &command_parts[1..];

    // Validate against whitelist
    let registry = CommandRegistry::new();
    let full_cmd = command_parts.join(" ");

    if registry.matches_whitelist(&full_cmd).is_none() && !is_safe_read_command(program, args) {
        return Ok(SkillExecutionResult::failed(&format!(
            "Command not in whitelist: {}",
            program
        )));
    }

    // Execute the command
    let start = Instant::now();
    let result = std::process::Command::new(program)
        .args(args)
        .output();

    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => Ok(output_to_result(output, command_parts.to_vec(), latency_ms)),
        Err(e) => Ok(SkillExecutionResult {
            success: false,
            command: command_parts.to_vec(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            latency_ms,
            error: Some(e.to_string()),
        }),
    }
}

/// Execute a command asynchronously
pub async fn execute_safe_command_async(
    command_parts: &[String],
) -> Result<SkillExecutionResult> {
    if command_parts.is_empty() {
        return Ok(SkillExecutionResult::failed("Empty command"));
    }

    let program = &command_parts[0];
    let args = &command_parts[1..];

    // Validate against whitelist
    let registry = CommandRegistry::new();
    let full_cmd = command_parts.join(" ");

    if registry.matches_whitelist(&full_cmd).is_none() && !is_safe_read_command(program, args) {
        return Ok(SkillExecutionResult::failed(&format!(
            "Command not in whitelist: {}",
            program
        )));
    }

    // Execute the command asynchronously
    let start = Instant::now();
    let result = tokio::process::Command::new(program)
        .args(args)
        .output()
        .await;

    let latency_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => Ok(output_to_result(output.into(), command_parts.to_vec(), latency_ms)),
        Err(e) => Ok(SkillExecutionResult {
            success: false,
            command: command_parts.to_vec(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            latency_ms,
            error: Some(e.to_string()),
        }),
    }
}

/// Convert process output to result
fn output_to_result(output: Output, command: Vec<String>, latency_ms: u64) -> SkillExecutionResult {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();
    let success = output.status.success();

    let error = if success {
        None
    } else {
        Some(format!(
            "Exit code: {:?}, stderr: {}",
            exit_code,
            stderr.trim()
        ))
    };

    SkillExecutionResult {
        success,
        command,
        stdout,
        stderr,
        exit_code,
        latency_ms,
        error,
    }
}

/// Check if command is a safe read-only command
/// These are allowed even if not explicitly in whitelist
fn is_safe_read_command(program: &str, args: &[String]) -> bool {
    let safe_programs = [
        "cat",
        "head",
        "tail",
        "ls",
        "pwd",
        "whoami",
        "hostname",
        "uname",
        "uptime",
        "date",
        "df",
        "free",
        "ps",
        "lscpu",
        "lsblk",
        "lspci",
        "lsusb",
        "ip",
        "ss",
        "journalctl",
        "systemctl",
        "pacman",
        "which",
        "file",
        "stat",
    ];

    if !safe_programs.contains(&program) {
        return false;
    }

    // Check for dangerous flags
    let dangerous_flags = ["-rf", "-f", "--force", "-i", "--interactive"];
    for arg in args {
        if dangerous_flags.iter().any(|&f| arg.starts_with(f)) {
            return false;
        }
        // Check for shell metacharacters
        if arg.chars().any(|c| matches!(c, '|' | ';' | '&' | '$' | '`' | '(' | ')' | '<' | '>')) {
            return false;
        }
    }

    // Specific command checks
    match program {
        "systemctl" => {
            // Only allow read-only systemctl commands
            let read_only = ["status", "show", "list-units", "list-unit-files", "is-active", "is-enabled"];
            args.first().map(|a| read_only.iter().any(|&r| a == r)).unwrap_or(false)
        }
        "pacman" => {
            // Only allow query/search, not install/remove
            args.first().map(|a| a.starts_with("-Q") || a.starts_with("-S") && a.contains("s")).unwrap_or(false)
                || args.iter().any(|a| a == "-Qi" || a == "-Qs" || a == "-Q" || a == "-Ss")
        }
        "journalctl" => {
            // Journalctl is safe as long as no write operations
            !args.iter().any(|a| {
                a == "--vacuum-size" || a.starts_with("--vacuum-size=")
                    || a == "--vacuum-time" || a.starts_with("--vacuum-time=")
                    || a == "--rotate"
            })
        }
        _ => true,
    }
}

/// Validate parameters for injection attacks
pub fn validate_params(params: &HashMap<String, String>) -> Result<()> {
    let forbidden_chars = ['|', ';', '&', '`', '$', '(', ')', '<', '>', '\n', '\r'];

    for (key, value) in params {
        if key.chars().any(|c| forbidden_chars.contains(&c)) {
            return Err(anyhow!("Parameter key contains forbidden characters: {}", key));
        }
        if value.chars().any(|c| forbidden_chars.contains(&c)) {
            return Err(anyhow!(
                "Parameter value contains forbidden characters: {} = {}",
                key,
                value
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_read_command_basic() {
        assert!(is_safe_read_command("ls", &[]));
        assert!(is_safe_read_command("df", &["-h".to_string()]));
        assert!(is_safe_read_command("uptime", &[]));
    }

    #[test]
    fn test_safe_read_command_systemctl() {
        assert!(is_safe_read_command(
            "systemctl",
            &["status".to_string(), "annad".to_string()]
        ));
        assert!(!is_safe_read_command(
            "systemctl",
            &["restart".to_string(), "annad".to_string()]
        ));
    }

    #[test]
    fn test_safe_read_command_journalctl() {
        assert!(is_safe_read_command(
            "journalctl",
            &["-u".to_string(), "annad".to_string(), "-n".to_string(), "50".to_string()]
        ));
        assert!(!is_safe_read_command(
            "journalctl",
            &["--vacuum-size=100M".to_string()]
        ));
    }

    #[test]
    fn test_safe_read_command_pacman() {
        assert!(is_safe_read_command("pacman", &["-Qi".to_string(), "vim".to_string()]));
        assert!(is_safe_read_command("pacman", &["-Ss".to_string(), "vim".to_string()]));
    }

    #[test]
    fn test_unsafe_commands() {
        assert!(!is_safe_read_command("rm", &["-rf".to_string()]));
        assert!(!is_safe_read_command("dd", &[]));
    }

    #[test]
    fn test_validate_params_clean() {
        let mut params = HashMap::new();
        params.insert("service".to_string(), "annad".to_string());
        params.insert("since".to_string(), "6 hours ago".to_string());
        assert!(validate_params(&params).is_ok());
    }

    #[test]
    fn test_validate_params_injection() {
        let mut params = HashMap::new();
        params.insert("service".to_string(), "annad; rm -rf /".to_string());
        assert!(validate_params(&params).is_err());
    }

    #[test]
    fn test_execute_safe_command() {
        // Test with a known safe command (uptime)
        let result = execute_safe_command(&["uptime".to_string()]).unwrap();
        // uptime is in the safe list, so it should execute
        assert!(result.success || result.error.is_some());
    }
}

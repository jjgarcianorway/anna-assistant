//! Executor Core - Safe command execution with capturing
//!
//! v6.41.0: The Executor takes command plans from the Planner and runs them
//! on the real system, with safety checks and output capturing.

use crate::planner_core::{CommandPlan, PlannedCommand, SafetyLevel};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

/// Result of executing a command plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Original plan that was executed
    pub plan: CommandPlan,

    /// Results from each command
    pub command_results: Vec<CommandResult>,

    /// Overall success flag
    pub success: bool,

    /// Overall execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Result of executing a single command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Command that was run
    pub command: String,

    /// Full command line (for display)
    pub full_command: String,

    /// Exit code
    pub exit_code: i32,

    /// stdout
    pub stdout: String,

    /// stderr
    pub stderr: String,

    /// Whether command succeeded
    pub success: bool,

    /// Execution time in milliseconds
    pub time_ms: u64,
}

/// Tool inventory - what's available on this system
#[derive(Debug, Clone)]
pub struct ToolInventory {
    /// Package managers
    pub package_managers: Vec<String>,

    /// Other tools
    pub tools: Vec<String>,
}

impl ToolInventory {
    /// Detect what tools are available on this system
    pub fn detect() -> Self {
        let mut package_managers = Vec::new();
        let mut tools = Vec::new();

        // Check package managers
        for pm in &["pacman", "yay", "paru", "apt", "dnf", "zypper", "flatpak", "snap"] {
            if check_command_exists(pm) {
                package_managers.push(pm.to_string());
            }
        }

        // Check common tools
        for tool in &["grep", "awk", "sed", "du", "df", "find", "ps", "systemctl", "lscpu", "lspci", "free"] {
            if check_command_exists(tool) {
                tools.push(tool.to_string());
            }
        }

        Self {
            package_managers,
            tools,
        }
    }

    /// Get all available tools as a single list
    pub fn all_tools(&self) -> Vec<String> {
        let mut all = self.package_managers.clone();
        all.extend(self.tools.clone());
        all
    }

    /// Check if a tool is available
    pub fn has_tool(&self, tool: &str) -> bool {
        self.package_managers.contains(&tool.to_string()) || self.tools.contains(&tool.to_string())
    }
}

/// Execute a command plan safely
pub fn execute_plan(plan: &CommandPlan) -> Result<ExecutionResult> {
    let start = std::time::Instant::now();

    // Safety check
    if plan.safety_level == SafetyLevel::Risky {
        return Err(anyhow!("Refusing to execute risky command plan"));
    }

    let inventory = ToolInventory::detect();
    let mut command_results = Vec::new();
    let mut all_success = true;

    for planned_cmd in &plan.commands {
        // Check if required tools exist
        let mut missing_tools = Vec::new();
        for required_tool in &planned_cmd.requires_tools {
            if !inventory.has_tool(required_tool) {
                missing_tools.push(required_tool.clone());
            }
        }

        if !missing_tools.is_empty() {
            // Skip this command, tool not available
            command_results.push(CommandResult {
                command: planned_cmd.command.clone(),
                full_command: format_full_command(planned_cmd),
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Required tools not found: {}", missing_tools.join(", ")),
                success: false,
                time_ms: 0,
            });
            all_success = false;
            continue;
        }

        // Execute command
        match execute_command(planned_cmd) {
            Ok(result) => {
                if !result.success {
                    all_success = false;
                }
                command_results.push(result);
            }
            Err(e) => {
                command_results.push(CommandResult {
                    command: planned_cmd.command.clone(),
                    full_command: format_full_command(planned_cmd),
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Execution error: {}", e),
                    success: false,
                    time_ms: 0,
                });
                all_success = false;
            }
        }
    }

    // Try fallbacks if primary commands failed
    if !all_success && !plan.fallbacks.is_empty() {
        for fallback_cmd in &plan.fallbacks {
            // Check if tools exist
            if fallback_cmd.requires_tools.iter().all(|t| inventory.has_tool(t)) {
                if let Ok(result) = execute_command(fallback_cmd) {
                    command_results.push(result);
                    // Don't override all_success - fallbacks are optional
                }
            }
        }
    }

    let execution_time_ms = start.elapsed().as_millis() as u64;

    Ok(ExecutionResult {
        plan: plan.clone(),
        command_results,
        success: all_success,
        execution_time_ms,
    })
}

/// Execute a single planned command
fn execute_command(planned: &PlannedCommand) -> Result<CommandResult> {
    let start = std::time::Instant::now();

    // Build command
    let output = Command::new(&planned.command)
        .args(&planned.args)
        .output()?;

    let time_ms = start.elapsed().as_millis() as u64;

    Ok(CommandResult {
        command: planned.command.clone(),
        full_command: format_full_command(planned),
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
        time_ms,
    })
}

/// Format a command for display
fn format_full_command(planned: &PlannedCommand) -> String {
    if planned.args.is_empty() {
        planned.command.clone()
    } else {
        format!("{} {}", planned.command, planned.args.join(" "))
    }
}

/// Check if a command exists
fn check_command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Validate command safety (no destructive operations)
pub fn is_command_safe(command: &str, args: &[String]) -> bool {
    // Blacklist of dangerous commands
    let dangerous_commands = ["rm", "dd", "mkfs", "fdisk", "parted", "shutdown", "reboot"];

    if dangerous_commands.contains(&command) {
        return false;
    }

    // Blacklist of dangerous arguments
    for arg in args {
        if arg.contains("--remove") || arg.contains("-R") || arg.contains("--delete") || arg.contains("-f") {
            // Be careful with force/remove flags
            if command == "pacman" || command == "yay" || command == "paru" {
                return false; // No package removal
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner_core::{CommandPlan, PlannedCommand, SafetyLevel};

    #[test]
    fn test_tool_inventory() {
        let inventory = ToolInventory::detect();
        // At least grep and ps should exist on any Linux system
        assert!(inventory.has_tool("grep") || inventory.has_tool("ps"));
    }

    #[test]
    fn test_command_safety() {
        assert!(is_command_safe("pacman", &["-Q".to_string()]));
        assert!(!is_command_safe("rm", &["-rf".to_string(), "/".to_string()]));
        assert!(!is_command_safe("pacman", &["-R".to_string(), "package".to_string()]));
    }

    #[test]
    fn test_execute_safe_command() {
        let planned = PlannedCommand {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            purpose: "test command".to_string(),
            requires_tools: vec![],
        };

        let result = execute_command(&planned);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_format_full_command() {
        let planned = PlannedCommand {
            command: "pacman".to_string(),
            args: vec!["-Q".to_string(), "steam".to_string()],
            purpose: "check steam".to_string(),
            requires_tools: vec!["pacman".to_string()],
        };

        assert_eq!(format_full_command(&planned), "pacman -Q steam");
    }
}

//! Command Validation Layer - v6.44.0
//!
//! Strict validation of planned commands before execution.
//! Ensures tools exist, syntax is sane, and operations are safe.

use crate::planner_core::PlannedCommand;
use crate::tool_inventory::ToolInventory;
use thiserror::Error;

/// A validated command that has passed all safety checks
#[derive(Debug, Clone)]
pub struct ValidatedCommand {
    /// Full command line as it will be executed
    pub cmdline: String,

    /// Primary tool being executed (e.g., "pacman", "free")
    pub tool: String,

    /// Arguments to the tool
    pub args: Vec<String>,
}

/// Errors that can occur during command validation
#[derive(Debug, Error, Clone)]
pub enum CommandValidationError {
    /// Tool not found in inventory
    #[error("Unknown tool: {0} (not installed or not in PATH)")]
    UnknownTool(String),

    /// Suspicious or malformed syntax
    #[error("Suspicious syntax: {0}")]
    SuspiciousSyntax(String),

    /// Operation not allowed in current safety mode
    #[error("Forbidden operation: {0}")]
    ForbiddenOperation(String),

    /// Empty command
    #[error("Empty command")]
    EmptyCommand,
}

/// Validate a planned command against tool inventory and safety rules
pub fn validate_planned_command(
    plan: &PlannedCommand,
    inventory: &ToolInventory,
) -> Result<ValidatedCommand, CommandValidationError> {
    // Check for empty command
    if plan.command.is_empty() {
        return Err(CommandValidationError::EmptyCommand);
    }

    // Extract tool name (handle shell invocations like "sh -c")
    let tool = extract_primary_tool(&plan.command, &plan.args);

    // Build full command line
    let cmdline = if plan.args.is_empty() {
        plan.command.clone()
    } else {
        format!("{} {}", plan.command, plan.args.join(" "))
    };

    // Check operation safety FIRST (v6.44.0: read-only only)
    // This way forbidden operations are rejected even if tool doesn't exist
    validate_operation_safety(&tool, &plan.args, &cmdline)?;

    // Check tool existence
    validate_tool_exists(&tool, inventory)?;

    // Check syntax sanity
    validate_syntax(&cmdline)?;

    Ok(ValidatedCommand {
        cmdline,
        tool,
        args: plan.args.clone(),
    })
}

/// Extract the primary tool from command and args
/// Handles cases like "sh -c 'pacman -Qq | grep games'"
fn extract_primary_tool(command: &str, args: &[String]) -> String {
    // If command is a shell invocation, extract the first tool in the pipeline
    if command == "sh" && args.len() >= 2 && args[0] == "-c" {
        // Parse the shell command string
        if let Some(shell_cmd) = args.get(1) {
            // Extract first word before space or pipe
            let first_tool = shell_cmd
                .split_whitespace()
                .next()
                .unwrap_or(command);
            return first_tool.to_string();
        }
    }

    command.to_string()
}

/// Validate that the tool exists in the inventory
fn validate_tool_exists(tool: &str, inventory: &ToolInventory) -> Result<(), CommandValidationError> {
    // Check if tool is in the inventory
    // ToolInventory has: pacman, yay, paru, flatpak, steam, lscpu, lspci, grep, awk, free, etc.

    // Standard system tools always available
    let always_available = ["sh", "bash", "cat", "grep", "awk", "sed", "head", "tail"];
    if always_available.contains(&tool) {
        return Ok(());
    }

    // Check package managers
    if tool == "pacman" && inventory.pacman {
        return Ok(());
    }
    if tool == "yay" && inventory.yay {
        return Ok(());
    }
    if tool == "paru" && inventory.paru {
        return Ok(());
    }
    if tool == "flatpak" && inventory.flatpak {
        return Ok(());
    }

    // Check system info tools
    if tool == "lscpu" && inventory.lscpu {
        return Ok(());
    }
    if tool == "lspci" && inventory.lspci {
        return Ok(());
    }
    if tool == "free" && inventory.free {
        return Ok(());
    }

    // Check other tools
    if tool == "steam" && inventory.steam {
        return Ok(());
    }
    if tool == "systemctl" && inventory.systemctl {
        return Ok(());
    }

    Err(CommandValidationError::UnknownTool(tool.to_string()))
}

/// Validate syntax for common issues
fn validate_syntax(cmdline: &str) -> Result<(), CommandValidationError> {
    // Check for empty pipes
    if cmdline.contains("| |") || cmdline.ends_with('|') || cmdline.starts_with('|') {
        return Err(CommandValidationError::SuspiciousSyntax(
            "Empty or trailing pipe".to_string()
        ));
    }

    // Check for trailing flags after awk/sed output redirects (classic LLM mistake)
    // Example: "free -m | awk '{print $3}' -m" is invalid
    if cmdline.contains("awk '{print") || cmdline.contains("awk {print") {
        let after_awk = cmdline.split("awk").nth(1).unwrap_or("");
        // Check if there's a closing } or }' followed by a flag
        if after_awk.contains("}' -") || after_awk.contains("} -") {
            return Err(CommandValidationError::SuspiciousSyntax(
                "Trailing flag after awk output block (e.g., awk '{print $3}' -m)".to_string()
            ));
        }
    }

    // Check for pacman piped to grep when pacman -Qs should be used
    // Example: "pacman -Q | grep games" should be "pacman -Qs games"
    if cmdline.contains("pacman -Q |") && cmdline.contains("grep") && !cmdline.contains("pacman -Qq") {
        return Err(CommandValidationError::SuspiciousSyntax(
            "Use 'pacman -Qs <pattern>' instead of 'pacman -Q | grep <pattern>'".to_string()
        ));
    }

    // Check for grep without pattern in pipe
    if cmdline.contains("| grep") {
        let after_grep = cmdline.split("| grep").nth(1).unwrap_or("").trim();
        if after_grep.is_empty() || after_grep.starts_with('|') {
            return Err(CommandValidationError::SuspiciousSyntax(
                "grep without pattern in pipe".to_string()
            ));
        }
    }

    Ok(())
}

/// Validate operation safety
/// v6.44.0: Only read-only operations allowed
fn validate_operation_safety(tool: &str, args: &[String], cmdline: &str) -> Result<(), CommandValidationError> {
    // Forbidden write operations
    let forbidden_write_tools = ["rm", "mv", "cp", "dd", "mkfs", "fdisk", "parted"];
    if forbidden_write_tools.contains(&tool) {
        return Err(CommandValidationError::ForbiddenOperation(
            format!("{} is a write operation (forbidden in v6.44.0)", tool)
        ));
    }

    // Package manager write operations
    if tool == "pacman" || tool == "yay" || tool == "paru" {
        // Check for install/remove/update flags
        let write_flags = ["-S", "-R", "-U", "--sync", "--remove", "--upgrade"];
        for arg in args {
            if write_flags.contains(&arg.as_str()) {
                return Err(CommandValidationError::ForbiddenOperation(
                    format!("{} {} is a package modification (forbidden in v6.44.0)", tool, arg)
                ));
            }
        }
    }

    // File editors
    let forbidden_editors = ["nano", "vim", "vi", "emacs", "ed"];
    if forbidden_editors.contains(&tool) {
        return Err(CommandValidationError::ForbiddenOperation(
            format!("{} is a file editor (forbidden in v6.44.0)", tool)
        ));
    }

    // Shell redirects
    if cmdline.contains('>') && !cmdline.contains("/dev/null") {
        return Err(CommandValidationError::ForbiddenOperation(
            "File redirection forbidden (use read-only commands)".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_inventory() -> ToolInventory {
        let mut inv = ToolInventory::default();
        inv.pacman = true;
        inv.lscpu = true;
        inv.lspci = true;
        inv.free = true;
        inv.systemctl = true;
        inv.grep = true;
        inv.awk = true;
        inv
    }

    #[test]
    fn test_accept_valid_free() {
        let plan = PlannedCommand {
            command: "free".to_string(),
            args: vec!["-m".to_string()],
            purpose: "Check RAM".to_string(),
            requires_tools: vec!["free".to_string()],
            risk_level: crate::planner_core::StepRiskLevel::ReadOnly,
            writes_files: false,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().cmdline, "free -m");
    }

    #[test]
    fn test_reject_broken_awk_flag() {
        // The infamous "free -m | awk '{print $3}' -m" bug
        let plan = PlannedCommand {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "free -m | awk '{print $3}' -m".to_string()],
            purpose: "Get RAM".to_string(),
            requires_tools: vec!["free".to_string(), "awk".to_string()],
            risk_level: crate::planner_core::StepRiskLevel::ReadOnly,
            writes_files: false,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandValidationError::SuspiciousSyntax(_)));
    }

    #[test]
    fn test_reject_pacman_pipe_grep() {
        // Should use "pacman -Qs games" not "pacman -Q | grep games"
        let plan = PlannedCommand {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "pacman -Q | grep games".to_string()],
            purpose: "Find games".to_string(),
            requires_tools: vec!["pacman".to_string()],
            risk_level: crate::planner_core::StepRiskLevel::ReadOnly,
            writes_files: false,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandValidationError::SuspiciousSyntax(_)));
    }

    #[test]
    fn test_reject_unknown_tool() {
        let plan = PlannedCommand {
            command: "nonexistent_tool".to_string(),
            args: vec![],
            purpose: "Test".to_string(),
            requires_tools: vec![],
            risk_level: crate::planner_core::StepRiskLevel::ReadOnly,
            writes_files: false,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandValidationError::UnknownTool(_)));
    }

    #[test]
    fn test_reject_write_operation_rm() {
        let plan = PlannedCommand {
            command: "rm".to_string(),
            args: vec!["-rf".to_string(), "/tmp/test".to_string()],
            purpose: "Delete files".to_string(),
            requires_tools: vec![],
            risk_level: crate::planner_core::StepRiskLevel::High,
            writes_files: true,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandValidationError::ForbiddenOperation(_)));
    }

    #[test]
    fn test_reject_pacman_install() {
        let plan = PlannedCommand {
            command: "pacman".to_string(),
            args: vec!["-S".to_string(), "steam".to_string()],
            purpose: "Install steam".to_string(),
            requires_tools: vec!["pacman".to_string()],
            risk_level: crate::planner_core::StepRiskLevel::High,
            writes_files: true,
            requires_root: true,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CommandValidationError::ForbiddenOperation(_)));
    }

    #[test]
    fn test_accept_valid_lscpu() {
        let plan = PlannedCommand {
            command: "lscpu".to_string(),
            args: vec![],
            purpose: "Check CPU".to_string(),
            requires_tools: vec!["lscpu".to_string()],
            risk_level: crate::planner_core::StepRiskLevel::ReadOnly,
            writes_files: false,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_ok());
    }

    #[test]
    fn test_accept_grep_proc_cpuinfo() {
        let plan = PlannedCommand {
            command: "grep".to_string(),
            args: vec!["-i".to_string(), "sse2".to_string(), "/proc/cpuinfo".to_string()],
            purpose: "Check CPU flags".to_string(),
            requires_tools: vec!["grep".to_string()],
            risk_level: crate::planner_core::StepRiskLevel::ReadOnly,
            writes_files: false,
            requires_root: false,
            expected_outcome: None,
            validation_hint: None,
        };

        let result = validate_planned_command(&plan, &mock_inventory());
        assert!(result.is_ok());
    }
}

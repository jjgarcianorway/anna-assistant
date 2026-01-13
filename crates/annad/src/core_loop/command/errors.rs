//! Error classification and recovery hints for command failures.

use super::types::CommandErrorType;

/// Classify error to enable intelligent recovery
/// v0.0.932: Expanded from 8 to 20+ error patterns
pub fn classify_command_error(
    output: &str,
    error: Option<&str>,
) -> (CommandErrorType, &'static str) {
    let combined = format!("{} {}", output, error.unwrap_or("")).to_lowercase();

    // Command not found patterns
    if combined.contains("command not found")
        || combined.contains("not found in path")
        || combined.contains("no such command")
        || combined.contains("not recognized as")
        || combined.contains(": not found")
    {
        return (
            CommandErrorType::CommandNotFound,
            "Install the package or use an alternative command",
        );
    }

    // Permission denied patterns
    if combined.contains("permission denied")
        || combined.contains("operation not permitted")
        || combined.contains("access denied")
        || combined.contains("not permitted")
        || combined.contains("insufficient permissions")
        || combined.contains("must be root")
        || combined.contains("requires root")
        || combined.contains("need to be root")
    {
        return (
            CommandErrorType::PermissionDenied,
            "Try with sudo or check file permissions",
        );
    }

    // Path not found patterns
    if combined.contains("no such file")
        || combined.contains("does not exist")
        || combined.contains("cannot find")
        || combined.contains("failed to open")
        || combined.contains("cannot access")
        || combined.contains("not a directory")
        || combined.contains("is a directory")
        || combined.contains("cannot stat")
    {
        return (
            CommandErrorType::PathNotFound,
            "Check if path exists or use correct location",
        );
    }

    // Timeout patterns
    if combined.contains("timed out")
        || combined.contains("timeout")
        || combined.contains("connection timed out")
        || combined.contains("read timed out")
        || combined.contains("operation timed out")
    {
        return (
            CommandErrorType::Timeout,
            "Command took too long - try a simpler query",
        );
    }

    // Syntax error patterns
    if combined.contains("syntax error")
        || combined.contains("invalid option")
        || combined.contains("unknown option")
        || combined.contains("unrecognized option")
        || combined.contains("illegal option")
        || combined.contains("bad flag")
        || combined.contains("missing argument")
        || combined.contains("requires an argument")
        || combined.contains("unexpected token")
        || combined.contains("parse error")
    {
        return (
            CommandErrorType::SyntaxError,
            "Fix command syntax or flags",
        );
    }

    // Missing dependency patterns
    if combined.contains("dependency")
        || combined.contains("not installed")
        || combined.contains("package not found")
        || combined.contains("unable to locate package")
        || combined.contains("no package")
        || combined.contains("missing library")
        || combined.contains("cannot load")
        || combined.contains("shared object")
    {
        return (
            CommandErrorType::MissingDependency,
            "Install required dependency first",
        );
    }

    // Empty output
    if output.trim().is_empty() {
        return (
            CommandErrorType::EmptyOutput,
            "Command produced no output",
        );
    }

    // Additional common error patterns that map to existing types
    if combined.contains("connection refused")
        || combined.contains("network unreachable")
        || combined.contains("host unreachable")
        || combined.contains("name resolution")
    {
        return (
            CommandErrorType::Unknown,
            "Network error - check connectivity",
        );
    }

    if combined.contains("disk full")
        || combined.contains("no space left")
        || combined.contains("out of memory")
        || combined.contains("cannot allocate")
    {
        return (
            CommandErrorType::Unknown,
            "Resource exhaustion - free up space/memory",
        );
    }

    if combined.contains("device busy")
        || combined.contains("resource busy")
        || combined.contains("cannot unmount")
    {
        return (
            CommandErrorType::Unknown,
            "Resource is busy - try closing related apps",
        );
    }

    (
        CommandErrorType::Unknown,
        "Unknown error - try alternative command",
    )
}

/// Get recovery hint based on error type
pub fn get_recovery_prompt(error_type: &CommandErrorType, cmd: &str) -> String {
    let base_cmd = cmd.split_whitespace().next().unwrap_or(cmd);
    match error_type {
        CommandErrorType::CommandNotFound => format!(
            "Command '{}' not installed. Suggest the Arch package or alternative.",
            base_cmd
        ),
        CommandErrorType::PermissionDenied => format!(
            "Permission denied for '{}'. Suggest sudo or permission fix.",
            cmd
        ),
        CommandErrorType::PathNotFound => format!(
            "Path not found in '{}'. Suggest how to find correct path.",
            cmd
        ),
        CommandErrorType::Timeout => {
            let hint = match base_cmd {
                "find" => "Use 'locate' or add '-maxdepth 2'",
                "grep" | "rg" => "Add 'head -20' to limit output",
                "du" => "Use 'du -d1' or 'df'",
                "journalctl" => "Add '--since \"1 hour ago\"' or '--lines=50'",
                _ => "Try with 'timeout 5s' or limit output",
            };
            format!("Command '{}' timed out. {}.", cmd, hint)
        }
        CommandErrorType::SyntaxError => {
            format!("Syntax error in '{}'. Fix flags/syntax.", cmd)
        }
        CommandErrorType::MissingDependency => {
            format!("Missing dependency for '{}'. Suggest install.", cmd)
        }
        CommandErrorType::EmptyOutput => {
            format!("No output from '{}'. Suggest alternative.", cmd)
        }
        CommandErrorType::Unknown => format!("Command '{}' failed. Suggest alternative.", cmd),
    }
}

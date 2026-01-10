//! Resolution state machine and tracking types.

/// v0.0.899: Resolution state machine for intelligent retry logic
#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionState {
    /// Initial state: gathering command output
    Gathering,
    /// Have output, checking if sufficient for answer
    Validating,
    /// Command failed, attempting recovery with diagnostics
    Recovering {
        error_type: CommandErrorType,
        attempts: u8,
    },
    /// Answer generated but validation failed, refining
    Refining { issues: String, attempts: u8 },
    /// Successfully converged on answer
    Complete,
    /// Unrecoverable error or max attempts reached
    Failed { reason: String },
}

impl ResolutionState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ResolutionState::Complete | ResolutionState::Failed { .. }
        )
    }

    pub fn can_continue(&self) -> bool {
        match self {
            ResolutionState::Recovering { attempts, .. } => *attempts < 3,
            ResolutionState::Refining { attempts, .. } => *attempts < 2,
            ResolutionState::Failed { .. } => false,
            _ => true,
        }
    }
}

/// v0.0.903: Track tried commands across iterations to prevent loops
#[derive(Debug, Clone, Default)]
pub struct TriedCommands {
    commands: Vec<String>,
}

impl TriedCommands {
    pub fn add(&mut self, cmd: &str) {
        let normalized = cmd
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        if !self
            .commands
            .iter()
            .any(|c| c == &normalized || cmd.starts_with(c))
        {
            self.commands.push(normalized);
        }
    }

    pub fn has_tried(&self, cmd: &str) -> bool {
        let normalized = cmd
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        self.commands
            .iter()
            .any(|c| c == &normalized || cmd.starts_with(c) || normalized.starts_with(c))
    }

    pub fn as_exclusion_hint(&self) -> String {
        if self.commands.is_empty() {
            String::new()
        } else {
            format!(
                "DO NOT suggest these commands (already tried and failed): {}",
                self.commands.join(", ")
            )
        }
    }
}

/// v0.0.897: Error categories for intelligent recovery
#[derive(Debug, Clone, PartialEq)]
pub enum CommandErrorType {
    /// Command not found (typo, not installed)
    NotFound,
    /// Permission denied (needs sudo, wrong user)
    PermissionDenied,
    /// Resource busy/locked (file in use, process running)
    ResourceBusy,
    /// Network error (DNS, connection refused)
    NetworkError,
    /// Timeout (slow command, hung process)
    Timeout,
    /// Generic/unknown error
    Unknown,
}

/// v0.0.897: Classify error to enable intelligent recovery
pub fn classify_command_error(output: &str, error: Option<&str>) -> (CommandErrorType, &'static str) {
    let combined = format!("{} {}", output, error.unwrap_or(""));
    let lower = combined.to_lowercase();

    if lower.contains("command not found")
        || lower.contains("not found")
        || lower.contains("no such file or directory")
    {
        return (
            CommandErrorType::NotFound,
            "Command or file not found - check spelling or install package",
        );
    }
    if lower.contains("permission denied")
        || lower.contains("operation not permitted")
        || lower.contains("access denied")
    {
        return (
            CommandErrorType::PermissionDenied,
            "Permission issue - may need sudo or different user",
        );
    }
    if lower.contains("resource busy")
        || lower.contains("device or resource busy")
        || lower.contains("lock")
    {
        return (
            CommandErrorType::ResourceBusy,
            "Resource locked - check for other processes using it",
        );
    }
    if lower.contains("network")
        || lower.contains("connection refused")
        || lower.contains("dns")
        || lower.contains("unreachable")
    {
        return (
            CommandErrorType::NetworkError,
            "Network error - check connectivity",
        );
    }
    if lower.contains("timeout") || lower.contains("timed out") {
        return (CommandErrorType::Timeout, "Command timed out");
    }
    (CommandErrorType::Unknown, "Command failed")
}

/// v0.0.897: Get recovery hint based on error type
pub fn get_recovery_prompt(error_type: &CommandErrorType, cmd: &str) -> String {
    match error_type {
        CommandErrorType::NotFound => format!(
            "The command '{}' was not found. Suggest an alternative command that:\n\
             1. Achieves the same goal\n\
             2. Is commonly available on Arch Linux\n\
             3. Does NOT require installing new packages unless necessary",
            cmd.split_whitespace().next().unwrap_or(cmd)
        ),
        CommandErrorType::PermissionDenied => format!(
            "The command '{}' failed with permission denied. Suggest:\n\
             1. The same command with sudo if appropriate\n\
             2. An alternative that doesn't need elevated privileges\n\
             3. A way to check/fix permissions first",
            cmd
        ),
        CommandErrorType::ResourceBusy => format!(
            "The resource is busy/locked. Suggest:\n\
             1. A command to identify what's using the resource\n\
             2. A safe way to release the lock\n\
             3. An alternative approach that doesn't need exclusive access",
        ),
        CommandErrorType::NetworkError => format!(
            "Network error encountered. Suggest:\n\
             1. A command to diagnose the network issue\n\
             2. An offline alternative if available\n\
             3. A way to check DNS/connectivity",
        ),
        CommandErrorType::Timeout => format!(
            "The command '{}' timed out. Suggest:\n\
             1. A faster alternative\n\
             2. A way to run it in background\n\
             3. A way to limit its scope",
            cmd
        ),
        CommandErrorType::Unknown => format!(
            "The command '{}' failed. Suggest an alternative approach.",
            cmd
        ),
    }
}

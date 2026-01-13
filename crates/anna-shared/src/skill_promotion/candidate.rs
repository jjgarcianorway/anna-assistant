//! Recipe Candidate - Intermediate representation before promotion.
//!
//! Candidates have additional safety metadata not in the final Recipe:
//! - Preconditions to check before execution
//! - Rollback steps if something goes wrong
//! - Risk level classification

use crate::memory::Experience;
use crate::recipe::{RecipeCommand, RecipeContext, RecipeSource, VerificationStep};
use serde::{Deserialize, Serialize};

/// Risk level for a recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Read-only commands, no system changes
    Safe,
    /// Minor changes, easily reversible
    Low,
    /// System modifications, backup recommended
    Medium,
    /// Destructive or hard to reverse
    High,
    /// Potentially dangerous, requires explicit confirmation
    Critical,
}

impl RiskLevel {
    /// Determine risk level from commands
    pub fn from_commands(commands: &[String]) -> Self {
        let mut max_risk = RiskLevel::Safe;

        for cmd in commands {
            let cmd_lower = cmd.to_lowercase();
            let risk = Self::classify_command(&cmd_lower);
            if risk as u8 > max_risk as u8 {
                max_risk = risk;
            }
        }

        max_risk
    }

    fn classify_command(cmd: &str) -> Self {
        // Critical: destructive, hard to reverse
        if cmd.contains("rm -rf")
            || cmd.contains("dd if=")
            || cmd.contains("mkfs")
            || cmd.contains("> /dev/")
            || cmd.contains("pacman -Rns")
            || cmd.contains("--force")
            || cmd.contains("chmod -R 777")
        {
            return RiskLevel::Critical;
        }

        // High: system modifications
        if cmd.contains("pacman -S")
            || cmd.contains("systemctl enable")
            || cmd.contains("systemctl disable")
            || cmd.contains("useradd")
            || cmd.contains("userdel")
            || cmd.contains("passwd")
            || cmd.starts_with("sudo")
        {
            return RiskLevel::High;
        }

        // Medium: config changes
        if cmd.contains("echo") && cmd.contains(">>")
            || cmd.contains("sed -i")
            || cmd.contains("systemctl restart")
            || cmd.contains("systemctl start")
            || cmd.contains("systemctl stop")
        {
            return RiskLevel::Medium;
        }

        // Low: minor changes
        if cmd.contains("touch")
            || cmd.contains("mkdir")
            || cmd.contains("cp")
            || cmd.contains("mv")
        {
            return RiskLevel::Low;
        }

        // Default: read-only
        RiskLevel::Safe
    }
}

/// Precondition that must be satisfied before running a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precondition {
    /// Type of precondition
    pub condition_type: PreconditionType,
    /// Human-readable description
    pub description: String,
    /// Command to check (if applicable)
    pub check_command: Option<String>,
    /// Expected result (for command checks)
    pub expected: Option<String>,
}

/// Types of preconditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreconditionType {
    /// A file must exist
    FileExists(String),
    /// A file must NOT exist
    FileNotExists(String),
    /// A command must be available
    CommandExists(String),
    /// A package must be installed
    PackageInstalled(String),
    /// A service must be running
    ServiceRunning(String),
    /// A service must be stopped
    ServiceStopped(String),
    /// Custom check command
    CustomCheck,
}

impl Precondition {
    pub fn file_exists(path: &str) -> Self {
        Self {
            condition_type: PreconditionType::FileExists(path.to_string()),
            description: format!("File {} must exist", path),
            check_command: Some(format!("test -f {}", path)),
            expected: None,
        }
    }

    pub fn command_exists(cmd: &str) -> Self {
        Self {
            condition_type: PreconditionType::CommandExists(cmd.to_string()),
            description: format!("Command '{}' must be available", cmd),
            check_command: Some(format!("command -v {}", cmd)),
            expected: None,
        }
    }

    pub fn package_installed(pkg: &str) -> Self {
        Self {
            condition_type: PreconditionType::PackageInstalled(pkg.to_string()),
            description: format!("Package '{}' must be installed", pkg),
            check_command: Some(format!("pacman -Qi {}", pkg)),
            expected: None,
        }
    }

    pub fn service_running(service: &str) -> Self {
        Self {
            condition_type: PreconditionType::ServiceRunning(service.to_string()),
            description: format!("Service '{}' must be running", service),
            check_command: Some(format!("systemctl is-active {}", service)),
            expected: Some("active".to_string()),
        }
    }
}

/// Step to roll back changes if something goes wrong
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Command to run for rollback
    pub command: String,
    /// Description of what this undoes
    pub description: String,
    /// Whether this needs root
    pub needs_root: bool,
}

/// A candidate recipe awaiting validation and promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCandidate {
    /// Generated ID
    pub id: String,
    /// Name derived from experience
    pub name: String,
    /// Keywords from experience
    pub keywords: Vec<String>,
    /// Question patterns
    pub patterns: Vec<String>,
    /// Context requirements
    pub context: RecipeContext,
    /// Commands to execute
    pub commands: Vec<RecipeCommand>,
    /// Verification step
    pub verification: Option<VerificationStep>,
    /// Preconditions to check
    pub preconditions: Vec<Precondition>,
    /// Rollback steps
    pub rollback: Vec<RollbackStep>,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Source experience ID
    pub source_experience_id: String,
    /// Number of successful uses in cluster
    pub cluster_success_count: u32,
}

/// Generate a recipe candidate from an experience
pub fn generate_candidate(experience: &Experience) -> RecipeCandidate {
    let commands: Vec<RecipeCommand> = experience
        .successful_commands
        .iter()
        .map(|cmd| {
            let needs_root = cmd.starts_with("sudo ") || cmd.contains("pacman -S");
            let modifies = needs_root
                || cmd.contains("echo")
                || cmd.contains("sed")
                || cmd.contains("systemctl");

            RecipeCommand {
                command: cmd.clone(),
                description: format!("Execute: {}", truncate_cmd(cmd, 50)),
                modifies_system: modifies,
                backup_file: extract_backup_target(cmd),
                needs_root,
            }
        })
        .collect();

    let risk_level = RiskLevel::from_commands(&experience.successful_commands);
    let preconditions = infer_preconditions(&experience.successful_commands);
    let rollback = infer_rollback(&experience.successful_commands);

    // Build context from experience context
    let context = RecipeContext {
        os: Some("Arch Linux".to_string()),
        editor: experience.context.get_tag("editor"),
        shell: experience.context.get_tag("shell"),
        bootloader: None,
        desktop: experience.context.get_tag("desktop"),
        display_server: None,
        filesystem: experience.context.get_tag("filesystem"),
    };

    RecipeCandidate {
        id: format!("candidate-{}", &experience.id[..8]),
        name: generate_name(&experience.question),
        keywords: experience.keywords.clone(),
        patterns: vec![experience.question.clone()],
        context,
        commands,
        verification: None, // TODO: infer from commands
        preconditions,
        rollback,
        risk_level,
        source_experience_id: experience.id.clone(),
        cluster_success_count: experience.usefulness_score,
    }
}

/// Infer preconditions from commands
fn infer_preconditions(commands: &[String]) -> Vec<Precondition> {
    let mut preconditions = Vec::new();

    for cmd in commands {
        // Extract command name
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let base_cmd = parts[0].trim_start_matches("sudo ");

        // Check if command needs to exist
        if !is_builtin_command(base_cmd) {
            preconditions.push(Precondition::command_exists(base_cmd));
        }

        // Systemctl commands need the service
        if cmd.contains("systemctl") && parts.len() >= 3 {
            let service = parts.last().unwrap();
            if !service.starts_with('-') {
                // For restart/start, service might need to exist
                if cmd.contains("restart") || cmd.contains("start") {
                    // Don't add precondition - service might be installed as part of recipe
                }
            }
        }

        // File operations need the file to exist
        if cmd.contains("cat ") || cmd.contains("less ") || cmd.contains("head ") {
            if let Some(file) = extract_file_path(cmd) {
                preconditions.push(Precondition::file_exists(&file));
            }
        }
    }

    // Deduplicate
    preconditions.dedup_by(|a, b| a.description == b.description);
    preconditions
}

/// Infer rollback steps from commands
fn infer_rollback(commands: &[String]) -> Vec<RollbackStep> {
    let mut rollback = Vec::new();

    for cmd in commands {
        // Package installation can be reversed
        if cmd.contains("pacman -S ") {
            if let Some(pkg) = extract_package_name(cmd) {
                rollback.push(RollbackStep {
                    command: format!("sudo pacman -Rns {}", pkg),
                    description: format!("Remove installed package: {}", pkg),
                    needs_root: true,
                });
            }
        }

        // Service enable can be disabled
        if cmd.contains("systemctl enable ") {
            if let Some(service) = extract_service_name(cmd) {
                rollback.push(RollbackStep {
                    command: format!("sudo systemctl disable {}", service),
                    description: format!("Disable service: {}", service),
                    needs_root: true,
                });
            }
        }

        // Service start can be stopped
        if cmd.contains("systemctl start ") {
            if let Some(service) = extract_service_name(cmd) {
                rollback.push(RollbackStep {
                    command: format!("sudo systemctl stop {}", service),
                    description: format!("Stop service: {}", service),
                    needs_root: true,
                });
            }
        }

        // File creation can be removed
        if cmd.contains("touch ") {
            if let Some(file) = extract_file_path(cmd) {
                rollback.push(RollbackStep {
                    command: format!("rm {}", file),
                    description: format!("Remove created file: {}", file),
                    needs_root: cmd.starts_with("sudo"),
                });
            }
        }
    }

    // Reverse order for rollback
    rollback.reverse();
    rollback
}

fn truncate_cmd(cmd: &str, max_len: usize) -> String {
    if cmd.len() <= max_len {
        cmd.to_string()
    } else {
        format!("{}...", &cmd[..max_len - 3])
    }
}

fn generate_name(question: &str) -> String {
    // Extract key words and create a name
    let words: Vec<&str> = question
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .take(4)
        .collect();
    words.join("-").to_lowercase()
}

fn extract_backup_target(cmd: &str) -> Option<String> {
    // Look for file paths being modified
    if cmd.contains("sed -i") || cmd.contains("echo") && cmd.contains(">>") {
        // Extract path after redirection or as sed target
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if part.starts_with('/') && !part.contains("dev/null") {
                return Some(part.to_string());
            }
            if *part == ">>" && i + 1 < parts.len() {
                return Some(parts[i + 1].to_string());
            }
        }
    }
    None
}

fn is_builtin_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "echo" | "cd" | "pwd" | "export" | "source" | "." | "test" | "[" | "true" | "false"
    )
}

fn extract_file_path(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    for part in parts {
        if part.starts_with('/') || part.starts_with("~/") {
            return Some(part.to_string());
        }
    }
    None
}

fn extract_package_name(cmd: &str) -> Option<String> {
    // Extract package from "pacman -S package" or "pacman -S --noconfirm package"
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "-S" || *part == "-Syu" {
            // Find first non-flag argument after -S
            for p in &parts[i + 1..] {
                if !p.starts_with('-') {
                    return Some(p.to_string());
                }
            }
        }
    }
    None
}

fn extract_service_name(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    // Last non-flag argument is usually the service name
    parts
        .iter()
        .rev()
        .find(|p| !p.starts_with('-') && !p.contains("systemctl"))
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(
            RiskLevel::from_commands(&["ls -la".to_string()]),
            RiskLevel::Safe
        );
        assert_eq!(
            RiskLevel::from_commands(&["sudo pacman -S vim".to_string()]),
            RiskLevel::High
        );
        assert_eq!(
            RiskLevel::from_commands(&["rm -rf /".to_string()]),
            RiskLevel::Critical
        );
    }

    #[test]
    fn test_precondition_generation() {
        let preconditions = infer_preconditions(&["cat /etc/pacman.conf".to_string()]);
        assert!(!preconditions.is_empty());
    }

    #[test]
    fn test_rollback_generation() {
        let rollback = infer_rollback(&["sudo pacman -S neovim".to_string()]);
        assert!(!rollback.is_empty());
        assert!(rollback[0].command.contains("pacman -Rns neovim"));
    }
}

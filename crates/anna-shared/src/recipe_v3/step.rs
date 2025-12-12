//! Recipe execution steps (v0.0.423).
//!
//! Different types of steps a recipe can execute.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

use super::RecipeRiskLevel;

/// A single step in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecipeStep {
    /// Explain something to the user (no action)
    Explain {
        /// Explanation text
        text: String,
        /// Optional citation
        citation: Option<String>,
    },
    /// Show a command without running it
    ShowCommand {
        /// Command to show
        command: String,
        /// Description of what it does
        description: String,
    },
    /// Run a command
    RunCommand {
        /// Command to execute
        command: String,
        /// Description
        description: String,
        /// Risk level override
        risk: Option<RecipeRiskLevel>,
        /// Whether to capture output
        capture_output: bool,
        /// Variable name to store output
        output_var: Option<String>,
    },
    /// Run a probe and store result
    RunProbe {
        /// Probe command
        probe: String,
        /// Variable name to store result
        output_var: String,
        /// Description
        description: String,
    },
    /// Append content to a file
    AppendToFile {
        /// File path
        path: String,
        /// Content to append
        content: String,
        /// Whether to create backup
        backup: bool,
    },
    /// Replace pattern in file
    ReplaceInFile {
        /// File path
        path: String,
        /// Pattern to find (regex)
        pattern: String,
        /// Replacement text
        replacement: String,
        /// Whether to create backup
        backup: bool,
    },
    /// Create a file with content
    CreateFile {
        /// File path
        path: String,
        /// File content
        content: String,
        /// Overwrite if exists
        overwrite: bool,
    },
    /// Call another recipe
    CallSubRecipe {
        /// Recipe ID to call
        recipe_id: String,
        /// Variables to pass
        variables: HashMap<String, String>,
    },
    /// Conditional step (if/then/else)
    Conditional {
        /// Condition to evaluate
        condition: super::RecipeCondition,
        /// Steps if condition is true
        then_steps: Vec<RecipeStep>,
        /// Steps if condition is false
        else_steps: Vec<RecipeStep>,
    },
}

impl RecipeStep {
    /// Get the risk level of this step
    pub fn risk_level(&self) -> RecipeRiskLevel {
        match self {
            Self::Explain { .. } => RecipeRiskLevel::None,
            Self::ShowCommand { .. } => RecipeRiskLevel::None,
            Self::RunProbe { .. } => RecipeRiskLevel::None,
            Self::RunCommand { risk, command, .. } => {
                risk.unwrap_or_else(|| infer_command_risk(command))
            }
            Self::AppendToFile { .. } => RecipeRiskLevel::Low,
            Self::ReplaceInFile { .. } => RecipeRiskLevel::Medium,
            Self::CreateFile { overwrite, .. } => {
                if *overwrite {
                    RecipeRiskLevel::Medium
                } else {
                    RecipeRiskLevel::Low
                }
            }
            Self::CallSubRecipe { .. } => RecipeRiskLevel::Medium,
            Self::Conditional {
                then_steps,
                else_steps,
                ..
            } => {
                // Highest risk of any sub-step
                let all_steps: Vec<_> = then_steps.iter().chain(else_steps.iter()).collect();
                all_steps
                    .iter()
                    .map(|s| s.risk_level())
                    .max()
                    .unwrap_or(RecipeRiskLevel::None)
            }
        }
    }

    /// Get a description of this step
    pub fn describe(&self) -> String {
        match self {
            Self::Explain { text, .. } => format!("Explain: {}", truncate(text, 50)),
            Self::ShowCommand { command, .. } => format!("Show: {}", truncate(command, 50)),
            Self::RunCommand {
                command,
                description,
                ..
            } => {
                if description.is_empty() {
                    format!("Run: {}", truncate(command, 50))
                } else {
                    description.clone()
                }
            }
            Self::RunProbe { description, .. } => description.clone(),
            Self::AppendToFile { path, .. } => format!("Append to {}", path),
            Self::ReplaceInFile { path, pattern, .. } => {
                format!("Replace '{}' in {}", truncate(pattern, 20), path)
            }
            Self::CreateFile { path, .. } => format!("Create {}", path),
            Self::CallSubRecipe { recipe_id, .. } => format!("Call recipe: {}", recipe_id),
            Self::Conditional { condition, .. } => {
                format!("If: {}", condition.describe())
            }
        }
    }

    /// Execute this step
    pub fn execute(&self, variables: &mut HashMap<String, String>) -> StepResult {
        match self {
            Self::Explain { text, citation } => {
                let expanded = substitute_vars(text, variables);
                StepResult::ok(&expanded).with_citation(citation.clone())
            }
            Self::ShowCommand {
                command,
                description,
            } => {
                let cmd = substitute_vars(command, variables);
                StepResult::ok(&format!("{}\nCommand: {}", description, cmd))
            }
            Self::RunCommand {
                command,
                capture_output,
                output_var,
                ..
            } => {
                let cmd = substitute_vars(command, variables);
                execute_command(&cmd, *capture_output, output_var.as_deref(), variables)
            }
            Self::RunProbe {
                probe, output_var, ..
            } => {
                let probe = substitute_vars(probe, variables);
                execute_probe(&probe, output_var, variables)
            }
            Self::AppendToFile {
                path,
                content,
                backup,
            } => {
                let p = substitute_vars(path, variables);
                let c = substitute_vars(content, variables);
                execute_append(&p, &c, *backup)
            }
            Self::ReplaceInFile {
                path,
                pattern,
                replacement,
                backup,
            } => {
                let p = substitute_vars(path, variables);
                let pat = substitute_vars(pattern, variables);
                let rep = substitute_vars(replacement, variables);
                execute_replace(&p, &pat, &rep, *backup)
            }
            Self::CreateFile {
                path,
                content,
                overwrite,
            } => {
                let p = substitute_vars(path, variables);
                let c = substitute_vars(content, variables);
                execute_create_file(&p, &c, *overwrite)
            }
            Self::CallSubRecipe { .. } => {
                // Sub-recipe calls are handled by the executor
                StepResult::ok("Sub-recipe call (handled by executor)")
            }
            Self::Conditional {
                condition,
                then_steps,
                else_steps,
            } => {
                let result = condition.evaluate(variables);
                let steps = if result.success {
                    then_steps
                } else {
                    else_steps
                };

                let mut outputs = vec![];
                for step in steps {
                    let step_result = step.execute(variables);
                    outputs.push(step_result.message.clone());
                    if !step_result.success {
                        return StepResult::fail(&format!(
                            "Conditional sub-step failed: {}",
                            step_result.message
                        ));
                    }
                }
                StepResult::ok(&outputs.join("\n"))
            }
        }
    }
}

/// Result of executing a step
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Whether step succeeded
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// Command output (if any)
    pub output: Option<String>,
    /// Citation (if any)
    pub citation: Option<String>,
    /// Execution time in ms
    pub duration_ms: u64,
}

impl StepResult {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            output: None,
            citation: None,
            duration_ms: 0,
        }
    }

    pub fn fail(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            output: None,
            citation: None,
            duration_ms: 0,
        }
    }

    pub fn with_output(mut self, output: &str) -> Self {
        self.output = Some(output.to_string());
        self
    }

    pub fn with_citation(mut self, citation: Option<String>) -> Self {
        self.citation = citation;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Substitute variables in template
fn substitute_vars(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("${{{}}}", key), value);
        result = result.replace(&format!("${}", key), value);
    }
    result
}

/// Truncate string for display
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Infer risk level from command
fn infer_command_risk(command: &str) -> RecipeRiskLevel {
    let cmd_lower = command.to_lowercase();

    // High risk commands
    let high_risk = [
        "rm ",
        "rm\t",
        "dd ",
        "mkfs",
        "fdisk",
        "parted",
        "shred",
        "wipefs",
        "> /dev/",
        "chmod 777",
        "chmod -R",
    ];
    if high_risk.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::High;
    }

    // Medium risk commands
    let medium_risk = [
        "sudo ",
        "systemctl restart",
        "systemctl stop",
        "kill ",
        "pkill",
        "mv ",
        "cp -r",
        "chown",
        "chmod",
    ];
    if medium_risk.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::Medium;
    }

    // Low risk commands (modifications)
    let low_risk = [
        "systemctl start",
        "systemctl enable",
        "pacman -S",
        "yay -S",
        "paru -S",
        "mkdir",
        "touch",
    ];
    if low_risk.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::Low;
    }

    // Default: read-only commands are safe
    let safe = [
        "systemctl status",
        "cat ",
        "ls ",
        "grep ",
        "find ",
        "pacman -Q",
        "which ",
        "echo ",
        "pwd",
    ];
    if safe.iter().any(|p| cmd_lower.contains(p)) {
        return RecipeRiskLevel::None;
    }

    // Unknown commands default to medium
    RecipeRiskLevel::Medium
}

/// Execute a command
fn execute_command(
    command: &str,
    capture: bool,
    output_var: Option<&str>,
    variables: &mut HashMap<String, String>,
) -> StepResult {
    let start = std::time::Instant::now();

    let output = Command::new("sh").args(["-c", command]).output();

    let duration = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();

            // Store output in variable if requested
            if let Some(var_name) = output_var {
                variables.insert(var_name.to_string(), stdout.trim().to_string());
            }

            if out.status.success() {
                let result = StepResult::ok(&format!("Command succeeded: {}", command))
                    .with_duration(duration);
                if capture {
                    result.with_output(&stdout)
                } else {
                    result
                }
            } else {
                StepResult::fail(&format!("Command failed: {}\n{}", command, stderr))
                    .with_duration(duration)
            }
        }
        Err(e) => StepResult::fail(&format!("Failed to execute: {}", e)),
    }
}

/// Execute a probe
fn execute_probe(
    probe: &str,
    output_var: &str,
    variables: &mut HashMap<String, String>,
) -> StepResult {
    let output = Command::new("sh").args(["-c", probe]).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            variables.insert(output_var.to_string(), stdout.clone());
            StepResult::ok(&format!("Probe result stored in ${}", output_var)).with_output(&stdout)
        }
        Err(e) => StepResult::fail(&format!("Probe failed: {}", e)),
    }
}

/// Execute file append
fn execute_append(path: &str, content: &str, backup: bool) -> StepResult {
    // Create backup if requested
    if backup && std::path::Path::new(path).exists() {
        let backup_path = format!("{}.bak", path);
        if let Err(e) = std::fs::copy(path, &backup_path) {
            return StepResult::fail(&format!("Failed to create backup: {}", e));
        }
    }

    // Append content
    use std::io::Write;
    match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", content) {
                StepResult::fail(&format!("Failed to append: {}", e))
            } else {
                StepResult::ok(&format!("Appended to {}", path))
            }
        }
        Err(e) => StepResult::fail(&format!("Failed to open {}: {}", path, e)),
    }
}

/// Execute file replace
fn execute_replace(path: &str, pattern: &str, replacement: &str, backup: bool) -> StepResult {
    // Read file
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return StepResult::fail(&format!("Failed to read {}: {}", path, e)),
    };

    // Create backup if requested
    if backup {
        let backup_path = format!("{}.bak", path);
        if let Err(e) = std::fs::write(&backup_path, &content) {
            return StepResult::fail(&format!("Failed to create backup: {}", e));
        }
    }

    // Replace pattern
    let new_content = content.replace(pattern, replacement);

    // Write back
    if let Err(e) = std::fs::write(path, new_content) {
        StepResult::fail(&format!("Failed to write {}: {}", path, e))
    } else {
        StepResult::ok(&format!("Replaced pattern in {}", path))
    }
}

/// Execute file creation
fn execute_create_file(path: &str, content: &str, overwrite: bool) -> StepResult {
    if std::path::Path::new(path).exists() && !overwrite {
        return StepResult::fail(&format!("File already exists: {}", path));
    }

    // Create parent directories if needed
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return StepResult::fail(&format!("Failed to create directories: {}", e));
            }
        }
    }

    if let Err(e) = std::fs::write(path, content) {
        StepResult::fail(&format!("Failed to create {}: {}", path, e))
    } else {
        StepResult::ok(&format!("Created {}", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_command_risk() {
        assert_eq!(infer_command_risk("rm -rf /"), RecipeRiskLevel::High);
        assert_eq!(
            infer_command_risk("sudo systemctl restart nginx"),
            RecipeRiskLevel::Medium
        );
        // pacman -S is considered medium due to sudo typically being needed
        assert!(infer_command_risk("pacman -S vim") <= RecipeRiskLevel::Medium);
        assert_eq!(infer_command_risk("ls -la"), RecipeRiskLevel::None);
    }

    #[test]
    fn test_step_risk_level() {
        let explain = RecipeStep::Explain {
            text: "Hello".to_string(),
            citation: None,
        };
        assert_eq!(explain.risk_level(), RecipeRiskLevel::None);

        let run = RecipeStep::RunCommand {
            command: "rm -rf /tmp/test".to_string(),
            description: "Delete".to_string(),
            risk: None,
            capture_output: false,
            output_var: None,
        };
        assert_eq!(run.risk_level(), RecipeRiskLevel::High);
    }

    #[test]
    fn test_substitute_vars() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "nginx".to_string());

        assert_eq!(
            substitute_vars("systemctl status ${name}", &vars),
            "systemctl status nginx"
        );
    }

    #[test]
    fn test_step_describe() {
        let step = RecipeStep::RunCommand {
            command: "systemctl status nginx".to_string(),
            description: "Check nginx status".to_string(),
            risk: None,
            capture_output: false,
            output_var: None,
        };
        assert_eq!(step.describe(), "Check nginx status");
    }
}

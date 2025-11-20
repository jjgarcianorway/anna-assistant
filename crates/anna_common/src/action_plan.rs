//! Safe ACTION_PLAN system for Beta.66
//!
//! Security-first action planning and execution with:
//! - Structured command representation (no shell strings)
//! - Mandatory validation before execution
//! - ANNA_BACKUP enforcement
//! - Risk classification
//! - Injection-resistant design

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Risk level for an action step
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionRisk {
    Low,
    Medium,
    High,
}

impl ActionRisk {
    pub fn description(&self) -> &'static str {
        match self {
            ActionRisk::Low => "Low risk - user-space only, easily reversible",
            ActionRisk::Medium => "Medium risk - may affect application functionality",
            ActionRisk::High => "High risk - may affect system services or require manual recovery",
        }
    }

    pub fn requires_confirmation(&self) -> bool {
        matches!(self, ActionRisk::Medium | ActionRisk::High)
    }
}

/// A single step in an action plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionStep {
    /// Unique identifier for this step
    pub id: String,

    /// Human-readable description of what this step does
    pub description: String,

    /// Risk level
    pub risk: ActionRisk,

    /// Whether user confirmation is required
    pub requires_confirmation: bool,

    /// Backup command (if applicable)
    /// MUST use ANNA_BACKUP naming: file.ANNA_BACKUP.YYYYMMDD-HHMMSS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup: Option<String>,

    /// Commands to execute
    /// Each command is a structured array: [program, arg1, arg2, ...]
    /// NOT a shell string - this prevents injection
    pub commands: Vec<Vec<String>>,

    /// How to restore if something goes wrong
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_hint: Option<String>,
}

/// Complete action plan from LLM
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionPlan {
    /// All steps in the plan
    pub steps: Vec<ActionStep>,
}

impl ActionPlan {
    /// Validate the entire plan before execution
    ///
    /// Security checks:
    /// - All required fields present
    /// - Risk levels valid
    /// - Commands not empty
    /// - Backup paths use ANNA_BACKUP naming
    /// - No shell metacharacters in unsafe positions
    pub fn validate(&self) -> Result<()> {
        if self.steps.is_empty() {
            return Err(anyhow!("Action plan has no steps"));
        }

        for (i, step) in self.steps.iter().enumerate() {
            self.validate_step(step, i)?;
        }

        Ok(())
    }

    fn validate_step(&self, step: &ActionStep, index: usize) -> Result<()> {
        // Validate ID
        if step.id.is_empty() {
            return Err(anyhow!("Step {} has empty id", index));
        }

        // Validate description
        if step.description.is_empty() {
            return Err(anyhow!("Step {} has empty description", index));
        }

        // Validate commands
        if step.commands.is_empty() {
            return Err(anyhow!("Step {} ('{}') has no commands", index, step.id));
        }

        for (cmd_idx, cmd) in step.commands.iter().enumerate() {
            if cmd.is_empty() {
                return Err(anyhow!("Step {} command {} is empty array", index, cmd_idx));
            }

            let program = &cmd[0];
            if program.is_empty() {
                return Err(anyhow!(
                    "Step {} command {} has empty program",
                    index,
                    cmd_idx
                ));
            }

            // Check for shell metacharacters in program name
            if program.contains(';')
                || program.contains('&')
                || program.contains('|')
                || program.contains('`')
                || program.contains('$')
            {
                return Err(anyhow!(
                    "Step {} command {} has suspicious metacharacters in program: '{}'",
                    index,
                    cmd_idx,
                    program
                ));
            }
        }

        // Validate backup naming if present
        if let Some(ref backup) = step.backup {
            if !backup.contains("ANNA_BACKUP") {
                return Err(anyhow!(
                    "Step {} backup does not use ANNA_BACKUP naming: '{}'",
                    index,
                    backup
                ));
            }

            // Check for proper timestamp format (YYYYMMDD-HHMMSS)
            if !backup.contains("ANNA_BACKUP.2") {
                // Should have ANNA_BACKUP.YYYY...
                eprintln!(
                    "Warning: Step {} backup may not have proper timestamp: '{}'",
                    index, backup
                );
            }
        }

        // Validate confirmation requirement matches risk
        match step.risk {
            ActionRisk::Low => {
                // Low risk doesn't strictly require confirmation, but it's ok if set
            }
            ActionRisk::Medium | ActionRisk::High => {
                if !step.requires_confirmation {
                    return Err(anyhow!(
                        "Step {} has {} risk but requires_confirmation=false",
                        index,
                        match step.risk {
                            ActionRisk::Medium => "medium",
                            ActionRisk::High => "high",
                            _ => unreachable!(),
                        }
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get highest risk level in this plan
    pub fn max_risk(&self) -> ActionRisk {
        self.steps
            .iter()
            .map(|s| s.risk)
            .max()
            .unwrap_or(ActionRisk::Low)
    }

    /// Check if any step requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        self.steps.iter().any(|s| s.requires_confirmation)
    }

    /// Get summary for user display
    pub fn summary(&self) -> String {
        format!(
            "{} steps, max risk: {}",
            self.steps.len(),
            match self.max_risk() {
                ActionRisk::Low => "low",
                ActionRisk::Medium => "medium",
                ActionRisk::High => "high",
            }
        )
    }
}

/// Safe command builder that prevents injection
pub struct SafeCommand {
    program: String,
    args: Vec<String>,
}

impl SafeCommand {
    /// Create a new safe command
    ///
    /// Unlike string parsing, this requires explicit program and args
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    /// Add an argument (automatically safe - no shell interpretation)
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    /// Convert to std::process::Command
    pub fn to_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.program);
        cmd.args(&self.args);
        cmd
    }

    /// Get program name
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Get arguments
    pub fn arguments(&self) -> &[String] {
        &self.args
    }
}

/// Convert ActionStep commands to SafeCommand
impl ActionStep {
    pub fn to_safe_commands(&self) -> Vec<SafeCommand> {
        self.commands
            .iter()
            .map(|cmd_parts| {
                let program = cmd_parts[0].clone();
                let args = cmd_parts[1..].to_vec();
                SafeCommand::new(program).args(args)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_action_plan() {
        let plan = ActionPlan {
            steps: vec![ActionStep {
                id: "step_1".to_string(),
                description: "Backup vimrc".to_string(),
                risk: ActionRisk::Low,
                requires_confirmation: false,
                backup: Some("cp ~/.vimrc ~/.vimrc.ANNA_BACKUP.20250118-123456".to_string()),
                commands: vec![vec![
                    "cp".to_string(),
                    "~/.vimrc".to_string(),
                    "~/.vimrc.ANNA_BACKUP.20250118-123456".to_string(),
                ]],
                restore_hint: Some("cp ~/.vimrc.ANNA_BACKUP.* ~/.vimrc".to_string()),
            }],
        };

        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_reject_empty_commands() {
        let plan = ActionPlan {
            steps: vec![ActionStep {
                id: "step_1".to_string(),
                description: "Do nothing".to_string(),
                risk: ActionRisk::Low,
                requires_confirmation: false,
                backup: None,
                commands: vec![], // ❌ Empty
                restore_hint: None,
            }],
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_reject_shell_metacharacters() {
        let plan = ActionPlan {
            steps: vec![ActionStep {
                id: "step_1".to_string(),
                description: "Suspicious command".to_string(),
                risk: ActionRisk::Low,
                requires_confirmation: false,
                backup: None,
                commands: vec![
                    vec!["echo test; rm -rf /".to_string()], // ❌ Semicolon in program
                ],
                restore_hint: None,
            }],
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_reject_bad_backup_naming() {
        let plan = ActionPlan {
            steps: vec![ActionStep {
                id: "step_1".to_string(),
                description: "Backup with wrong naming".to_string(),
                risk: ActionRisk::Low,
                requires_confirmation: false,
                backup: Some("cp file file.bak".to_string()), // ❌ No ANNA_BACKUP
                commands: vec![vec![
                    "cp".to_string(),
                    "file".to_string(),
                    "file.bak".to_string(),
                ]],
                restore_hint: None,
            }],
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_high_risk_requires_confirmation() {
        let plan = ActionPlan {
            steps: vec![ActionStep {
                id: "step_1".to_string(),
                description: "High risk action".to_string(),
                risk: ActionRisk::High,
                requires_confirmation: false, // ❌ High risk MUST require confirmation
                backup: None,
                commands: vec![vec![
                    "systemctl".to_string(),
                    "restart".to_string(),
                    "important-service".to_string(),
                ]],
                restore_hint: None,
            }],
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_safe_command_builder() {
        let cmd = SafeCommand::new("cp")
            .arg("file with spaces.txt")
            .arg("destination with spaces.txt");

        assert_eq!(cmd.program(), "cp");
        assert_eq!(
            cmd.arguments(),
            &[
                "file with spaces.txt".to_string(),
                "destination with spaces.txt".to_string()
            ]
        );

        // When converted to std::process::Command, spaces are handled safely
        let _std_cmd = cmd.to_command();
        // Arguments are kept separate - no shell interpretation needed
    }
}

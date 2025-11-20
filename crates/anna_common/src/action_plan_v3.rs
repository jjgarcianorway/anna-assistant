//! Action Plan Schema - Runtime LLM Contract
//!
//! Beta.143: JSON-based action plan schema for runtime LLM
//!
//! The runtime LLM (Ollama with Llama/Qwen/etc) MUST output this exact schema.
//! This is the contract between Anna and the planning brain.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Risk level for commands and checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    /// Pure inspection, no changes
    Info,
    /// Reversible user config (wallpaper, fonts, theming)
    Low,
    /// Package installs, service changes with clear rollback
    Medium,
    /// Bootloader, network, filesystem - could lock user out
    High,
}

impl RiskLevel {
    /// Check if this risk level requires user confirmation
    pub fn requires_confirmation(&self) -> bool {
        match self {
            RiskLevel::Info => false,
            RiskLevel::Low | RiskLevel::Medium | RiskLevel::High => true,
        }
    }

    /// Get color for display
    pub fn color(&self) -> &'static str {
        match self {
            RiskLevel::Info => "blue",
            RiskLevel::Low => "green",
            RiskLevel::Medium => "yellow",
            RiskLevel::High => "red",
        }
    }

    /// Get emoji indicator
    pub fn emoji(&self) -> &'static str {
        match self {
            RiskLevel::Info => "ℹ️",
            RiskLevel::Low => "✅",
            RiskLevel::Medium => "⚠️",
            RiskLevel::High => "🚨",
        }
    }
}

/// Necessary check before executing main plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NecessaryCheck {
    /// Unique identifier for this check
    pub id: String,

    /// Human-readable description of what this checks
    pub description: String,

    /// Command to run (safe diagnostic only)
    pub command: String,

    /// Risk level (should be INFO for checks)
    pub risk_level: RiskLevel,

    /// Whether this check is required before proceeding
    pub required: bool,
}

/// Single step in the command plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStep {
    /// Unique identifier for this step
    pub id: String,

    /// Human-readable description of what this step does
    pub description: String,

    /// Exact shell command to execute
    pub command: String,

    /// Risk level for this command
    pub risk_level: RiskLevel,

    /// ID of corresponding rollback step (if any)
    pub rollback_id: Option<String>,

    /// Whether this requires explicit user confirmation
    pub requires_confirmation: bool,
}

/// Rollback step to undo a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Unique identifier for this rollback step
    pub id: String,

    /// Human-readable description of what this undoes
    pub description: String,

    /// Command to restore previous state
    pub command: String,
}

/// Detection results from environment scanning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectionResults {
    /// Desktop environment (GNOME, KDE, XFCE, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub de: Option<String>,

    /// Window manager (Hyprland, sway, i3, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wm: Option<String>,

    /// Wallpaper backends detected
    #[serde(default)]
    pub wallpaper_backends: Vec<String>,

    /// Display protocol (wayland or x11)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_protocol: Option<String>,

    /// Additional detected features
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Metadata about the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMeta {
    /// What was detected in the environment
    #[serde(default)]
    pub detection_results: DetectionResults,

    /// Which template was used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_used: Option<String>,

    /// LLM version identifier
    pub llm_version: String,
}

/// Complete action plan from runtime LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    /// High-level reasoning about the problem using telemetry
    pub analysis: String,

    /// User-visible goals this plan achieves
    pub goals: Vec<String>,

    /// Checks to run before executing the plan
    #[serde(default)]
    pub necessary_checks: Vec<NecessaryCheck>,

    /// Main command sequence
    #[serde(default)]
    pub command_plan: Vec<CommandStep>,

    /// Rollback procedures
    #[serde(default)]
    pub rollback_plan: Vec<RollbackStep>,

    /// Plain English explanation for the user
    pub notes_for_user: String,

    /// Metadata about this plan
    pub meta: PlanMeta,
}

impl ActionPlan {
    /// Get highest risk level in the command plan
    pub fn max_risk_level(&self) -> Option<RiskLevel> {
        self.command_plan
            .iter()
            .map(|step| &step.risk_level)
            .max_by_key(|r| match r {
                RiskLevel::Info => 0,
                RiskLevel::Low => 1,
                RiskLevel::Medium => 2,
                RiskLevel::High => 3,
            })
            .cloned()
    }

    /// Check if any step requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        self.command_plan
            .iter()
            .any(|step| step.requires_confirmation)
    }

    /// Get all commands (checks + plan steps)
    pub fn all_commands(&self) -> Vec<&str> {
        let mut commands = Vec::new();
        for check in &self.necessary_checks {
            commands.push(check.command.as_str());
        }
        for step in &self.command_plan {
            commands.push(step.command.as_str());
        }
        commands
    }

    /// Validate basic schema requirements
    pub fn validate(&self) -> Result<(), String> {
        // Check for empty analysis
        if self.analysis.trim().is_empty() {
            return Err("Analysis cannot be empty".to_string());
        }

        // Check for empty notes
        if self.notes_for_user.trim().is_empty() {
            return Err("Notes for user cannot be empty".to_string());
        }

        // Validate necessary_checks
        for check in &self.necessary_checks {
            if check.id.is_empty() {
                return Err("Check ID cannot be empty".to_string());
            }
            if check.command.trim().is_empty() {
                return Err(format!("Check '{}' has empty command", check.id));
            }
        }

        // Validate command_plan
        for step in &self.command_plan {
            if step.id.is_empty() {
                return Err("Step ID cannot be empty".to_string());
            }
            if step.command.trim().is_empty() {
                return Err(format!("Step '{}' has empty command", step.id));
            }
        }

        // Validate rollback_plan references
        for step in &self.command_plan {
            if let Some(rollback_id) = &step.rollback_id {
                if !self.rollback_plan.iter().any(|r| &r.id == rollback_id) {
                    return Err(format!(
                        "Step '{}' references non-existent rollback '{}'",
                        step.id, rollback_id
                    ));
                }
            }
        }

        // Validate rollback commands
        for rollback in &self.rollback_plan {
            if rollback.id.is_empty() {
                return Err("Rollback ID cannot be empty".to_string());
            }
            if rollback.command.trim().is_empty() {
                return Err(format!("Rollback '{}' has empty command", rollback.id));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Info < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
    }

    #[test]
    fn test_risk_level_requires_confirmation() {
        assert!(!RiskLevel::Info.requires_confirmation());
        assert!(RiskLevel::Low.requires_confirmation());
        assert!(RiskLevel::Medium.requires_confirmation());
        assert!(RiskLevel::High.requires_confirmation());
    }

    #[test]
    fn test_action_plan_validation() {
        let plan = ActionPlan {
            analysis: "Test analysis".to_string(),
            goals: vec!["Goal 1".to_string()],
            necessary_checks: vec![],
            command_plan: vec![CommandStep {
                id: "step1".to_string(),
                description: "Test step".to_string(),
                command: "echo test".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            }],
            rollback_plan: vec![],
            notes_for_user: "Test notes".to_string(),
            meta: PlanMeta {
                detection_results: DetectionResults::default(),
                template_used: None,
                llm_version: "anna_runtime_v3".to_string(),
            },
        };

        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_action_plan_validation_empty_command() {
        let plan = ActionPlan {
            analysis: "Test".to_string(),
            goals: vec![],
            necessary_checks: vec![],
            command_plan: vec![CommandStep {
                id: "step1".to_string(),
                description: "Test".to_string(),
                command: "".to_string(), // Empty command
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            }],
            rollback_plan: vec![],
            notes_for_user: "Test".to_string(),
            meta: PlanMeta {
                detection_results: DetectionResults::default(),
                template_used: None,
                llm_version: "anna_runtime_v3".to_string(),
            },
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_action_plan_validation_missing_rollback() {
        let plan = ActionPlan {
            analysis: "Test".to_string(),
            goals: vec![],
            necessary_checks: vec![],
            command_plan: vec![CommandStep {
                id: "step1".to_string(),
                description: "Test".to_string(),
                command: "echo test".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("rollback_missing".to_string()), // References non-existent rollback
                requires_confirmation: true,
            }],
            rollback_plan: vec![],
            notes_for_user: "Test".to_string(),
            meta: PlanMeta {
                detection_results: DetectionResults::default(),
                template_used: None,
                llm_version: "anna_runtime_v3".to_string(),
            },
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_max_risk_level() {
        let plan = ActionPlan {
            analysis: "Test".to_string(),
            goals: vec![],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "step1".to_string(),
                    description: "Low risk".to_string(),
                    command: "echo test".to_string(),
                    risk_level: RiskLevel::Low,
                    rollback_id: None,
                    requires_confirmation: false,
                },
                CommandStep {
                    id: "step2".to_string(),
                    description: "High risk".to_string(),
                    command: "sudo something".to_string(),
                    risk_level: RiskLevel::High,
                    rollback_id: None,
                    requires_confirmation: true,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Test".to_string(),
            meta: PlanMeta {
                detection_results: DetectionResults::default(),
                template_used: None,
                llm_version: "anna_runtime_v3".to_string(),
            },
        };

        assert_eq!(plan.max_risk_level(), Some(RiskLevel::High));
    }
}

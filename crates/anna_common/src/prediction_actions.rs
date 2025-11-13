// Prediction Actions - Bridge from Predictions to Self-Healing
// Phase 3.9: Wire prediction engine to self-healing (low-risk only)
//
// This module connects the prediction engine to the self-healing framework,
// mapping predictions to concrete actions while respecting risk levels.

use crate::prediction::{Prediction, PredictionType, Priority};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Risk level for self-healing actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionRisk {
    /// No risk - monitoring, logging only
    None,
    /// Low risk - safe automatic actions (clear cache, restart non-critical services)
    Low,
    /// Medium risk - requires user approval (update packages, restart critical services)
    Medium,
    /// High risk - dangerous operations (modify configs, system-wide changes)
    High,
    /// Critical risk - potentially destructive (format, delete data)
    Critical,
}

impl ActionRisk {
    /// Check if action can be executed automatically when self-healing is enabled
    pub fn is_auto_executable(&self) -> bool {
        matches!(self, ActionRisk::None | ActionRisk::Low)
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionRisk::None => "none",
            ActionRisk::Low => "low",
            ActionRisk::Medium => "medium",
            ActionRisk::High => "high",
            ActionRisk::Critical => "critical",
        }
    }
}

/// Actionable item derived from prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionAction {
    /// Prediction ID this action is based on
    pub prediction_id: String,

    /// Action title
    pub title: String,

    /// Action description
    pub description: String,

    /// Risk level of performing this action
    pub risk: ActionRisk,

    /// Command to execute (if applicable)
    pub command: Option<String>,

    /// Whether this action can be auto-executed
    pub auto_executable: bool,

    /// Expected outcome if action succeeds
    pub expected_outcome: String,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl PredictionAction {
    /// Create new action
    pub fn new(
        prediction_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        risk: ActionRisk,
    ) -> Self {
        let auto_executable = risk.is_auto_executable();
        Self {
            prediction_id: prediction_id.into(),
            title: title.into(),
            description: description.into(),
            risk,
            command: None,
            auto_executable,
            expected_outcome: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set command
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set expected outcome
    pub fn with_outcome(mut self, outcome: impl Into<String>) -> Self {
        self.expected_outcome = outcome.into();
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Convert predictions to actionable items
pub struct PredictionActionMapper;

impl PredictionActionMapper {
    /// Map predictions to actions
    ///
    /// Phase 3.9 Safety Rules:
    /// - Low-risk actions: Can be auto-executed when self-healing enabled
    /// - Medium+ risk: Only show suggestions with command one-liners
    /// - Never perform changes without explicit enablement
    pub fn map_predictions_to_actions(predictions: &[Prediction]) -> Vec<PredictionAction> {
        let mut actions = Vec::new();

        for prediction in predictions {
            // Map prediction to actions based on type and priority
            let mut pred_actions = Self::map_single_prediction(prediction);
            actions.append(&mut pred_actions);
        }

        actions
    }

    /// Map single prediction to actions
    fn map_single_prediction(prediction: &Prediction) -> Vec<PredictionAction> {
        let mut actions = Vec::new();

        match prediction.prediction_type {
            PredictionType::MaintenanceWindow => {
                // Maintenance window predictions are informational
                if !prediction.recommended_actions.is_empty() {
                    for (idx, action_text) in prediction.recommended_actions.iter().enumerate() {
                        // Classify risk based on the action text
                        let risk = Self::classify_action_risk(action_text);

                        let action = PredictionAction::new(
                            format!("{}-{}", prediction.id, idx),
                            format!("Maintenance: {}", action_text),
                            format!("Predicted maintenance action with {}% confidence", prediction.confidence),
                            risk,
                        )
                        .with_outcome("System maintenance completed");

                        actions.push(action);
                    }
                }
            }

            PredictionType::ServiceFailure => {
                // Service failure predictions may require restarts
                actions.push(
                    PredictionAction::new(
                        prediction.id.clone(),
                        format!("Service Recovery: {}", prediction.title),
                        prediction.description.clone(),
                        ActionRisk::Low, // Restarting failed services is low-risk
                    )
                    .with_command("systemctl restart <service>")
                    .with_outcome("Service restored to healthy state"),
                );
            }

            PredictionType::ResourceExhaustion => {
                // Resource exhaustion requires cleanup actions
                if prediction.title.contains("disk") || prediction.title.contains("cache") {
                    actions.push(
                        PredictionAction::new(
                            prediction.id.clone(),
                            format!("Clear Cache: {}", prediction.title),
                            prediction.description.clone(),
                            ActionRisk::Low, // Clearing cache is low-risk
                        )
                        .with_command("paccache -r")
                        .with_outcome("Disk space freed"),
                    );
                } else {
                    // Memory/CPU exhaustion is medium risk
                    actions.push(
                        PredictionAction::new(
                            prediction.id.clone(),
                            format!("Resource Management: {}", prediction.title),
                            prediction.description.clone(),
                            ActionRisk::Medium, // Resource management requires approval
                        )
                        .with_outcome("Resource usage reduced"),
                    );
                }
            }

            PredictionType::PerformanceDegradation => {
                // Performance issues may require investigation
                actions.push(
                    PredictionAction::new(
                        prediction.id.clone(),
                        format!("Performance: {}", prediction.title),
                        prediction.description.clone(),
                        ActionRisk::Low, // Investigation is low-risk
                    )
                    .with_outcome("Performance issue identified"),
                );
            }

            PredictionType::Recommendation => {
                // General recommendations - classify by action text
                for (idx, action_text) in prediction.recommended_actions.iter().enumerate() {
                    let risk = Self::classify_action_risk(action_text);

                    let action = PredictionAction::new(
                        format!("{}-{}", prediction.id, idx),
                        action_text.clone(),
                        prediction.description.clone(),
                        risk,
                    )
                    .with_outcome("Recommendation applied");

                    actions.push(action);
                }
            }
        }

        actions
    }

    /// Classify action risk based on action text
    fn classify_action_risk(action_text: &str) -> ActionRisk {
        let lower = action_text.to_lowercase();

        // Critical risk keywords
        if lower.contains("delete")
            || lower.contains("remove")
            || lower.contains("format")
            || lower.contains("destroy") {
            return ActionRisk::Critical;
        }

        // High risk keywords
        if lower.contains("modify config")
            || lower.contains("edit")
            || lower.contains("system-wide")
            || lower.contains("kernel") {
            return ActionRisk::High;
        }

        // Medium risk keywords
        if lower.contains("update")
            || lower.contains("upgrade")
            || lower.contains("install")
            || lower.contains("restart critical") {
            return ActionRisk::Medium;
        }

        // Low risk keywords
        if lower.contains("clear cache")
            || lower.contains("restart")
            || lower.contains("reload")
            || lower.contains("clean") {
            return ActionRisk::Low;
        }

        // Default to medium for safety
        ActionRisk::Medium
    }
}

/// Phase 3.9: Execution Policy
///
/// Controls which actions can be auto-executed based on configuration
/// and safety rules.
pub struct ActionExecutionPolicy {
    /// Whether self-healing is enabled
    pub self_healing_enabled: bool,

    /// Maximum actions per hour
    pub max_actions_per_hour: u32,

    /// Whether to allow service restarts
    pub allow_service_restart: bool,

    /// Whether to allow package operations
    pub allow_package_operations: bool,
}

impl Default for ActionExecutionPolicy {
    fn default() -> Self {
        Self {
            self_healing_enabled: false, // Disabled by default for safety
            max_actions_per_hour: 3,
            allow_service_restart: false,
            allow_package_operations: false,
        }
    }
}

impl ActionExecutionPolicy {
    /// Check if action can be executed
    pub fn can_execute(&self, action: &PredictionAction) -> bool {
        // Rule 1: Self-healing must be enabled
        if !self.self_healing_enabled {
            return false;
        }

        // Rule 2: Only auto-executable actions (Low or None risk)
        if !action.auto_executable {
            return false;
        }

        // Rule 3: Check specific permissions
        if let Some(command) = &action.command {
            let lower = command.to_lowercase();

            if (lower.contains("restart") || lower.contains("systemctl"))
                && !self.allow_service_restart {
                return false;
            }

            if (lower.contains("pacman") || lower.contains("yay"))
                && !self.allow_package_operations {
                return false;
            }
        }

        true
    }

    /// Get execution decision for action
    pub fn get_decision(&self, action: &PredictionAction) -> ActionDecision {
        if !self.self_healing_enabled {
            return ActionDecision::Disabled;
        }

        if !action.auto_executable {
            return ActionDecision::RequiresApproval(format!(
                "Action has {} risk and requires manual execution",
                action.risk.as_str()
            ));
        }

        if self.can_execute(action) {
            ActionDecision::AutoExecute
        } else {
            ActionDecision::RequiresApproval("Policy restrictions prevent auto-execution".to_string())
        }
    }
}

/// Action execution decision
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionDecision {
    /// Action can be auto-executed
    AutoExecute,
    /// Action requires user approval
    RequiresApproval(String),
    /// Self-healing is disabled
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_classification() {
        assert_eq!(
            PredictionActionMapper::classify_action_risk("Clear pacman cache"),
            ActionRisk::Low
        );

        assert_eq!(
            PredictionActionMapper::classify_action_risk("Update system packages"),
            ActionRisk::Medium
        );

        assert_eq!(
            PredictionActionMapper::classify_action_risk("Delete old logs"),
            ActionRisk::Critical
        );
    }

    #[test]
    fn test_auto_executable() {
        assert!(ActionRisk::Low.is_auto_executable());
        assert!(ActionRisk::None.is_auto_executable());
        assert!(!ActionRisk::Medium.is_auto_executable());
        assert!(!ActionRisk::High.is_auto_executable());
    }

    #[test]
    fn test_execution_policy_default_safe() {
        let policy = ActionExecutionPolicy::default();
        assert!(!policy.self_healing_enabled);

        let action = PredictionAction::new(
            "test-1",
            "Test Action",
            "Test",
            ActionRisk::Low,
        );

        assert!(!policy.can_execute(&action));
    }
}

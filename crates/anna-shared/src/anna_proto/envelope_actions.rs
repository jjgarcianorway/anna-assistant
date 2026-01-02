//! Action-related types - Action, ActionType, ActionPayload, RiskLevel.

use serde::{Deserialize, Serialize};

/// A proposed action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Type of action.
    #[serde(rename = "type")]
    pub action_type: ActionType,
    /// Action-specific payload.
    pub payload: ActionPayload,
    /// Risk level.
    #[serde(default)]
    pub risk: RiskLevel,
    /// Whether user confirmation is required.
    #[serde(default)]
    pub requires_confirmation: bool,
}

impl Action {
    /// Create a probe action.
    pub fn probe(probe_id: &str) -> Self {
        Self {
            action_type: ActionType::Probe,
            payload: ActionPayload::Probe {
                probe_id: probe_id.to_string(),
            },
            risk: RiskLevel::Safe,
            requires_confirmation: false,
        }
    }

    /// Create an ask_user action.
    pub fn ask_user(question: &str) -> Self {
        Self {
            action_type: ActionType::AskUser,
            payload: ActionPayload::AskUser {
                question: question.to_string(),
            },
            risk: RiskLevel::Safe,
            requires_confirmation: false,
        }
    }

    /// Create a propose_change action.
    pub fn propose_change(description: &str, command: &str) -> Self {
        Self {
            action_type: ActionType::ProposeChange,
            payload: ActionPayload::ProposeChange {
                description: description.to_string(),
                command: command.to_string(),
            },
            risk: RiskLevel::Risky,
            requires_confirmation: true,
        }
    }

    /// Create an install_helper action.
    pub fn install_helper(helper: &str) -> Self {
        Self {
            action_type: ActionType::InstallHelper,
            payload: ActionPayload::InstallHelper {
                helper_name: helper.to_string(),
            },
            risk: RiskLevel::Risky,
            requires_confirmation: true,
        }
    }
}

/// Type of action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Run a probe.
    Probe,
    /// Ask the user a question.
    AskUser,
    /// Propose a system change.
    ProposeChange,
    /// Install a helper tool.
    InstallHelper,
}

/// Action-specific payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionPayload {
    /// Probe action payload.
    Probe { probe_id: String },
    /// Ask user payload.
    AskUser { question: String },
    /// Propose change payload.
    ProposeChange {
        description: String,
        command: String,
    },
    /// Install helper payload.
    InstallHelper { helper_name: String },
    /// Generic/unknown payload.
    Other(serde_json::Value),
}

/// Risk level for an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Safe action (read-only, no side effects).
    #[default]
    Safe,
    /// Risky action (may modify system).
    Risky,
}

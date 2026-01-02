//! Action-related types for specialist protocol.

use serde::{Deserialize, Serialize};

/// Actions that can be taken
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseActions {
    /// Actions that Anna COULD run if user confirms
    #[serde(default)]
    pub proposed: Vec<ProposedAction>,

    /// Actions already applied (rare, low-risk only)
    #[serde(default)]
    pub auto_applied: Vec<AppliedAction>,
}

/// An action that can be proposed to the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Unique action ID
    pub id: String,

    /// Human-readable description
    pub description: String,

    /// Risk level
    pub risk: RiskLevel,

    /// Whether sudo is required
    #[serde(default)]
    pub requires_sudo: bool,

    /// Commands to execute
    #[serde(default)]
    pub commands: Vec<String>,
}

/// An action that was auto-applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedAction {
    /// Action ID
    pub id: String,

    /// What was done
    pub description: String,

    /// Result
    pub result: String,
}

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

impl std::cmp::PartialOrd for RiskLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for RiskLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_rank = match self {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
        };
        let other_rank = match other {
            RiskLevel::Low => 0,
            RiskLevel::Medium => 1,
            RiskLevel::High => 2,
        };
        self_rank.cmp(&other_rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
    }
}

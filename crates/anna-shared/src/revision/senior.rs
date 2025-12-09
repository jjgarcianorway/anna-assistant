//! Senior escalation types (v0.0.208).

use serde::{Deserialize, Serialize};

use super::types::RevisionInstruction;

/// Senior escalation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorEscalation {
    /// Whether senior was able to provide useful guidance
    pub successful: bool,
    /// Revision instructions from senior
    pub instruction: RevisionInstruction,
    /// Optional explanation for why escalation was needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SeniorEscalation {
    /// Create a successful escalation with instructions
    pub fn success(instruction: RevisionInstruction) -> Self {
        Self {
            successful: true,
            instruction,
            reason: None,
        }
    }

    /// Create a failed escalation (senior couldn't help)
    pub fn failed(reason: impl Into<String>) -> Self {
        Self {
            successful: false,
            instruction: RevisionInstruction::none(),
            reason: Some(reason.into()),
        }
    }
}

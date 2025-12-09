//! Junior verification types (v0.0.208).

use serde::{Deserialize, Serialize};

use super::types::RevisionInstruction;

/// Junior verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorVerification {
    /// Reliability score (0-100)
    pub score: u8,
    /// Whether the answer meets the threshold
    pub verified: bool,
    /// Revision instructions if not verified
    pub instruction: RevisionInstruction,
}

impl JuniorVerification {
    /// Create a verified result (score meets threshold)
    pub fn verified(score: u8) -> Self {
        Self {
            score,
            verified: true,
            instruction: RevisionInstruction::none(),
        }
    }

    /// Create a result requiring revision
    pub fn needs_revision(score: u8, instruction: RevisionInstruction) -> Self {
        Self {
            score,
            verified: false,
            instruction,
        }
    }
}

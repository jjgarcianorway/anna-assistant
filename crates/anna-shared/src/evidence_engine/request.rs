//! Evidence request types

use serde::{Deserialize, Serialize};

use super::domain::EvidenceDomain;
use super::intent::EvidenceIntent;

/// Request for evidence gathering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequest {
    /// Ticket ID for tracking
    pub ticket_id: String,
    /// Domain classification
    pub domain: EvidenceDomain,
    /// Intent classification
    pub intent: EvidenceIntent,
    /// Original user question
    pub question: String,
    /// Tags extracted by translator (e.g., ["vim", "syntax", "editor"])
    pub tags: Vec<String>,
}

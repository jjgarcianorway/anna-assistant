//! Intent handler types and result structures.

use crate::strict_contract::StrictSpecialistResponse;

/// Intent handler result
pub enum HandlerResult {
    /// Successfully handled - return this response
    Success(StrictSpecialistResponse),
    /// Missing required probe - specify which one
    MissingProbe { probe_name: String, reason: String },
    /// Cannot handle deterministically - fall back to LLM
    NeedsSpecialist { reason: String },
}

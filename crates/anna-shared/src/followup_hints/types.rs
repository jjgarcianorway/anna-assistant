//! Follow-up hint types.

/// A follow-up hint to append to an answer
#[derive(Debug, Clone)]
pub struct FollowupHint {
    /// The suggestion text
    pub hint: String,
    /// Optional command to try
    pub command: Option<String>,
    /// Relevance score (0-100)
    pub relevance: u8,
}

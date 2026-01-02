//! Progress tracking for streaming responses.

use serde::{Deserialize, Serialize};

/// Progress frame (optional streaming support).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressFrame {
    /// Frame type.
    #[serde(rename = "type")]
    pub frame_type: ProgressType,
    /// Progress percentage (0-100).
    pub progress: Option<u8>,
    /// Status message.
    pub message: Option<String>,
}

impl ProgressFrame {
    /// Create a progress update.
    pub fn progress(percent: u8, message: &str) -> Self {
        Self {
            frame_type: ProgressType::Progress,
            progress: Some(percent.min(100)),
            message: Some(message.to_string()),
        }
    }

    /// Create a thinking status.
    pub fn thinking(message: &str) -> Self {
        Self {
            frame_type: ProgressType::Thinking,
            progress: None,
            message: Some(message.to_string()),
        }
    }
}

/// Progress frame type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressType {
    /// Progress update.
    Progress,
    /// Model is thinking.
    Thinking,
    /// Result is ready.
    Result,
}

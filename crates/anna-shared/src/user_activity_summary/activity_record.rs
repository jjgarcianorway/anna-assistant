//! Activity record types

use serde::{Deserialize, Serialize};

/// A single activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    /// Timestamp of activity
    pub timestamp: u64,
    /// Type of activity
    pub activity_type: String,
    /// Topic/category (if detected)
    pub topic: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Duration in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
}

impl ActivityRecord {
    /// Create a new activity record
    pub fn new(activity_type: impl Into<String>, timestamp: u64) -> Self {
        Self {
            timestamp,
            activity_type: activity_type.into(),
            topic: None,
            session_id: None,
            duration_ms: None,
        }
    }
}

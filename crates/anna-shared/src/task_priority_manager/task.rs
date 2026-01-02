//! Managed Task Types

use serde::{Deserialize, Serialize};
use super::priority::TaskPriority;
use super::state::TaskState;

/// A managed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedTask {
    /// Task ID
    pub id: String,
    /// Task description
    pub description: String,
    /// Priority level
    pub priority: TaskPriority,
    /// Current state
    pub state: TaskState,
    /// Created timestamp
    pub created_at: u64,
    /// Started timestamp
    pub started_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
    /// Blocked reason
    pub blocked_reason: Option<String>,
}

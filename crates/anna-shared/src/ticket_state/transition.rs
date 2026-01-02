//! State transition tracking

use serde::{Deserialize, Serialize};

use super::state::TicketState;

/// A state transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Previous state
    pub from: TicketState,
    /// New state
    pub to: TicketState,
    /// Timestamp (Unix millis)
    pub at: u64,
    /// Optional reason/context
    pub reason: Option<String>,
}

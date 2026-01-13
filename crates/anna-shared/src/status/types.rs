//! Basic status enums and types.

use serde::{Deserialize, Serialize};

/// Daemon state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DaemonState {
    #[default]
    Starting,
    Ready,
    Error,
}

impl std::fmt::Display for DaemonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonState::Starting => write!(f, "STARTING"),
            DaemonState::Ready => write!(f, "READY"),
            DaemonState::Error => write!(f, "ERROR"),
        }
    }
}

/// Update check state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UpdateCheckState {
    #[default]
    NeverChecked,
    Success,
    Failed,
    Checking,
}

impl std::fmt::Display for UpdateCheckState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateCheckState::NeverChecked => write!(f, "NEVER_CHECKED"),
            UpdateCheckState::Success => write!(f, "OK"),
            UpdateCheckState::Failed => write!(f, "FAILED"),
            UpdateCheckState::Checking => write!(f, "CHECKING"),
        }
    }
}

/// v0.3.3: Ticket status
/// v0.3.29: Added Investigating and Experimenting for UX clarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TicketStatus {
    #[default]
    Open,
    /// v0.3.29: Actively investigating - running probes, gathering info
    Investigating,
    /// v0.3.29: Running experiments to test hypotheses
    Experimenting,
    InProgress,
    Escalated,
    Resolved,
    Failed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketStatus::Open => write!(f, "open"),
            TicketStatus::Investigating => write!(f, "investigating"),
            TicketStatus::Experimenting => write!(f, "experimenting"),
            TicketStatus::InProgress => write!(f, "in-progress"),
            TicketStatus::Escalated => write!(f, "escalated"),
            TicketStatus::Resolved => write!(f, "resolved"),
            TicketStatus::Failed => write!(f, "failed"),
        }
    }
}

/// v0.3.3: Specialist availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpecialistStatus {
    #[default]
    Available,
    Busy,
    Offline,
}

impl std::fmt::Display for SpecialistStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialistStatus::Available => write!(f, "available"),
            SpecialistStatus::Busy => write!(f, "busy"),
            SpecialistStatus::Offline => write!(f, "offline"),
        }
    }
}

/// v0.3.25: Default true for serde
pub fn default_true() -> bool {
    true
}

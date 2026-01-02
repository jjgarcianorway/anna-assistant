// v0.0.528: Team Specialist Roster - Type Definitions
// Enums and type definitions for the specialist roster system

use serde::{Deserialize, Serialize};

/// Specialist seniority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeniorityLevel {
    Junior,
    Senior,
}

impl Default for SeniorityLevel {
    fn default() -> Self {
        Self::Junior
    }
}

impl std::fmt::Display for SeniorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Junior => write!(f, "Junior"),
            Self::Senior => write!(f, "Senior"),
        }
    }
}

/// Department/team type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Department {
    Desktop,
    Network,
    Security,
    Storage,
    Audio,
    Video,
    System,
    Database,
    DevOps,
    Support,
}

impl Default for Department {
    fn default() -> Self {
        Self::System
    }
}

impl std::fmt::Display for Department {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "Desktop"),
            Self::Network => write!(f, "Network"),
            Self::Security => write!(f, "Security"),
            Self::Storage => write!(f, "Storage"),
            Self::Audio => write!(f, "Audio"),
            Self::Video => write!(f, "Video"),
            Self::System => write!(f, "System"),
            Self::Database => write!(f, "Database"),
            Self::DevOps => write!(f, "DevOps"),
            Self::Support => write!(f, "Support"),
        }
    }
}

/// Availability status of a specialist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AvailabilityStatus {
    #[default]
    Available,
    Busy,
    OnTicket,
    Unavailable,
}

impl std::fmt::Display for AvailabilityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::Busy => write!(f, "Busy"),
            Self::OnTicket => write!(f, "On Ticket"),
            Self::Unavailable => write!(f, "Unavailable"),
        }
    }
}

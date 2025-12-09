//! Transcript actor types (v0.0.178).

use serde::{Deserialize, Serialize};

/// Actor in the transcript (who is speaking/acting)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    You,        // The user
    Anna,       // Anna's final response
    Translator, // LLM translator stage
    Dispatcher, // Probe dispatcher
    Probe,      // System probe execution
    Specialist, // Domain specialist LLM
    Supervisor, // Quality/reliability validator
    Junior,     // Junior reviewer (v0.0.25 tickets)
    Senior,     // Senior reviewer (v0.0.25 tickets)
    Annad,      // Daemon for probe execution (v0.0.25 tickets)
    System,     // System messages (errors, timeouts, etc.)
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::You => write!(f, "you"),
            Self::Anna => write!(f, "anna"),
            Self::Translator => write!(f, "translator"),
            Self::Dispatcher => write!(f, "dispatcher"),
            Self::Probe => write!(f, "probe"),
            Self::Specialist => write!(f, "specialist"),
            Self::Supervisor => write!(f, "supervisor"),
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
            Self::Annad => write!(f, "annad"),
            Self::System => write!(f, "system"),
        }
    }
}

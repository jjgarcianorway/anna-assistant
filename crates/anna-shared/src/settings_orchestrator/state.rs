// v0.0.574: Orchestrator State Types
// State management for the settings orchestrator

use serde::{Deserialize, Serialize};

/// Orchestrator state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OrchestratorState {
    /// Not initialized
    #[default]
    Uninitialized,
    /// Initializing
    Initializing,
    /// Ready
    Ready,
    /// Busy (processing)
    Busy,
    /// Error state
    Error,
}

impl std::fmt::Display for OrchestratorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "Uninitialized"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Ready => write!(f, "Ready"),
            Self::Busy => write!(f, "Busy"),
            Self::Error => write!(f, "Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_state_display() {
        assert_eq!(format!("{}", OrchestratorState::Ready), "Ready");
        assert_eq!(format!("{}", OrchestratorState::Busy), "Busy");
    }
}

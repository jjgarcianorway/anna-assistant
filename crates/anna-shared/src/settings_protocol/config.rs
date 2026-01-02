// v0.0.728: Settings Protocol - Configuration

use serde::{Deserialize, Serialize};
use super::types::{ProtocolType, ProtocolStatus};

/// Protocol config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Name
    pub name: String,
    /// Protocol type
    pub protocol_type: ProtocolType,
    /// Status
    pub status: ProtocolStatus,
    /// Max clauses
    pub max_clauses: usize,
}

impl ProtocolConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            protocol_type: ProtocolType::Amendment,
            status: ProtocolStatus::Draft,
            max_clauses: 100,
        }
    }

    /// Set type
    pub fn protocol_type(mut self, pt: ProtocolType) -> Self {
        self.protocol_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ProtocolStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max clauses
    pub fn max_clauses(mut self, max: usize) -> Self {
        self.max_clauses = max;
        self
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

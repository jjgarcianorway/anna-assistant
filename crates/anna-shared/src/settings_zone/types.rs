// v0.0.742: Settings Zone Types (Phase 318)

use serde::{Deserialize, Serialize};

/// Zone type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ZoneType {
    /// Free trade zone
    #[default]
    FreeTrade,
    /// Economic zone
    Economic,
    /// Security zone
    Security,
    /// Buffer zone
    Buffer,
}

impl std::fmt::Display for ZoneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FreeTrade => write!(f, "free-trade"),
            Self::Economic => write!(f, "economic"),
            Self::Security => write!(f, "security"),
            Self::Buffer => write!(f, "buffer"),
        }
    }
}

/// Zone status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ZoneStatus {
    /// Proposed status
    #[default]
    Proposed,
    /// Established status
    Established,
    /// Operational status
    Operational,
    /// Suspended status
    Suspended,
}

impl std::fmt::Display for ZoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Established => write!(f, "established"),
            Self::Operational => write!(f, "operational"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_type_display() {
        assert_eq!(format!("{}", ZoneType::FreeTrade), "free-trade");
        assert_eq!(format!("{}", ZoneType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ZoneStatus::Proposed), "proposed");
        assert_eq!(format!("{}", ZoneStatus::Operational), "operational");
    }
}

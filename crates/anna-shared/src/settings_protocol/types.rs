// v0.0.728: Settings Protocol - Type Definitions

use serde::{Deserialize, Serialize};

/// Protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProtocolType {
    /// Amendment protocol
    #[default]
    Amendment,
    /// Optional protocol
    Optional,
    /// Supplementary protocol
    Supplementary,
    /// Implementation protocol
    Implementation,
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Amendment => write!(f, "amendment"),
            Self::Optional => write!(f, "optional"),
            Self::Supplementary => write!(f, "supplementary"),
            Self::Implementation => write!(f, "implementation"),
        }
    }
}

/// Protocol status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProtocolStatus {
    /// Draft status
    #[default]
    Draft,
    /// Open status
    Open,
    /// Active status
    Active,
    /// Closed status
    Closed,
}

impl std::fmt::Display for ProtocolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Open => write!(f, "open"),
            Self::Active => write!(f, "active"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

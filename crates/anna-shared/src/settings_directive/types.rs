// v0.0.718: Settings Directive Types (Phase 294)
// Directive type and authority definitions

use serde::{Deserialize, Serialize};

/// Directive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DirectiveType {
    /// Mandatory directive
    #[default]
    Mandatory,
    /// Recommended directive
    Recommended,
    /// Advisory directive
    Advisory,
    /// Optional directive
    Optional,
}

impl std::fmt::Display for DirectiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mandatory => write!(f, "mandatory"),
            Self::Recommended => write!(f, "recommended"),
            Self::Advisory => write!(f, "advisory"),
            Self::Optional => write!(f, "optional"),
        }
    }
}

/// Directive authority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DirectiveAuthority {
    /// System authority
    #[default]
    System,
    /// Admin authority
    Admin,
    /// Policy authority
    Policy,
    /// Executive authority
    Executive,
}

impl std::fmt::Display for DirectiveAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Admin => write!(f, "admin"),
            Self::Policy => write!(f, "policy"),
            Self::Executive => write!(f, "executive"),
        }
    }
}

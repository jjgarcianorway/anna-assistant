// v0.0.664: Settings Resolution Types
// Basic enums and type definitions

use serde::{Deserialize, Serialize};

/// Resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ResolutionStrategy {
    /// Direct value lookup
    #[default]
    Direct,
    /// Follow references
    Reference,
    /// Compute from dependencies
    Computed,
    /// Use cached value
    Cached,
    /// Use default value
    Default,
}

impl std::fmt::Display for ResolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::Reference => write!(f, "reference"),
            Self::Computed => write!(f, "computed"),
            Self::Cached => write!(f, "cached"),
            Self::Default => write!(f, "default"),
        }
    }
}

/// Resolution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResolutionStatus {
    /// Resolved successfully
    #[default]
    Resolved,
    /// Pending resolution
    Pending,
    /// Failed to resolve
    Failed,
    /// Circular reference detected
    Circular,
    /// Not found
    NotFound,
}

impl std::fmt::Display for ResolutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolved => write!(f, "resolved"),
            Self::Pending => write!(f, "pending"),
            Self::Failed => write!(f, "failed"),
            Self::Circular => write!(f, "circular"),
            Self::NotFound => write!(f, "not_found"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_display() {
        assert_eq!(format!("{}", ResolutionStrategy::Direct), "direct");
        assert_eq!(format!("{}", ResolutionStrategy::Reference), "reference");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ResolutionStatus::Resolved), "resolved");
        assert_eq!(format!("{}", ResolutionStatus::Circular), "circular");
    }
}

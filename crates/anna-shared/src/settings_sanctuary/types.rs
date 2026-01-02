// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Types

use serde::{Deserialize, Serialize};

/// Sanctuary type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SanctuaryType {
    /// Wildlife sanctuary
    #[default]
    Wildlife,
    /// Marine sanctuary
    Marine,
    /// Bird sanctuary
    Bird,
    /// Forest sanctuary
    Forest,
}

impl std::fmt::Display for SanctuaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wildlife => write!(f, "wildlife"),
            Self::Marine => write!(f, "marine"),
            Self::Bird => write!(f, "bird"),
            Self::Forest => write!(f, "forest"),
        }
    }
}

/// Sanctuary status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SanctuaryStatus {
    /// Protected status
    #[default]
    Protected,
    /// Monitored status
    Monitored,
    /// Rehabilitating status
    Rehabilitating,
    /// Expanding status
    Expanding,
}

impl std::fmt::Display for SanctuaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protected => write!(f, "protected"),
            Self::Monitored => write!(f, "monitored"),
            Self::Rehabilitating => write!(f, "rehabilitating"),
            Self::Expanding => write!(f, "expanding"),
        }
    }
}

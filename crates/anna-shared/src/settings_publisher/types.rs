// v0.0.634: Settings Publisher Types (Phase 210)
// Basic types for settings publisher

use serde::{Deserialize, Serialize};

/// Publisher type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PublisherType {
    /// System publisher
    #[default]
    System,
    /// Application publisher
    Application,
    /// Service publisher
    Service,
    /// Plugin publisher
    Plugin,
    /// External publisher
    External,
}

impl std::fmt::Display for PublisherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Application => write!(f, "application"),
            Self::Service => write!(f, "service"),
            Self::Plugin => write!(f, "plugin"),
            Self::External => write!(f, "external"),
        }
    }
}

/// Publication scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PublicationScope {
    /// Local scope
    #[default]
    Local,
    /// Module scope
    Module,
    /// Application scope
    Application,
    /// System scope
    System,
    /// Global scope
    Global,
}

impl std::fmt::Display for PublicationScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Module => write!(f, "module"),
            Self::Application => write!(f, "application"),
            Self::System => write!(f, "system"),
            Self::Global => write!(f, "global"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publisher_type_display() {
        assert_eq!(format!("{}", PublisherType::System), "system");
        assert_eq!(format!("{}", PublisherType::Application), "application");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", PublicationScope::Local), "local");
        assert_eq!(format!("{}", PublicationScope::Global), "global");
    }
}

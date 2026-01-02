//! Recipe category classification.

use serde::{Deserialize, Serialize};

/// Recipe category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecipeCategory {
    /// Package management
    Package,
    /// Service management
    Service,
    /// Configuration files
    Config,
    /// System monitoring
    System,
    /// Network operations
    Network,
    /// Storage management
    Storage,
    /// Docker/containers
    Container,
    /// Git operations
    Git,
    /// Editor configuration
    Editor,
    /// Security-related
    Security,
    /// Custom/other
    Custom,
}

impl RecipeCategory {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Package => "Package",
            Self::Service => "Service",
            Self::Config => "Config",
            Self::System => "System",
            Self::Network => "Network",
            Self::Storage => "Storage",
            Self::Container => "Container",
            Self::Git => "Git",
            Self::Editor => "Editor",
            Self::Security => "Security",
            Self::Custom => "Custom",
        }
    }

    /// All categories
    pub fn all() -> Vec<Self> {
        vec![
            Self::Package,
            Self::Service,
            Self::Config,
            Self::System,
            Self::Network,
            Self::Storage,
            Self::Container,
            Self::Git,
            Self::Editor,
            Self::Security,
            Self::Custom,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_display() {
        assert_eq!(RecipeCategory::Package.display_name(), "Package");
        assert_eq!(RecipeCategory::Container.display_name(), "Container");
    }
}

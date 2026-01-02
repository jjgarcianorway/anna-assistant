// v0.0.680: Settings Expander Types (Phase 256)
// Core types and enums for settings expansion

use serde::{Deserialize, Serialize};

/// Expand mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ExpandMode {
    /// Expand environment variables
    #[default]
    Environment,
    /// Expand references to other settings
    Reference,
    /// Expand template strings
    Template,
    /// Expand all
    All,
}

impl std::fmt::Display for ExpandMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Environment => write!(f, "environment"),
            Self::Reference => write!(f, "reference"),
            Self::Template => write!(f, "template"),
            Self::All => write!(f, "all"),
        }
    }
}

/// Variable syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VariableSyntax {
    /// Shell-style: ${VAR}
    #[default]
    Shell,
    /// Mustache-style: {{VAR}}
    Mustache,
    /// Percent-style: %VAR%
    Percent,
}

impl std::fmt::Display for VariableSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shell => write!(f, "shell"),
            Self::Mustache => write!(f, "mustache"),
            Self::Percent => write!(f, "percent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_mode_display() {
        assert_eq!(format!("{}", ExpandMode::Environment), "environment");
        assert_eq!(format!("{}", ExpandMode::Reference), "reference");
    }

    #[test]
    fn test_variable_syntax_display() {
        assert_eq!(format!("{}", VariableSyntax::Shell), "shell");
        assert_eq!(format!("{}", VariableSyntax::Mustache), "mustache");
    }
}

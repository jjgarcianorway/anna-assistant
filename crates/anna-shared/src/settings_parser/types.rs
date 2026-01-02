// v0.0.646: Settings Parser Types (Phase 222)
// Basic types and enums for settings parsing

use serde::{Deserialize, Serialize};

/// Parse source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ParseSource {
    /// JSON source
    #[default]
    Json,
    /// TOML source
    Toml,
    /// YAML source
    Yaml,
    /// INI source
    Ini,
    /// Environment source
    Env,
}

impl std::fmt::Display for ParseSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::Ini => write!(f, "ini"),
            Self::Env => write!(f, "env"),
        }
    }
}

/// Parse mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ParseMode {
    /// Strict mode
    #[default]
    Strict,
    /// Lenient mode
    Lenient,
    /// Permissive mode
    Permissive,
    /// Validate only
    ValidateOnly,
}

impl std::fmt::Display for ParseMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Lenient => write!(f, "lenient"),
            Self::Permissive => write!(f, "permissive"),
            Self::ValidateOnly => write!(f, "validate_only"),
        }
    }
}

/// Parse error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Line number
    pub line: Option<usize>,
    /// Column number
    pub column: Option<usize>,
    /// Key path
    pub path: Option<String>,
}

impl ParseError {
    /// Create new error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
            path: None,
        }
    }

    /// Set location
    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_source_display() {
        assert_eq!(format!("{}", ParseSource::Json), "json");
        assert_eq!(format!("{}", ParseSource::Toml), "toml");
    }

    #[test]
    fn test_parse_mode_display() {
        assert_eq!(format!("{}", ParseMode::Strict), "strict");
        assert_eq!(format!("{}", ParseMode::Lenient), "lenient");
    }

    #[test]
    fn test_error_new() {
        let e = ParseError::new("test error").at(1, 5);
        assert_eq!(e.line, Some(1));
        assert_eq!(e.column, Some(5));
    }
}

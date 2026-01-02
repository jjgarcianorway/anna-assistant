//! Config hint types for routing to specialists.

use serde::{Deserialize, Serialize};

/// v0.0.264: Hint for specialists about what config change the user wants.
/// This is NOT the answer - it's context to help the specialist understand the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHint {
    /// The editor/app being configured (vim, nano, helix, etc.)
    pub app_id: String,
    /// What feature the user wants (syntax, line_numbers, theme, etc.)
    pub feature: ConfigFeatureHint,
    /// Whether they want to enable or disable it
    pub enable: bool,
    /// Optional parameter value (e.g., theme name, tab width)
    pub param: Option<String>,
}

/// v0.0.264: Feature categories for config hints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFeatureHint {
    /// Syntax highlighting
    Syntax,
    /// Line numbers (absolute, relative, or off)
    LineNumbers,
    /// Color theme/colorscheme
    Theme,
    /// Indentation settings
    Indent,
    /// Mouse support
    Mouse,
    /// Word wrap
    WordWrap,
    /// Search highlighting
    SearchHighlight,
    /// Cursor line
    CursorLine,
    /// Status line
    StatusLine,
    /// Unknown feature - needs specialist interpretation
    Unknown,
}

impl ConfigFeatureHint {
    /// Parse from query keywords
    pub fn from_query(query: &str) -> Self {
        let q = query.to_lowercase();
        if q.contains("syntax") || q.contains("highlight") && !q.contains("search") {
            Self::Syntax
        } else if q.contains("line number") || q.contains("linenumber") {
            Self::LineNumbers
        } else if q.contains("theme") || q.contains("colorscheme") || q.contains("color scheme") {
            Self::Theme
        } else if q.contains("indent") || q.contains("tab") || q.contains("spaces") {
            Self::Indent
        } else if q.contains("mouse") {
            Self::Mouse
        } else if q.contains("wrap") {
            Self::WordWrap
        } else if q.contains("search") && q.contains("highlight") {
            Self::SearchHighlight
        } else if q.contains("cursor line") || q.contains("cursorline") {
            Self::CursorLine
        } else if q.contains("status") {
            Self::StatusLine
        } else {
            Self::Unknown
        }
    }
}

impl std::fmt::Display for ConfigFeatureHint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Syntax => "syntax highlighting",
            Self::LineNumbers => "line numbers",
            Self::Theme => "color theme",
            Self::Indent => "indentation",
            Self::Mouse => "mouse support",
            Self::WordWrap => "word wrap",
            Self::SearchHighlight => "search highlighting",
            Self::CursorLine => "cursor line",
            Self::StatusLine => "status line",
            Self::Unknown => "configuration",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_feature_hint_display() {
        assert_eq!(ConfigFeatureHint::Syntax.to_string(), "syntax highlighting");
        assert_eq!(ConfigFeatureHint::LineNumbers.to_string(), "line numbers");
        assert_eq!(ConfigFeatureHint::Theme.to_string(), "color theme");
    }
}

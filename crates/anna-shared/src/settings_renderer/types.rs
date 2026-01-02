// v0.0.647: Settings Renderer Types (Phase 223)
// Core types for render targets and themes

use serde::{Deserialize, Serialize};

/// Render target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RenderTarget {
    /// Terminal output
    #[default]
    Terminal,
    /// HTML output
    Html,
    /// Markdown output
    Markdown,
    /// Plain text
    PlainText,
    /// Rich text
    RichText,
}

impl std::fmt::Display for RenderTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terminal => write!(f, "terminal"),
            Self::Html => write!(f, "html"),
            Self::Markdown => write!(f, "markdown"),
            Self::PlainText => write!(f, "plain_text"),
            Self::RichText => write!(f, "rich_text"),
        }
    }
}

/// Render theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RenderTheme {
    /// Default theme
    #[default]
    Default,
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Minimal theme
    Minimal,
    /// Custom theme
    Custom,
}

impl std::fmt::Display for RenderTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
            Self::Minimal => write!(f, "minimal"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_target_display() {
        assert_eq!(format!("{}", RenderTarget::Terminal), "terminal");
        assert_eq!(format!("{}", RenderTarget::Html), "html");
    }

    #[test]
    fn test_render_theme_display() {
        assert_eq!(format!("{}", RenderTheme::Dark), "dark");
        assert_eq!(format!("{}", RenderTheme::Light), "light");
    }
}

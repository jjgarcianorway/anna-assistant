// v0.0.647: Render Output (Phase 223)
// Output representation for rendered settings

use serde::{Deserialize, Serialize};
use super::types::{RenderTarget, RenderTheme};

/// Render output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderOutput {
    /// Content
    pub content: String,
    /// Target used
    pub target: RenderTarget,
    /// Theme used
    pub theme: RenderTheme,
    /// Line count
    pub line_count: usize,
}

impl RenderOutput {
    /// Create new output
    pub fn new(content: impl Into<String>, target: RenderTarget, theme: RenderTheme) -> Self {
        let content = content.into();
        let line_count = content.lines().count();
        Self {
            content,
            target,
            theme,
            line_count,
        }
    }

    /// Get content length
    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_new() {
        let o = RenderOutput::new("line1\nline2", RenderTarget::Terminal, RenderTheme::Default);
        assert_eq!(o.line_count, 2);
    }

    #[test]
    fn test_output_empty() {
        let o = RenderOutput::new("", RenderTarget::Terminal, RenderTheme::Default);
        assert!(o.is_empty());
    }
}

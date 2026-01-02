// v0.0.647: Renderer Configuration (Phase 223)
// Configuration for settings rendering

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{RenderTarget, RenderTheme};

/// Renderer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    /// Render target
    pub target: RenderTarget,
    /// Render theme
    pub theme: RenderTheme,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Show headers
    pub show_headers: bool,
    /// Show borders
    pub show_borders: bool,
    /// Max width
    pub max_width: Option<usize>,
}

impl RendererConfig {
    /// Create new config
    pub fn new(target: RenderTarget) -> Self {
        Self {
            target,
            theme: RenderTheme::Default,
            category: None,
            show_headers: true,
            show_borders: false,
            max_width: None,
        }
    }

    /// Set theme
    pub fn theme(mut self, theme: RenderTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set show headers
    pub fn show_headers(mut self, show: bool) -> Self {
        self.show_headers = show;
        self
    }

    /// Set show borders
    pub fn show_borders(mut self, show: bool) -> Self {
        self.show_borders = show;
        self
    }

    /// Set max width
    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self::new(RenderTarget::Terminal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = RendererConfig::new(RenderTarget::Terminal);
        assert!(c.show_headers);
    }

    #[test]
    fn test_config_builder() {
        let c = RendererConfig::new(RenderTarget::Html)
            .theme(RenderTheme::Dark)
            .show_borders(true);
        assert_eq!(c.theme, RenderTheme::Dark);
        assert!(c.show_borders);
    }
}

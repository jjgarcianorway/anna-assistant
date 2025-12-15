// v0.0.647: Settings Renderer (Phase 223)
// Renderer for displaying settings in various output formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

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

/// Renderer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RendererStats {
    /// Total renders
    pub total_renders: usize,
    /// By target
    pub by_target: HashMap<String, usize>,
    /// By theme
    pub by_theme: HashMap<String, usize>,
    /// Total lines rendered
    pub total_lines: usize,
}

impl RendererStats {
    /// Record render
    pub fn record(&mut self, target: RenderTarget, theme: RenderTheme, line_count: usize) {
        self.total_renders += 1;
        *self.by_target.entry(target.to_string()).or_insert(0) += 1;
        *self.by_theme.entry(theme.to_string()).or_insert(0) += 1;
        self.total_lines += line_count;
    }

    /// Average lines per render
    pub fn average_lines(&self) -> f64 {
        if self.total_renders == 0 {
            0.0
        } else {
            self.total_lines as f64 / self.total_renders as f64
        }
    }
}

/// Settings renderer
#[derive(Debug, Clone, Default)]
pub struct SettingsRenderer {
    /// Config
    config: RendererConfig,
    /// Outputs
    outputs: Vec<RenderOutput>,
    /// Stats
    stats: RendererStats,
}

impl SettingsRenderer {
    /// Create new renderer
    pub fn new(config: RendererConfig) -> Self {
        Self {
            config,
            outputs: Vec::new(),
            stats: RendererStats::default(),
        }
    }

    /// Render settings
    pub fn render(&mut self, settings: &[(String, String)]) -> RenderOutput {
        let content = self.do_render(settings);
        let output = RenderOutput::new(content, self.config.target, self.config.theme);

        self.stats.record(
            self.config.target,
            self.config.theme,
            output.line_count,
        );
        self.outputs.push(output.clone());
        output
    }

    /// Do render
    fn do_render(&self, settings: &[(String, String)]) -> String {
        let mut output = String::new();

        match self.config.target {
            RenderTarget::Terminal => {
                for (key, value) in settings {
                    output.push_str(&format!("{}: {}\n", key, value));
                }
            }
            RenderTarget::Html => {
                output.push_str("<dl>\n");
                for (key, value) in settings {
                    output.push_str(&format!("  <dt>{}</dt>\n  <dd>{}</dd>\n", key, value));
                }
                output.push_str("</dl>\n");
            }
            RenderTarget::Markdown => {
                for (key, value) in settings {
                    output.push_str(&format!("- **{}**: {}\n", key, value));
                }
            }
            RenderTarget::PlainText => {
                for (key, value) in settings {
                    output.push_str(&format!("{}={}\n", key, value));
                }
            }
            RenderTarget::RichText => {
                if self.config.show_headers {
                    output.push_str("Settings:\n");
                }
                for (key, value) in settings {
                    output.push_str(&format!("  {} → {}\n", key, value));
                }
            }
        }

        output
    }

    /// Get outputs
    pub fn outputs(&self) -> &[RenderOutput] {
        &self.outputs
    }

    /// Get stats
    pub fn stats(&self) -> &RendererStats {
        &self.stats
    }

    /// Output count
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Clear outputs
    pub fn clear(&mut self) {
        self.outputs.clear();
    }
}

/// Settings renderer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsRendererRegistry {
    /// Renderers by ID
    renderers: HashMap<String, SettingsRenderer>,
}

impl SettingsRendererRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register renderer
    pub fn register(&mut self, id: impl Into<String>, renderer: SettingsRenderer) {
        self.renderers.insert(id.into(), renderer);
    }

    /// Unregister renderer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.renderers.remove(id).is_some()
    }

    /// Get renderer
    pub fn get(&self, id: &str) -> Option<&SettingsRenderer> {
        self.renderers.get(id)
    }

    /// Get renderer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsRenderer> {
        self.renderers.get_mut(id)
    }

    /// Renderer count
    pub fn count(&self) -> usize {
        self.renderers.len()
    }
}

/// Format renderer registry
pub fn format_renderer_registry(registry: &SettingsRendererRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Renderer Registry:\n");
    output.push_str(&format!("  Renderers: {}\n", registry.count()));
    output
}

/// Check if query is about renderer
pub fn is_renderer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("renderer") || lower.contains("render settings") || lower.contains("display settings")
}

/// Fun fact about renderer
pub fn renderer_fun_fact() -> &'static str {
    "Anna's settings renderers display configs beautifully!"
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

    #[test]
    fn test_stats_record() {
        let mut s = RendererStats::default();
        s.record(RenderTarget::Terminal, RenderTheme::Default, 10);
        s.record(RenderTarget::Html, RenderTheme::Dark, 20);
        assert_eq!(s.total_renders, 2);
        assert_eq!(s.total_lines, 30);
    }

    #[test]
    fn test_renderer_new() {
        let r = SettingsRenderer::new(RendererConfig::new(RenderTarget::Terminal));
        assert_eq!(r.output_count(), 0);
    }

    #[test]
    fn test_renderer_render_terminal() {
        let mut r = SettingsRenderer::new(RendererConfig::new(RenderTarget::Terminal));
        let settings = vec![("key".to_string(), "value".to_string())];
        let o = r.render(&settings);
        assert!(o.content.contains("key: value"));
    }

    #[test]
    fn test_renderer_render_markdown() {
        let mut r = SettingsRenderer::new(RendererConfig::new(RenderTarget::Markdown));
        let settings = vec![("key".to_string(), "value".to_string())];
        let o = r.render(&settings);
        assert!(o.content.contains("**key**"));
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsRendererRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsRendererRegistry::new();
        r.register("rend1", SettingsRenderer::new(RendererConfig::new(RenderTarget::Terminal)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_renderer_query() {
        assert!(is_renderer_query("settings renderer"));
        assert!(!is_renderer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = renderer_fun_fact();
        assert!(fact.contains("renderer"));
    }
}

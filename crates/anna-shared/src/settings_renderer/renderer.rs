// v0.0.647: Settings Renderer Implementation (Phase 223)
// Main renderer implementation for displaying settings

use super::config::RendererConfig;
use super::output::RenderOutput;
use super::stats::RendererStats;
use super::types::RenderTarget;

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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::RenderTheme;

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
}

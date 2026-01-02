// v0.0.647: Settings Renderer Registry (Phase 223)
// Registry for managing multiple settings renderers

use std::collections::HashMap;
use super::renderer::SettingsRenderer;

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
    use super::super::config::RendererConfig;
    use super::super::types::RenderTarget;

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

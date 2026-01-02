// v0.0.647: Settings Renderer Module (Phase 223)
// Renderer for displaying settings in various output formats

mod types;
mod config;
mod output;
mod stats;
mod renderer;
mod registry;

// Re-export public API
pub use types::{RenderTarget, RenderTheme};
pub use config::RendererConfig;
pub use output::RenderOutput;
pub use stats::RendererStats;
pub use renderer::SettingsRenderer;
pub use registry::{
    SettingsRendererRegistry,
    format_renderer_registry,
    is_renderer_query,
    renderer_fun_fact,
};

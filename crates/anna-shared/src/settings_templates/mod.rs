// v0.0.569: Settings Templates Module (Phase 145)
// Create and apply reusable settings templates for different scenarios

mod types;
mod manager;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use types::{TemplateScope, TemplateUseCase, TemplateMeta, SettingsTemplate};
pub use manager::TemplateManager;
pub use helpers::{apply_category, format_templates, is_template_query, template_fun_fact};

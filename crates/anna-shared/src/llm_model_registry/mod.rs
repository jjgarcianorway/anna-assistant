// v0.0.531: LLM Model Registry (Phase 107)
// Tracks installed LLM models and their assignments to specialists per VISION.md

mod types;
mod registry;
mod formatting;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain the original API
pub use types::{ModelCapability, ModelStatus, InstalledBy, ModelRecord};
pub use registry::LlmModelRegistry;
pub use formatting::{format_model, format_model_compact, format_model_oneline, format_registry_summary};
pub use utils::{is_model_query, model_fun_fact};

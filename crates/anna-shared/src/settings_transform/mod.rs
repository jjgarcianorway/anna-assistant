// v0.0.666: Settings Transform (Phase 242)
// Transform settings between different formats and structures

mod types;
mod config;
mod rule;
mod stats;
mod transformer;
mod registry;
mod utils;

// Re-export all public types to maintain API compatibility
pub use types::{TransformType, TransformDirection};
pub use config::TransformerConfig;
pub use rule::{TransformRule, TransformResult};
pub use stats::TransformerStats;
pub use transformer::SettingsTransformer;
pub use registry::{TransformerRegistry, format_transformer_registry};
pub use utils::{is_transformer_query, transformer_fun_fact};

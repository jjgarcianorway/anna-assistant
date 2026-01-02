// v0.0.598: Settings Transformer Module (Phase 174)
// Transformation pipeline for settings values

mod types;
mod pipeline;
mod manager;
mod utils;
#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use types::{
    TransformType,
    TransformDirection,
    TransformDef,
    TransformResult,
};

pub use pipeline::TransformPipeline;
pub use manager::TransformerManager;

pub use utils::{
    format_transform_pipeline,
    is_transformer_query,
    transformer_fun_fact,
};

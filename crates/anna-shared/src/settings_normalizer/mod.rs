// v0.0.645: Settings Normalizer Module (Phase 221)
// Normalizer for standardizing settings values

mod types;
mod config;
mod result;
mod stats;
mod normalizer;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{NormalizationType, NormalizationRule};
pub use config::NormalizerConfig;
pub use result::NormalizationResult;
pub use stats::NormalizerStats;
pub use normalizer::SettingsNormalizer;
pub use registry::{SettingsNormalizerRegistry, format_normalizer_registry};
pub use utils::{is_normalizer_query, normalizer_fun_fact};

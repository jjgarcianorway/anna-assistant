// v0.0.667: Settings Normalization (Phase 243)
// Normalize settings to a canonical format

mod types;
mod result;
mod normalizer;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public items to maintain the public API
pub use types::{NormalizationType, CaseStyle, NormalizerConfig};
pub use result::{NormalizationResult, NormalizerStats};
pub use normalizer::SettingsNormalizer;
pub use registry::NormalizerRegistry;
pub use utils::{format_normalizer_registry, is_normalizer_query, normalizer_fun_fact};

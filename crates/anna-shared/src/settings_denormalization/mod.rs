// v0.0.668: Settings Denormalization (Phase 244)
// Denormalize settings from canonical to target format
//
// This module is organized into the following submodules:
// - types: Core enums (DenormalizationType, TargetFormat)
// - config: Configuration for denormalizer
// - result: Result types for denormalization operations
// - stats: Statistics tracking
// - denormalizer: Main denormalizer implementation
// - registry: Registry for managing multiple denormalizers
// - utils: Utility functions

mod types;
mod config;
mod result;
mod stats;
mod denormalizer;
mod registry;
mod utils;

// Re-export all public items to preserve the original API
pub use types::{DenormalizationType, TargetFormat};
pub use config::DenormalizerConfig;
pub use result::DenormalizationResult;
pub use stats::DenormalizerStats;
pub use denormalizer::SettingsDenormalizer;
pub use registry::DenormalizerRegistry;
pub use utils::{format_denormalizer_registry, is_denormalizer_query, denormalizer_fun_fact};

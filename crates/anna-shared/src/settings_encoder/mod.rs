// v0.0.648: Settings Encoder (Phase 224)
// Encoder for serializing settings to various formats

mod format;
mod config;
mod result;
mod stats;
mod encoder;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use format::{EncodingFormat, EncodingOptions};
pub use config::EncoderConfig;
pub use result::EncodeResult;
pub use stats::EncoderStats;
pub use encoder::SettingsEncoder;
pub use registry::SettingsEncoderRegistry;
pub use utils::{format_encoder_registry, is_encoder_query, encoder_fun_fact};

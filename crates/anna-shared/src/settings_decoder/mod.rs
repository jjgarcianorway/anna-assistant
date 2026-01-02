// v0.0.649: Settings Decoder Module (Phase 225)
// Decoder for deserializing settings from various formats

mod types;
mod config;
mod result;
mod stats;
mod decoder;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{DecodingFormat, DecodingMode};
pub use config::DecoderConfig;
pub use result::{DecodeError, DecodeResult};
pub use stats::DecoderStats;
pub use decoder::SettingsDecoder;
pub use registry::SettingsDecoderRegistry;
pub use utils::{format_decoder_registry, is_decoder_query, decoder_fun_fact};

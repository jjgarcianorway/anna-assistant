// v0.0.649: Settings Decoder Utils (Phase 225)
// Utility functions for decoder

use super::registry::SettingsDecoderRegistry;

/// Format decoder registry
pub fn format_decoder_registry(registry: &SettingsDecoderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Decoder Registry:\n");
    output.push_str(&format!("  Decoders: {}\n", registry.count()));
    output
}

/// Check if query is about decoder
pub fn is_decoder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("decoder") || lower.contains("decode settings") || lower.contains("deserialize settings")
}

/// Fun fact about decoder
pub fn decoder_fun_fact() -> &'static str {
    "Anna's settings decoders parse configs from any format!"
}

// v0.0.648: Settings Encoder (Phase 224)
// Utility functions

use super::registry::SettingsEncoderRegistry;

/// Format encoder registry
pub fn format_encoder_registry(registry: &SettingsEncoderRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Encoder Registry:\n");
    output.push_str(&format!("  Encoders: {}\n", registry.count()));
    output
}

/// Check if query is about encoder
pub fn is_encoder_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("encoder") || lower.contains("encode settings") || lower.contains("serialize settings")
}

/// Fun fact about encoder
pub fn encoder_fun_fact() -> &'static str {
    "Anna's settings encoders serialize configs to any format!"
}

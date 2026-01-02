// v0.0.728: Settings Protocol - Helper Functions

use super::registry::ProtocolRegistry;

/// Format protocol registry
pub fn format_protocol_registry(registry: &ProtocolRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Protocol Registry:\n");
    output.push_str(&format!("  Protocols: {}\n", registry.count()));
    output
}

/// Check if query is about protocol
pub fn is_protocol_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings protocol") || lower.contains("protocol settings") || lower.contains("formal procedure")
}

/// Fun fact about protocol
pub fn protocol_fun_fact() -> &'static str {
    "Anna's settings protocol establishes formal governance procedures!"
}

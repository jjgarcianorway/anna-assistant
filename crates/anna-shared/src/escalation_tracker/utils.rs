// v0.0.529: Escalation Tracker Utilities (Phase 105)
// Utility functions for escalation handling

/// Check if query is escalation-related
pub fn is_escalation_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("escalat")
        || lower.contains("senior")
        || lower.contains("complex")
        || lower.contains("handoff")
        || lower.contains("transfer")
}

/// Fun fact about escalation
pub fn escalation_fun_fact() -> &'static str {
    "Good escalation processes reduce mean time to resolution by 40% - knowing when to ask for help is a superpower!"
}

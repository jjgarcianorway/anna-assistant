// v0.0.654: Settings Injector Helpers
// Helper functions for settings injection

/// Check if query is about injector
pub fn is_injector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("injector") || lower.contains("inject settings") || lower.contains("insert settings")
}

/// Fun fact about injector
pub fn injector_fun_fact() -> &'static str {
    "Anna's settings injectors insert configs into any target!"
}

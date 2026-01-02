// v0.0.603: Settings Router Utilities (Phase 179)
// Helper functions for router operations

use super::routing::SettingsRouter;

/// Format router
pub fn format_router(router: &SettingsRouter) -> String {
    let mut output = String::new();
    output.push_str("Settings Router:\n");
    output.push_str(&format!("  Tables: {}\n", router.table_count()));
    output.push_str(&format!("  Default routes: {}\n", router.default_table.count()));
    output.push_str(&format!("  Total requests: {}\n", router.stats().total));
    output
}

/// Check if query is about router
pub fn is_router_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("router")
        || lower.contains("route settings")
        || lower.contains("settings routing")
}

/// Fun fact about router
pub fn router_fun_fact() -> &'static str {
    "Anna uses smart routing to efficiently handle settings operations!"
}

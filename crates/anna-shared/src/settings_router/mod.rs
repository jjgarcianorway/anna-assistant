// v0.0.603: Settings Router (Phase 179)
// Routing logic for settings operations

mod types;
mod routing;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{RouteAction, RouteDef, RouteMatch, RouteStats, RouteType};
pub use routing::{RouteTable, SettingsRouter};
pub use utils::{format_router, is_router_query, router_fun_fact};

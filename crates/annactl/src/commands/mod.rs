//! Command handlers for annactl (v0.0.446).
//! v0.0.144: Simplified - removed unnecessary flags, natural language for everything.
//! v0.0.205: Modularized into domain-focused submodules.
//! v0.0.237: Added config command handler for natural language settings.
//! v0.0.323: Added learning command to show probe learning stats.
//! v0.0.328: Added query test option to learning command.
//! v0.0.406: Added suggest-recipes command for recipe candidate analysis.
//! v0.0.444: Added debug command for diagnostics.
//! v0.0.446: Enhanced debug with 4 levels and trace command.

pub mod config;
mod debug;
mod debug_trace;
mod feedback;
mod handlers;
mod learning;
mod repl;

// Re-export all handlers
#[allow(unused_imports)]
pub use config::{show_config_status, try_handle_config, ConfigResult};
pub use debug::{handle_debug, DebugCommand};
pub use handlers::{handle_request, handle_reset, handle_stats, handle_status, handle_uninstall};
pub use learning::{handle_learning_with_query, handle_suggest_recipes};
pub use repl::handle_repl;

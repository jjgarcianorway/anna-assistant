// v0.0.718: Settings Directive Module (Phase 294)
// Authoritative directives for settings management

mod types;
mod config;
mod order;
mod stats;
mod directive;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{DirectiveType, DirectiveAuthority};
pub use config::DirectiveConfig;
pub use order::{DirectiveOrder, DirectiveSupplement};
pub use stats::DirectiveStats;
pub use directive::SettingsDirective;
pub use registry::DirectiveRegistry;
pub use utils::{format_directive_registry, is_directive_query, directive_fun_fact};

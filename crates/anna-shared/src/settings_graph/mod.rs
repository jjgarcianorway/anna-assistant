// v0.0.663: Settings Graph (Phase 239)
// Graph for modeling settings relationships and dependencies

mod link_types;
mod linker_config;
mod settings_link;
mod linker;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use link_types::{LinkDirection, LinkType};
pub use linker_config::LinkerConfig;
pub use settings_link::{LinkResult, LinkerStats, SettingsLink};
pub use linker::{SettingsLinker, SettingsLinkerRegistry};
pub use utils::{format_graph_registry, graph_fun_fact, is_graph_query};

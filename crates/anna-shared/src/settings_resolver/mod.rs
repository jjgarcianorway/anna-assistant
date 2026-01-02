// v0.0.599: Settings Resolver Module (Phase 175)
// Resolution logic for settings conflicts and dependencies

mod config;
mod resolver;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use config::ResolverConfig;
pub use resolver::SettingsResolver;
pub use types::{Conflict, ConflictType, Dependency, Resolution, ResolutionStrategy};
pub use utils::{format_resolver, is_resolver_query, resolver_fun_fact};

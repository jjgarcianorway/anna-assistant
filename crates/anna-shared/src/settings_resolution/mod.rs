// v0.0.664: Settings Resolution (Phase 240)
// Resolve settings values with reference following and computation

pub mod types;
pub mod config;
pub mod result;
pub mod stats;
pub mod resolver;
pub mod registry;

// Re-export all public types
pub use types::{ResolutionStrategy, ResolutionStatus};
pub use config::ResolverConfig;
pub use result::{ResolutionResult, ResolutionRequest};
pub use stats::ResolverStats;
pub use resolver::SettingsResolver;
pub use registry::{
    SettingsResolverRegistry,
    format_resolver_registry,
    is_resolver_query,
    resolver_fun_fact,
};

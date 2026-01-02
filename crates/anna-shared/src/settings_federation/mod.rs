// v0.0.737: Settings Federation (Phase 313)
// Federal union for settings governance

mod types;
mod core;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain the original API
pub use types::{
    FederationType,
    FederationStatus,
    FederationConfig,
    FederationArticle,
    FederationState,
    FederationStats,
};

pub use core::SettingsFederation;
pub use registry::FederationRegistry;
pub use utils::{
    format_federation_registry,
    is_federation_query,
    federation_fun_fact,
};

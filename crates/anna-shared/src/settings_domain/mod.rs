// v0.0.743: Settings Domain Module (Phase 319)
// Sovereign domain for settings jurisdiction

mod domain_types;
mod domain_config;
mod domain_right;
mod domain_stats;
mod domain;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use domain_types::{DomainType, DomainStatus};
pub use domain_config::DomainConfig;
pub use domain_right::{DomainRight, DomainHolder};
pub use domain_stats::DomainStats;
pub use domain::{SettingsDomain, DomainRegistry};
pub use utils::{format_domain_registry, is_domain_query, domain_fun_fact};

// v0.0.722: Settings Ordinance (Phase 298)
// Local ordinances for settings regulation

mod types;
mod ordinance;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain the same API
pub use types::{
    OrdinanceAmendment,
    OrdinanceConfig,
    OrdinanceJurisdiction,
    OrdinanceProvision,
    OrdinanceStats,
    OrdinanceType,
};

pub use ordinance::SettingsOrdinance;
pub use registry::OrdinanceRegistry;
pub use helpers::{format_ordinance_registry, is_ordinance_query, ordinance_fun_fact};

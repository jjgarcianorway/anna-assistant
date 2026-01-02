// v0.0.721: Settings Mandate (Phase 297)
// Authoritative mandates for settings compliance

mod mandate;
mod registry;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use mandate::SettingsMandate;
pub use registry::MandateRegistry;
pub use types::{
    MandateCompliance, MandateConfig, MandateEvidence, MandateRequirement, MandateStats, MandateType,
};
pub use utils::{format_mandate_registry, is_mandate_query, mandate_fun_fact};

// v0.0.726: Settings Covenant (Phase 302)
// Binding agreement for settings governance

mod types;
mod covenant;
mod registry;
mod helpers;

// Re-export all public types and functions to maintain the original API
pub use types::{
    CovenantType,
    CovenantStatus,
    CovenantConfig,
    CovenantTerm,
    CovenantObligation,
    CovenantStats,
};

pub use covenant::SettingsCovenant;
pub use registry::CovenantRegistry;
pub use helpers::{
    format_covenant_registry,
    is_covenant_query,
    covenant_fun_fact,
};

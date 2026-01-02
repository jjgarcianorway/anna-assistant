// v0.0.730: Settings Accord (Phase 306)
// Formal accord for settings governance

mod accord;
mod helpers;
mod registry;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use accord::SettingsAccord;
pub use helpers::{accord_fun_fact, format_accord_registry, is_accord_query};
pub use registry::AccordRegistry;
pub use types::{
    AccordConfig, AccordProvision, AccordSignatory, AccordStats, AccordStatus, AccordType,
};

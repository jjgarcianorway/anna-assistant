// v0.0.697: Settings Dossier Module (Phase 273)
// Comprehensive dossier of settings information

mod types;
mod config;
mod document;
mod entry;
mod stats;
mod dossier;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use types::{DossierType, DossierClassification};
pub use config::DossierConfig;
pub use document::DossierDocument;
pub use entry::DossierEntry;
pub use stats::DossierStats;
pub use dossier::SettingsDossier;
pub use registry::DossierRegistry;
pub use utils::{format_dossier_registry, is_dossier_query, dossier_fun_fact};

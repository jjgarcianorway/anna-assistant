// v0.0.734: Settings Entente (Phase 310)
// Informal understanding for settings governance

mod types;
mod config;
mod understanding;
mod partner;
mod stats;
mod entente;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve API
pub use types::{EntenteType, EntenteStatus};
pub use config::EntenteConfig;
pub use understanding::EntenteUnderstanding;
pub use partner::EntentePartner;
pub use stats::EntenteStats;
pub use entente::SettingsEntente;
pub use registry::EntenteRegistry;
pub use utils::{format_entente_registry, is_entente_query, entente_fun_fact};

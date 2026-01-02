// v0.0.703: Settings Repertoire Module (Phase 279)
// Performance repertoire of available settings

mod types;
mod repertoire;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain the original public API
pub use types::{
    RepertoireType,
    RepertoireStatus,
    RepertoireConfig,
    RepertoirePiece,
    RepertoireItem,
    RepertoireStats,
};

pub use repertoire::SettingsRepertoire;
pub use registry::RepertoireRegistry;
pub use helpers::{
    format_repertoire_registry,
    is_repertoire_query,
    repertoire_fun_fact,
};

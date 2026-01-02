// v0.0.732: Settings Concordat Module (Phase 308)
// Religious agreement for settings governance

mod types;
mod core;
mod helpers;

// Re-export all public types
pub use types::{
    ConcordatType,
    ConcordatStatus,
    ConcordatConfig,
    ConcordatArticle,
    ConcordatSignatory,
    ConcordatStats,
};

// Re-export core structures
pub use core::{
    SettingsConcordat,
    ConcordatRegistry,
};

// Re-export helper functions
pub use helpers::{
    format_concordat_registry,
    is_concordat_query,
    concordat_fun_fact,
};

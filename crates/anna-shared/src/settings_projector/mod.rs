// v0.0.672: Settings Projector Module (Phase 248)
// Project settings to specific fields/views

mod types;
mod projector;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{
    FieldMapping,
    ProjectionResult,
    ProjectionType,
    ProjectorConfig,
    ProjectorStats,
};

pub use projector::SettingsProjector;

pub use registry::{
    ProjectorRegistry,
    format_projector_registry,
};

pub use utils::{
    is_projector_query,
    projector_fun_fact,
};

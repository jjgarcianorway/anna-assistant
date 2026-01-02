// v0.0.751: Settings Borough (Phase 327)
// Borough subdivision for settings local governance
// Modular structure with files under 400 lines

mod types;
mod borough;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve API
pub use types::{
    BoroughConfig,
    BoroughRepresentative,
    BoroughResolution,
    BoroughStats,
    BoroughStatus,
    BoroughType,
};

pub use borough::SettingsBorough;
pub use registry::BoroughRegistry;
pub use utils::{borough_fun_fact, format_borough_registry, is_borough_query};

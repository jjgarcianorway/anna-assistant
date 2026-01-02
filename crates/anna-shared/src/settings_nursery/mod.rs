// v0.0.769: Settings Nursery (Phase 345)
// Plant nursery for settings propagation

mod types;
mod config;
mod seedling;
mod propagator;
mod stats;
mod nursery;
mod registry;
mod utils;

// Re-export all public types and functions to maintain the public API
pub use types::{NurseryType, NurseryStatus};
pub use config::NurseryConfig;
pub use seedling::NurserySeedling;
pub use propagator::NurseryPropagator;
pub use stats::NurseryStats;
pub use nursery::SettingsNursery;
pub use registry::{NurseryRegistry, format_nursery_registry};
pub use utils::{is_nursery_query, nursery_fun_fact};

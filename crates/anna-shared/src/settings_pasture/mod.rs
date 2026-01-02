// v0.0.764: Settings Pasture (Phase 340)
// Grazing pasture for settings livestock

mod types;
mod config;
mod herd;
mod stats;
mod pasture;
mod registry;
mod utils;

// Re-export all public items to maintain the same API
pub use types::{PastureType, PastureStatus};
pub use config::PastureConfig;
pub use herd::{PastureHerd, PastureHerder};
pub use stats::PastureStats;
pub use pasture::SettingsPasture;
pub use registry::{PastureRegistry, format_pasture_registry};
pub use utils::{is_pasture_query, pasture_fun_fact};

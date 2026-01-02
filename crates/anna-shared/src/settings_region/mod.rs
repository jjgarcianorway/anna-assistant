// v0.0.747: Settings Region Module (Phase 323)
// Geographic region for settings organization

mod types;
mod config;
mod policy;
mod coordinator;
mod stats;
mod region;
mod registry;
mod utils;

// Re-export all public types and functions
pub use types::{RegionType, RegionStatus};
pub use config::RegionConfig;
pub use policy::RegionPolicy;
pub use coordinator::RegionCoordinator;
pub use stats::RegionStats;
pub use region::SettingsRegion;
pub use registry::RegionRegistry;
pub use utils::{format_region_registry, is_region_query, region_fun_fact};

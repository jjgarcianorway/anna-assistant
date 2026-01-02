// v0.0.746: Settings Province (Phase 322)
// Administrative province for settings governance

mod types;
mod config;
mod edict;
mod governor;
mod stats;
mod province;
mod registry;
mod utils;

// Re-export all public types
pub use types::{ProvinceType, ProvinceStatus};
pub use config::ProvinceConfig;
pub use edict::ProvinceEdict;
pub use governor::ProvinceGovernor;
pub use stats::ProvinceStats;
pub use province::SettingsProvince;
pub use registry::{ProvinceRegistry, format_province_registry};
pub use utils::{is_province_query, province_fun_fact};

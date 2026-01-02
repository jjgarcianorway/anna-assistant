// v0.0.748: Settings District Module (Phase 324)
// Local district for settings administration

mod types;
mod config;
mod bylaw;
mod official;
mod stats;
mod district;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Public re-exports
pub use types::{DistrictType, DistrictStatus};
pub use config::DistrictConfig;
pub use bylaw::DistrictBylaw;
pub use official::DistrictOfficial;
pub use stats::DistrictStats;
pub use district::SettingsDistrict;
pub use registry::DistrictRegistry;
pub use utils::{format_district_registry, is_district_query, district_fun_fact};

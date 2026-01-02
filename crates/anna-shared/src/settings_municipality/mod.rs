// v0.0.750: Settings Municipality Module (Phase 326)
// Municipal corporation for settings self-governance

mod types;
mod config;
mod code;
mod councilor;
mod stats;
mod municipality;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use types::{MunicipalityType, MunicipalityStatus};
pub use config::MunicipalityConfig;
pub use code::MunicipalityCode;
pub use councilor::MunicipalityCouncilor;
pub use stats::MunicipalityStats;
pub use municipality::SettingsMunicipality;
pub use registry::MunicipalityRegistry;
pub use utils::{format_municipality_registry, is_municipality_query, municipality_fun_fact};

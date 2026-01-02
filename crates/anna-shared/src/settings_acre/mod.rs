// v0.0.760: Settings Acre (Phase 336)
// Land acre for settings measurement

mod types;
mod config;
mod measurement;
mod stats;
mod acre;
mod registry;
mod tests;

// Re-export all public types to preserve API
pub use types::{AcreType, AcreStatus};
pub use config::AcreConfig;
pub use measurement::{AcreMeasurement, AcreSurveyor};
pub use stats::AcreStats;
pub use acre::SettingsAcre;
pub use registry::{AcreRegistry, format_acre_registry, is_acre_query, acre_fun_fact};

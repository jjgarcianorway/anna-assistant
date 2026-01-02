// v0.0.762: Settings Field Module (Phase 338)
// Agricultural field for settings cultivation

mod types;
mod config;
mod crop;
mod farmer;
mod stats;
mod field;
mod registry;
mod utils;

// Re-export all public types to preserve the original API
pub use types::{FieldType, FieldStatus};
pub use config::FieldConfig;
pub use crop::FieldCrop;
pub use farmer::FieldFarmer;
pub use stats::FieldStats;
pub use field::SettingsField;
pub use registry::FieldRegistry;
pub use utils::{format_field_registry, is_field_query, field_fun_fact};

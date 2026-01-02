// v0.0.641: Settings Inspector Module (Phase 217)
// Inspector for settings structure and values

mod types;
mod config;
mod finding;
mod result;
mod stats;
mod inspector;
mod registry;
mod utils;

// Re-export all public types and functions to preserve API
pub use types::{InspectionType, InspectionDepth};
pub use config::InspectorConfig;
pub use finding::InspectionFinding;
pub use result::InspectionResult;
pub use stats::InspectorStats;
pub use inspector::SettingsInspector;
pub use registry::{SettingsInspectorRegistry, format_inspector_registry};
pub use utils::{is_inspector_query, inspector_fun_fact};

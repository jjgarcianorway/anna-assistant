// v0.0.659: Settings Restorer Module
// Modular settings restorer implementation

pub mod mode;
pub mod config;
pub mod source;
pub mod result;
pub mod stats;
pub mod restorer;
pub mod registry;
pub mod utils;

// Re-export all public types to preserve API
pub use mode::{RestoreMode, RestoreStrategy};
pub use config::RestorerConfig;
pub use source::RestoreSource;
pub use result::RestoreResult;
pub use stats::RestorerStats;
pub use restorer::SettingsRestorer;
pub use registry::{SettingsRestorerRegistry, format_restorer_registry};
pub use utils::{is_restorer_query, restorer_fun_fact};

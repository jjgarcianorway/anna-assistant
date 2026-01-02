// v0.0.715: Settings Communique (Phase 291)
// Official communications about settings

mod types;
mod config;
mod message;
mod stats;
mod communique;
mod registry;

// Re-export all public types
pub use types::{CommuniqueType, CommuniqueClassification};
pub use config::CommuniqueConfig;
pub use message::{CommuniqueMessage, CommuniqueAttachment};
pub use stats::CommuniqueStats;
pub use communique::SettingsCommunique;
pub use registry::{CommuniqueRegistry, format_communique_registry, is_communique_query, communique_fun_fact};

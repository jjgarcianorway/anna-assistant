// v0.0.744: Settings Realm (Phase 320)
// Royal realm for settings sovereignty

mod types;
mod config;
mod decree;
mod vassal;
mod stats;
mod realm;
mod registry;
mod utils;

// Re-export all public types and functions
pub use types::{RealmType, RealmStatus};
pub use config::RealmConfig;
pub use decree::RealmDecree;
pub use vassal::RealmVassal;
pub use stats::RealmStats;
pub use realm::SettingsRealm;
pub use registry::RealmRegistry;
pub use utils::{format_realm_registry, is_realm_query, realm_fun_fact};

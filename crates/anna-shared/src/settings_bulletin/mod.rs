// v0.0.706: Settings Bulletin (Phase 282)
// Bulletin board for settings updates

mod types;
mod config;
mod post;
mod stats;
mod bulletin;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{BulletinType, BulletinPriority};
pub use config::BulletinConfig;
pub use post::{BulletinPost, BulletinItem};
pub use stats::BulletinStats;
pub use bulletin::SettingsBulletin;
pub use registry::BulletinRegistry;
pub use utils::{format_bulletin_registry, is_bulletin_query, bulletin_fun_fact};

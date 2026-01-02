// v0.0.704: Settings Gazette Module (Phase 280)
// Official gazette of settings announcements

mod types;
mod config;
mod notice;
mod stats;
mod gazette;
mod registry;
mod helpers;

// Re-export all public types and functions to maintain the same API
pub use types::{GazetteType, GazetteStatus};
pub use config::GazetteConfig;
pub use notice::{GazetteNotice, GazetteEntry};
pub use stats::GazetteStats;
pub use gazette::SettingsGazette;
pub use registry::GazetteRegistry;
pub use helpers::{format_gazette_registry, is_gazette_query, gazette_fun_fact};

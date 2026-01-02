// v0.0.742: Settings Zone Module (Phase 318)
// Designated zone for settings boundaries

mod types;
mod config;
mod regulation;
mod participant;
mod stats;
mod zone;
mod registry;
mod utils;

// Re-export all public types and functions
pub use types::{ZoneType, ZoneStatus};
pub use config::ZoneConfig;
pub use regulation::ZoneRegulation;
pub use participant::ZoneParticipant;
pub use stats::ZoneStats;
pub use zone::SettingsZone;
pub use registry::ZoneRegistry;
pub use utils::{format_zone_registry, is_zone_query, zone_fun_fact};

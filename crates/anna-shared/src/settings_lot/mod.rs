// v0.0.756: Settings Lot Module (Phase 332)
// Land lot for settings property

mod types;
mod config;
mod deed;
mod assessor;
mod stats;
mod lot;
mod registry;
#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{LotType, LotStatus};
pub use config::LotConfig;
pub use deed::LotDeed;
pub use assessor::LotAssessor;
pub use stats::LotStats;
pub use lot::SettingsLot;
pub use registry::{LotRegistry, format_lot_registry, is_lot_query, lot_fun_fact};

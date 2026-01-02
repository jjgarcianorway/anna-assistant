// v0.0.717: Settings Circular (Phase 293)
// Circular notices distributed to all

mod types;
mod config;
mod notice;
mod stats;
mod circular;
mod registry;
mod utils;
#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{CircularType, CircularScope};
pub use config::CircularConfig;
pub use notice::{CircularNotice, CircularAttachment};
pub use stats::CircularStats;
pub use circular::SettingsCircular;
pub use registry::CircularRegistry;
pub use utils::{format_circular_registry, is_circular_query, circular_fun_fact};

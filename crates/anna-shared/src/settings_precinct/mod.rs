// v0.0.753: Settings Precinct Module (Phase 329)
// Voting precinct for settings participation

mod types;
mod config;
mod ballot;
mod captain;
mod stats;
mod precinct;
mod registry;
mod tests;

// Re-export all public types
pub use types::{PrecinctType, PrecinctStatus};
pub use config::PrecinctConfig;
pub use ballot::PrecinctBallot;
pub use captain::PrecinctCaptain;
pub use stats::PrecinctStats;
pub use precinct::SettingsPrecinct;
pub use registry::{PrecinctRegistry, format_precinct_registry, is_precinct_query, precinct_fun_fact};

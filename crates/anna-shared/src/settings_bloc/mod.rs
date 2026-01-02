// v0.0.740: Settings Bloc Module (Phase 316)
// Regional bloc for settings coordination

mod types;
mod config;
mod policy;
mod member;
mod stats;
mod bloc;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve API
pub use types::{BlocType, BlocStatus};
pub use config::BlocConfig;
pub use policy::BlocPolicy;
pub use member::BlocMember;
pub use stats::BlocStats;
pub use bloc::SettingsBloc;
pub use registry::BlocRegistry;
pub use utils::{format_bloc_registry, is_bloc_query, bloc_fun_fact};

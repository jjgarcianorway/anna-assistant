// v0.0.714: Settings Dispatch (Phase 290)
// Dispatching settings changes to targets

mod types;
mod config;
mod item;
mod stats;
mod dispatch;
mod registry;
mod utils;

// Re-export types
pub use types::{DispatchType, DispatchStatus};
pub use config::DispatchConfig;
pub use item::{DispatchItem, DispatchMetadata};
pub use stats::DispatchStats;
pub use dispatch::SettingsDispatch;
pub use registry::DispatchRegistry;
pub use utils::{format_dispatch_registry, is_dispatch_query, dispatch_fun_fact};

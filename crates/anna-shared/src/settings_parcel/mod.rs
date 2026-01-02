// v0.0.757: Settings Parcel (Phase 333)
// Land parcel for settings ownership

mod types;
mod config;
mod title;
mod examiner;
mod stats;
mod parcel;
mod registry;
mod utils;

// Re-export all public types
pub use types::{ParcelType, ParcelStatus};
pub use config::ParcelConfig;
pub use title::ParcelTitle;
pub use examiner::ParcelExaminer;
pub use stats::ParcelStats;
pub use parcel::SettingsParcel;
pub use registry::ParcelRegistry;
pub use utils::{format_parcel_registry, is_parcel_query, parcel_fun_fact};

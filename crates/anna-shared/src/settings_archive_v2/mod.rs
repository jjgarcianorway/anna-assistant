// v0.0.702: Settings Archive V2 (Phase 278)
// Long-term archive of settings history

mod types;
mod config;
mod record;
mod stats;
mod archive;
mod registry;
mod utils;

// Re-export all public types to maintain the same API
pub use types::{ArchiveTypeV2, ArchiveRetention};
pub use config::ArchiveConfigV2;
pub use record::{ArchiveRecord, ArchiveBox};
pub use stats::ArchiveStatsV2;
pub use archive::SettingsArchiveV2;
pub use registry::ArchiveRegistryV2;
pub use utils::{format_archive_registry_v2, is_archive_v2_query, archive_v2_fun_fact};

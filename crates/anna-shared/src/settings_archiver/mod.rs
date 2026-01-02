// v0.0.658: Settings Archiver Module (Phase 234)
// Modularized archiver for backing up settings configurations

mod types;
mod config;
mod metadata;
mod stats;
mod archiver;
mod registry;
mod utils;

// Re-export all public types to preserve API
pub use types::{ArchiveFormat, ArchiveType};
pub use config::ArchiverConfig;
pub use metadata::{ArchiveMetadata, ArchiveResult};
pub use stats::ArchiverStats;
pub use archiver::SettingsArchiver;
pub use registry::{SettingsArchiverRegistry, format_archiver_registry};
pub use utils::{is_archiver_query, archiver_fun_fact};

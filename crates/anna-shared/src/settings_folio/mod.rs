// v0.0.695: Settings Folio (Phase 271)
// Portfolio of settings collections

mod types;
mod config;
mod section;
mod item;
mod stats;
mod folio;
mod registry;
mod utils;

// Re-export all public types to preserve API
pub use types::{FolioType, FolioStatus};
pub use config::FolioConfig;
pub use section::FolioSection;
pub use item::FolioItem;
pub use stats::FolioStats;
pub use folio::SettingsFolio;
pub use registry::FolioRegistry;
pub use utils::{format_folio_registry, is_folio_query, folio_fun_fact};

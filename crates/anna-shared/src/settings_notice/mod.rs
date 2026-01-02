// v0.0.713: Settings Notice (Phase 289)
// Official notices about settings changes

mod types;
mod config;
mod entry;
mod stats;
mod notice;
mod registry;
mod utils;

// Re-export all public types to preserve API
pub use types::{NoticeType, NoticePriority};
pub use config::NoticeConfig;
pub use entry::{NoticeEntry, NoticeMetadata};
pub use stats::NoticeStats;
pub use notice::SettingsNotice;
pub use registry::NoticeRegistry;
pub use utils::{format_notice_registry, is_notice_query, notice_fun_fact};

// v0.0.694: Settings Diary (Phase 270)
// Daily diary of settings activities

mod types;
mod config;
mod entry;
mod page;
mod stats;
mod diary;
mod registry;
mod helpers;

#[cfg(test)]
mod tests;

// Re-export all public types to maintain the same API
pub use types::{DiaryEntryType, DiaryImportance};
pub use config::DiaryConfig;
pub use entry::DiaryEntry;
pub use page::DailyPage;
pub use stats::DiaryStats;
pub use diary::SettingsDiary;
pub use registry::DiaryRegistry;
pub use helpers::{format_diary_registry, is_diary_query, diary_fun_fact};

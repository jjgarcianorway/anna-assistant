// v0.0.707: Settings Journal (Phase 283)
// Personal journal for settings reflections

mod enums;
mod config;
mod entry;
mod item;
mod stats;
mod journal;
mod registry;
mod utils;

// Re-export public API
pub use enums::{JournalType, JournalMood};
pub use config::JournalConfig;
pub use entry::JournalEntry;
pub use item::JournalItem;
pub use stats::JournalStats;
pub use journal::SettingsJournal;
pub use registry::JournalRegistry;
pub use utils::{format_journal_registry, is_journal_query, journal_fun_fact};

// v0.0.708: Settings Memo (Phase 284)
// Internal memos for settings communication

mod types;
mod config;
mod message;
mod stats;
mod memo;
mod registry;

// Re-export all public types to preserve the original API
pub use types::{MemoType, MemoStatus};
pub use config::MemoConfig;
pub use message::{MemoMessage, MemoAttachment};
pub use stats::MemoStats;
pub use memo::SettingsMemo;
pub use registry::{MemoRegistry, format_memo_registry, is_memo_query, memo_fun_fact};

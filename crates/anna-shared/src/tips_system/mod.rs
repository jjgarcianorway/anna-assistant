// v0.0.541: Tips System Module (Phase 117)
// Tips for greetings about config options per VISION.md

mod formatters;
mod manager;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use formatters::{format_tip, format_tip_compact, format_tips_summary};
pub use manager::TipsSystem;
pub use types::{Tip, TipCategory, TipPriority};
pub use utils::{is_tips_query, tips_fun_fact};

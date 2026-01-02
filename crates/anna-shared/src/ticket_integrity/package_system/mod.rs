//! Clean Package vs System Logic (Part 3) - v0.0.442.
//!
//! Stop confusing "package presence" with "system feature".
//!
//! WRONG: "do I have swap?" → "**swap** package is not installed"
//! RIGHT: "do I have swap?" → check /proc/swaps, report swap status
//!
//! Clear, separate intents:
//! - `system.swap_configured` - System swap memory
//! - `packages.check_installed` - Package installation
//! - `packages.search_by_name` - Search for packages

mod helpers;
mod intents;
mod status;
mod types;

// Re-export public API
pub use intents::classify_question;
pub use status::{PackageStatus, SwapStatus};
pub use types::{PackageIntent, QuestionClassification, SwapKind, SystemIntent};

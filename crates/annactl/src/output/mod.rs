//! Output Module - Answer normalization for CLI and TUI (Beta.210)
//!
//! Provides unified normalization following the canonical [SUMMARY]/[DETAILS]/[COMMANDS]
//! format defined in docs/ANSWER_FORMAT.md.

mod normalizer;

pub use normalizer::{generate_fallback_message, normalize_for_cli, normalize_for_tui};

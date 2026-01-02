//! Command Shortcuts System (v0.0.483).
//!
//! Provides short aliases for common operations.
//! Users can say brief commands and Anna expands them.
//!
//! Examples:
//! - "du" -> "show disk usage"
//! - "mem" -> "show memory usage"
//! - "top5" -> "show top 5 processes by CPU"

mod types;
mod shortcuts;
mod operations;
mod formatting;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve API
pub use types::{CommandShortcut, ShortcutCategory};
pub use shortcuts::{builtin_shortcuts, shortcuts_by_category};
pub use operations::{
    build_shortcut_map, expand_shortcut, is_shortcut, is_shortcuts_query,
};
pub use formatting::{format_shortcuts, format_category_shortcuts};

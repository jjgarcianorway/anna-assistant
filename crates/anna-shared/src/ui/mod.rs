//! Terminal UI helpers for consistent output styling (v0.0.213).
//!
//! v0.0.213: Modularized into domain-focused submodules.

pub mod colors;
pub mod formatting;
pub mod printing;
pub mod spinner;
pub mod stage;
pub mod symbols;
pub mod terminal;

#[cfg(test)]
mod tests;

// Re-export commonly used items for backwards compatibility
pub use formatting::{format_bytes, format_duration, progress_bar};
pub use printing::{print_err, print_footer, print_header, print_kv, print_kv_status, print_ok, print_section, HR};
pub use spinner::Spinner;
pub use stage::{StageProgress, StageStatus};
pub use terminal::{clear_line, cursor_up, print_inline};

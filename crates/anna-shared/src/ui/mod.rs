//! Terminal UI helpers for consistent output styling (v0.0.337).
//!
//! v0.0.213: Modularized into domain-focused submodules.
//! v0.0.337: Enhanced printing helpers for consistent output.

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
pub use printing::{
    kv, kv_colored, print_err, print_footer, print_header, print_hint, print_kv,
    print_kv_status, print_label, print_ok, print_section, print_section_header, print_warn,
    HR, KEY_WIDTH,
};
pub use spinner::Spinner;
pub use stage::{StageProgress, StageStatus};
pub use terminal::{clear_line, cursor_up, print_inline};

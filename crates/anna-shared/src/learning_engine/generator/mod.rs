//! Recipe generator from tickets (v0.0.427).
//!
//! Creates new recipes from successful specialist responses:
//! - Extracts pattern from ticket
//! - Captures probes used
//! - Generates answer templates
//! - Tracks citations

pub mod builder;
pub mod core;
pub mod inference;
pub mod types;

pub use builder::*;
pub use core::*;
pub use inference::*;
pub use types::*;

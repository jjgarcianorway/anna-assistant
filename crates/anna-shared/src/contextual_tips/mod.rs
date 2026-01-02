//! Contextual Tips System (v0.0.482).
//!
//! Provides relevant tips based on the user's current context:
//! - What they just asked about
//! - What action was just performed
//! - What topic they're working on
//!
//! Unlike greeting_tips which are random, these are targeted.

mod types;
mod tips;
mod handlers;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{ContextualTip, TipContext};
pub use handlers::{
    get_contextual_tips,
    select_tip,
    format_tip,
    get_tip_for_query,
    should_show_tip,
};

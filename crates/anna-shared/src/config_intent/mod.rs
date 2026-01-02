//! Config edit intent detection (v0.0.264).
//!
//! Detects when user requests are config edits and provides ROUTING HINTS to specialists.
//! Anna learns actual solutions from specialists, not from hardcoded patterns here.
//!
//! v0.0.263: Added neovim, nano, helix, and colorscheme support.
//! v0.0.264: Refactored to provide routing hints instead of hardcoded answers.
//!           Anna now learns recipes from specialists (Sofia for Desktop team).

mod types;
mod parsing;
mod detection;

// Re-export types from config_types for backward compatibility
pub use crate::config_types::{ConfigEditAction, ConfigIntent, ConfigTarget};

// Re-export types from submodules
pub use types::{ConfigFeatureHint, ConfigHint};
pub use parsing::is_config_edit_request;
pub use detection::{
    detect_config_intent, detect_emacs_config_intent, detect_helix_config_intent,
    detect_nano_config_intent, detect_neovim_config_intent, detect_vim_config_intent,
};

//! Clarification engine v2 (v0.0.197).
//!
//! Clean request/response flow with verification integration.
//! Auto-select when only one option. Installed-only menus.
//!
//! v0.0.197: Modularized into domain-focused submodules.

mod facts;
mod processing;
mod types;

// Re-export all types and functions
pub use facts::{invalidate_on_uninstall, should_skip, store_fact};
pub use processing::{editor_request, find_installed_alternatives, process_response};
pub use types::{
    ClarifyOption, ClarifyRequest, ClarifyResponse, ClarifyResult, VerifyFailureTracker,
    KEY_CANCEL, KEY_OTHER,
};

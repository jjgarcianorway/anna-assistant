//! Clarification questions with verification probes (v0.0.191).
//!
//! When information is missing to answer a query, we ask concrete clarification
//! questions with associated probes to verify the answer.
//!
//! v0.0.39: Uses InventoryCache for installed tool detection.
//! v0.0.42: Added menu-based prompts with numeric keys (0=cancel, 9=other).
//! v0.0.44: Moved v2 types to clarify_v2.rs module.
//! v0.0.191: Modularized into domain-focused submodules.

mod detection;
mod editors;
mod legacy;
mod menu;

// Re-export v0.0.44 types from clarify_v2
pub use crate::clarify_v2::{
    editor_request, find_installed_alternatives, invalidate_on_uninstall, process_response,
    should_skip, store_fact, ClarifyOption as ClarifyOptionV2, ClarifyRequest, ClarifyResponse,
    ClarifyResult, VerifyFailureTracker, KEY_CANCEL, KEY_OTHER,
};

// Re-export menu types (v0.0.42)
pub use menu::{ClarifyOutcome, ClarifyPrompt, MenuOption};

// Re-export detection functions
pub use detection::{extract_service_name, needs_clarification};

// Re-export editor functions
pub use editors::{
    editor_menu_prompt, find_installed_alternative, generate_editor_clarification,
    generate_editor_options_sync, generate_editor_options_with_cache, is_cancel_selection,
    is_other_selection, verify_editor_installed, CLARIFY_CANCEL_KEY, CLARIFY_OTHER_KEY,
    KNOWN_EDITORS,
};

// Re-export legacy types
pub use legacy::{
    build_verify_command, generate_question, kind_to_fact_key, ClarifyAnswer, ClarifyKind,
    ClarifyOption, ClarifyQuestion, ClarifyResultLegacy,
};

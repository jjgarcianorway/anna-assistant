//! LLM-based translator for query classification (v0.0.448).
//!
//! Converts user text to structured TranslatorTicket JSON.
//! v0.0.74: Now includes AnswerContract for answer shaping.
//! v0.0.164: Probe registry extracted to separate module.
//! v0.0.290: Strip reasoning tags from translator responses.
//! v0.0.318: Added TranslatorResult with debug info for LLM call visibility.
//! v0.0.322: Integrated probe learning - recommends probes based on past effectiveness.
//! v0.0.327: Uses load_with_decay() for automatic learning decay.
//! v0.0.333: Only uses learning when confidence is sufficient.
//! v0.0.374: Filter out probe combinations known to fail for similar queries.
//! v0.0.448: DETERMINISTIC PROBES FIRST - check intent-specific probe rules before LLM.

// Module declarations
mod api;
mod learning;
mod parsers;
mod prompt;
mod types;

// Re-export public API
pub use api::{translate, translate_with_context, translate_with_debug};
pub use prompt::build_translator_request;
pub use types::{TranslatorInput, TranslatorResult, MAX_TRANSLATOR_PAYLOAD_SIZE};

// Re-export probe registry for backwards compatibility
// v0.0.797: Added probe_id_to_command_dynamic for dynamic probe support
pub use crate::probe_registry::{
    filter_valid_probes, probe_id_to_command, probe_id_to_command_dynamic, PROBE_IDS,
};

// Re-export fallback translator for backwards compatibility
pub use crate::translator_fallback::translate_fallback;

//! Debug block footer (v0.0.444).
//!
//! Standardized debug output appended to responses at debug level 1+.
//!
//! This module re-exports from the modular components for backward compatibility.

// Re-export all types from the modular structure
pub use super::debug_block::DebugBlock;
pub use super::types::{
    EvidenceDebug, LlmCallDebug, ModelsUsedDebug, ProbeDebugInfo, ProbeStatus, TimeoutDebug,
    TimingDebug, TranslatorDecision,
};

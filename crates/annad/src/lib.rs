//! Anna daemon - LLM-driven Linux assistant.
//!
//! Core functionality:
//! - Pure LLM intelligence (no pattern matching) - v0.2.0
//! - Multi-stage investigation
//! - Grounded answers based on actual command output
//! - Smart fix suggestions based on findings
//! - Auto-update from GitHub
//! - Unix socket server for client communication
//! - Self-healing infrastructure (v0.3.36)
//! - Assisted operations (Phase 39): Supervised, human-executed fixes
//! - Capability routing (Phase 34): Deterministic, no LLM

// =============================================================================
// Phase 34: LLM Call Tracking Infrastructure (Test-Only)
// =============================================================================
// This counter is incremented by LLM entrypoints and asserted to be zero
// in capability path tests, providing hard proof that LLM is not called.

#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for LLM calls (test-only).
#[cfg(test)]
pub static LLM_CALL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Increment the LLM call counter (called by LLM entrypoints in test mode).
#[cfg(test)]
pub fn record_llm_call() {
    LLM_CALL_COUNTER.fetch_add(1, Ordering::SeqCst);
}

/// Reset the LLM call counter (called at start of tests).
#[cfg(test)]
pub fn reset_llm_call_counter() {
    LLM_CALL_COUNTER.store(0, Ordering::SeqCst);
}

/// Get the current LLM call count.
#[cfg(test)]
pub fn get_llm_call_count() -> u64 {
    LLM_CALL_COUNTER.load(Ordering::SeqCst)
}

/// No-op in non-test builds.
#[cfg(not(test))]
pub fn record_llm_call() {}

pub mod assisted_ops;  // Phase 39: Supervised assistance layer
pub mod autofix;  // TODO: Remove after full migration to llm_core
pub mod binary_watcher;
pub mod changes;
pub mod core_loop;
pub mod department;
pub mod intent;
pub mod llm_core;  // NEW: Pure LLM-driven core loop
pub mod ollama;
pub mod patterns;  // TODO: Remove after full migration to llm_core
pub mod plan_executor;  // Phase 16: Action plan execution
pub mod plan_generator;  // Phase 16: Generate plans from LLM
pub mod plan_stash;  // Phase 17: State capture for rollback
pub mod plan_templates;  // Phase 17: Template plans for common operations
pub mod plan_verify;  // Phase 17: Post-action verification
pub mod ralph;
pub mod recipes;
pub mod recovery;  // v0.3.36: Self-healing recovery
pub mod server;
pub mod specialist;  // v0.3.37: Specialist dispatch system
pub mod state;
pub mod team_speak;
pub mod translator;
pub mod update;
pub mod update_loop;
pub mod update_ops;
pub mod validation;

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

pub mod agent_registry;
pub mod artifact_registry;
pub mod automation_creator;
pub mod battery;
pub mod gpu_monitor;
pub mod pkg_suggestions;
pub mod power_profile;
pub mod system_learner;
pub mod xwayland;
pub mod wallpaper;
pub mod ssh_auditor;
pub mod user_management;
pub mod kernel_builder;
pub mod agents;
pub mod anomaly;
pub mod assisted_ops;
pub mod autonomous_loop;
pub mod autofix;
pub mod autohealing;
pub mod briefing;
pub mod cache;
pub mod chart_generator;
pub mod binary_watcher;
pub mod context_resolver;
pub mod de_config;
pub mod disk_health;
pub mod dynamic_plan;
pub mod intelligence;
pub mod meta_learning;
pub mod universal_handler;
pub mod temporal_tasks;
pub mod feasibility;
pub mod system_identity;
pub mod adaptive_intelligence;
pub mod smart_file_ops;
pub mod opportunity_detector;
pub mod future_planner;
pub mod pattern_learning;
pub mod failure_memory;
pub mod anomaly_analysis;
pub mod cleanup_detector;
pub mod regression_detector;
pub mod predictive_maintenance;
pub mod teaching_mode;
pub mod cross_module_intelligence;
pub mod llm_orchestration;
pub mod self_improvement;
pub mod proactive_monitoring;
pub mod change_tracking;
pub mod historical_narrative;
pub mod trust_calibration;
pub mod opportunistic_maintenance;
pub mod action_execution;
pub mod multi_perspective_analysis;
pub mod user_context;
pub mod changes;
pub mod core_loop;
pub mod department;
pub mod intent;
pub mod llm_core;
pub mod memory;
pub mod model_router;
pub mod ollama;
pub mod orchestrator;
pub mod personality;
pub mod plan_executor;
pub mod plan_generator;
pub mod plan_stash;
pub mod plan_verify;
pub mod ralph;
pub mod recovery;
pub mod report;
pub mod root_cause;
pub mod scheduler_loop;
pub mod self_healing;
pub mod server;
pub mod smart_timing;
pub mod state;
pub mod suggestions;
pub mod team_speak;
pub mod telegram;
pub mod tool_manager;
pub mod translator;
pub mod wiki_sync;
pub mod update;
pub mod update_loop;
pub mod update_ops;
pub mod update_system;
pub mod validation;

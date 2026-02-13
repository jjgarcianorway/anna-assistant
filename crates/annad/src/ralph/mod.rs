//! Ralph-style autonomous iteration loop for answering questions.
//!
//! The Ralph Wiggum approach: iteration beats perfection.
//! Instead of complex branching, use a simple loop with clear completion criteria.
//!
//! Principles:
//! 1. Define "done" upfront - what does success look like?
//! 2. Iterate until done - trust the loop, not complexity
//! 3. Self-evaluate - LLM checks its own work before declaring done
//! 4. Learn from attempts - each iteration improves the next

mod answer_gen;
mod commands;
pub mod confidence;
mod config_handler;
mod config_flow;
mod criteria;
pub mod evidence;
mod early_handlers;
mod finish;
mod loop_early;
mod loop_fallback;
mod loop_impl;
mod parallel;
mod recipe_learning;
mod run_loop;
mod streaming;
pub mod streaming_helpers;
mod suggestions;
mod system_probe;
mod temporal;
mod verification;

pub use suggestions::{generate_suggestions, format_suggestions};
pub use parallel::{should_parallelize, run_parallel_investigation, synthesize_parallel_results};

// Re-export public API
pub use criteria::{determine_criteria, AnswerType, CompletionCriteria};
pub use streaming::ralph_loop_streaming;

use anyhow::Result;
use anna_shared::rpc::AskResult;

/// The Ralph loop: iterate until done (non-streaming version)
/// LLM-first: no bypass paths. Every question goes through the LLM.
/// v0.3.162: Universal capability system with feasibility checking and temporal tasks.
/// v0.3.166: Pattern learning, failure memory, and automation suggestions.
pub async fn ralph_loop(model: &str, question: &str) -> Result<AskResult> {
    loop_impl::ralph_loop_impl(model, question).await
}

//! Anna v7 Brain - Clean Architecture
//!
//! A complete rewrite of the LLM orchestration layer with:
//! - Strict data contracts (no vibes)
//! - Three-phase pipeline: PLAN → EXECUTE → INTERPRET
//! - Reliability scoring with retry logic
//! - Fixed tool catalog (LLM cannot invent tools)
//!
//! Design principles:
//! 1. If JSON parsing fails, the phase fails. No partial plans.
//! 2. LLM sees tool descriptions, not commands.
//! 3. Rust owns all tool execution.
//! 4. Reliability score drives retry decisions.

pub mod contracts;
pub mod orchestrator;
pub mod prompts;
pub mod tools;

pub use contracts::*;
pub use orchestrator::BrainOrchestrator;
pub use tools::ToolCatalog;

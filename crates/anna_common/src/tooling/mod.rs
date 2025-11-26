//! Tooling - v6.62.0 Hybrid LLM Orchestration
//!
//! Single source of truth for all tools Anna can execute.
//! Rust orchestrator controls the loop, LLM follows strict contract.
//!
//! v6.62.0: Hybrid architecture with reliability scoring
//! - Tool catalog defines WHAT tools exist
//! - LLM planner outputs structured plan (subtasks, tool_calls, expected_evidence)
//! - Rust executes tools and collects EvidenceBundle
//! - LLM interpreter produces answer with reliability score
//! - Rust retries if reliability < 0.8 (up to 2 times)

pub mod catalog;
pub mod actions;
pub mod executor;
pub mod llm_orchestrator;

pub use catalog::{ToolId, ToolSpec, ToolKind, ToolResult, tool_catalog, get_tool};
pub use actions::{Action, ActionResult, map_action_to_tools};
pub use executor::{ToolExecutor, ExecutionError};
pub use llm_orchestrator::{
    LlmOrchestrator, OrchestrationResult, PlannerOutput, EvidenceBundle,
    InterpreterOutput, ReliabilityScore, get_tool_catalog_for_llm
};

//! Tooling - v6.60.0 LLM-Driven Orchestration
//!
//! Single source of truth for all tools Anna can execute.
//! The orchestrator delegates ALL decisions to the LLM.
//!
//! v6.60.0: Pure LLM-driven orchestration
//! - Tool catalog defines WHAT tools exist
//! - LLM planner decides WHICH tools to run
//! - LLM interpreter decides HOW to summarize results
//! - Orchestrator ONLY executes and forwards results

pub mod catalog;
pub mod actions;
pub mod executor;
pub mod llm_orchestrator;

pub use catalog::{ToolId, ToolSpec, ToolKind, ToolResult, tool_catalog, get_tool};
pub use actions::{Action, ActionResult, map_action_to_tools};
pub use executor::{ToolExecutor, ExecutionError};
pub use llm_orchestrator::{LlmOrchestrator, CommandPlan, OrchestrationResult, get_tool_catalog_for_llm};

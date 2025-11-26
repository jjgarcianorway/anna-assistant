//! Anna Brain Core v1.0 - Pure LLM-Driven Architecture
//!
//! INPUT → LLM → TOOLS → LLM → ANSWER
//!
//! No hardcoded logic. The LLM decides what tools to run.
//! Anna only executes tools and relays results.

pub mod contracts;
pub mod prompt;
pub mod tools;
pub mod orchestrator;

pub use contracts::{BrainOutput, BrainMode, ToolRequest, ToolResult, BrainState, ToolSchema};
pub use orchestrator::{BrainOrchestrator, BrainResult};
pub use tools::{ToolCatalog, execute_tool};

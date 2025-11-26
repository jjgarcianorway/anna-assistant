//! Brain v8 - Pure LLM-Driven Architecture
//!
//! The LLM is the brain.
//! annad is only:
//!   - The memory of the machine (telemetry)
//!   - The hands of the machine (tools)
//!   - The message relay
//!
//! Everything else belongs to the LLM.
//! No second LLM call needed. Single think→answer loop.

pub mod contracts;
pub mod prompt;
pub mod tools;
pub mod orchestrator;

pub use contracts::{BrainOutput, ToolRequest, BrainMode};
pub use orchestrator::BrainOrchestrator;
pub use tools::{ToolCatalog, execute_tool};

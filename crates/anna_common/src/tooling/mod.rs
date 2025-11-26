//! Tooling - v7.0.0 Clean Tool Catalog
//!
//! Single source of truth for all tools Anna can execute.
//! v7.0.0: Moved orchestration to brain_v7 module.
//!
//! This module now provides:
//! - Tool catalog defines WHAT tools exist
//! - Tool executor runs individual tools
//! - Actions map high-level intents to tool sequences

pub mod catalog;
pub mod actions;
pub mod executor;

pub use catalog::{ToolId, ToolSpec, ToolKind, ToolResult, tool_catalog, get_tool};
pub use actions::{Action, ActionResult, map_action_to_tools};
pub use executor::{ToolExecutor, ExecutionError};

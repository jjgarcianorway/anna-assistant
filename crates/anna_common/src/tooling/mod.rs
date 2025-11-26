//! Tooling - v6.59.0 Unified Tool Catalog
//!
//! Single source of truth for all tools Anna can execute.
//! Both the health checker and the NL executor use this catalog.

pub mod catalog;
pub mod actions;
pub mod executor;

pub use catalog::{ToolId, ToolSpec, ToolKind, ToolResult, tool_catalog, get_tool};
pub use actions::{Action, ActionResult, map_action_to_tools};
pub use executor::{ToolExecutor, ExecutionError};

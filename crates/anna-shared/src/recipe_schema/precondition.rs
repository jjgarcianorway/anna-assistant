//! Precondition types for recipe execution.
//!
//! Preconditions are checks that must pass before a recipe can be executed.

use serde::{Deserialize, Serialize};

/// Precondition that must be true before recipe execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Precondition {
    /// Check if a tool/command exists
    ToolExists { tool: String },
    /// Check if a file exists
    FileExists { path: String },
    /// Check if a directory exists
    DirExists { path: String },
    /// Check probe result contains expected value
    ProbeContains { probe: String, contains: String },
    /// Check probe result matches regex
    ProbeMatches { probe: String, pattern: String },
    /// Check systemd service exists
    ServiceExists { service: String },
    /// Custom probe check
    ProbeCheck { probe: String, condition: String },
}

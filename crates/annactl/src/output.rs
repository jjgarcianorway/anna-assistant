//! Structured output formatting for annactl
//!
//! Phase 0.3a: JSON + human-readable output for all commands
//! Citation: [archwiki:system_maintenance]

use serde::{Deserialize, Serialize};

/// Standard output envelope for all annactl commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    /// API version
    pub version: String,

    /// Whether the command succeeded
    pub ok: bool,

    /// Current system state
    pub state: String,

    /// Command that was executed
    pub command: String,

    /// Whether command is allowed in current state
    pub allowed: bool,

    /// Wiki citation for state or command
    pub citation: String,

    /// Human-readable message
    pub message: String,
}

impl CommandOutput {
    /// Print both JSON and human-readable output
    pub fn print(&self) {
        // Print JSON to stdout
        if let Ok(json) = serde_json::to_string_pretty(self) {
            println!("{}", json);
        }

        println!();

        // Print human-readable summary
        println!("[anna] current state: {}", self.state);

        if self.allowed {
            println!("[anna] command: {} — {}", self.command, self.message);
        } else {
            println!(
                "[anna] command: {} — not available in this state",
                self.command
            );
        }

        // Print citation
        let wiki_page = self
            .citation
            .trim_start_matches("[archwiki:")
            .trim_end_matches(']')
            .replace('_', " ");
        println!("See Arch Wiki: {}", wiki_page);
    }

    /// Create output for command not available
    pub fn not_available(state: String, command: String, citation: String) -> Self {
        Self {
            version: env!("ANNA_VERSION").to_string(),
            ok: false,
            state,
            command,
            allowed: false,
            citation,
            message: "Command not available in current state.".to_string(),
        }
    }

    /// Create output for daemon unavailable
    pub fn daemon_unavailable(command: String) -> Self {
        Self {
            version: env!("ANNA_VERSION").to_string(),
            ok: false,
            state: "unknown".to_string(),
            command,
            allowed: false,
            citation: "[archwiki:system_maintenance]".to_string(),
            message: "Daemon unavailable. Start with: sudo systemctl start annad".to_string(),
        }
    }

    /// Create output for invalid daemon response
    pub fn invalid_response(command: String, error: String) -> Self {
        Self {
            version: env!("ANNA_VERSION").to_string(),
            ok: false,
            state: "unknown".to_string(),
            command,
            allowed: false,
            citation: "[archwiki:system_maintenance]".to_string(),
            message: format!("Invalid daemon response: {}", error),
        }
    }
}

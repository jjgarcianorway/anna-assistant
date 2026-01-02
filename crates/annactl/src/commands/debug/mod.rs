//! Debug command handlers (v0.0.446).
//!
//! Provides diagnostics for troubleshooting:
//! - `annactl debug last`: Show debug block from the last request
//! - `annactl debug trace`: Show canonical TraceBlock from the last request
//! - `annactl debug request <id>`: Show debug block for a specific request
//! - `annactl debug llm last`: Show last LLM call details
//! - `annactl debug probes last`: Show last probe outputs
//! - `annactl debug config`: Show current debug configuration
//!
//! Debug levels (0-3):
//! - 0 (off):     Normal user output only
//! - 1 (summary): Domain, intent, probes, outcome, reliability, failures
//! - 2 (trace):   Above + probe commands, exit codes, parsed values, LLM tokens
//! - 3 (full):    Above + full prompts/responses, raw probe output, parser errors

mod formatting;
mod handlers;
mod llm;
mod probes;
mod types;

use anyhow::Result;

use super::debug_trace::show_last_trace;
use handlers::{show_debug_config, show_debug_last, show_debug_request};
use llm::show_llm_last;
use probes::show_probes_last;

// Re-export types
pub use types::{DebugCommand, LlmSubCommand, ProbesSubCommand};

/// Handle debug commands.
pub async fn handle_debug(cmd: DebugCommand) -> Result<()> {
    match cmd {
        DebugCommand::Last => show_debug_last().await,
        DebugCommand::Trace { level } => show_last_trace(&level).await,
        DebugCommand::Request { id } => show_debug_request(&id).await,
        DebugCommand::Llm {
            cmd: LlmSubCommand::Last,
        } => show_llm_last().await,
        DebugCommand::Probes {
            cmd: ProbesSubCommand::Last,
        } => show_probes_last().await,
        DebugCommand::Config => show_debug_config(),
    }
}

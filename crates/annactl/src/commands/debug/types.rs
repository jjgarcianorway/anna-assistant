//! Debug command type definitions (v0.0.446).

use clap::Subcommand;

/// Debug subcommands
#[derive(Subcommand, Clone, Debug)]
pub enum DebugCommand {
    /// Show debug block from the last request
    Last,
    /// Show canonical TraceBlock from the last request (structured trace)
    Trace {
        /// Output level: summary, trace, or full (default: trace)
        #[arg(short, long, default_value = "trace")]
        level: String,
    },
    /// Show debug block for a specific request ID
    Request {
        /// Request ID to look up
        id: String,
    },
    /// Show last LLM call details
    Llm {
        #[command(subcommand)]
        cmd: LlmSubCommand,
    },
    /// Show last probe outputs
    Probes {
        #[command(subcommand)]
        cmd: ProbesSubCommand,
    },
    /// Show current debug configuration
    Config,
}

/// LLM subcommands
#[derive(Subcommand, Clone, Debug)]
pub enum LlmSubCommand {
    /// Show the last LLM call
    Last,
}

/// Probes subcommands
#[derive(Subcommand, Clone, Debug)]
pub enum ProbesSubCommand {
    /// Show the last probe outputs
    Last,
}

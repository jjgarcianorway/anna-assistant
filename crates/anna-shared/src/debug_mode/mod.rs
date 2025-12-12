//! Debug Mode System (v0.0.446).
//!
//! Provides 4 debug levels for diagnosing failures:
//! - Level 0 (OFF): Normal user output only
//! - Level 1 (SUMMARY): Domain, intent, probes, outcome, reliability score, failure reason
//! - Level 2 (TRACE): Above + probe commands, exit codes, parsed values, LLM tokens, gate report
//! - Level 3 (FULL): Above + full prompts/responses, raw probe output, parser errors
//!
//! Key components:
//! - `DebugLevel`: The four levels (0-3)
//! - `DebugConfig`: Configuration for debug output
//! - `TraceBlock`: Canonical trace structure for all debug output
//! - `Redactor`: Enhanced redaction with mandatory secret removal
//! - `ReasonCode`: Machine-readable failure reasons

pub mod block;
pub mod config;
pub mod reason_codes;
pub mod redact;
pub mod sanitize;
pub mod trace;

#[cfg(test)]
pub mod tests;

// Re-exports
pub use block::{DebugBlock, ProbeDebugInfo, TranslatorDecision};
pub use config::{DebugConfig, DebugLevel, RedactConfig};
pub use reason_codes::ReasonCode;
pub use redact::{is_sensitive_path, redact_journal_line, redact_proc_cmdline, Redactor};
pub use sanitize::{SanitizeResult, Sanitizer};
pub use trace::{
    FailureDetail, GateResult, LlmTrace, ProbeTrace, TimeoutInfo, TimingTrace, TraceBlock,
    TraceOutcome,
};

/// Version of the debug mode system.
pub const DEBUG_MODE_VERSION: &str = "0.0.446";

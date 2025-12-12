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

use super::debug_trace::show_last_trace;
use anna_shared::debug_mode::{DebugConfig, DebugLevel};
use anna_shared::state_dir;
use anyhow::Result;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

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

/// Path to debug log file.
fn debug_log_path() -> PathBuf {
    PathBuf::from(state_dir()).join("debug_log.json")
}

/// Show debug block from last request.
async fn show_debug_last() -> Result<()> {
    let path = debug_log_path();

    if !path.exists() {
        println!("No debug log found at {}", path.display());
        println!("\nTo enable debug logging, set debug.level in Anna's config:");
        println!("  level = \"trace\"   # Shows routing, timings, models, reason codes");
        println!("  level = \"full\"    # Shows full prompts/responses (sanitized)");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;

    // Parse last entry (file has one JSON object per line)
    if let Some(last_line) = content.lines().rev().find(|l| !l.trim().is_empty()) {
        println!("{}", format_debug_block(last_line)?);
    } else {
        println!("Debug log is empty.");
    }

    Ok(())
}

/// Show debug block for specific request.
async fn show_debug_request(id: &str) -> Result<()> {
    let path = debug_log_path();

    if !path.exists() {
        println!("No debug log found.");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;

    // Search for request ID
    for line in content.lines() {
        if line.contains(id) {
            println!("{}", format_debug_block(line)?);
            return Ok(());
        }
    }

    println!("Request {} not found in debug log.", id);
    println!("Recent request IDs:");

    // Show last 5 request IDs
    for line in content.lines().rev().take(5) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(req_id) = json.get("request_id").and_then(|v| v.as_str()) {
                println!("  {}", req_id);
            }
        }
    }

    Ok(())
}

/// Show last LLM call details.
async fn show_llm_last() -> Result<()> {
    let path = debug_log_path();

    if !path.exists() {
        println!("No debug log found.");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;

    // Find last entry with llm_calls
    for line in content.lines().rev() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(llm_calls) = json.get("llm_calls") {
                if let Some(calls) = llm_calls.as_array() {
                    if !calls.is_empty() {
                        println!("[llm_calls]");
                        for call in calls {
                            print_llm_call(call);
                        }
                        return Ok(());
                    }
                }
            }
        }
    }

    println!("No LLM calls found in debug log.");
    println!("\nNote: LLM call details require debug.level = \"full\"");

    Ok(())
}

/// Print a single LLM call.
fn print_llm_call(call: &serde_json::Value) {
    let role = call
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let model = call
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let duration = call
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let parse_success = call
        .get("parse_success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    println!("  --- {} ({}) ---", role, model);
    println!("  duration: {}ms", duration);
    println!("  parse_success: {}", parse_success);

    if let Some(err) = call.get("parse_error").and_then(|v| v.as_str()) {
        println!("  parse_error: {}", err);
    }

    if let Some(prompt) = call.get("prompt").and_then(|v| v.as_str()) {
        println!("  prompt:");
        for line in prompt.lines().take(20) {
            println!("    {}", line);
        }
        if prompt.lines().count() > 20 {
            println!("    ...[truncated]");
        }
    }

    if let Some(response) = call.get("response").and_then(|v| v.as_str()) {
        println!("  response:");
        for line in response.lines().take(20) {
            println!("    {}", line);
        }
        if response.lines().count() > 20 {
            println!("    ...[truncated]");
        }
    }
}

/// Show last probe outputs.
async fn show_probes_last() -> Result<()> {
    let path = debug_log_path();

    if !path.exists() {
        println!("No debug log found.");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;

    // Find last entry with probes
    for line in content.lines().rev() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(probes) = json.get("probes_run") {
                if let Some(probes_arr) = probes.as_array() {
                    if !probes_arr.is_empty() {
                        println!("[probes_run]");
                        for probe in probes_arr {
                            print_probe(probe);
                        }
                        return Ok(());
                    }
                }
            }
        }
    }

    println!("No probe outputs found in debug log.");

    Ok(())
}

/// Print a single probe.
fn print_probe(probe: &serde_json::Value) {
    let id = probe
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let cmd = probe.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let status = probe
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let exit_code = probe
        .get("exit_code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    let duration = probe
        .get("duration_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("  --- {} ({}) ---", id, cmd);
    println!("  status: {} (exit {})", status, exit_code);
    println!("  duration: {}ms", duration);

    if let Some(stdout) = probe.get("stdout").and_then(|v| v.as_str()) {
        if !stdout.is_empty() {
            println!("  stdout:");
            for line in stdout.lines().take(10) {
                println!("    {}", line);
            }
            if stdout.lines().count() > 10 {
                println!("    ...[truncated]");
            }
        }
    }

    if let Some(stderr) = probe.get("stderr").and_then(|v| v.as_str()) {
        if !stderr.is_empty() {
            println!("  stderr:");
            for line in stderr.lines().take(5) {
                println!("    {}", line);
            }
        }
    }
}

/// Show current debug configuration.
fn show_debug_config() -> Result<()> {
    // Load config from file or show defaults
    let config_path = PathBuf::from(state_dir()).join("debug_config.toml");

    let config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        toml::from_str::<DebugConfig>(&content).unwrap_or_default()
    } else {
        DebugConfig::default()
    };

    println!("[debug configuration]");
    println!(
        "  level: {}",
        match config.level {
            DebugLevel::Off => "off (0) - normal user output only",
            DebugLevel::Summary => "summary (1) - domain, intent, probes, outcome, failures",
            DebugLevel::Trace => "trace (2) - above + probe details, LLM tokens, gate report",
            DebugLevel::Full => "full (3) - above + raw prompts/responses, raw probe output",
        }
    );
    println!();
    println!("[redaction settings]");
    println!("  redact_emails: {}", config.redact.redact_emails);
    println!("  redact_private_ips: {}", config.redact.redact_private_ips);
    println!("  redact_secrets: {}", config.redact.redact_secrets);
    println!(
        "  redact_sensitive_files: {}",
        config.redact.redact_sensitive_files
    );
    println!("  max_probe_lines: {}", config.redact.max_probe_lines);
    println!(
        "  max_llm_output_chars: {}",
        config.redact.max_llm_output_chars
    );
    println!();
    println!("Config file: {}", config_path.display());

    if !config_path.exists() {
        println!(
            "\nTo enable debug mode, create {} with:",
            config_path.display()
        );
        println!("  level = \"summary\"  # or \"trace\" or \"full\"");
    }

    Ok(())
}

/// Format a debug block for display.
fn format_debug_block(json_line: &str) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(json_line)?;
    let mut out = String::new();

    out.push_str("[debug]\n");

    if let Some(v) = json.get("request_id").and_then(|v| v.as_str()) {
        out.push_str(&format!("  request_id          {}\n", v));
    }
    if let Some(v) = json.get("outcome").and_then(|v| v.as_str()) {
        out.push_str(&format!("  outcome             {}\n", v));
    }
    if let Some(v) = json.get("routed_topic").and_then(|v| v.as_str()) {
        out.push_str(&format!("  routed_topic        {}\n", v));
    }

    // Models
    if let Some(models) = json.get("models") {
        let mut parts = Vec::new();
        if let Some(t) = models.get("translator").and_then(|v| v.as_str()) {
            parts.push(format!("translator:{}", t));
        }
        if let Some(s) = models.get("specialist").and_then(|v| v.as_str()) {
            parts.push(format!("specialist:{}", s));
        }
        if !parts.is_empty() {
            out.push_str(&format!("  models_used         {}\n", parts.join(", ")));
        }
    }

    // Timings
    if let Some(timings) = json.get("timings") {
        let total = timings
            .get("total_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let probe = timings
            .get("probe_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let llm = timings.get("llm_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        out.push_str(&format!(
            "  timings             total:{}ms probe:{}ms llm:{}ms\n",
            total, probe, llm
        ));
    }

    // Reason codes
    if let Some(codes) = json.get("reason_codes").and_then(|v| v.as_array()) {
        let code_strs: Vec<&str> = codes.iter().filter_map(|c| c.as_str()).collect();
        out.push_str(&format!(
            "  reason_codes        [{}]\n",
            code_strs.join(", ")
        ));
    }

    // Timeout
    if let Some(timeout) = json.get("timeout") {
        let stage = timeout
            .get("stage")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let configured = timeout
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let elapsed = timeout
            .get("elapsed_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        out.push_str(&format!(
            "  timeout             {} (configured: {}ms, elapsed: {}ms)\n",
            stage, configured, elapsed
        ));
    }

    Ok(out)
}

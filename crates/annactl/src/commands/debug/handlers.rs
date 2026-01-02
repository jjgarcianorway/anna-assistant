//! Core debug command handlers (v0.0.446).

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use anna_shared::debug_mode::{DebugConfig, DebugLevel};
use anna_shared::state_dir;

use super::formatting::format_debug_block;

/// Path to debug log file.
fn debug_log_path() -> PathBuf {
    PathBuf::from(state_dir()).join("debug_log.json")
}

/// Show debug block from last request.
pub async fn show_debug_last() -> Result<()> {
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
pub async fn show_debug_request(id: &str) -> Result<()> {
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

/// Show current debug configuration.
pub fn show_debug_config() -> Result<()> {
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

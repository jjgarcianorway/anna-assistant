//! LLM-specific debug handlers (v0.0.446).

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use anna_shared::state_dir;

/// Path to debug log file.
fn debug_log_path() -> PathBuf {
    PathBuf::from(state_dir()).join("debug_log.json")
}

/// Show last LLM call details.
pub async fn show_llm_last() -> Result<()> {
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

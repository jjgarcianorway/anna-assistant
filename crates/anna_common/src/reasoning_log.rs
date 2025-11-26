//! Anna Reasoning Logger - v10.2.1
//!
//! Debug logging for chain-of-thought reasoning.
//! Enabled via config.toml [dev] log_reasoning = true or ANNA_LOG_REASONING=1
//!
//! Logs to: ~/.local/share/anna/reasoning.log

use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialize reasoning logger
pub fn init(enabled: bool) {
    LOGGING_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled {
        if let Ok(path) = log_file_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
        }
    }
}

/// Check if reasoning logging is enabled
pub fn is_enabled() -> bool {
    LOGGING_ENABLED.load(Ordering::SeqCst)
        || std::env::var("ANNA_LOG_REASONING").map(|v| v == "1").unwrap_or(false)
}

/// Get the log file path
fn log_file_path() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = PathBuf::from(home)
        .join(".local/share/anna/reasoning.log");
    Ok(path)
}

/// Log a reasoning step
pub fn log_step(iteration: usize, step_type: &str, reasoning: &str) {
    if !is_enabled() {
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!(
        "\n[{}] Iteration {} - {}\n{}\n{}\n",
        timestamp,
        iteration,
        step_type,
        "-".repeat(60),
        reasoning
    );

    if let Ok(path) = log_file_path() {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

/// Log a tool execution
pub fn log_tool(iteration: usize, tool: &str, args: &str, why: &str) {
    if !is_enabled() {
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!(
        "\n[{}] Iteration {} - TOOL: {}\nArgs: {}\nWhy: {}\n",
        timestamp,
        iteration,
        tool,
        args,
        why
    );

    if let Ok(path) = log_file_path() {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

/// Log evidence added
pub fn log_evidence(iteration: usize, evidence_id: &str, source: &str, content_preview: &str) {
    if !is_enabled() {
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let preview = if content_preview.len() > 200 {
        format!("{}...", &content_preview[..200])
    } else {
        content_preview.to_string()
    };

    let entry = format!(
        "\n[{}] Iteration {} - EVIDENCE [{}] from {}\n{}\n",
        timestamp,
        iteration,
        evidence_id,
        source,
        preview
    );

    if let Ok(path) = log_file_path() {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

/// Log the final answer
pub fn log_answer(query: &str, answer: &str, reliability: f32, label: &str) {
    if !is_enabled() {
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!(
        "\n\n{}\n[{}] QUERY: {}\n{}\n\nANSWER ({} - {:.0}%):\n{}\n{}\n\n",
        "=".repeat(80),
        timestamp,
        query,
        "=".repeat(80),
        label,
        reliability * 100.0,
        answer,
        "=".repeat(80)
    );

    if let Ok(path) = log_file_path() {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

/// Start a new query session in the log
pub fn log_query_start(query: &str) {
    if !is_enabled() {
        return;
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!(
        "\n\n{}\n[{}] NEW QUERY: {}\n{}\n",
        "=".repeat(80),
        timestamp,
        query,
        "=".repeat(80)
    );

    if let Ok(path) = log_file_path() {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_disabled_by_default() {
        // Should not crash when logging is disabled
        log_step(1, "decide_tool", "test reasoning");
        log_tool(1, "run_shell", "pacman -Qs test", "testing");
        log_evidence(1, "E1", "test", "content");
        log_answer("test query?", "test answer", 0.9, "HIGH");
    }
}

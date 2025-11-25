//! Version Banner and Update Status Display
//!
//! Displays Anna's version, mode, and update status at startup

use anna_common::context::db::{ContextDb, DbLocation};
use anna_common::llm::{LlmConfig, LlmMode};
use anna_common::terminal_format as fmt;
use anyhow::Result;

/// Display startup banner with version and mode
///
/// This is the FIRST thing users see when running annactl.
/// Shows: version, assistance mode, and auto-update status (if applicable)
pub async fn display_startup_banner(db: &ContextDb) {
    let version = env!("CARGO_PKG_VERSION");

    // Get LLM config
    let mode_str = match db.load_llm_config().await {
        Ok(config) => format_llm_mode(&config),
        Err(_) => "Rules + Arch Wiki (LLM not configured)".to_string(),
    };

    // Single-line banner: "Anna Assistant v5.7.0-beta.1 · mode: Rules + Arch Wiki"
    println!(
        "{} {} {} {}",
        fmt::bold("Anna Assistant"),
        fmt::bold(&format!("v{}", version)),
        fmt::dimmed("·"),
        fmt::dimmed(&format!("mode: {}", mode_str))
    );

    // Check if this is a post-update first run
    if let Ok(true) = check_and_show_update_notice(version).await {
        // Update notice was shown, blank line already printed
    }

    println!(); // Blank line after banner/notice
}

/// Format LLM mode for display (v6.54.1: shows config source)
pub fn format_llm_mode(config: &LlmConfig) -> String {
    match config.mode {
        LlmMode::NotConfigured => "Rules + Arch Wiki (LLM not configured)".to_string(),
        LlmMode::Disabled => "Rules + Arch Wiki only".to_string(),
        LlmMode::Local => {
            let model = config.model.as_deref().unwrap_or("llama3.2");

            // v6.54.1: Check if model matches user config
            let user_config = anna_common::config::Config::load().ok();
            let preferred_model = user_config
                .as_ref()
                .and_then(|c| c.llm.model.as_deref());

            if let Some(preferred) = preferred_model {
                if preferred == model {
                    format!("Local LLM via Ollama: {} (from config)", model)
                } else {
                    format!("Local LLM via Ollama: {} (config model '{}' missing)", model, preferred)
                }
            } else {
                format!("Local LLM via Ollama: {}", model)
            }
        }
        LlmMode::Remote => {
            // Show just the hostname for brevity
            if let Some(base_url) = &config.base_url {
                if let Some(host) = base_url
                    .strip_prefix("https://")
                    .or_else(|| base_url.strip_prefix("http://"))
                {
                    let hostname = host.split('/').next().unwrap_or(host);
                    format!("Remote API: {}", hostname)
                } else {
                    "Remote API: OpenAI-compatible".to_string()
                }
            } else {
                "Remote API: OpenAI-compatible".to_string()
            }
        }
    }
}

/// Check if version changed since last run and show update notice
///
/// Returns: true if notice was shown, false otherwise
async fn check_and_show_update_notice(current_version: &str) -> Result<bool> {
    // Use a simple file-based approach for version tracking
    const LAST_VERSION_FILE: &str = "/var/lib/anna/last_seen_version";

    let last_version = tokio::fs::read_to_string(LAST_VERSION_FILE)
        .await
        .ok()
        .map(|s| s.trim().to_string());

    // If version changed, show notice
    if let Some(last) = last_version {
        if last != current_version {
            // Show auto-update notice (Beta.89: removed non-existent 'annactl changelog' reference)
            println!(
                "{}",
                fmt::success(&format!(
                    "Anna auto-updated from v{} to v{}",
                    last, current_version
                ))
            );

            // Update last seen version
            let _ = tokio::fs::write(LAST_VERSION_FILE, current_version).await;
            return Ok(true);
        }
    } else {
        // First run ever, just record current version
        let _ = tokio::fs::write(LAST_VERSION_FILE, current_version).await;
    }

    Ok(false)
}

/// Display version and mode only (for `annactl version` command)
pub async fn display_version_only() {
    let version = env!("CARGO_PKG_VERSION");

    // Try to get LLM config
    let db_location = DbLocation::auto_detect();
    let mode_str = match ContextDb::open(db_location).await {
        Ok(db) => match db.load_llm_config().await {
            Ok(config) => format_llm_mode(&config),
            Err(_) => "Rules + Arch Wiki (LLM not configured)".to_string(),
        },
        Err(_) => "Rules + Arch Wiki (LLM not configured)".to_string(),
    };

    println!("Anna Assistant v{}", version);
    println!("Mode: {}", mode_str);
}

//! LLM Bootstrap - Auto-detect and configure LLM on first run
//!
//! When Anna starts and finds no LLM config, but Ollama is running with a model,
//! automatically detect and save the configuration.

use anyhow::{Context, Result};
use anna_common::context::db::{ContextDb, DbLocation};
use anna_common::llm::{LlmConfig, LlmMode};
use std::process::Command;
use tracing::{info, warn};

/// Bootstrap LLM configuration if not already set up
///
/// This runs on daemon startup to auto-detect Ollama installations
/// that were set up by the installer but not yet configured in Anna's database.
pub async fn bootstrap_llm_if_needed() -> Result<()> {
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Check if LLM is already configured
    let existing_config = db.load_llm_config().await?;

    if existing_config.mode != LlmMode::NotConfigured {
        info!("LLM already configured: {}", existing_config.description);
        return Ok(());
    }

    info!("LLM not configured, checking for Ollama installation...");

    // Check if Ollama service is running
    let ollama_running = Command::new("systemctl")
        .args(&["is-active", "--quiet", "ollama"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ollama_running {
        info!("Ollama service not running, skipping LLM bootstrap");
        return Ok(());
    }

    // Check if Ollama API is reachable
    let api_check = Command::new("curl")
        .args(&["-s", "-f", "http://localhost:11434/api/version", "--max-time", "2"])
        .output();

    if !api_check.map(|o| o.status.success()).unwrap_or(false) {
        warn!("Ollama service running but API not reachable");
        return Ok(());
    }

    // Get list of available models
    let model_list = Command::new("ollama")
        .args(&["list"])
        .output()
        .context("Failed to list Ollama models")?;

    if !model_list.status.success() {
        warn!("Failed to get Ollama model list");
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&model_list.stdout);

    // Look for llama3.2 models (1b or 3b)
    let model = if stdout.contains("llama3.2:3b") {
        "llama3.2:3b"
    } else if stdout.contains("llama3.2:1b") {
        "llama3.2:1b"
    } else if let Some(line) = stdout.lines().nth(1) {
        // Grab the first model name from the second line (first line is headers)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            parts[0]
        } else {
            warn!("No Ollama models found");
            return Ok(());
        }
    } else {
        warn!("No Ollama models found");
        return Ok(());
    };

    info!("Detected Ollama model: {}", model);

    // Create and save LLM config
    let config = LlmConfig::local("http://127.0.0.1:11434/v1", model);

    db.save_llm_config(&config).await
        .context("Failed to save LLM config")?;

    info!("✓ LLM auto-configured: Ollama with {}", model);

    Ok(())
}

//! LLM Bootstrap - Auto-detect and configure LLM on first run
//!
//! When Anna starts and finds no LLM config, but Ollama is running with a model,
//! automatically detect and save the configuration.

use anna_common::context::db::{ContextDb, DbLocation};
use anna_common::llm::{LlmConfig, LlmMode};
use anyhow::{Context, Result};
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
        .args(&[
            "-s",
            "-f",
            "http://localhost:11434/api/version",
            "--max-time",
            "2",
        ])
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

    // Parse available models from Ollama
    let available_models: Vec<String> = stdout
        .lines()
        .skip(1) // Skip header line
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect();

    if available_models.is_empty() {
        warn!("No Ollama models found");
        return Ok(());
    }

    info!("Available models: {:?}", available_models);

    // Detect hardware capabilities
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();

    let ram_gb = sys.total_memory() as f64 / 1_073_741_824.0; // bytes to GB
    let cores = sys.physical_core_count().unwrap_or(2);

    info!("Hardware: {:.1}GB RAM, {} CPU cores", ram_gb, cores);

    // Get recommended model profiles based on hardware
    use anna_common::model_profiles::{get_available_profiles, get_recommended_with_fallbacks};

    let (recommended, _fallbacks) = get_recommended_with_fallbacks(ram_gb, cores);

    // Try to find a suitable model from available Ollama models
    let all_profiles = get_available_profiles();

    // First priority: recommended model if available
    if let Some(rec) = recommended {
        if available_models.contains(&rec.model_name) {
            info!(
                "✓ Using recommended model for hardware: {} ({})",
                rec.model_name, rec.description
            );
            let config = LlmConfig::local("http://127.0.0.1:11434/v1", &rec.model_name);
            db.save_llm_config(&config).await?;
            info!("✓ LLM auto-configured: Ollama with {}", rec.model_name);
            return Ok(());
        }
    }

    // Second priority: find best available model from profiles
    let best_available = all_profiles
        .into_iter()
        .filter(|p| {
            available_models.contains(&p.model_name) && p.is_suitable_for(ram_gb, cores)
        })
        .max_by_key(|p| p.quality_tier);

    let model = if let Some(profile) = best_available {
        info!(
            "✓ Selected best available model: {} ({})",
            profile.model_name, profile.description
        );
        profile.model_name
    } else {
        // Fallback: just use first available model
        warn!(
            "No optimal model found in profiles, using first available: {}",
            available_models[0]
        );
        available_models[0].clone()
    };

    // Create and save LLM config
    let config = LlmConfig::local("http://127.0.0.1:11434/v1", &model);

    db.save_llm_config(&config)
        .await
        .context("Failed to save LLM config")?;

    info!("✓ LLM auto-configured: Ollama with {}", model);

    Ok(())
}

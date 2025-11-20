//! LLM Bootstrap - Auto-detect and configure LLM on first run
//!
//! When Anna starts and finds no LLM config, but Ollama is running with a model,
//! automatically detect and save the configuration.

use anna_common::context::db::{ContextDb, DbLocation};
use anna_common::llm::{LlmConfig, LlmMode};
use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, warn};

/// Extract parameter size from model name (e.g., "llama3.2:3b" → Some(3.0))
/// Handles formats like "3b", "8b", "7b", "1.5b", "13b", etc.
fn extract_param_size(model_name: &str) -> Option<f64> {
    // Look for pattern like ":3b", ":8b", ":1.5b", etc.
    let parts: Vec<&str> = model_name.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let size_part = parts.last()?.trim().to_lowercase();

    // Remove 'b' suffix
    if !size_part.ends_with('b') {
        return None;
    }

    let number_part = &size_part[..size_part.len() - 1];
    number_part.parse::<f64>().ok()
}

/// Bootstrap LLM configuration if not already set up
///
/// This runs on daemon startup to auto-detect Ollama installations
/// that were set up by the installer but not yet configured in Anna's database.
pub async fn bootstrap_llm_if_needed() -> Result<()> {
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Check if LLM is already configured
    let existing_config = db.load_llm_config().await?;

    let is_configured = existing_config.mode != LlmMode::NotConfigured;

    if is_configured {
        info!("LLM already configured: {}", existing_config.description);
        // Still check if a better model is available for upgrade
    } else {
        info!("LLM not configured, checking for Ollama installation...");
    }

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
            // Beta.89 FIX: Don't override user's explicit model choice
            // Only auto-configure if NOT already configured
            if is_configured {
                let current_model = existing_config.model.as_deref().unwrap_or("unknown");

                // If user already has this model, we're good
                if current_model == rec.model_name {
                    info!("✓ Already using recommended model: {}", current_model);
                    return Ok(());
                }

                // Check if recommended model is actually better
                let current_profile = get_available_profiles()
                    .into_iter()
                    .find(|p| p.model_name == current_model);

                if let Some(current) = current_profile {
                    if rec.quality_tier <= current.quality_tier {
                        info!(
                            "✓ Current model {} ({:?}) is adequate, not downgrading to {} ({:?})",
                            current.model_name,
                            current.quality_tier,
                            rec.model_name,
                            rec.quality_tier
                        );
                        return Ok(());
                    } else {
                        info!(
                            "⚡ Upgrading to recommended model: {} ({:?}) → {} ({:?})",
                            current.model_name,
                            current.quality_tier,
                            rec.model_name,
                            rec.quality_tier
                        );
                    }
                } else {
                    // Current model not in profiles - parse parameter size to prevent downgrade
                    let current_params = extract_param_size(current_model);
                    let rec_params = extract_param_size(&rec.model_name);

                    if let (Some(current_size), Some(rec_size)) = (current_params, rec_params) {
                        if rec_size < current_size {
                            info!(
                                "✓ Current model {} ({} params) is better than recommended {} ({} params), not downgrading",
                                current_model, current_size, rec.model_name, rec_size
                            );
                            return Ok(());
                        }
                    }

                    info!(
                        "⚡ Switching to recommended model: {} → {}",
                        current_model, rec.model_name
                    );
                }
            }

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
        .filter(|p| available_models.contains(&p.model_name) && p.is_suitable_for(ram_gb, cores))
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

    // Check if we should upgrade the existing model
    if is_configured {
        // Extract current model name from existing config
        let current_model = existing_config.model.as_deref().unwrap_or("unknown");

        if current_model == model {
            info!(
                "✓ Already using optimal model: {} (no upgrade needed)",
                current_model
            );
            return Ok(());
        }

        // Check if new model is significantly better
        let current_profile = get_available_profiles()
            .into_iter()
            .find(|p| p.model_name == current_model);
        let new_profile = get_available_profiles()
            .into_iter()
            .find(|p| p.model_name == model);

        if let (Some(current), Some(new)) = (current_profile, new_profile) {
            if new.quality_tier > current.quality_tier {
                info!(
                    "⚡ Upgrading LLM: {} ({:?}) → {} ({:?})",
                    current.model_name, current.quality_tier, new.model_name, new.quality_tier
                );
            } else {
                info!(
                    "✓ Current model {} is adequate ({:?} vs {:?})",
                    current.model_name, current.quality_tier, new.quality_tier
                );
                return Ok(());
            }
        } else {
            // One or both models not in profiles - compare parameter sizes
            let current_params = extract_param_size(current_model);
            let new_params = extract_param_size(&model);

            if let (Some(current_size), Some(new_size)) = (current_params, new_params) {
                if new_size <= current_size {
                    info!(
                        "✓ Current model {} ({} params) is adequate, not downgrading to {} ({} params)",
                        current_model, current_size, model, new_size
                    );
                    return Ok(());
                } else {
                    info!(
                        "⚡ Upgrading to better model: {} ({} params) → {} ({} params)",
                        current_model, current_size, model, new_size
                    );
                }
            } else {
                info!(
                    "⚡ Switching to better model: {} → {}",
                    current_model, model
                );
            }
        }
    }

    // Create and save LLM config
    let config = LlmConfig::local("http://127.0.0.1:11434/v1", &model);

    db.save_llm_config(&config)
        .await
        .context("Failed to save LLM config")?;

    if is_configured {
        info!("✓ LLM upgraded to: Ollama with {}", model);
    } else {
        info!("✓ LLM auto-configured: Ollama with {}", model);
    }

    Ok(())
}

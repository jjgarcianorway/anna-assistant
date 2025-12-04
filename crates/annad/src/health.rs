//! Health check module - monitors Ollama and model availability.

use crate::ollama;
use crate::permissions;
use crate::state::SharedState;
use anna_shared::status::LlmState;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

/// Health check interval in seconds
const HEALTH_CHECK_INTERVAL: u64 = 30;

/// Run the health check loop
pub async fn health_check_loop(state: SharedState) {
    let mut interval = interval(Duration::from_secs(HEALTH_CHECK_INTERVAL));

    loop {
        interval.tick().await;

        if let Err(e) = check_and_repair(state.clone()).await {
            error!("Health check failed: {}", e);
        }
    }
}

/// Check system health and repair if needed
async fn check_and_repair(state: SharedState) -> anyhow::Result<()> {
    // Check 0: Permissions - ensure anna group and user access
    check_permissions();

    // Get current expected model
    let model_name = {
        let state = state.read().await;
        state
            .llm
            .models
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "llama3.2:1b".to_string())
    };

    // Check 1: Is Ollama installed?
    if !ollama::is_installed() {
        warn!("Health check: Ollama not installed, triggering repair");
        trigger_repair(state.clone(), "ollama_missing").await?;
        return Ok(());
    }

    // Check 2: Is Ollama running?
    if !ollama::is_running().await {
        warn!("Health check: Ollama not running, attempting to start");

        // Try to start it
        if let Err(e) = ollama::start_service().await {
            warn!("Failed to start Ollama: {}, triggering full repair", e);
            trigger_repair(state.clone(), "ollama_not_running").await?;
            return Ok(());
        }

        info!("Health check: Ollama restarted successfully");
    }

    // Check 3: Is the model available?
    if !ollama::has_model(&model_name).await {
        warn!("Health check: Model {} not available, triggering repair", model_name);
        trigger_repair(state.clone(), "model_missing").await?;
        return Ok(());
    }

    // All checks passed - ensure state is Ready
    {
        let mut state = state.write().await;
        if state.llm.state != LlmState::Ready {
            state.set_llm_ready();
            info!("Health check: All systems operational, marking Ready");
        }
    }

    Ok(())
}

/// Trigger a repair sequence
async fn trigger_repair(state: SharedState, reason: &str) -> anyhow::Result<()> {
    info!("Starting repair sequence: {}", reason);

    // Set state to bootstrapping
    {
        let mut state = state.write().await;
        state.llm.state = LlmState::Bootstrapping;
        state.llm.phase = Some(format!("repairing: {}", reason));
    }

    // Step 1: Ensure Ollama is installed
    {
        let mut state = state.write().await;
        state.set_llm_phase("installing_ollama");
    }

    if !ollama::is_installed() {
        ollama::install().await?;
    }

    // Step 2: Ensure Ollama is running
    {
        let mut state = state.write().await;
        state.set_llm_phase("starting_ollama");
    }

    if !ollama::is_running().await {
        ollama::start_service().await?;
    }

    // Update Ollama status
    {
        let mut state = state.write().await;
        state.ollama = ollama::get_status().await;
    }

    // Step 3: Get the model name and ensure it's pulled
    let model_name = {
        let state = state.read().await;
        state
            .llm
            .models
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "llama3.2:1b".to_string())
    };

    {
        let mut state = state.write().await;
        state.set_llm_phase("pulling_models");
    }

    if !ollama::has_model(&model_name).await {
        ollama::pull_model(&model_name).await?;
    }

    // Step 4: Mark ready
    {
        let mut state = state.write().await;
        state.set_llm_ready();
    }

    info!("Repair sequence completed successfully");
    Ok(())
}

/// Check and fix permissions (group setup, user access)
fn check_permissions() {
    // Ensure anna group exists and directories are properly configured
    if let Err(e) = permissions::ensure_permissions() {
        warn!("Permissions check failed: {}", e);
        return;
    }

    // Check if any regular users need to be added to the anna group
    let fixed = permissions::check_and_fix_user_access();
    if !fixed.is_empty() {
        info!(
            "Added {} user(s) to anna group: {}",
            fixed.len(),
            fixed.join(", ")
        );
    }
}

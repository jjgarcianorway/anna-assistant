//! Ollama model management - pull, list, delete, check models.
//! v0.0.825: Extracted from ollama.rs for modularity.

use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Duration;
use tracing::{info, warn};

const OLLAMA_API: &str = "http://127.0.0.1:11434";

/// Create an ollama command with required environment variables
fn ollama_cmd() -> Command {
    let mut cmd = Command::new("ollama");
    cmd.env("HOME", "/root");
    cmd.env("OLLAMA_MODELS", "/var/lib/anna/models");
    cmd
}

/// Pull a model with retry logic for network resilience
/// v0.0.291: Added exponential backoff retry for transient network failures
pub async fn pull_model(model: &str) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            // Exponential backoff: 2s, 4s, 8s
            let delay_secs = 2 * (1 << (attempt - 1));
            info!(
                "Model pull retry {} for {} after {}s delay",
                attempt, model, delay_secs
            );
            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
        }

        match pull_model_single_attempt(model) {
            Ok(()) => return Ok(()),
            Err(e) => {
                warn!(
                    "Model pull attempt {} failed for {}: {}",
                    attempt + 1,
                    model,
                    e
                );
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        anyhow!(
            "Failed to pull model {} after {} retries",
            model,
            MAX_RETRIES
        )
    }))
}

/// Single model pull attempt (internal helper)
fn pull_model_single_attempt(model: &str) -> Result<()> {
    info!("Pulling model: {}", model);

    let output = ollama_cmd().args(["pull", model]).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to pull model {}: {}", model, stderr));
    }

    info!("Model {} pulled successfully", model);
    Ok(())
}

/// Check if a model is available locally
pub async fn has_model(model: &str) -> bool {
    let client = reqwest::Client::new();

    let response = client.get(format!("{}/api/tags", OLLAMA_API)).send().await;

    match response {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                    return models.iter().any(|m| {
                        m.get("name")
                            .and_then(|n| n.as_str())
                            .map(|n| n.starts_with(model))
                            .unwrap_or(false)
                    });
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// v0.0.303: Delete a model from Ollama
/// Used to clean up unused models and free disk space
pub async fn delete_model(model: &str) -> Result<()> {
    info!("Deleting model: {}", model);

    let output = ollama_cmd().args(["rm", model]).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to delete model {}: {}", model, stderr));
    }

    info!("Model {} deleted successfully", model);
    Ok(())
}

/// List all locally available models from Ollama
/// v0.0.269: Added for intelligent model auto-selection
pub async fn list_models() -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .get(format!("{}/api/tags", OLLAMA_API))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to list models: HTTP {}", response.status()));
    }

    let json: serde_json::Value = response.json().await?;
    let models = json
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

/// Run a simple benchmark to test throughput
pub async fn benchmark(model: &str) -> Result<f64> {
    info!("Running benchmark for model: {}", model);

    let start = std::time::Instant::now();
    let prompt = "Count from 1 to 10.";

    let _ = super::ollama::chat(model, prompt).await?;

    let elapsed = start.elapsed();
    let tokens_per_sec = 50.0 / elapsed.as_secs_f64(); // Rough estimate

    info!("Benchmark: ~{:.1} tokens/sec", tokens_per_sec);
    Ok(tokens_per_sec)
}

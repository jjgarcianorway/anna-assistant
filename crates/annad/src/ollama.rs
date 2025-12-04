//! Ollama management - install, run, and interact with Ollama.

use anna_shared::status::OllamaStatus;
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

/// Detect the system's package manager
fn detect_package_manager() -> Option<&'static str> {
    // Check for pacman (Arch, Manjaro, EndeavourOS)
    if Command::new("which")
        .arg("pacman")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("pacman");
    }

    // Check for apt (Debian, Ubuntu)
    if Command::new("which")
        .arg("apt")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("apt");
    }

    // Check for dnf (Fedora)
    if Command::new("which")
        .arg("dnf")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("dnf");
    }

    None
}

/// Check if Ollama is installed
pub fn is_installed() -> bool {
    Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install Ollama using the system package manager
pub async fn install() -> Result<()> {
    info!("Installing Ollama...");

    let pkg_manager = detect_package_manager();

    let result = match pkg_manager {
        Some("pacman") => {
            info!("Using pacman to install Ollama");
            Command::new("pacman")
                .args(["-S", "--noconfirm", "ollama"])
                .output()
        }
        Some("apt") => {
            info!("Using apt to install Ollama");
            // For apt, we need to add the repo first or use the script
            Command::new("bash")
                .args(["-c", "curl -fsSL https://ollama.com/install.sh | sh"])
                .output()
        }
        Some("dnf") => {
            info!("Using dnf to install Ollama");
            Command::new("bash")
                .args(["-c", "curl -fsSL https://ollama.com/install.sh | sh"])
                .output()
        }
        _ => {
            info!("Using official installer script");
            Command::new("bash")
                .args(["-c", "curl -fsSL https://ollama.com/install.sh | sh"])
                .output()
        }
    };

    match result {
        Ok(output) if output.status.success() => {
            info!("Ollama installed successfully");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to install Ollama: {}", stderr))
        }
        Err(e) => Err(anyhow!("Failed to run installer: {}", e)),
    }
}

/// Check if Ollama service is running
pub async fn is_running() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    client
        .get(format!("{}/api/tags", OLLAMA_API))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Start Ollama service
pub async fn start_service() -> Result<()> {
    info!("Starting Ollama service...");

    // Try systemctl first
    let output = Command::new("systemctl")
        .args(["start", "ollama"])
        .output()?;

    if output.status.success() {
        // Wait for service to be ready
        for _ in 0..30 {
            if is_running().await {
                info!("Ollama service started");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    // Fallback: try starting directly
    warn!("systemctl failed, trying direct start");
    let _child = ollama_cmd().arg("serve").spawn()?;

    for _ in 0..30 {
        if is_running().await {
            info!("Ollama started directly");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(anyhow!("Failed to start Ollama service"))
}

/// Get Ollama version
pub async fn get_version() -> Option<String> {
    let output = ollama_cmd().arg("--version").output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Some(stdout.trim().to_string())
    } else {
        None
    }
}

/// Get full Ollama status
pub async fn get_status() -> OllamaStatus {
    OllamaStatus {
        installed: is_installed(),
        running: is_running().await,
        version: get_version().await,
    }
}

/// Pull a model
pub async fn pull_model(model: &str) -> Result<()> {
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

/// Send a chat request to Ollama (default timeout)
pub async fn chat(model: &str, prompt: &str) -> Result<String> {
    chat_with_timeout(model, prompt, 120).await
}

/// Send a chat request to Ollama with explicit timeout
pub async fn chat_with_timeout(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false
    });

    let response = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Ollama request failed: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await?;
    let response_text = json
        .get("response")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();

    Ok(response_text)
}

/// Run a simple benchmark to test throughput
pub async fn benchmark(model: &str) -> Result<f64> {
    info!("Running benchmark for model: {}", model);

    let start = std::time::Instant::now();
    let prompt = "Count from 1 to 10.";

    let _ = chat(model, prompt).await?;

    let elapsed = start.elapsed();
    let tokens_per_sec = 50.0 / elapsed.as_secs_f64(); // Rough estimate

    info!("Benchmark: ~{:.1} tokens/sec", tokens_per_sec);
    Ok(tokens_per_sec)
}

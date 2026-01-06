//! Ollama management - install, run, and interact with Ollama.
//! v0.0.143: Added streaming support for word-by-word output.
//! v0.0.158: Streaming functions extracted to ollama_streaming.rs.
//! v0.0.825: Model management extracted to ollama_models.rs.

use anna_shared::status::OllamaStatus;
use anyhow::{anyhow, Result};
use std::process::Command;
use std::time::Duration;
use tracing::{info, warn};

// Re-export streaming functions from dedicated module (v0.0.158)
pub use crate::ollama_streaming::{chat_streaming, chat_streaming_with_retry};

// Re-export model management functions (v0.0.825)
pub use crate::ollama_models::{benchmark, delete_model, has_model, list_models, pull_model};

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

/// Clean up manually installed Ollama files that conflict with pacman packages
/// v0.0.820: Required because curl installer leaves files that conflict with pacman
fn cleanup_manual_ollama_install() {
    use std::path::Path;

    // Files/dirs left by the curl installer that conflict with pacman
    let manual_install_paths = [
        "/usr/bin/ollama",
        "/usr/lib/ollama",
        "/usr/lib/systemd/system/ollama.service",
        "/usr/lib/sysusers.d/ollama.conf",
        "/usr/lib/tmpfiles.d/ollama.conf",
        "/usr/share/licenses/ollama",
        "/usr/share/ollama",
    ];

    // Check if ollama binary exists but isn't owned by pacman (manual install)
    let is_manual = Command::new("pacman")
        .args(["-Qo", "/usr/bin/ollama"])
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(false);

    if !is_manual && Path::new("/usr/bin/ollama").exists() {
        // Owned by pacman, don't touch
        return;
    }

    info!("Cleaning up manually installed Ollama files...");

    // Stop service first
    let _ = Command::new("systemctl").args(["stop", "ollama"]).output();
    let _ = Command::new("pkill").args(["-9", "ollama"]).output();
    std::thread::sleep(std::time::Duration::from_secs(1));

    for path in &manual_install_paths {
        let p = Path::new(path);
        if p.exists() {
            if p.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(p) {
                    warn!("Failed to remove {}: {}", path, e);
                } else {
                    info!("Removed {}", path);
                }
            } else if let Err(e) = std::fs::remove_file(p) {
                warn!("Failed to remove {}: {}", path, e);
            } else {
                info!("Removed {}", path);
            }
        }
    }
}

/// Install Ollama using the system package manager
/// v0.0.820: Clean up manual install first, then install via pacman
pub async fn install() -> Result<()> {
    info!("Installing Ollama...");

    let pkg_manager = detect_package_manager();

    // v0.0.820: Clean up manual install files that conflict with pacman
    if pkg_manager == Some("pacman") {
        cleanup_manual_ollama_install();
    }

    let result = match pkg_manager {
        Some("pacman") => {
            info!("Using pacman to install Ollama");
            // v0.0.824: Use --overwrite to handle any file conflicts
            Command::new("pacman")
                .args(["-S", "--noconfirm", "--overwrite", "*", "ollama"])
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
        .unwrap_or_else(|_| reqwest::Client::new());

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

/// Send a chat request to Ollama (default timeout)
pub async fn chat(model: &str, prompt: &str) -> Result<String> {
    chat_with_timeout(model, prompt, 120).await
}

/// Send a chat request to Ollama with explicit timeout and retry logic
/// v0.0.140: Added retry with exponential backoff for reliability
pub async fn chat_with_timeout(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
    const MAX_RETRIES: u32 = 2;
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            // Exponential backoff: 500ms, 1000ms
            let delay_ms = 500 * (1 << (attempt - 1));
            info!("LLM retry {} after {}ms delay", attempt, delay_ms);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        match chat_single_attempt(model, prompt, timeout_secs).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                warn!("LLM attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("LLM request failed after {} retries", MAX_RETRIES)))
}

/// Single LLM request attempt (public for use by ollama_streaming)
pub async fn chat_single_attempt(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
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

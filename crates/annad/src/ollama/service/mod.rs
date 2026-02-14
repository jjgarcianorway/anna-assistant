//! Ollama service management and model operations.

mod health;

pub use health::{
    ensure_gpu_acceleration, get_ollama_diagnostics, needs_gpu_variant_upgrade,
    upgrade_to_gpu_variant,
};

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tracing::{info, warn};

use super::hardware::{detect_hardware, GpuType};
use super::OLLAMA_API;

const ANNA_REGISTRY: &str = "/var/lib/anna/registry.json";

/// Registry of resources installed by Anna
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AnnaRegistry {
    pub models: Vec<String>,
    pub packages: Vec<String>,
}

impl AnnaRegistry {
    pub fn load() -> Self {
        std::fs::read_to_string(ANNA_REGISTRY)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let dir = Path::new(ANNA_REGISTRY)
            .parent()
            .ok_or_else(|| anyhow!("Invalid registry path: no parent directory"))?;
        std::fs::create_dir_all(dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(ANNA_REGISTRY, json)?;
        Ok(())
    }

    pub fn add_model(&mut self, model: &str) {
        if !self.models.contains(&model.to_string()) {
            self.models.push(model.to_string());
        }
    }

    pub fn add_package(&mut self, pkg: &str) {
        if !self.packages.contains(&pkg.to_string()) {
            self.packages.push(pkg.to_string());
        }
    }
}

/// Create an ollama command with required environment variables
pub(crate) fn ollama_cmd() -> Command {
    // Always set a full PATH so ollama can be found regardless of how the daemon was started
    let full_path = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    let mut cmd = Command::new("ollama");
    cmd.env("HOME", "/root");
    // Do NOT override OLLAMA_MODELS — use whatever the running service uses.
    // Overriding it causes models downloaded by the service to be invisible to the CLI
    // and forces re-download of models the user already has.
    cmd.env("PATH", full_path);
    cmd
}

/// Check if Ollama is installed
pub fn is_installed() -> bool {
    // Check known paths directly — avoids depending on `which` being in PATH
    if std::path::Path::new("/usr/bin/ollama").exists()
        || std::path::Path::new("/usr/local/bin/ollama").exists()
        || std::path::Path::new("/bin/ollama").exists()
    {
        return true;
    }
    // Fallback: try running it
    Command::new("ollama")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}


/// Install Ollama using pacman (Arch Linux).
/// Uses spawn_blocking so the download doesn't starve the tokio runtime.
pub async fn install() -> Result<()> {
    info!("Installing Ollama via pacman...");

    let hw = detect_hardware();
    let pkg = match hw.gpu_type {
        GpuType::NvidiaCuda => "ollama-cuda",
        GpuType::AmdRocm => "ollama-rocm",
        _ => "ollama",
    };

    let pkg_owned = pkg.to_string();
    let installed_pkg = tokio::task::spawn_blocking(move || -> Result<String> {
        let output = Command::new("/usr/bin/pacman")
            .args(["-S", "--noconfirm", "--needed", &pkg_owned])
            .output()
            .with_context(|| format!("Failed to run /usr/bin/pacman to install {}", pkg_owned))?;

        if output.status.success() {
            return Ok(pkg_owned);
        }

        // GPU variant not available — fall back to base ollama
        if pkg_owned != "ollama" {
            warn!("{} not available, trying base ollama", pkg_owned);
            let output = Command::new("/usr/bin/pacman")
                .args(["-S", "--noconfirm", "--needed", "ollama"])
                .output()
                .with_context(|| "Failed to run /usr/bin/pacman to install ollama")?;
            if output.status.success() {
                return Ok("ollama".to_string());
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("pacman failed to install ollama: {}", stderr.trim()));
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("pacman failed to install {}: {}", pkg_owned, stderr.trim()))
    })
    .await
    .with_context(|| "spawn_blocking for pacman panicked")??;

    let mut registry = AnnaRegistry::load();
    registry.add_package(&installed_pkg);
    registry.save()?;

    info!("Ollama installed successfully: {}", installed_pkg);
    Ok(())
}

/// Verify a model actually responds to a trivial prompt.
/// Returns Err if the model is not installed or ollama is not responding.
pub async fn test_model(model: &str) -> Result<()> {
    // 120s timeout: first /api/generate call loads model into GPU memory (60-120s cold start)
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": "hi",
        "stream": false,
        "options": { "num_predict": 1 }
    });

    let response = client
        .post(format!("{}/api/generate", super::OLLAMA_API))
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("Ollama unreachable: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Model {} not available ({})", model, response.status()))
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

    let output = tokio::task::spawn_blocking(|| {
        Command::new("/usr/bin/systemctl")
            .args(["start", "ollama"])
            .output()
            .with_context(|| "Failed to run /usr/bin/systemctl start ollama")
    })
    .await
    .with_context(|| "spawn_blocking panicked for systemctl start")??;

    if output.status.success() {
        for _ in 0..30 {
            if is_running().await {
                info!("Ollama service started");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    warn!("systemctl failed, trying direct start");
    let _child = ollama_cmd().arg("serve").spawn()
        .with_context(|| "Failed to spawn ollama serve — is ollama installed?")?;

    for _ in 0..30 {
        if is_running().await {
            info!("Ollama started directly");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Err(anyhow!("Failed to start Ollama service"))
}

/// List available models
pub async fn list_models() -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client
        .get(format!("{}/api/tags", OLLAMA_API))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to list models: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await?;
    let models = json
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

/// Pull a model and register it
pub async fn pull_model(model: &str) -> Result<()> {
    info!("Pulling model: {} (this may take several minutes)", model);

    let model = model.to_string();
    let model_clone = model.clone();

    let result = tokio::task::spawn_blocking(move || {
        ollama_cmd().args(["pull", &model]).output()
    })
    .await?;

    match result {
        Ok(output) if output.status.success() => {
            info!("Model pulled successfully: {}", model_clone);
            let mut registry = AnnaRegistry::load();
            registry.add_model(&model_clone);
            registry.save()?;
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to pull model: {}", stderr))
        }
        Err(e) => Err(anyhow!("Failed to run ollama pull: {}", e)),
    }
}

/// Delete a model
pub async fn delete_model(model: &str) -> Result<()> {
    info!("Deleting model: {}", model);

    let model = model.to_string();
    let model_clone = model.clone();
    let result = tokio::task::spawn_blocking(move || {
        ollama_cmd().args(["rm", &model]).output()
    })
    .await?;

    match result {
        Ok(output) if output.status.success() => {
            info!("Model deleted: {}", model_clone);
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Failed to delete model: {}", stderr))
        }
        Err(e) => Err(anyhow!("Failed to run ollama rm: {}", e)),
    }
}

/// Clean up all Anna-installed resources
pub async fn cleanup_anna_resources() -> Result<()> {
    info!("Cleaning up Anna-installed resources...");

    let registry = AnnaRegistry::load();

    for model in &registry.models {
        if let Err(e) = delete_model(model).await {
            warn!("Failed to delete model {}: {}", model, e);
        }
    }

    if !registry.packages.is_empty() {
        info!(
            "Anna installed these packages (not auto-removing): {:?}",
            registry.packages
        );
    }

    std::fs::remove_file(ANNA_REGISTRY).ok();
    std::fs::remove_dir("/var/lib/anna/models").ok();
    std::fs::remove_dir("/var/lib/anna").ok();

    Ok(())
}

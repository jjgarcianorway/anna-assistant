//! Ollama service management and model operations.

use anyhow::{anyhow, Result};
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
fn ollama_cmd() -> Command {
    let mut cmd = Command::new("ollama");
    cmd.env("HOME", "/root");
    cmd.env("OLLAMA_MODELS", "/var/lib/anna/models");
    cmd
}

/// Check if Ollama is installed
pub fn is_installed() -> bool {
    Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install Ollama using pacman (Arch Linux)
pub async fn install() -> Result<()> {
    info!("Installing Ollama via pacman...");

    let hw = detect_hardware();
    let packages = match hw.gpu_type {
        GpuType::NvidiaCuda => vec!["ollama-cuda"],
        GpuType::AmdRocm => vec!["ollama-rocm"],
        _ => vec!["ollama"],
    };

    let mut registry = AnnaRegistry::load();

    for pkg in &packages {
        let output = Command::new("pacman")
            .args(["-S", "--noconfirm", "--needed", pkg])
            .output()?;

        if output.status.success() {
            info!("Installed package: {}", pkg);
            registry.add_package(pkg);
        } else {
            if *pkg != "ollama" {
                warn!("{} not available, trying base ollama", pkg);
                let output = Command::new("pacman")
                    .args(["-S", "--noconfirm", "--needed", "ollama"])
                    .output()?;
                if output.status.success() {
                    registry.add_package("ollama");
                }
            }
        }
    }

    registry.save()?;
    info!("Ollama installed successfully");
    Ok(())
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

/// Get detailed diagnostics when Ollama is not working
pub fn get_ollama_diagnostics() -> Vec<String> {
    let mut diagnostics = Vec::new();

    let ollama_exists = Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !ollama_exists {
        diagnostics
            .push("Ollama not installed. Install with: sudo pacman -S ollama".to_string());
        return diagnostics;
    }

    let process_running = Command::new("pgrep")
        .arg("-x")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !process_running {
        diagnostics.push("Ollama process not running".to_string());
        diagnostics.push("Start with: sudo systemctl start ollama".to_string());
    }

    let port_check = Command::new("ss")
        .args(["-tlnp"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(":11434"))
        .unwrap_or(false);

    if !port_check && process_running {
        diagnostics.push("Port 11434 not listening - Ollama may be starting up".to_string());
    }

    let service_status = Command::new("systemctl")
        .args(["is-active", "ollama"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    if service_status != "active" {
        diagnostics.push(format!("Ollama service status: {}", service_status));
        if service_status == "failed" {
            diagnostics.push("Check logs: journalctl -u ollama -n 20".to_string());
        }
    }

    let disk_check = Command::new("df")
        .args(["-h", "/usr/share/ollama"])
        .output()
        .ok()
        .and_then(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.lines().nth(1).and_then(|line| {
                line.split_whitespace().nth(4).map(|s| s.to_string())
            })
        });

    if let Some(usage) = disk_check {
        if let Ok(pct) = usage.trim_end_matches('%').parse::<u32>() {
            if pct > 95 {
                diagnostics.push(format!(
                    "Low disk space ({}% used) - may affect model loading",
                    pct
                ));
            }
        }
    }

    if diagnostics.is_empty() {
        diagnostics
            .push("Ollama appears configured correctly but API not responding".to_string());
        diagnostics.push("Try restarting: sudo systemctl restart ollama".to_string());
    }

    diagnostics
}

/// Start Ollama service
pub async fn start_service() -> Result<()> {
    info!("Starting Ollama service...");

    let output = Command::new("systemctl")
        .args(["start", "ollama"])
        .output()?;

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
        let mut cmd = std::process::Command::new("ollama");
        cmd.env("HOME", "/root");
        cmd.env("OLLAMA_MODELS", "/var/lib/anna/models");
        cmd.args(["pull", &model]);
        cmd.output()
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
        let mut cmd = std::process::Command::new("ollama");
        cmd.env("HOME", "/root");
        cmd.env("OLLAMA_MODELS", "/var/lib/anna/models");
        cmd.args(["rm", &model]);
        cmd.output()
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

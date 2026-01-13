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

/// v0.0.999: Check if we have the right ollama variant for our GPU
/// Returns true if ollama-cuda/ollama-rocm is needed but not installed
pub fn needs_gpu_variant_upgrade() -> Option<&'static str> {
    let hw = detect_hardware();

    let needed_pkg = match hw.gpu_type {
        GpuType::NvidiaCuda => "ollama-cuda",
        GpuType::AmdRocm => "ollama-rocm",
        _ => return None, // CPU-only doesn't need upgrade
    };

    // Check if the GPU variant is already installed
    let output = Command::new("pacman")
        .args(["-Q", needed_pkg])
        .output()
        .ok()?;

    if output.status.success() {
        None // Already have the right package
    } else {
        Some(needed_pkg)
    }
}

/// v0.0.999: Upgrade to GPU-accelerated ollama variant if needed
pub async fn upgrade_to_gpu_variant() -> Result<bool> {
    let Some(needed_pkg) = needs_gpu_variant_upgrade() else {
        return Ok(false); // No upgrade needed
    };

    info!("Upgrading to {} for GPU acceleration...", needed_pkg);

    // Stop ollama service first
    let _ = Command::new("systemctl")
        .args(["stop", "ollama"])
        .output();

    // Install the GPU variant (will replace base ollama)
    let output = Command::new("pacman")
        .args(["-S", "--noconfirm", needed_pkg])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("Failed to install {}: {}", needed_pkg, stderr);
        // Try to restart ollama anyway
        let _ = Command::new("systemctl")
            .args(["start", "ollama"])
            .output();
        return Err(anyhow!("Failed to install {}: {}", needed_pkg, stderr));
    }

    let mut registry = AnnaRegistry::load();
    registry.add_package(needed_pkg);
    registry.save()?;

    // Start ollama service
    let _ = Command::new("systemctl")
        .args(["start", "ollama"])
        .output();

    // Wait for it to be ready
    for _ in 0..30 {
        if is_running().await {
            info!("{} installed and running", needed_pkg);
            return Ok(true);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!("{} installed, but service not responding yet", needed_pkg);
    Ok(true)
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
/// v0.3.36: Removed manual command suggestions - Anna handles recovery automatically
pub fn get_ollama_diagnostics() -> Vec<String> {
    let mut diagnostics = Vec::new();

    let ollama_exists = Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !ollama_exists {
        diagnostics.push("Ollama is not installed".to_string());
        return diagnostics;
    }

    let process_running = Command::new("pgrep")
        .arg("-x")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !process_running {
        diagnostics.push("Ollama process is not running".to_string());
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
        diagnostics.push(format!("Ollama systemd service status: {}", service_status));
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
        diagnostics.push("Ollama appears configured correctly but API is not responding".to_string());
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

/// v0.0.999: Check if Ollama is using GPU and restart if not
pub async fn ensure_gpu_acceleration() -> Result<bool> {
    let hw = detect_hardware();

    // Only check for GPU systems
    if matches!(hw.gpu_type, GpuType::CpuOnly) {
        return Ok(true); // CPU-only is expected
    }

    // Check if any model is loaded
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .get(format!("{}/api/ps", OLLAMA_API))
        .send()
        .await;

    let Ok(response) = response else {
        return Ok(true); // Can't check, assume OK
    };

    let json: serde_json::Value = response.json().await.unwrap_or_default();
    let models = json.get("models").and_then(|m| m.as_array());

    if let Some(models) = models {
        for model in models {
            let size_vram = model.get("size_vram").and_then(|v| v.as_u64()).unwrap_or(0);
            let size = model.get("size").and_then(|v| v.as_u64()).unwrap_or(1);

            // If less than 10% of model is in VRAM on a GPU system, something is wrong
            if size_vram == 0 || (size_vram as f64 / size as f64) < 0.1 {
                warn!("Model not using GPU (VRAM: {}MB, Size: {}MB). Attempting restart...",
                      size_vram / 1024 / 1024, size / 1024 / 1024);

                // Try to restart ollama service
                if let Err(e) = restart_ollama_service().await {
                    warn!("Failed to restart ollama: {}", e);
                    return Ok(false);
                }

                info!("Ollama restarted - GPU should now be active");
                return Ok(true);
            }
        }
    }

    Ok(true)
}

/// Try to restart Ollama service to fix GPU issues
/// v0.0.999: NEVER use sudo - it triggers pam_faillock and locks out the user!
/// v0.3.36: Updated error messages to not include manual commands
async fn restart_ollama_service() -> Result<()> {
    info!("Attempting to restart Ollama for GPU acceleration...");

    // v0.0.999: DO NOT attempt systemctl restart - it requires sudo and failed attempts
    // will trigger pam_faillock, locking out the user's account!
    // Instead, we try to work with what we have or fall back gracefully.

    // Try to kill only the ollama runner (model process), not the main serve
    // The runner might reload with GPU support
    let _ = Command::new("pkill").args(["-f", "ollama runner"]).output();
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check if ollama is still running
    if is_running().await {
        info!("Ollama runner killed, will reload on next request");
        return Ok(());
    }

    // If main ollama died too, try starting as user process
    warn!("Ollama not running, starting as user process...");
    let _child = ollama_cmd().arg("serve").spawn()?;

    // Wait for it to be ready
    for _ in 0..30 {
        if is_running().await {
            info!("Ollama started as user process");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // If we still can't get GPU working, continue without GPU
    warn!("GPU acceleration unavailable - continuing with CPU mode");
    Err(anyhow!("GPU acceleration could not be enabled - running in CPU mode"))
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

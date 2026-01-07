//! Ollama management - install, run, and interact with Ollama.
//!
//! Includes hardware detection and automatic model selection.

use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tracing::{info, warn};

const OLLAMA_API: &str = "http://127.0.0.1:11434";
const ANNA_REGISTRY: &str = "/var/lib/anna/registry.json";

/// Hardware capabilities detected on the system
#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub gpu_type: GpuType,
    pub vram_mb: u64,
    pub ram_gb: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuType {
    NvidiaCuda,
    AmdRocm,
    IntelArc,
    CpuOnly,
}

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
        let dir = Path::new(ANNA_REGISTRY).parent().unwrap();
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

/// Detect hardware capabilities
pub fn detect_hardware() -> HardwareInfo {
    let (gpu_type, vram_mb) = detect_gpu();
    let ram_gb = detect_ram();

    info!(
        "Hardware detected: {:?}, VRAM: {}MB, RAM: {}GB",
        gpu_type, vram_mb, ram_gb
    );

    HardwareInfo {
        gpu_type,
        vram_mb,
        ram_gb,
    }
}

fn detect_gpu() -> (GpuType, u64) {
    // Try NVIDIA first
    if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
        .output()
    {
        if output.status.success() {
            let vram_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(vram) = vram_str.trim().parse::<u64>() {
                return (GpuType::NvidiaCuda, vram);
            }
        }
    }

    // Try AMD ROCm
    if let Ok(output) = Command::new("rocm-smi")
        .args(["--showmeminfo", "vram"])
        .output()
    {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Parse ROCm output for VRAM (format varies)
            for line in output_str.lines() {
                if line.contains("Total") {
                    if let Some(mb) = extract_mb_from_line(line) {
                        return (GpuType::AmdRocm, mb);
                    }
                }
            }
        }
    }

    // Try Intel Arc via lspci
    if let Ok(output) = Command::new("lspci").output() {
        let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if output_str.contains("intel") && output_str.contains("arc") {
            // Intel Arc typically has 8-16GB, estimate 8GB
            return (GpuType::IntelArc, 8192);
        }
    }

    (GpuType::CpuOnly, 0)
}

fn extract_mb_from_line(line: &str) -> Option<u64> {
    // Try to find a number followed by MB or MiB
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if part.to_lowercase().contains("mb") || part.to_lowercase().contains("mib") {
            if i > 0 {
                if let Ok(val) = parts[i - 1].parse::<u64>() {
                    return Some(val);
                }
            }
        }
        // Also try parsing the part itself if it's just a number
        if let Ok(val) = part.replace(",", "").parse::<u64>() {
            if val > 1000 {
                // Likely MB value
                return Some(val);
            }
        }
    }
    None
}

fn detect_ram() -> u64 {
    if let Ok(output) = Command::new("free").args(["-g"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(gb) = parts[1].parse::<u64>() {
                        return gb;
                    }
                }
            }
        }
    }
    8 // Default fallback
}

/// Select the best model based on hardware
pub fn select_best_model(hw: &HardwareInfo) -> &'static str {
    // Model selection based on available VRAM
    // Larger models = better reasoning but need more VRAM

    match hw.gpu_type {
        GpuType::NvidiaCuda | GpuType::AmdRocm | GpuType::IntelArc => {
            if hw.vram_mb >= 16000 {
                // 16GB+ VRAM: Can run 14B+ models
                "qwen2.5:14b"
            } else if hw.vram_mb >= 10000 {
                // 10-16GB VRAM: 7-8B models comfortably
                "qwen2.5:7b"
            } else if hw.vram_mb >= 6000 {
                // 6-10GB VRAM: 7B models with some offload
                "qwen2.5:7b"
            } else if hw.vram_mb >= 4000 {
                // 4-6GB VRAM: smaller models
                "qwen2.5:3b"
            } else {
                // < 4GB: tiny models
                "qwen2.5:1.5b"
            }
        }
        GpuType::CpuOnly => {
            // CPU-only: depends on RAM
            if hw.ram_gb >= 32 {
                "qwen2.5:7b"
            } else if hw.ram_gb >= 16 {
                "qwen2.5:3b"
            } else {
                "qwen2.5:1.5b"
            }
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

    // Check for CUDA support and install appropriate package
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
            // Try base ollama if cuda/rocm variant fails
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

/// Pull a model and register it (runs in blocking thread)
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

            // Register in Anna's registry
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

    // Delete models we installed
    for model in &registry.models {
        if let Err(e) = delete_model(model).await {
            warn!("Failed to delete model {}: {}", model, e);
        }
    }

    // Note: We don't auto-remove packages as user might want ollama
    // Just log what we installed
    if !registry.packages.is_empty() {
        info!("Anna installed these packages (not auto-removing): {:?}", registry.packages);
    }

    // Remove registry file
    std::fs::remove_file(ANNA_REGISTRY).ok();

    // Remove models directory if empty
    std::fs::remove_dir("/var/lib/anna/models").ok();
    std::fs::remove_dir("/var/lib/anna").ok();

    Ok(())
}

/// Send a chat request to Ollama with timeout and retry
pub async fn chat_with_timeout(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
    const MAX_RETRIES: u32 = 2;
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
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

    Err(last_error.unwrap_or_else(|| anyhow!("LLM request failed after retries")))
}

/// Single LLM request attempt
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

/// Streaming LLM request - writes tokens to the provided async writer
pub async fn chat_streaming_to_writer<W>(
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    writer: &mut W,
) -> Result<String>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    use anna_shared::rpc::StreamingResponse;
    use futures_util::StreamExt;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": true
    });

    let response = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Ollama request failed: {}", response.status()));
    }

    let mut full_response = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);

        // Ollama streams JSON objects, one per line
        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(token) = json.get("response").and_then(|r| r.as_str()) {
                    full_response.push_str(token);
                    // Send token to client
                    let response = StreamingResponse::Token { token: token.to_string() };
                    let json_str = serde_json::to_string(&response)?;
                    writer.write_all(format!("{}\n", json_str).as_bytes()).await?;
                    writer.flush().await?;
                }
            }
        }
    }

    Ok(full_response)
}

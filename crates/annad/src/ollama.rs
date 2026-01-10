//! Ollama management - install, run, and interact with Ollama.
//!
//! Includes hardware detection and automatic model selection.

use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use tracing::{info, warn, error};

const OLLAMA_API: &str = "http://127.0.0.1:11434";
const ANNA_REGISTRY: &str = "/var/lib/anna/registry.json";

/// Circuit breaker state for Ollama
/// Opens after consecutive failures, auto-resets after cooldown
/// v0.0.891: Using SeqCst ordering to prevent race conditions
static CONSECUTIVE_FAILURES: AtomicU32 = AtomicU32::new(0);
static CIRCUIT_OPENED_AT: AtomicU64 = AtomicU64::new(0);

/// Number of consecutive failures before circuit opens
const CIRCUIT_OPEN_THRESHOLD: u32 = 3;
/// Cooldown period in seconds before retrying after circuit opens
const CIRCUIT_COOLDOWN_SECS: u64 = 30;

/// Get current time as seconds since epoch
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Check if circuit breaker is open (should fail fast)
/// v0.0.891: Using SeqCst for proper synchronization
fn is_circuit_open() -> bool {
    let failures = CONSECUTIVE_FAILURES.load(Ordering::SeqCst);
    if failures >= CIRCUIT_OPEN_THRESHOLD {
        let opened_at = CIRCUIT_OPENED_AT.load(Ordering::SeqCst);
        let now = now_secs();

        // Check if cooldown has passed
        if now.saturating_sub(opened_at) < CIRCUIT_COOLDOWN_SECS {
            return true;
        }

        // Cooldown passed - allow a test request (half-open state)
        // Reset failure count to allow test request through
        // Use compare_exchange to avoid race with other threads
        if CONSECUTIVE_FAILURES.compare_exchange(
            failures,
            CIRCUIT_OPEN_THRESHOLD - 1, // Allow one test request
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).is_ok() {
            info!("Circuit breaker half-open, allowing test request");
        }
    }
    false
}

/// Record a successful request (closes circuit)
/// v0.0.891: Using SeqCst for proper synchronization
fn record_success() {
    let prev = CONSECUTIVE_FAILURES.swap(0, Ordering::SeqCst);
    if prev >= CIRCUIT_OPEN_THRESHOLD - 1 {
        info!("Circuit breaker closed after successful request");
    }
}

/// Record a failed request (may open circuit)
/// v0.0.891: Using SeqCst and atomic check for threshold
fn record_failure() {
    let failures = CONSECUTIVE_FAILURES.fetch_add(1, Ordering::SeqCst) + 1;
    if failures == CIRCUIT_OPEN_THRESHOLD {
        // Atomically set opened_at timestamp
        CIRCUIT_OPENED_AT.store(now_secs(), Ordering::SeqCst);
        error!("Circuit breaker OPEN - Ollama has {} consecutive failures, cooling down for {}s",
               failures, CIRCUIT_COOLDOWN_SECS);
    }
}

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
        // v0.0.891: Safe parent extraction instead of unwrap
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
/// v0.0.907: Considers GPU+RAM hybrid execution for larger models
/// Ollama automatically offloads layers to CPU when VRAM is insufficient
pub fn select_best_model(hw: &HardwareInfo) -> &'static str {
    // Model memory requirements (approximate, in GB):
    // qwen2.5:32b  ~20GB
    // qwen2.5:14b  ~9GB
    // qwen2.5:7b   ~5GB
    // qwen2.5:3b   ~2GB
    // qwen2.5:1.5b ~1GB

    // Total usable memory = VRAM + (RAM for offload)
    // With GPU: use VRAM fully + some RAM for offload (hybrid execution)
    // Ollama handles this automatically - we just need to ensure total fits

    // Use ceiling division to avoid edge cases (8188MB should be treated as 8GB, not 7)
    let vram_gb = (hw.vram_mb + 512) / 1024;  // Round to nearest GB
    let total_memory_gb = vram_gb + hw.ram_gb;

    match hw.gpu_type {
        GpuType::NvidiaCuda | GpuType::AmdRocm | GpuType::IntelArc => {
            // GPU available - can use hybrid execution
            // Prefer models that fit mostly in VRAM for speed, but allow offload

            if vram_gb >= 24 {
                // 24GB+ VRAM: Run 32B fully on GPU
                "qwen2.5:32b"
            } else if vram_gb >= 16 {
                // 16GB+ VRAM: 14B on GPU
                "qwen2.5:14b"
            } else if vram_gb >= 8 && hw.ram_gb >= 24 {
                // 8GB VRAM + 24GB+ RAM: Can run 14B with hybrid (GPU + CPU offload)
                // This is your case! RTX 4060 8GB + 32GB RAM
                "qwen2.5:14b"
            } else if vram_gb >= 6 && hw.ram_gb >= 16 {
                // 6GB VRAM + 16GB RAM: 7B hybrid comfortably
                "qwen2.5:7b"
            } else if vram_gb >= 4 {
                // 4-6GB VRAM: 7B with heavy offload or 3B
                if hw.ram_gb >= 16 {
                    "qwen2.5:7b"
                } else {
                    "qwen2.5:3b"
                }
            } else if vram_gb >= 2 {
                "qwen2.5:3b"
            } else {
                "qwen2.5:1.5b"
            }
        }
        GpuType::CpuOnly => {
            // CPU-only: depends entirely on RAM
            // Models run slower but still work
            if hw.ram_gb >= 48 {
                "qwen2.5:14b"  // Plenty of RAM for 14B on CPU
            } else if hw.ram_gb >= 32 {
                "qwen2.5:7b"
            } else if hw.ram_gb >= 16 {
                "qwen2.5:3b"
            } else if hw.ram_gb >= 8 {
                "qwen2.5:1.5b"
            } else {
                "qwen2.5:0.5b"
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

/// Get detailed diagnostics when Ollama is not working
pub fn get_ollama_diagnostics() -> Vec<String> {
    let mut diagnostics = Vec::new();

    // Check if ollama binary exists
    let ollama_exists = Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !ollama_exists {
        diagnostics.push("Ollama not installed. Install with: sudo pacman -S ollama".to_string());
        return diagnostics;
    }

    // Check if ollama process is running
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

    // Check if port 11434 is in use
    let port_check = Command::new("ss")
        .args(["-tlnp"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(":11434"))
        .unwrap_or(false);

    if !port_check && process_running {
        diagnostics.push("Port 11434 not listening - Ollama may be starting up".to_string());
    }

    // Check systemd service status
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

    // Check disk space for models
    let disk_check = Command::new("df")
        .args(["-h", "/usr/share/ollama"])
        .output()
        .ok()
        .and_then(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            // Parse usage percentage from df output
            output.lines().nth(1).and_then(|line| {
                line.split_whitespace().nth(4).map(|s| s.to_string())
            })
        });

    if let Some(usage) = disk_check {
        if let Ok(pct) = usage.trim_end_matches('%').parse::<u32>() {
            if pct > 95 {
                diagnostics.push(format!("Low disk space ({}% used) - may affect model loading", pct));
            }
        }
    }

    if diagnostics.is_empty() {
        diagnostics.push("Ollama appears configured correctly but API not responding".to_string());
        diagnostics.push("Try restarting: sudo systemctl restart ollama".to_string());
    }

    diagnostics
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
/// Uses circuit breaker to fail fast when Ollama is unhealthy
pub async fn chat_with_timeout(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
    // Check circuit breaker first
    if is_circuit_open() {
        return Err(anyhow!(
            "Circuit breaker OPEN - Ollama is unavailable (too many failures). \
             Waiting for cooldown before retrying."
        ));
    }

    const MAX_RETRIES: u32 = 2;
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay_ms = 500 * (1 << (attempt - 1));
            info!("LLM retry {} after {}ms delay", attempt, delay_ms);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        match chat_single_attempt(model, prompt, timeout_secs).await {
            Ok(response) => {
                record_success();
                return Ok(response);
            }
            Err(e) => {
                warn!("LLM attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
            }
        }
    }

    // All retries failed
    record_failure();
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
/// Uses circuit breaker to fail fast when Ollama is unhealthy
pub async fn chat_streaming_to_writer<W>(
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    writer: &mut W,
) -> Result<String>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    // Use the validating version with empty command output (no validation)
    chat_streaming_validated(model, prompt, timeout_secs, "", writer).await
}

/// Streaming LLM request with validation (v0.0.889)
/// Validates the answer against command output as it streams
pub async fn chat_streaming_validated<W>(
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    command_output: &str,
    writer: &mut W,
) -> Result<String>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    use anna_shared::rpc::StreamingResponse;
    use crate::validation::StreamingValidator;
    use futures_util::StreamExt;

    // Check circuit breaker first
    if is_circuit_open() {
        return Err(anyhow!(
            "Circuit breaker OPEN - Ollama is unavailable (too many failures). \
             Waiting for cooldown before retrying."
        ));
    }

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
        record_failure();
        return Err(anyhow!("Ollama request failed: {}", response.status()));
    }

    let mut full_response = String::new();
    let mut stream = response.bytes_stream();

    // Create validator if we have command output to validate against
    let mut validator = if !command_output.is_empty() {
        Some(StreamingValidator::new(command_output))
    } else {
        None
    };

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                record_failure();
                return Err(anyhow!("Stream error: {}", e));
            }
        };
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

                    // Validate and send any warnings (v0.0.889)
                    if let Some(ref mut v) = validator {
                        let warnings = v.add_token(token);
                        for warning in warnings {
                            let warning_response = StreamingResponse::Validation { warning };
                            let warning_json = serde_json::to_string(&warning_response)?;
                            writer.write_all(format!("{}\n", warning_json).as_bytes()).await?;
                            writer.flush().await?;
                        }
                    }
                }
            }
        }
    }

    record_success();
    Ok(full_response)
}

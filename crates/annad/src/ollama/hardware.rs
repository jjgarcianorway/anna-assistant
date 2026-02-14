//! Hardware detection and model selection for Ollama.

use std::process::Command;
use tracing::info;

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
            return (GpuType::IntelArc, 8192);
        }
    }

    (GpuType::CpuOnly, 0)
}

fn extract_mb_from_line(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        if part.to_lowercase().contains("mb") || part.to_lowercase().contains("mib") {
            if i > 0 {
                if let Ok(val) = parts[i - 1].parse::<u64>() {
                    return Some(val);
                }
            }
        }
        if let Ok(val) = part.replace(",", "").parse::<u64>() {
            if val > 1000 {
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

/// Approximate parameter count in billions for a model name string.
/// Used to rank installed models by size.
fn model_params_b(name: &str) -> f32 {
    let lower = name.to_lowercase();
    for tag in &["72b","70b","32b","30b","27b","22b","14b","13b","12b","11b","8b","7b","6b","4b","3.8b","3b","2b","1.5b","1b","0.5b"] {
        if lower.contains(tag) {
            return tag.trim_end_matches('b').parse().unwrap_or(0.0);
        }
    }
    // Unknown size — treat as medium
    7.0
}

/// Maximum safe parameter count for CPU-only inference with <15s response time.
/// Even on a fast CPU, a 7b model takes ~60s. 3b is the practical limit.
fn cpu_max_params_b() -> f32 { 3.0 }

/// Maximum safe parameter count given available VRAM (in MB).
fn gpu_max_params_b(vram_mb: u64) -> f32 {
    let vram_gb = (vram_mb + 512) / 1024;
    match vram_gb {
        0..=1  => 1.5,
        2..=3  => 3.0,
        4..=5  => 7.0,
        6..=7  => 7.0,
        8..=11 => 8.0,
        12..=23 => 14.0,
        _ => 32.0,
    }
}

/// Returns true if this model is an embedding/reranking model — not usable for chat.
fn is_embedding_model(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("embed") || lower.contains("minilm") || lower.contains("rerank")
        || lower.contains("bge-") || lower.contains("e5-") || lower.contains("gte-")
}

/// Select the best already-installed model that fits in available memory.
/// Prefers larger (better quality) models within the hardware limit.
/// Falls back to `select_best_model` target if nothing installed matches.
pub fn select_from_installed(hw: &HardwareInfo, installed: &[String]) -> String {
    let max_params = match hw.gpu_type {
        GpuType::CpuOnly => cpu_max_params_b(),
        _ => gpu_max_params_b(hw.vram_mb),
    };

    // Filter out embedding models and models too large for hardware
    let mut candidates: Vec<(&String, f32)> = installed.iter()
        .filter(|m| !is_embedding_model(m))
        .map(|m| (m, model_params_b(m)))
        .filter(|(_, p)| *p > 0.0 && *p <= max_params)
        .collect();
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if let Some((best, params)) = candidates.first() {
        info!("Selected installed model {} ({:.1}B) within {:.1}B limit", best, params, max_params);
        return best.to_string();
    }

    // Nothing installed fits — return the ideal target to download
    let fallback = select_best_model(hw);
    info!("No suitable installed model found, will download {}", fallback);
    fallback.to_string()
}

/// Ideal model target based on hardware (may need to be downloaded).
/// NOTE: For CPU-only, capped at 3b — larger models exceed response time limits.
pub fn select_best_model(hw: &HardwareInfo) -> &'static str {
    let vram_gb = (hw.vram_mb + 512) / 1024;

    match hw.gpu_type {
        GpuType::NvidiaCuda | GpuType::AmdRocm | GpuType::IntelArc => {
            if vram_gb >= 24 { "qwen2.5:32b" }
            else if vram_gb >= 12 { "qwen2.5:14b" }
            else if vram_gb >= 8 { "qwen2.5:7b" }
            else if vram_gb >= 4 { "qwen2.5:7b" }
            else if vram_gb >= 2 { "qwen2.5:3b" }
            else { "qwen2.5:1.5b" }
        }
        // CPU-only: hard cap at 3b — 7b+ causes constant timeouts on any CPU
        GpuType::CpuOnly => {
            if hw.ram_gb >= 16 { "qwen2.5:3b" }
            else if hw.ram_gb >= 8 { "qwen2.5:1.5b" }
            else { "qwen2.5:0.5b" }
        }
    }
}

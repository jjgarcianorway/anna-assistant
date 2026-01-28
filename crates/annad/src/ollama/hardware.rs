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

/// Select the best model based on hardware
/// v0.0.907: Considers GPU+RAM hybrid execution for larger models
pub fn select_best_model(hw: &HardwareInfo) -> &'static str {
    let vram_gb = (hw.vram_mb + 512) / 1024;
    let _total_memory_gb = vram_gb + hw.ram_gb;

    match hw.gpu_type {
        GpuType::NvidiaCuda | GpuType::AmdRocm | GpuType::IntelArc => {
            if vram_gb >= 24 {
                "qwen2.5:32b"
            } else if vram_gb >= 12 {
                "qwen2.5:14b"
            } else if vram_gb >= 8 {
                "qwen2.5:7b"
            } else if vram_gb >= 6 && hw.ram_gb >= 16 {
                "qwen2.5:7b"
            } else if vram_gb >= 4 {
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
            if hw.ram_gb >= 48 {
                "qwen2.5:14b"
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

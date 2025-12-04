//! Hardware probing for model selection.

use anna_shared::status::{GpuInfo, HardwareInfo};
use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Probe system hardware
pub fn probe_hardware() -> Result<HardwareInfo> {
    let cpu_cores = num_cpus();
    let cpu_model = cpu_model_name();
    let ram_bytes = total_ram();
    let gpu = detect_gpu();

    let info = HardwareInfo {
        cpu_cores,
        cpu_model,
        ram_bytes,
        gpu,
    };

    info!(
        "Hardware: {} cores, {} RAM, GPU: {}",
        info.cpu_cores,
        format_bytes(info.ram_bytes),
        info.gpu.as_ref().map(|g| g.model.as_str()).unwrap_or("none")
    );

    Ok(info)
}

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|p| p.get() as u32)
        .unwrap_or(1)
}

fn cpu_model_name() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn total_ram() -> u64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("MemTotal"))
                .and_then(|line| {
                    line.split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse::<u64>().ok())
                })
                .map(|kb| kb * 1024) // Convert to bytes
        })
        .unwrap_or(0)
}

fn detect_gpu() -> Option<GpuInfo> {
    // Try nvidia-smi first
    if let Some(gpu) = detect_nvidia_gpu() {
        return Some(gpu);
    }

    // Try AMD ROCm
    if let Some(gpu) = detect_amd_gpu() {
        return Some(gpu);
    }

    None
}

fn detect_nvidia_gpu() -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() >= 2 {
        let model = parts[0].to_string();
        let vram_mb: u64 = parts[1].parse().unwrap_or(0);

        info!("Detected NVIDIA GPU: {} with {} VRAM", model, format_bytes(vram_mb * 1024 * 1024));

        Some(GpuInfo {
            vendor: "NVIDIA".to_string(),
            model,
            vram_bytes: vram_mb * 1024 * 1024,
        })
    } else {
        None
    }
}

fn detect_amd_gpu() -> Option<GpuInfo> {
    let output = Command::new("rocm-smi")
        .args(["--showproductname", "--showmeminfo", "vram"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Simplified AMD detection - actual parsing would need more work
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("GPU") {
        warn!("AMD GPU detected but detailed parsing not implemented");
        Some(GpuInfo {
            vendor: "AMD".to_string(),
            model: "Unknown AMD GPU".to_string(),
            vram_bytes: 0,
        })
    } else {
        None
    }
}

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Select appropriate model based on hardware
pub fn select_model(hardware: &HardwareInfo) -> String {
    let ram_gb = hardware.ram_bytes / (1024 * 1024 * 1024);
    let has_gpu = hardware.gpu.is_some();
    let vram_gb = hardware
        .gpu
        .as_ref()
        .map(|g| g.vram_bytes / (1024 * 1024 * 1024))
        .unwrap_or(0);

    // Model selection logic based on available resources
    let model = if has_gpu && vram_gb >= 8 {
        // Good GPU - can run larger models
        "llama3.2:3b"
    } else if ram_gb >= 16 {
        // Plenty of RAM for CPU inference
        "llama3.2:3b"
    } else if ram_gb >= 8 {
        // Moderate RAM
        "llama3.2:1b"
    } else {
        // Limited resources - smallest model
        "qwen2.5:0.5b"
    };

    info!(
        "Selected model: {} (RAM: {} GB, GPU VRAM: {} GB)",
        model, ram_gb, vram_gb
    );

    model.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_bytes(512 * 1024 * 1024), "512.0 MB");
    }

    #[test]
    fn test_select_model_low_ram() {
        let hw = HardwareInfo {
            cpu_cores: 4,
            cpu_model: "Test".to_string(),
            ram_bytes: 4 * 1024 * 1024 * 1024,
            gpu: None,
        };
        assert_eq!(select_model(&hw), "qwen2.5:0.5b");
    }

    #[test]
    fn test_select_model_good_ram() {
        let hw = HardwareInfo {
            cpu_cores: 8,
            cpu_model: "Test".to_string(),
            ram_bytes: 16 * 1024 * 1024 * 1024,
            gpu: None,
        };
        assert_eq!(select_model(&hw), "llama3.2:3b");
    }
}

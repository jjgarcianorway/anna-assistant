//! System Information - Deterministic system facts (NO LLM HALLUCINATION)
//!
//! v6.41.0: All system information must come from real sources:
//! - /proc filesystem
//! - lscpu, lspci, lsblk commands
//! - systemctl, free, df, du
//! - sysfs (/sys/class/drm, etc.)
//!
//! LLM must NEVER generate CPU info, VRAM, disk usage, processes, etc.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

// ============================================================================
// CPU Information
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
    pub architecture: String,
    pub vendor: String,
    pub frequency_mhz: Option<f64>,
}

/// Get CPU information from lscpu (deterministic)
pub fn get_cpu_info() -> Result<CpuInfo> {
    let output = Command::new("lscpu")
        .output()
        .context("Failed to execute lscpu")?;

    if !output.status.success() {
        anyhow::bail!("lscpu failed");
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut model = String::from("Unknown");
    let mut cores = 0u32;
    let mut threads = 0u32;
    let mut architecture = String::from("Unknown");
    let mut vendor = String::from("Unknown");
    let mut frequency_mhz: Option<f64> = None;

    for line in text.lines() {
        if line.starts_with("Model name:") {
            model = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("CPU(s):") {
            if let Some(val) = line.split(':').nth(1) {
                threads = val.trim().parse().unwrap_or(0);
            }
        } else if line.starts_with("Core(s) per socket:") {
            if let Some(val) = line.split(':').nth(1) {
                cores = val.trim().parse().unwrap_or(0);
            }
        } else if line.starts_with("Architecture:") {
            architecture = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("Vendor ID:") {
            vendor = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("CPU MHz:") || line.starts_with("CPU max MHz:") {
            if let Some(val) = line.split(':').nth(1) {
                if let Ok(freq) = val.trim().parse::<f64>() {
                    frequency_mhz = Some(freq);
                }
            }
        }
    }

    Ok(CpuInfo {
        model,
        cores,
        threads,
        architecture,
        vendor,
        frequency_mhz,
    })
}

// ============================================================================
// GPU Information
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub model: String,
    pub driver: Option<String>,
    pub vram_total_mb: Option<u64>,
    pub vram_used_mb: Option<u64>,
}

/// Get GPU information from lspci and sysfs (deterministic)
pub fn get_gpu_info() -> Result<Vec<GpuInfo>> {
    let output = Command::new("lspci")
        .output()
        .context("Failed to execute lspci")?;

    if !output.status.success() {
        anyhow::bail!("lspci failed");
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    for line in text.lines() {
        if line.contains("VGA compatible controller") || line.contains("3D controller") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                let model = parts[2].trim().to_string();
                let vendor = if model.contains("NVIDIA") {
                    "NVIDIA".to_string()
                } else if model.contains("AMD") || model.contains("ATI") {
                    "AMD".to_string()
                } else if model.contains("Intel") {
                    "Intel".to_string()
                } else {
                    "Unknown".to_string()
                };

                // Get VRAM info if available
                let (vram_total, vram_used) = get_vram_info(&vendor);

                gpus.push(GpuInfo {
                    vendor,
                    model,
                    driver: get_gpu_driver(),
                    vram_total_mb: vram_total,
                    vram_used_mb: vram_used,
                });
            }
        }
    }

    if gpus.is_empty() {
        anyhow::bail!("No GPU found");
    }

    Ok(gpus)
}

/// Get VRAM information (vendor-specific, deterministic)
fn get_vram_info(vendor: &str) -> (Option<u64>, Option<u64>) {
    match vendor {
        "NVIDIA" => get_nvidia_vram(),
        "AMD" => get_amd_vram(),
        "Intel" => (None, None), // Intel integrated graphics don't expose VRAM separately
        _ => (None, None),
    }
}

/// Get NVIDIA VRAM using nvidia-smi
fn get_nvidia_vram() -> (Option<u64>, Option<u64>) {
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=memory.total,memory.used", "--format=csv,noheader,nounits"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = text.lines().next() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() == 2 {
                    let total = parts[0].trim().parse::<u64>().ok();
                    let used = parts[1].trim().parse::<u64>().ok();
                    return (total, used);
                }
            }
        }
    }

    // Fallback: try sysfs (newer kernels)
    if let Ok(total) = fs::read_to_string("/sys/class/drm/card0/device/mem_info_vram_total") {
        if let Ok(total_bytes) = total.trim().parse::<u64>() {
            let total_mb = total_bytes / 1024 / 1024;
            let used_mb = fs::read_to_string("/sys/class/drm/card0/device/mem_info_vram_used")
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(|bytes| bytes / 1024 / 1024);
            return (Some(total_mb), used_mb);
        }
    }

    (None, None)
}

/// Get AMD VRAM from sysfs
fn get_amd_vram() -> (Option<u64>, Option<u64>) {
    // Try multiple card indices
    for card_idx in 0..4 {
        let total_path = format!("/sys/class/drm/card{}/device/mem_info_vram_total", card_idx);
        let used_path = format!("/sys/class/drm/card{}/device/mem_info_vram_used", card_idx);

        if let Ok(total) = fs::read_to_string(&total_path) {
            if let Ok(total_bytes) = total.trim().parse::<u64>() {
                let total_mb = total_bytes / 1024 / 1024;
                let used_mb = fs::read_to_string(&used_path)
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .map(|bytes| bytes / 1024 / 1024);
                return (Some(total_mb), used_mb);
            }
        }
    }

    (None, None)
}

/// Get GPU driver information
fn get_gpu_driver() -> Option<String> {
    // Check for NVIDIA driver
    if Command::new("nvidia-smi").output().is_ok() {
        return Some("nvidia".to_string());
    }

    // Check for AMD driver (amdgpu)
    if Path::new("/sys/module/amdgpu").exists() {
        return Some("amdgpu".to_string());
    }

    // Check for Intel driver (i915)
    if Path::new("/sys/module/i915").exists() {
        return Some("i915".to_string());
    }

    None
}

// ============================================================================
// Memory Information
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub available_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

/// Get memory information from /proc/meminfo (deterministic)
pub fn get_ram_info() -> Result<MemoryInfo> {
    let meminfo = fs::read_to_string("/proc/meminfo")
        .context("Failed to read /proc/meminfo")?;

    let mut total_kb = 0u64;
    let mut available_kb = 0u64;
    let mut swap_total_kb = 0u64;
    let mut swap_free_kb = 0u64;

    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = parse_meminfo_line(line);
        } else if line.starts_with("MemAvailable:") {
            available_kb = parse_meminfo_line(line);
        } else if line.starts_with("SwapTotal:") {
            swap_total_kb = parse_meminfo_line(line);
        } else if line.starts_with("SwapFree:") {
            swap_free_kb = parse_meminfo_line(line);
        }
    }

    let total_mb = total_kb / 1024;
    let available_mb = available_kb / 1024;
    let used_mb = total_mb.saturating_sub(available_mb);
    let swap_total_mb = swap_total_kb / 1024;
    let swap_used_mb = swap_total_mb.saturating_sub(swap_free_kb / 1024);

    Ok(MemoryInfo {
        total_mb,
        used_mb,
        available_mb,
        swap_total_mb,
        swap_used_mb,
    })
}

fn parse_meminfo_line(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// ============================================================================
// Disk Information
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub filesystem: String,
    pub mount_point: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub use_percent: u8,
}

/// Get disk usage from df (deterministic)
pub fn get_disk_usage() -> Result<Vec<DiskInfo>> {
    let output = Command::new("df")
        .args(&["-h", "--output=source,target,size,used,avail,pcent"])
        .output()
        .context("Failed to execute df")?;

    if !output.status.success() {
        anyhow::bail!("df failed");
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        if idx == 0 {
            continue; // Skip header
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let filesystem = parts[0].to_string();
            let mount_point = parts[1].to_string();

            // Skip pseudo-filesystems
            if filesystem.starts_with("tmpfs")
                || filesystem.starts_with("devtmpfs")
                || filesystem.starts_with("udev")
            {
                continue;
            }

            let total_gb = parse_size_to_gb(parts[2]);
            let used_gb = parse_size_to_gb(parts[3]);
            let available_gb = parse_size_to_gb(parts[4]);
            let use_percent = parts[5].trim_end_matches('%').parse().unwrap_or(0);

            disks.push(DiskInfo {
                filesystem,
                mount_point,
                total_gb,
                used_gb,
                available_gb,
                use_percent,
            });
        }
    }

    Ok(disks)
}

fn parse_size_to_gb(size_str: &str) -> f64 {
    let size_str = size_str.trim();
    if size_str.is_empty() || size_str == "-" {
        return 0.0;
    }

    let (num_str, unit) = if size_str.ends_with('T') {
        (size_str.trim_end_matches('T'), 1024.0)
    } else if size_str.ends_with('G') {
        (size_str.trim_end_matches('G'), 1.0)
    } else if size_str.ends_with('M') {
        (size_str.trim_end_matches('M'), 1.0 / 1024.0)
    } else if size_str.ends_with('K') {
        (size_str.trim_end_matches('K'), 1.0 / (1024.0 * 1024.0))
    } else {
        (size_str, 1.0 / (1024.0 * 1024.0 * 1024.0))
    };

    num_str.parse::<f64>().unwrap_or(0.0) * unit
}

/// Get disk usage for a specific path
pub fn get_disk_free(path: &str) -> Result<DiskInfo> {
    let output = Command::new("df")
        .args(&["-h", "--output=source,target,size,used,avail,pcent", path])
        .output()
        .context("Failed to execute df")?;

    if !output.status.success() {
        anyhow::bail!("df failed for path: {}", path);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text.lines().collect();

    if lines.len() < 2 {
        anyhow::bail!("df returned no data");
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() >= 6 {
        Ok(DiskInfo {
            filesystem: parts[0].to_string(),
            mount_point: parts[1].to_string(),
            total_gb: parse_size_to_gb(parts[2]),
            used_gb: parse_size_to_gb(parts[3]),
            available_gb: parse_size_to_gb(parts[4]),
            use_percent: parts[5].trim_end_matches('%').parse().unwrap_or(0),
        })
    } else {
        anyhow::bail!("Failed to parse df output");
    }
}

// ============================================================================
// Directory Size Analysis
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorySize {
    pub path: String,
    pub size_gb: f64,
}

/// Get biggest directories using du (deterministic)
pub fn get_biggest_dirs(path: &str, count: usize) -> Result<Vec<DirectorySize>> {
    let output = Command::new("du")
        .args(&["-xh", "--max-depth=1", path])
        .output()
        .context("Failed to execute du")?;

    if !output.status.success() {
        anyhow::bail!("du failed for path: {}", path);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut dirs = Vec::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let size_gb = parse_size_to_gb(parts[0]);
            let dir_path = parts[1..].join(" ");

            // Skip the total line (same as input path)
            if dir_path != path {
                dirs.push(DirectorySize {
                    path: dir_path,
                    size_gb,
                });
            }
        }
    }

    // Sort by size descending
    dirs.sort_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap_or(std::cmp::Ordering::Equal));

    // Take top N
    dirs.truncate(count);

    Ok(dirs)
}

// ============================================================================
// OS Information
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub kernel: String,
    pub architecture: String,
}

/// Get OS information from /etc/os-release and uname (deterministic)
pub fn get_os_info() -> Result<OsInfo> {
    let os_release = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .context("Failed to read os-release")?;

    let mut name = String::from("Unknown");
    let mut version = String::from("Unknown");

    for line in os_release.lines() {
        if line.starts_with("NAME=") {
            name = line.split('=').nth(1).unwrap_or("").trim_matches('"').to_string();
        } else if line.starts_with("VERSION=") {
            version = line.split('=').nth(1).unwrap_or("").trim_matches('"').to_string();
        }
    }

    let kernel = get_kernel_info()?;
    let architecture = std::env::consts::ARCH.to_string();

    Ok(OsInfo {
        name,
        version,
        kernel,
        architecture,
    })
}

/// Get kernel version from uname (deterministic)
pub fn get_kernel_info() -> Result<String> {
    let output = Command::new("uname")
        .arg("-r")
        .output()
        .context("Failed to execute uname")?;

    if !output.status.success() {
        anyhow::bail!("uname failed");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ============================================================================
// Process Information
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub mem_percent: f32,
}

/// Get top processes by CPU or memory (deterministic)
pub fn get_top_processes(count: usize, sort_by: &str) -> Result<Vec<ProcessInfo>> {
    let sort_arg = match sort_by {
        "cpu" => "-pcpu",
        "mem" => "-pmem",
        _ => "-pcpu",
    };

    let output = Command::new("ps")
        .args(&["aux", "--sort", sort_arg])
        .output()
        .context("Failed to execute ps")?;

    if !output.status.success() {
        anyhow::bail!("ps failed");
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for (idx, line) in text.lines().enumerate() {
        if idx == 0 {
            continue; // Skip header
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 11 {
            let pid = parts[1].parse().unwrap_or(0);
            let cpu_percent = parts[2].parse().unwrap_or(0.0);
            let mem_percent = parts[3].parse().unwrap_or(0.0);
            let name = parts[10].to_string();

            processes.push(ProcessInfo {
                pid,
                name,
                cpu_percent,
                mem_percent,
            });

            if processes.len() >= count {
                break;
            }
        }
    }

    Ok(processes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cpu_info() {
        let result = get_cpu_info();
        assert!(result.is_ok());
        let cpu = result.unwrap();
        assert!(!cpu.model.is_empty());
        assert!(cpu.threads > 0);
    }

    #[test]
    fn test_get_ram_info() {
        let result = get_ram_info();
        assert!(result.is_ok());
        let mem = result.unwrap();
        assert!(mem.total_mb > 0);
    }

    #[test]
    fn test_get_disk_usage() {
        let result = get_disk_usage();
        assert!(result.is_ok());
        let disks = result.unwrap();
        assert!(!disks.is_empty());
    }

    #[test]
    fn test_get_os_info() {
        let result = get_os_info();
        assert!(result.is_ok());
        let os = result.unwrap();
        assert!(!os.name.is_empty());
        assert!(!os.kernel.is_empty());
    }

    #[test]
    fn test_parse_size_to_gb() {
        assert_eq!(parse_size_to_gb("1T"), 1024.0);
        assert_eq!(parse_size_to_gb("1G"), 1.0);
        assert_eq!(parse_size_to_gb("1024M"), 1.0);
    }

    #[test]
    fn test_get_top_processes() {
        let result = get_top_processes(5, "cpu");
        assert!(result.is_ok());
        let procs = result.unwrap();
        assert!(procs.len() <= 5);
    }
}

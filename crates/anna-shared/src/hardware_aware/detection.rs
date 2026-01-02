//! Hardware detection utilities (v0.0.434).

use crate::hardware_aware::types::{CpuInfo, GpuInfo, GpuVendor, OsInfo, StorageInfo};
use std::path::Path;

/// Detect CPU information.
pub(super) fn detect_cpu() -> CpuInfo {
    let mut info = CpuInfo::default();

    // Read /proc/cpuinfo
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        let mut physical_ids = std::collections::HashSet::new();
        let mut processor_count = 0u32;

        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(val) = line.split(':').nth(1) {
                    info.model_name = val.trim().to_string();
                }
            } else if line.starts_with("processor") {
                processor_count += 1;
            } else if line.starts_with("physical id") {
                if let Some(val) = line.split(':').nth(1) {
                    physical_ids.insert(val.trim().to_string());
                }
            } else if line.starts_with("flags") {
                info.avx2_supported = line.contains(" avx2 ");
            } else if line.starts_with("cpu cores") {
                if let Some(val) = line.split(':').nth(1) {
                    if let Ok(cores) = val.trim().parse() {
                        info.core_count = cores;
                    }
                }
            }
        }

        info.thread_count = processor_count;
        if info.core_count == 1 && processor_count > 1 {
            // Fallback: estimate cores from threads
            info.core_count = processor_count / 2;
        }
    }

    info
}

/// Detect RAM information.
pub(super) fn detect_ram() -> (f32, f32) {
    let mut total_gb = 0.0f32;
    let mut free_gb = 0.0f32;

    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb) = extract_kb_value(line) {
                    total_gb = kb as f32 / 1024.0 / 1024.0;
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(kb) = extract_kb_value(line) {
                    free_gb = kb as f32 / 1024.0 / 1024.0;
                }
            }
        }
    }

    (total_gb, free_gb)
}

/// Detect GPU information.
pub(super) fn detect_gpu() -> GpuInfo {
    let mut info = GpuInfo::default();

    // Check for NVIDIA GPU
    if Path::new("/proc/driver/nvidia/version").exists() {
        info.discrete = true;
        info.vendor = GpuVendor::Nvidia;
        info.model_name = detect_nvidia_model();
        info.vram_gb = detect_nvidia_vram();
        return info;
    }

    // Check lspci output via /sys
    if let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") {
        for entry in entries.flatten() {
            let class_path = entry.path().join("class");
            if let Ok(class) = std::fs::read_to_string(&class_path) {
                // VGA compatible controller (0x030000) or 3D controller (0x030200)
                let class = class.trim();
                if class.starts_with("0x0302") || class.starts_with("0x0300") {
                    let vendor_path = entry.path().join("vendor");
                    if let Ok(vendor) = std::fs::read_to_string(&vendor_path) {
                        let vendor = vendor.trim();
                        match vendor {
                            "0x10de" => {
                                info.discrete = true;
                                info.vendor = GpuVendor::Nvidia;
                            }
                            "0x1002" => {
                                info.discrete = true;
                                info.vendor = GpuVendor::Amd;
                            }
                            "0x8086" => {
                                // Intel - could be integrated
                                if info.vendor == GpuVendor::None {
                                    info.vendor = GpuVendor::Intel;
                                }
                            }
                            _ => {
                                if info.vendor == GpuVendor::None {
                                    info.vendor = GpuVendor::Other;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info
}

/// Detect storage information.
pub(super) fn detect_storage() -> StorageInfo {
    let mut info = StorageInfo::default();

    // Check data dir
    info.data_dir_available_gb = get_available_space_gb(super::ANNA_DATA_DIR);

    // Check Ollama model storage (default: ~/.ollama/models)
    let ollama_dir = dirs::home_dir()
        .map(|h| h.join(".ollama").join("models"))
        .unwrap_or_default();

    if ollama_dir.exists() {
        info.model_storage_available_gb =
            get_available_space_gb(ollama_dir.to_str().unwrap_or("/"));
        info.model_storage_total_gb = get_total_space_gb(ollama_dir.to_str().unwrap_or("/"));
    } else {
        // Fall back to home directory
        if let Some(home) = dirs::home_dir() {
            info.model_storage_available_gb =
                get_available_space_gb(home.to_str().unwrap_or("/"));
            info.model_storage_total_gb = get_total_space_gb(home.to_str().unwrap_or("/"));
        }
    }

    info
}

/// Detect OS information.
pub(super) fn detect_os() -> OsInfo {
    let mut info = OsInfo::default();

    // Read /etc/os-release
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                info.distro = line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }

    // Read kernel version
    if let Ok(content) = std::fs::read_to_string("/proc/version") {
        if let Some(version) = content.split_whitespace().nth(2) {
            info.kernel_version = version.to_string();
        }
    }

    info
}

/// Extract kB value from /proc/meminfo line.
fn extract_kb_value(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1)?.parse().ok()
}

/// Detect NVIDIA model name from nvidia-smi or sysfs.
fn detect_nvidia_model() -> Option<String> {
    // Try sysfs first
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("card") {
                let device_path = entry.path().join("device/product_name");
                if let Ok(name) = std::fs::read_to_string(&device_path) {
                    return Some(name.trim().to_string());
                }
            }
        }
    }
    None
}

/// Detect NVIDIA VRAM from sysfs.
fn detect_nvidia_vram() -> Option<u32> {
    // Try to read from sysfs
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let mem_path = entry.path().join("device/mem_info_vram_total");
            if let Ok(content) = std::fs::read_to_string(&mem_path) {
                if let Ok(bytes) = content.trim().parse::<u64>() {
                    return Some((bytes / 1024 / 1024 / 1024) as u32);
                }
            }
        }
    }
    None
}

/// Get available space in GB for a path.
pub(super) fn get_available_space_gb(path: &str) -> u32 {
    use std::process::Command;

    let output = Command::new("df")
        .args(["--output=avail", "-B1", path])
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                if let Ok(bytes) = line.trim().parse::<u64>() {
                    return (bytes / 1024 / 1024 / 1024) as u32;
                }
            }
        }
    }
    0
}

/// Get total space in GB for a path.
pub(super) fn get_total_space_gb(path: &str) -> u32 {
    use std::process::Command;

    let output = Command::new("df")
        .args(["--output=size", "-B1", path])
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().nth(1) {
                if let Ok(bytes) = line.trim().parse::<u64>() {
                    return (bytes / 1024 / 1024 / 1024) as u32;
                }
            }
        }
    }
    0
}

/// Get current timestamp as ISO string.
pub(super) fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO-ish format without chrono dependency
    format!("{}", secs)
}

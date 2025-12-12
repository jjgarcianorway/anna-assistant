//! Hardware profiling and capability tiers (v0.0.434).
//!
//! Collects system information and maps to capability tiers.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Capability tier based on hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityTier {
    /// <= 8 GB RAM, low disk, no discrete GPU.
    Tiny,
    /// 8-16 GB RAM, decent disk.
    Small,
    /// 16-32 GB RAM or small GPU.
    Medium,
    /// > 32 GB RAM or strong GPU.
    Large,
}

impl CapabilityTier {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Tiny => "Tiny",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }

    /// Recommended max concurrent model memory in GB.
    pub fn max_model_memory_gb(&self) -> u32 {
        match self {
            Self::Tiny => 4,
            Self::Small => 8,
            Self::Medium => 16,
            Self::Large => 32,
        }
    }
}

impl Default for CapabilityTier {
    fn default() -> Self {
        Self::Small
    }
}

/// CPU information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// Model name (e.g., "Intel Core i9-14900HX").
    pub model_name: String,
    /// Physical core count.
    pub core_count: u32,
    /// Thread count (including hyperthreading).
    pub thread_count: u32,
    /// Whether AVX2 is supported.
    pub avx2_supported: bool,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            model_name: "Unknown".to_string(),
            core_count: 1,
            thread_count: 1,
            avx2_supported: false,
        }
    }
}

/// GPU vendor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
    None,
}

impl GpuVendor {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Nvidia => "NVIDIA",
            Self::Amd => "AMD",
            Self::Intel => "Intel",
            Self::Other => "Other",
            Self::None => "None",
        }
    }
}

/// GPU information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// Whether a discrete GPU is present.
    pub discrete: bool,
    /// GPU vendor.
    pub vendor: GpuVendor,
    /// GPU model name (if detected).
    pub model_name: Option<String>,
    /// Estimated VRAM in GB (if detectable).
    pub vram_gb: Option<u32>,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            discrete: false,
            vendor: GpuVendor::None,
            model_name: None,
            vram_gb: None,
        }
    }
}

/// Storage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Available space at data_dir in GB.
    pub data_dir_available_gb: u32,
    /// Available space at model storage in GB.
    pub model_storage_available_gb: u32,
    /// Total space at model storage in GB.
    pub model_storage_total_gb: u32,
}

impl Default for StorageInfo {
    fn default() -> Self {
        Self {
            data_dir_available_gb: 0,
            model_storage_available_gb: 0,
            model_storage_total_gb: 0,
        }
    }
}

/// OS information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    /// Distribution name (e.g., "Arch Linux").
    pub distro: String,
    /// Kernel version.
    pub kernel_version: String,
}

impl Default for OsInfo {
    fn default() -> Self {
        Self {
            distro: "Unknown".to_string(),
            kernel_version: "Unknown".to_string(),
        }
    }
}

/// Complete hardware profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// Profile version for migration.
    pub profile_version: u32,
    /// When this profile was created.
    pub last_profiled_at: String,
    /// CPU information.
    pub cpu: CpuInfo,
    /// Total RAM in GB.
    pub ram_total_gb: f32,
    /// Free RAM at profiling time in GB.
    pub ram_free_gb: f32,
    /// GPU information.
    pub gpu: GpuInfo,
    /// Storage information.
    pub storage: StorageInfo,
    /// OS information.
    pub os: OsInfo,
    /// Computed capability tier.
    pub tier: CapabilityTier,
}

impl HardwareProfile {
    /// Profile the current system.
    pub fn profile_system() -> Self {
        let cpu = Self::detect_cpu();
        let (ram_total_gb, ram_free_gb) = Self::detect_ram();
        let gpu = Self::detect_gpu();
        let storage = Self::detect_storage();
        let os = Self::detect_os();
        let tier = Self::compute_tier(ram_total_gb, &gpu);

        Self {
            profile_version: super::PROFILE_VERSION,
            last_profiled_at: chrono_now(),
            cpu,
            ram_total_gb,
            ram_free_gb,
            gpu,
            storage,
            os,
            tier,
        }
    }

    /// Detect CPU information.
    fn detect_cpu() -> CpuInfo {
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
    fn detect_ram() -> (f32, f32) {
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
    fn detect_gpu() -> GpuInfo {
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
    fn detect_storage() -> StorageInfo {
        let mut info = StorageInfo::default();

        // Check data dir
        info.data_dir_available_gb = get_available_space_gb(super::ANNA_DATA_DIR);

        // Check Ollama model storage (default: ~/.ollama/models)
        let ollama_dir = dirs::home_dir()
            .map(|h| h.join(".ollama").join("models"))
            .unwrap_or_default();

        if ollama_dir.exists() {
            info.model_storage_available_gb = get_available_space_gb(ollama_dir.to_str().unwrap_or("/"));
            info.model_storage_total_gb = get_total_space_gb(ollama_dir.to_str().unwrap_or("/"));
        } else {
            // Fall back to home directory
            if let Some(home) = dirs::home_dir() {
                info.model_storage_available_gb = get_available_space_gb(home.to_str().unwrap_or("/"));
                info.model_storage_total_gb = get_total_space_gb(home.to_str().unwrap_or("/"));
            }
        }

        info
    }

    /// Detect OS information.
    fn detect_os() -> OsInfo {
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

    /// Compute capability tier from hardware.
    fn compute_tier(ram_gb: f32, gpu: &GpuInfo) -> CapabilityTier {
        // GPU can bump tier but needs adequate RAM too
        let has_strong_gpu = gpu.discrete && matches!(gpu.vendor, GpuVendor::Nvidia | GpuVendor::Amd);
        let vram_ok = gpu.vram_gb.unwrap_or(0) >= 6;

        // Large: high RAM, or good RAM + strong GPU
        if ram_gb > 32.0 || (ram_gb > 16.0 && has_strong_gpu && vram_ok) {
            CapabilityTier::Large
        } else if ram_gb > 16.0 || (ram_gb > 8.0 && has_strong_gpu) {
            CapabilityTier::Medium
        } else if ram_gb > 8.0 {
            CapabilityTier::Small
        } else {
            CapabilityTier::Tiny
        }
    }

    /// Load from file.
    pub fn load(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    /// Check if reprofile is needed.
    pub fn needs_reprofile(&self) -> bool {
        self.profile_version < super::PROFILE_VERSION
    }

    /// Format for display.
    pub fn format_summary(&self) -> String {
        format!(
            "RAM: {:.1} GB | CPU: {} ({} cores) | GPU: {} | Tier: {}",
            self.ram_total_gb,
            self.cpu.model_name,
            self.cpu.core_count,
            self.gpu.vendor.label(),
            self.tier.label()
        )
    }
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
fn get_available_space_gb(path: &str) -> u32 {
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
fn get_total_space_gb(path: &str) -> u32 {
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
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO-ish format without chrono dependency
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_tier_labels() {
        assert_eq!(CapabilityTier::Tiny.label(), "Tiny");
        assert_eq!(CapabilityTier::Large.label(), "Large");
    }

    #[test]
    fn test_compute_tier() {
        let no_gpu = GpuInfo::default();

        assert_eq!(HardwareProfile::compute_tier(4.0, &no_gpu), CapabilityTier::Tiny);
        assert_eq!(HardwareProfile::compute_tier(12.0, &no_gpu), CapabilityTier::Small);
        assert_eq!(HardwareProfile::compute_tier(24.0, &no_gpu), CapabilityTier::Medium);
        assert_eq!(HardwareProfile::compute_tier(64.0, &no_gpu), CapabilityTier::Large);
    }

    #[test]
    fn test_gpu_bumps_tier() {
        let nvidia_gpu = GpuInfo {
            discrete: true,
            vendor: GpuVendor::Nvidia,
            model_name: Some("RTX 4060".to_string()),
            vram_gb: Some(8),
        };

        // 12GB RAM would be Small, but with NVIDIA GPU becomes Medium
        assert_eq!(HardwareProfile::compute_tier(12.0, &nvidia_gpu), CapabilityTier::Medium);

        // 20GB RAM with strong GPU becomes Large
        assert_eq!(HardwareProfile::compute_tier(20.0, &nvidia_gpu), CapabilityTier::Large);
    }

    #[test]
    fn test_profile_serialization() {
        let profile = HardwareProfile {
            profile_version: 1,
            last_profiled_at: "12345".to_string(),
            cpu: CpuInfo::default(),
            ram_total_gb: 16.0,
            ram_free_gb: 8.0,
            gpu: GpuInfo::default(),
            storage: StorageInfo::default(),
            os: OsInfo::default(),
            tier: CapabilityTier::Small,
        };

        let json = serde_json::to_string(&profile).unwrap();
        let restored: HardwareProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.ram_total_gb, 16.0);
        assert_eq!(restored.tier, CapabilityTier::Small);
    }
}

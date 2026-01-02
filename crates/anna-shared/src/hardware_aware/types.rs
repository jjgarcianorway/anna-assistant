//! Hardware profile types (v0.0.434).

use serde::{Deserialize, Serialize};

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

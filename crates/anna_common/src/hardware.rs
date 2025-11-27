//! Hardware and Driver Detection for Anna v0.5.0
//!
//! Detects hardware capabilities and driver status for model selection.
//! GPU with working drivers = GPU model. GPU without drivers = CPU model.

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// CPU model tiers for selection
pub const CPU_MODEL_SMALL: &str = "llama3.2:3b";
pub const CPU_MODEL_MEDIUM: &str = "qwen2.5:7b";

/// GPU model tiers for selection
pub const GPU_MODEL_MEDIUM: &str = "qwen2.5:14b";
pub const GPU_MODEL_LARGE: &str = "qwen2.5:32b";

/// Performance profile based on hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceProfile {
    Low,
    Medium,
    High,
}

/// GPU vendor detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpuVendor {
    None,
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

impl GpuVendor {
    pub fn as_str(&self) -> &'static str {
        match self {
            GpuVendor::None => "none",
            GpuVendor::Nvidia => "nvidia",
            GpuVendor::Amd => "amd",
            GpuVendor::Intel => "intel",
            GpuVendor::Unknown => "unknown",
        }
    }
}

/// Hardware detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu_vendor: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub ram_gb: u64,
    pub gpu_vendor: GpuVendor,
    pub gpu_name: Option<String>,
    pub vram_gb: Option<u64>,
    pub gpu_driver_loaded: bool,
    pub gpu_driver_functional: bool,
    pub performance_profile: PerformanceProfile,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            cpu_vendor: "unknown".to_string(),
            cpu_cores: 1,
            cpu_threads: 1,
            ram_gb: 4,
            gpu_vendor: GpuVendor::None,
            gpu_name: None,
            vram_gb: None,
            gpu_driver_loaded: false,
            gpu_driver_functional: false,
            performance_profile: PerformanceProfile::Low,
        }
    }
}

impl HardwareProfile {
    /// Detect current hardware profile
    pub fn detect() -> Self {
        let mut profile = Self::default();

        // Detect CPU
        profile.detect_cpu();

        // Detect RAM
        profile.detect_ram();

        // Detect GPU and drivers
        profile.detect_gpu();

        // Calculate performance profile
        profile.calculate_performance_profile();

        profile
    }

    fn detect_cpu(&mut self) {
        // Read /proc/cpuinfo
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            let mut cores = 0;
            let mut threads = 0;

            for line in content.lines() {
                if line.starts_with("vendor_id") {
                    if let Some(vendor) = line.split(':').nth(1) {
                        self.cpu_vendor = vendor.trim().to_string();
                    }
                }
                if line.starts_with("processor") {
                    threads += 1;
                }
                if line.starts_with("cpu cores") {
                    if let Some(c) = line.split(':').nth(1) {
                        cores = c.trim().parse().unwrap_or(1);
                    }
                }
            }

            self.cpu_threads = threads.max(1);
            self.cpu_cores = if cores > 0 { cores } else { threads };
        }
    }

    fn detect_ram(&mut self) {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            self.ram_gb = kb / 1024 / 1024;
                        }
                    }
                    break;
                }
            }
        }
    }

    fn detect_gpu(&mut self) {
        // Check for NVIDIA GPU using lspci
        if let Ok(output) = Command::new("lspci").output() {
            let lspci = String::from_utf8_lossy(&output.stdout);

            if lspci.contains("NVIDIA") {
                self.gpu_vendor = GpuVendor::Nvidia;
                // Extract GPU name
                for line in lspci.lines() {
                    if line.contains("NVIDIA") && line.contains("VGA") {
                        if let Some(name) = line.split(':').next_back() {
                            self.gpu_name = Some(name.trim().to_string());
                        }
                        break;
                    }
                }
                // Check if NVIDIA driver is loaded
                self.detect_nvidia_driver();
            } else if lspci.contains("AMD") && lspci.contains("VGA") {
                self.gpu_vendor = GpuVendor::Amd;
                for line in lspci.lines() {
                    if line.contains("AMD") && line.contains("VGA") {
                        if let Some(name) = line.split(':').next_back() {
                            self.gpu_name = Some(name.trim().to_string());
                        }
                        break;
                    }
                }
                self.detect_amd_driver();
            } else if lspci.contains("Intel") && lspci.contains("VGA") {
                self.gpu_vendor = GpuVendor::Intel;
                self.gpu_driver_loaded = true; // Intel usually works with kernel
                self.gpu_driver_functional = true;
            }
        }
    }

    fn detect_nvidia_driver(&mut self) {
        // Check if nvidia kernel module is loaded
        if let Ok(modules) = fs::read_to_string("/proc/modules") {
            self.gpu_driver_loaded = modules.contains("nvidia");
        }

        // Check if nvidia-smi works
        if let Ok(output) = Command::new("nvidia-smi")
            .arg("--query-gpu=memory.total")
            .arg("--format=csv,noheader,nounits")
            .output()
        {
            if output.status.success() {
                self.gpu_driver_functional = true;
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(mb_str) = stdout.lines().next() {
                    if let Ok(mb) = mb_str.trim().parse::<u64>() {
                        self.vram_gb = Some(mb / 1024);
                    }
                }
            }
        }
    }

    fn detect_amd_driver(&mut self) {
        // Check for amdgpu kernel module
        if let Ok(modules) = fs::read_to_string("/proc/modules") {
            self.gpu_driver_loaded = modules.contains("amdgpu");
        }

        // Check for ROCm via rocm-smi
        if let Ok(output) = Command::new("rocm-smi").arg("--showmeminfo").output() {
            if output.status.success() {
                self.gpu_driver_functional = true;
            }
        }
    }

    fn calculate_performance_profile(&mut self) {
        // Determine performance profile
        if self.gpu_driver_functional && self.vram_gb.unwrap_or(0) >= 8 {
            self.performance_profile = PerformanceProfile::High;
        } else if self.ram_gb >= 16 && self.cpu_cores >= 8 {
            self.performance_profile = PerformanceProfile::Medium;
        } else {
            self.performance_profile = PerformanceProfile::Low;
        }
    }

    /// Select the appropriate model based on hardware
    pub fn select_model(&self) -> ModelRecommendation {
        let (model, reason) = if self.gpu_driver_functional {
            // GPU with working drivers
            if self.vram_gb.unwrap_or(0) >= 16 && self.ram_gb >= 32 {
                (GPU_MODEL_LARGE, "GPU detected with sufficient VRAM")
            } else {
                (GPU_MODEL_MEDIUM, "GPU detected with moderate VRAM")
            }
        } else if self.gpu_vendor != GpuVendor::None && !self.gpu_driver_loaded {
            // GPU exists but driver not loaded - check CPU capability
            if self.ram_gb >= 16 && self.cpu_cores >= 8 {
                (
                    CPU_MODEL_MEDIUM,
                    "GPU detected but driver not loaded - using CPU model (7b)",
                )
            } else {
                (
                    CPU_MODEL_SMALL,
                    "GPU detected but driver not loaded - using CPU model (3b)",
                )
            }
        } else if self.gpu_vendor != GpuVendor::None && !self.gpu_driver_functional {
            // GPU exists, driver loaded but not functional - check CPU capability
            if self.ram_gb >= 16 && self.cpu_cores >= 8 {
                (
                    CPU_MODEL_MEDIUM,
                    "GPU driver not functional - using CPU model (7b)",
                )
            } else {
                (
                    CPU_MODEL_SMALL,
                    "GPU driver not functional - using CPU model (3b)",
                )
            }
        } else if self.ram_gb >= 16 && self.cpu_cores >= 8 {
            // Good CPU system without GPU
            (CPU_MODEL_MEDIUM, "High-performance CPU system")
        } else {
            // Low-end system
            (CPU_MODEL_SMALL, "Standard CPU system")
        };

        ModelRecommendation {
            model: model.to_string(),
            fallback: CPU_MODEL_SMALL.to_string(),
            reason: reason.to_string(),
            can_upgrade: !self.gpu_driver_functional && self.gpu_vendor != GpuVendor::None,
            upgrade_trigger: if !self.gpu_driver_functional && self.gpu_vendor != GpuVendor::None {
                Some("Install GPU drivers to unlock larger models".to_string())
            } else {
                None
            },
        }
    }

    /// Check if hardware has improved compared to a previous profile
    pub fn has_improved_from(&self, previous: &HardwareProfile) -> Option<HardwareImprovement> {
        let mut improvements = Vec::new();

        // GPU driver became functional
        if self.gpu_driver_functional && !previous.gpu_driver_functional {
            improvements.push("GPU driver is now functional".to_string());
        }

        // GPU driver became loaded
        if self.gpu_driver_loaded && !previous.gpu_driver_loaded {
            improvements.push("GPU driver is now loaded".to_string());
        }

        // RAM upgrade
        if self.ram_gb > previous.ram_gb {
            improvements.push(format!(
                "RAM upgraded from {}GB to {}GB",
                previous.ram_gb, self.ram_gb
            ));
        }

        // VRAM became available
        if self.vram_gb.is_some() && previous.vram_gb.is_none() {
            improvements.push(format!(
                "VRAM now available: {}GB",
                self.vram_gb.unwrap_or(0)
            ));
        }

        if improvements.is_empty() {
            None
        } else {
            Some(HardwareImprovement {
                improvements,
                new_recommendation: self.select_model(),
                previous_model: previous.select_model().model,
            })
        }
    }
}

/// Model recommendation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub model: String,
    pub fallback: String,
    pub reason: String,
    pub can_upgrade: bool,
    pub upgrade_trigger: Option<String>,
}

/// Hardware improvement detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareImprovement {
    pub improvements: Vec<String>,
    pub new_recommendation: ModelRecommendation,
    pub previous_model: String,
}

/// Runtime state for hardware monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareState {
    pub last_profile: Option<HardwareProfile>,
    pub last_check: Option<i64>,
    pub pending_upgrade: Option<ModelRecommendation>,
    pub auto_upgraded: bool,
}

impl HardwareState {
    /// Get state file path
    pub fn state_path() -> std::path::PathBuf {
        std::path::PathBuf::from("/var/lib/anna/hardware_state.json")
    }

    /// Load state from disk
    pub fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save state to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_profile_default() {
        let profile = HardwareProfile::default();
        assert_eq!(profile.gpu_vendor, GpuVendor::None);
        assert!(!profile.gpu_driver_loaded);
        assert!(!profile.gpu_driver_functional);
    }

    #[test]
    fn test_model_selection_no_gpu() {
        let profile = HardwareProfile {
            cpu_cores: 4,
            ram_gb: 8,
            gpu_vendor: GpuVendor::None,
            gpu_driver_functional: false,
            ..Default::default()
        };
        let rec = profile.select_model();
        assert_eq!(rec.model, CPU_MODEL_SMALL);
    }

    #[test]
    fn test_model_selection_gpu_without_driver_good_cpu() {
        // GPU without driver but good CPU - should use 7b model
        let profile = HardwareProfile {
            cpu_cores: 8,
            ram_gb: 32,
            gpu_vendor: GpuVendor::Nvidia,
            gpu_name: Some("GeForce RTX 3080".to_string()),
            gpu_driver_loaded: false,
            gpu_driver_functional: false,
            ..Default::default()
        };
        let rec = profile.select_model();
        // Should use CPU_MODEL_MEDIUM (7b) because CPU is capable
        assert_eq!(rec.model, CPU_MODEL_MEDIUM);
        assert!(rec.can_upgrade);
        assert!(rec.upgrade_trigger.is_some());
    }

    #[test]
    fn test_model_selection_gpu_without_driver_weak_cpu() {
        // GPU without driver and weak CPU - should use 3b model
        let profile = HardwareProfile {
            cpu_cores: 4,
            ram_gb: 8,
            gpu_vendor: GpuVendor::Nvidia,
            gpu_name: Some("GeForce GTX 1050".to_string()),
            gpu_driver_loaded: false,
            gpu_driver_functional: false,
            ..Default::default()
        };
        let rec = profile.select_model();
        // Should use CPU_MODEL_SMALL (3b) because CPU is not capable
        assert_eq!(rec.model, CPU_MODEL_SMALL);
        assert!(rec.can_upgrade);
        assert!(rec.upgrade_trigger.is_some());
    }

    #[test]
    fn test_model_selection_gpu_with_driver() {
        let profile = HardwareProfile {
            cpu_cores: 8,
            ram_gb: 32,
            gpu_vendor: GpuVendor::Nvidia,
            gpu_name: Some("GeForce RTX 3080".to_string()),
            vram_gb: Some(10),
            gpu_driver_loaded: true,
            gpu_driver_functional: true,
            performance_profile: PerformanceProfile::High,
            ..Default::default()
        };
        let rec = profile.select_model();
        assert_eq!(rec.model, GPU_MODEL_MEDIUM);
        assert!(!rec.can_upgrade);
    }

    #[test]
    fn test_model_selection_high_vram() {
        let profile = HardwareProfile {
            cpu_cores: 16,
            ram_gb: 64,
            gpu_vendor: GpuVendor::Nvidia,
            vram_gb: Some(24),
            gpu_driver_loaded: true,
            gpu_driver_functional: true,
            performance_profile: PerformanceProfile::High,
            ..Default::default()
        };
        let rec = profile.select_model();
        assert_eq!(rec.model, GPU_MODEL_LARGE);
    }

    #[test]
    fn test_hardware_improvement_detection() {
        let old = HardwareProfile {
            gpu_vendor: GpuVendor::Nvidia,
            gpu_driver_loaded: false,
            gpu_driver_functional: false,
            ..Default::default()
        };

        let new = HardwareProfile {
            gpu_vendor: GpuVendor::Nvidia,
            gpu_driver_loaded: true,
            gpu_driver_functional: true,
            vram_gb: Some(8),
            ..Default::default()
        };

        let improvement = new.has_improved_from(&old);
        assert!(improvement.is_some());
        let imp = improvement.unwrap();
        assert!(!imp.improvements.is_empty());
    }

    #[test]
    fn test_performance_profile() {
        // Low-end system
        let low = HardwareProfile {
            cpu_cores: 2,
            ram_gb: 4,
            gpu_driver_functional: false,
            performance_profile: PerformanceProfile::Low,
            ..Default::default()
        };
        assert_eq!(low.performance_profile, PerformanceProfile::Low);

        // High-end system
        let high = HardwareProfile {
            cpu_cores: 8,
            ram_gb: 32,
            gpu_driver_functional: true,
            vram_gb: Some(12),
            performance_profile: PerformanceProfile::High,
            ..Default::default()
        };
        assert_eq!(high.performance_profile, PerformanceProfile::High);
    }

    #[test]
    fn test_gpu_vendor_as_str() {
        assert_eq!(GpuVendor::None.as_str(), "none");
        assert_eq!(GpuVendor::Nvidia.as_str(), "nvidia");
        assert_eq!(GpuVendor::Amd.as_str(), "amd");
        assert_eq!(GpuVendor::Intel.as_str(), "intel");
    }
}

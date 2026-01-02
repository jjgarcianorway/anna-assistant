//! Hardware profiling and system capability assessment (v0.0.434).

use crate::hardware_aware::detection::{
    chrono_now, detect_cpu, detect_gpu, detect_os, detect_ram, detect_storage,
};
use crate::hardware_aware::types::{
    CapabilityTier, CpuInfo, GpuInfo, GpuVendor, OsInfo, StorageInfo,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
        let cpu = detect_cpu();
        let (ram_total_gb, ram_free_gb) = detect_ram();
        let gpu = detect_gpu();
        let storage = detect_storage();
        let os = detect_os();
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

    /// Compute capability tier from hardware.
    fn compute_tier(ram_gb: f32, gpu: &GpuInfo) -> CapabilityTier {
        // GPU can bump tier but needs adequate RAM too
        let has_strong_gpu =
            gpu.discrete && matches!(gpu.vendor, GpuVendor::Nvidia | GpuVendor::Amd);
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

        assert_eq!(
            HardwareProfile::compute_tier(4.0, &no_gpu),
            CapabilityTier::Tiny
        );
        assert_eq!(
            HardwareProfile::compute_tier(12.0, &no_gpu),
            CapabilityTier::Small
        );
        assert_eq!(
            HardwareProfile::compute_tier(24.0, &no_gpu),
            CapabilityTier::Medium
        );
        assert_eq!(
            HardwareProfile::compute_tier(64.0, &no_gpu),
            CapabilityTier::Large
        );
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
        assert_eq!(
            HardwareProfile::compute_tier(12.0, &nvidia_gpu),
            CapabilityTier::Medium
        );

        // 20GB RAM with strong GPU becomes Large
        assert_eq!(
            HardwareProfile::compute_tier(20.0, &nvidia_gpu),
            CapabilityTier::Large
        );
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

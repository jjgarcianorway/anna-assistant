//! System profile section for status display (v0.0.434).

use super::super::profile::HardwareProfile;
use serde::{Deserialize, Serialize};

/// System profile section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfileSection {
    /// Total RAM in GB.
    pub ram_total_gb: f32,
    /// CPU model name.
    pub cpu_model: String,
    /// CPU core count.
    pub cpu_cores: u32,
    /// AVX2 supported.
    pub avx2: bool,
    /// GPU description (if any).
    pub gpu: Option<String>,
    /// Capability tier.
    pub tier: String,
    /// Last profiled timestamp.
    pub last_profiled: String,
}

impl SystemProfileSection {
    /// Build from hardware profile.
    pub fn from_profile(profile: &HardwareProfile) -> Self {
        let gpu = if profile.gpu.discrete {
            Some(format!(
                "{} {}",
                profile.gpu.vendor.label(),
                profile
                    .gpu
                    .model_name
                    .clone()
                    .unwrap_or_else(|| "GPU".to_string())
            ))
        } else if profile.gpu.vendor != super::super::profile::GpuVendor::None {
            Some(format!("{} (integrated)", profile.gpu.vendor.label()))
        } else {
            None
        };

        Self {
            ram_total_gb: profile.ram_total_gb,
            cpu_model: profile.cpu.model_name.clone(),
            cpu_cores: profile.cpu.core_count,
            avx2: profile.cpu.avx2_supported,
            gpu,
            tier: profile.tier.label().to_string(),
            last_profiled: profile.last_profiled_at.clone(),
        }
    }
}

//! Test fixtures for hardware-aware acceptance tests (v0.0.434).

use super::profile::{
    CapabilityTier, CpuInfo, GpuInfo, GpuVendor, HardwareProfile, OsInfo, StorageInfo,
};

pub fn tiny_profile() -> HardwareProfile {
    HardwareProfile {
        profile_version: 1,
        last_profiled_at: "0".to_string(),
        cpu: CpuInfo {
            model_name: "Intel Celeron".to_string(),
            core_count: 2,
            thread_count: 2,
            avx2_supported: false,
        },
        ram_total_gb: 6.0,
        ram_free_gb: 4.0,
        gpu: GpuInfo::default(),
        storage: StorageInfo {
            data_dir_available_gb: 10,
            model_storage_available_gb: 20,
            model_storage_total_gb: 50,
        },
        os: OsInfo {
            distro: "Arch Linux".to_string(),
            kernel_version: "6.6.0".to_string(),
        },
        tier: CapabilityTier::Tiny,
    }
}

pub fn medium_profile() -> HardwareProfile {
    HardwareProfile {
        profile_version: 1,
        last_profiled_at: "0".to_string(),
        cpu: CpuInfo {
            model_name: "Intel Core i7-12700".to_string(),
            core_count: 12,
            thread_count: 20,
            avx2_supported: true,
        },
        ram_total_gb: 32.0,
        ram_free_gb: 24.0,
        gpu: GpuInfo {
            discrete: true,
            vendor: GpuVendor::Nvidia,
            model_name: Some("RTX 4060".to_string()),
            vram_gb: Some(8),
        },
        storage: StorageInfo {
            data_dir_available_gb: 50,
            model_storage_available_gb: 100,
            model_storage_total_gb: 500,
        },
        os: OsInfo {
            distro: "Arch Linux".to_string(),
            kernel_version: "6.6.0".to_string(),
        },
        tier: CapabilityTier::Medium,
    }
}

pub fn large_profile() -> HardwareProfile {
    HardwareProfile {
        profile_version: 1,
        last_profiled_at: "0".to_string(),
        cpu: CpuInfo {
            model_name: "AMD Ryzen 9 7950X".to_string(),
            core_count: 16,
            thread_count: 32,
            avx2_supported: true,
        },
        ram_total_gb: 64.0,
        ram_free_gb: 48.0,
        gpu: GpuInfo {
            discrete: true,
            vendor: GpuVendor::Nvidia,
            model_name: Some("RTX 4090".to_string()),
            vram_gb: Some(24),
        },
        storage: StorageInfo {
            data_dir_available_gb: 200,
            model_storage_available_gb: 500,
            model_storage_total_gb: 2000,
        },
        os: OsInfo {
            distro: "Arch Linux".to_string(),
            kernel_version: "6.6.0".to_string(),
        },
        tier: CapabilityTier::Large,
    }
}

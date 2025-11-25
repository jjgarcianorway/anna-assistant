//! Machine Identity - Deterministic machine fingerprinting
//!
//! v6.54.0: Identity, Persistence, and Multi-User Awareness
//!
//! This module provides stable machine identification without secrets,
//! allowing Anna to detect reinstalls, clones, and different machines.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Environment type for the machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentType {
    BareMetal,
    VM,
    Container,
}

/// Machine fingerprint for stable identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineFingerprint {
    /// Stable ID string (hash of key attributes)
    pub id: String,

    /// CPU model string
    pub cpu_model: String,

    /// Total RAM in GiB
    pub total_ram_gib: u64,

    /// Primary disk model
    pub primary_disk_model: String,

    /// Primary disk size in GB
    pub primary_disk_size_gb: u64,

    /// System hostname
    pub hostname: String,

    /// OS release string
    pub os_release: String,

    /// Whether this is a virtual machine
    pub is_virtual_machine: bool,

    /// VM hint (qemu, vmware, virtualbox, etc)
    pub vm_hint: Option<String>,

    /// Environment type
    pub environment: EnvironmentType,
}

/// Relationship between two machines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineRelation {
    /// Same machine, no changes
    SameMachine,

    /// Same hardware, likely reinstalled OS
    SameHardwareReinstalled,

    /// Probably a cloned disk
    ProbablyCloned,

    /// Different machine entirely
    DifferentMachine,
}

impl MachineFingerprint {
    /// Collect current machine fingerprint
    pub fn collect() -> Self {
        let cpu_model = Self::read_cpu_model();
        let total_ram_gib = Self::read_total_ram_gib();
        let (primary_disk_model, primary_disk_size_gb) = Self::read_primary_disk();
        let hostname = Self::read_hostname();
        let os_release = Self::read_os_release();
        let (is_virtual_machine, vm_hint) = Self::detect_vm();

        let environment = if is_virtual_machine {
            EnvironmentType::VM
        } else {
            // TODO: Detect containers in future
            EnvironmentType::BareMetal
        };

        // Generate stable ID from key attributes
        let id = Self::generate_id(
            &cpu_model,
            total_ram_gib,
            &primary_disk_model,
            primary_disk_size_gb,
        );

        Self {
            id,
            cpu_model,
            total_ram_gib,
            primary_disk_model,
            primary_disk_size_gb,
            hostname,
            os_release,
            is_virtual_machine,
            vm_hint,
            environment,
        }
    }

    /// Calculate similarity score (0.0 to 1.0)
    pub fn similarity_score(&self, other: &Self) -> f32 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // High weight for primary disk (most stable hardware identifier)
        let disk_weight = 0.35;
        if self.primary_disk_model == other.primary_disk_model
            && self.primary_disk_size_gb == other.primary_disk_size_gb {
            score += disk_weight;
        }
        weight_sum += disk_weight;

        // High weight for CPU model
        let cpu_weight = 0.30;
        if self.cpu_model == other.cpu_model {
            score += cpu_weight;
        }
        weight_sum += cpu_weight;

        // Medium weight for total RAM
        let ram_weight = 0.20;
        if self.total_ram_gib == other.total_ram_gib {
            score += ram_weight;
        }
        weight_sum += ram_weight;

        // Lower weight for hostname (changes frequently)
        let hostname_weight = 0.08;
        if self.hostname == other.hostname {
            score += hostname_weight;
        }
        weight_sum += hostname_weight;

        // Lower weight for OS (can change with reinstall)
        let os_weight = 0.07;
        if self.os_release == other.os_release {
            score += os_weight;
        }
        weight_sum += os_weight;

        score / weight_sum
    }

    /// Classify relationship to another machine
    pub fn classification(&self, other: &Self) -> MachineRelation {
        let similarity = self.similarity_score(other);

        // Exact match on ID
        if self.id == other.id {
            return MachineRelation::SameMachine;
        }

        // Very high similarity but different OS → reinstall
        if similarity >= 0.85 && self.os_release != other.os_release {
            return MachineRelation::SameHardwareReinstalled;
        }

        // High similarity but same OS → probably cloned
        if similarity >= 0.85 && self.os_release == other.os_release {
            return MachineRelation::ProbablyCloned;
        }

        // Hardware matches but hostname/OS different → reinstall
        if self.cpu_model == other.cpu_model
            && self.primary_disk_model == other.primary_disk_model
            && (self.hostname != other.hostname || self.os_release != other.os_release) {
            return MachineRelation::SameHardwareReinstalled;
        }

        // Environment type mismatch (bare metal vs VM) → different or cloned
        if self.environment != other.environment {
            return MachineRelation::ProbablyCloned;
        }

        // Low similarity → different machine
        MachineRelation::DifferentMachine
    }

    // === Helper methods for system detection ===

    fn generate_id(cpu: &str, ram: u64, disk: &str, disk_size: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cpu.as_bytes());
        hasher.update(&ram.to_le_bytes());
        hasher.update(disk.as_bytes());
        hasher.update(&disk_size.to_le_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string()
    }

    fn read_cpu_model() -> String {
        std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("model name"))
                    .and_then(|line| line.split(':').nth(1))
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| "Unknown CPU".to_string())
    }

    fn read_total_ram_gib() -> u64 {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("MemTotal:"))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                    })
            })
            .map(|kb| kb / (1024 * 1024)) // Convert KB to GiB
            .unwrap_or(0)
    }

    fn read_primary_disk() -> (String, u64) {
        // Try to read from /sys/block for the first non-loop device
        if let Ok(entries) = std::fs::read_dir("/sys/block") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Skip loop devices, ram devices, etc
                if name_str.starts_with("loop") || name_str.starts_with("ram") {
                    continue;
                }

                // Try to read model
                let model_path = entry.path().join("device/model");
                let model = std::fs::read_to_string(&model_path)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| name_str.to_string());

                // Try to read size
                let size_path = entry.path().join("size");
                let size_gb = std::fs::read_to_string(&size_path)
                    .ok()
                    .and_then(|s| s.trim().parse::<u64>().ok())
                    .map(|blocks| blocks * 512 / (1000 * 1000 * 1000)) // 512-byte blocks to GB
                    .unwrap_or(0);

                return (model, size_gb);
            }
        }

        ("Unknown Disk".to_string(), 0)
    }

    fn read_hostname() -> String {
        std::fs::read_to_string("/etc/hostname")
            .ok()
            .map(|s| s.trim().to_string())
            .or_else(|| {
                std::process::Command::new("hostname")
                    .output()
                    .ok()
                    .and_then(|out| String::from_utf8(out.stdout).ok())
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| "localhost".to_string())
    }

    fn read_os_release() -> String {
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("PRETTY_NAME="))
                    .map(|line| {
                        line.trim_start_matches("PRETTY_NAME=")
                            .trim_matches('"')
                            .to_string()
                    })
            })
            .unwrap_or_else(|| "Unknown OS".to_string())
    }

    fn detect_vm() -> (bool, Option<String>) {
        // Check DMI system information
        if let Ok(product_name) = std::fs::read_to_string("/sys/class/dmi/id/product_name") {
            let product = product_name.to_lowercase();

            if product.contains("virtualbox") {
                return (true, Some("VirtualBox".to_string()));
            } else if product.contains("vmware") {
                return (true, Some("VMware".to_string()));
            } else if product.contains("qemu") || product.contains("kvm") {
                return (true, Some("QEMU/KVM".to_string()));
            } else if product.contains("microsoft") || product.contains("virtual") {
                return (true, Some("Hyper-V".to_string()));
            }
        }

        // Check CPU vendor for known VM indicators
        if let Some(cpu) = std::fs::read_to_string("/proc/cpuinfo").ok() {
            if cpu.contains("QEMU") || cpu.contains("TCG") {
                return (true, Some("QEMU".to_string()));
            }
        }

        // Check systemd-detect-virt if available
        if let Ok(output) = std::process::Command::new("systemd-detect-virt").output() {
            if output.status.success() {
                let virt = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if virt != "none" && !virt.is_empty() {
                    return (true, Some(virt));
                }
            }
        }

        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_collection() {
        let fp = MachineFingerprint::collect();

        // Basic sanity checks
        assert!(!fp.id.is_empty());
        assert!(!fp.cpu_model.is_empty());
        assert!(!fp.hostname.is_empty());
        assert!(!fp.os_release.is_empty());
    }

    #[test]
    fn test_same_machine() {
        let fp1 = MachineFingerprint {
            id: "test123".to_string(),
            cpu_model: "Intel Core i7-9700K".to_string(),
            total_ram_gib: 16,
            primary_disk_model: "Samsung SSD 970".to_string(),
            primary_disk_size_gb: 500,
            hostname: "myhost".to_string(),
            os_release: "Arch Linux".to_string(),
            is_virtual_machine: false,
            vm_hint: None,
            environment: EnvironmentType::BareMetal,
        };

        let fp2 = fp1.clone();

        assert_eq!(fp1.classification(&fp2), MachineRelation::SameMachine);
        assert_eq!(fp1.similarity_score(&fp2), 1.0);
    }

    #[test]
    fn test_reinstalled() {
        let fp1 = MachineFingerprint {
            id: "test123".to_string(),
            cpu_model: "Intel Core i7-9700K".to_string(),
            total_ram_gib: 16,
            primary_disk_model: "Samsung SSD 970".to_string(),
            primary_disk_size_gb: 500,
            hostname: "myhost".to_string(),
            os_release: "Arch Linux".to_string(),
            is_virtual_machine: false,
            vm_hint: None,
            environment: EnvironmentType::BareMetal,
        };

        let fp2 = MachineFingerprint {
            os_release: "Ubuntu 22.04".to_string(),
            id: "different".to_string(), // ID changed due to different attributes
            ..fp1.clone()
        };

        let relation = fp1.classification(&fp2);
        assert_eq!(relation, MachineRelation::SameHardwareReinstalled);
    }

    #[test]
    fn test_cloned() {
        let fp1 = MachineFingerprint {
            id: "test123".to_string(),
            cpu_model: "Intel Core i7-9700K".to_string(),
            total_ram_gib: 16,
            primary_disk_model: "Samsung SSD 970".to_string(),
            primary_disk_size_gb: 500,
            hostname: "original".to_string(),
            os_release: "Arch Linux".to_string(),
            is_virtual_machine: false,
            vm_hint: None,
            environment: EnvironmentType::BareMetal,
        };

        let fp2 = MachineFingerprint {
            hostname: "cloned".to_string(),
            id: "different".to_string(),
            ..fp1.clone()
        };

        let relation = fp1.classification(&fp2);
        assert_eq!(relation, MachineRelation::ProbablyCloned);
    }

    #[test]
    fn test_different_machine() {
        let fp1 = MachineFingerprint {
            id: "test123".to_string(),
            cpu_model: "Intel Core i7-9700K".to_string(),
            total_ram_gib: 16,
            primary_disk_model: "Samsung SSD 970".to_string(),
            primary_disk_size_gb: 500,
            hostname: "machine1".to_string(),
            os_release: "Arch Linux".to_string(),
            is_virtual_machine: false,
            vm_hint: None,
            environment: EnvironmentType::BareMetal,
        };

        let fp2 = MachineFingerprint {
            id: "different".to_string(),
            cpu_model: "AMD Ryzen 9 5900X".to_string(),
            total_ram_gib: 32,
            primary_disk_model: "WD Blue".to_string(),
            primary_disk_size_gb: 1000,
            hostname: "machine2".to_string(),
            os_release: "Ubuntu 22.04".to_string(),
            is_virtual_machine: false,
            vm_hint: None,
            environment: EnvironmentType::BareMetal,
        };

        let relation = fp1.classification(&fp2);
        assert_eq!(relation, MachineRelation::DifferentMachine);

        let score = fp1.similarity_score(&fp2);
        assert!(score < 0.3); // Very different
    }

    #[test]
    fn test_vm_to_bare_metal_is_cloned() {
        let fp1 = MachineFingerprint {
            id: "test123".to_string(),
            cpu_model: "Intel Core i7-9700K".to_string(),
            total_ram_gib: 16,
            primary_disk_model: "Samsung SSD 970".to_string(),
            primary_disk_size_gb: 500,
            hostname: "myhost".to_string(),
            os_release: "Arch Linux".to_string(),
            is_virtual_machine: false,
            vm_hint: None,
            environment: EnvironmentType::BareMetal,
        };

        let fp2 = MachineFingerprint {
            id: "different_id".to_string(), // Different ID due to environment change
            is_virtual_machine: true,
            vm_hint: Some("QEMU/KVM".to_string()),
            environment: EnvironmentType::VM,
            ..fp1.clone()
        };

        let relation = fp1.classification(&fp2);
        assert_eq!(relation, MachineRelation::ProbablyCloned);
    }

    #[test]
    fn test_similarity_scoring() {
        let base = MachineFingerprint {
            id: "test123".to_string(),
            cpu_model: "Intel Core i7-9700K".to_string(),
            total_ram_gib: 16,
            primary_disk_model: "Samsung SSD 970".to_string(),
            primary_disk_size_gb: 500,
            hostname: "myhost".to_string(),
            os_release: "Arch Linux".to_string(),
            is_virtual_machine: false,
            vm_hint: None,
            environment: EnvironmentType::BareMetal,
        };

        // Only hostname different
        let hostname_diff = MachineFingerprint {
            hostname: "newhost".to_string(),
            ..base.clone()
        };
        let score = base.similarity_score(&hostname_diff);
        assert!(score > 0.9); // Very similar

        // Only CPU different
        let cpu_diff = MachineFingerprint {
            cpu_model: "AMD Ryzen 5 3600".to_string(),
            ..base.clone()
        };
        let score = base.similarity_score(&cpu_diff);
        assert!(score < 0.8 && score > 0.6); // Moderately similar

        // Disk and CPU different
        let major_diff = MachineFingerprint {
            cpu_model: "AMD Ryzen 5 3600".to_string(),
            primary_disk_model: "WD Blue".to_string(),
            ..base.clone()
        };
        let score = base.similarity_score(&major_diff);
        assert!(score < 0.4); // Not very similar
    }
}

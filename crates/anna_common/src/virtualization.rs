//! Virtualization and containerization detection
//!
//! Detects virtualization and containerization capabilities:
//! - KVM/SVM support (hardware virtualization)
//! - VFIO/IOMMU for PCI passthrough
//! - Docker, Podman, libvirt status
//! - Running containers and VMs

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// Virtualization technology support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualizationInfo {
    /// Hardware virtualization available (KVM on Intel, SVM on AMD)
    pub hw_virt_available: bool,
    /// Virtualization technology type (Intel VT-x or AMD-V)
    pub virt_type: Option<String>,
    /// KVM kernel module loaded
    pub kvm_loaded: bool,
    /// IOMMU enabled for PCI passthrough
    pub iommu_enabled: bool,
    /// VFIO kernel module loaded
    pub vfio_loaded: bool,
    /// VFIO devices bound
    pub vfio_devices: Vec<String>,
    /// libvirt service status
    pub libvirt_running: bool,
    /// libvirt service enabled
    pub libvirt_enabled: bool,
    /// Docker service status
    pub docker_running: bool,
    /// Docker service enabled
    pub docker_enabled: bool,
    /// Podman installed
    pub podman_installed: bool,
    /// QEMU/KVM installed
    pub qemu_installed: bool,
    /// VirtualBox installed
    pub virtualbox_installed: bool,
    /// Number of running Docker containers
    pub docker_containers_running: Option<usize>,
    /// Number of libvirt VMs
    pub libvirt_vms: Option<usize>,
}

impl VirtualizationInfo {
    /// Detect virtualization capabilities
    pub fn detect() -> Self {
        let (hw_virt_available, virt_type) = detect_hardware_virt();
        let kvm_loaded = is_module_loaded("kvm");
        let iommu_enabled = check_iommu_enabled();
        let vfio_loaded = is_module_loaded("vfio");
        let vfio_devices = get_vfio_devices();
        let libvirt_running = is_service_running("libvirtd");
        let libvirt_enabled = is_service_enabled("libvirtd");
        let docker_running = is_service_running("docker");
        let docker_enabled = is_service_enabled("docker");
        let podman_installed = is_command_available("podman");
        let qemu_installed = is_command_available("qemu-system-x86_64");
        let virtualbox_installed = is_command_available("VBoxManage");
        let docker_containers_running = if docker_running {
            count_docker_containers()
        } else {
            None
        };
        let libvirt_vms = if libvirt_running {
            count_libvirt_vms()
        } else {
            None
        };

        Self {
            hw_virt_available,
            virt_type,
            kvm_loaded,
            iommu_enabled,
            vfio_loaded,
            vfio_devices,
            libvirt_running,
            libvirt_enabled,
            docker_running,
            docker_enabled,
            podman_installed,
            qemu_installed,
            virtualbox_installed,
            docker_containers_running,
            libvirt_vms,
        }
    }
}

/// Detect hardware virtualization support
fn detect_hardware_virt() -> (bool, Option<String>) {
    // Check CPU flags for virtualization support
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("flags") || line.starts_with("Features") {
                if let Some(flags_str) = line.split(':').nth(1) {
                    let flags_lower = flags_str.to_lowercase();

                    // Check for Intel VT-x (vmx flag)
                    if flags_lower.contains("vmx") {
                        return (true, Some("Intel VT-x".to_string()));
                    }

                    // Check for AMD-V (svm flag)
                    if flags_lower.contains("svm") {
                        return (true, Some("AMD-V".to_string()));
                    }
                }
                break;
            }
        }
    }

    // Fallback: check lscpu output
    if let Ok(output) = Command::new("lscpu").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Virtualization:") {
                    if let Some(virt_type) = line.split(':').nth(1) {
                        let virt_type = virt_type.trim().to_string();
                        return (true, Some(virt_type));
                    }
                }
            }
        }
    }

    (false, None)
}

/// Check if a kernel module is loaded
fn is_module_loaded(module_name: &str) -> bool {
    if let Ok(output) = Command::new("lsmod").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.lines().any(|line| line.starts_with(module_name));
        }
    }
    false
}

/// Check if IOMMU is enabled
fn check_iommu_enabled() -> bool {
    // Check kernel command line for IOMMU
    if let Ok(content) = fs::read_to_string("/proc/cmdline") {
        if content.contains("intel_iommu=on") || content.contains("amd_iommu=on") {
            return true;
        }
    }

    // Check dmesg for IOMMU initialization
    if let Ok(output) = Command::new("dmesg").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("IOMMU enabled")
                || stdout.contains("DMAR: Intel-IOMMU")
                || stdout.contains("AMD-Vi")
            {
                return true;
            }
        }
    }

    // Check for IOMMU groups
    if fs::read_dir("/sys/kernel/iommu_groups").is_ok() {
        return true;
    }

    false
}

/// Get list of VFIO-bound devices
fn get_vfio_devices() -> Vec<String> {
    let mut devices = Vec::new();

    // Check /sys/bus/pci/drivers/vfio-pci for bound devices
    if let Ok(entries) = fs::read_dir("/sys/bus/pci/drivers/vfio-pci") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // PCI addresses look like 0000:01:00.0
            if name.contains(':') && name.contains('.') {
                // Try to get device name
                let device_path = format!("/sys/bus/pci/devices/{}/", name);
                if let Ok(device_name) = fs::read_to_string(format!("{}uevent", device_path)) {
                    for line in device_name.lines() {
                        if line.starts_with("PCI_SLOT_NAME=") {
                            if let Some(slot) = line.split('=').nth(1) {
                                devices.push(slot.to_string());
                            }
                        }
                    }
                }
                if !devices.iter().any(|d| d.contains(&name)) {
                    devices.push(name);
                }
            }
        }
    }

    devices
}

/// Check if a systemd service is running
fn is_service_running(service_name: &str) -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-active")
        .arg(service_name)
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim() == "active";
        }
    }
    false
}

/// Check if a systemd service is enabled
fn is_service_enabled(service_name: &str) -> bool {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-enabled")
        .arg(service_name)
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim() == "enabled";
        }
    }
    false
}

/// Check if a command is available
fn is_command_available(command: &str) -> bool {
    if let Ok(output) = Command::new("which").arg(command).output() {
        return output.status.success();
    }
    false
}

/// Count running Docker containers
fn count_docker_containers() -> Option<usize> {
    if let Ok(output) = Command::new("docker").arg("ps").arg("-q").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let count = stdout.lines().filter(|line| !line.is_empty()).count();
            return Some(count);
        }
    }
    None
}

/// Count libvirt VMs
fn count_libvirt_vms() -> Option<usize> {
    if let Ok(output) = Command::new("virsh").arg("list").arg("--all").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Count non-header lines
            let count = stdout
                .lines()
                .skip(2) // Skip header lines
                .filter(|line| {
                    !line.trim().is_empty() && line.contains("running") || line.contains("shut off")
                })
                .count();
            return Some(count);
        }
    }
    None
}

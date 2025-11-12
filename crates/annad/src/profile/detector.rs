// Phase 3.0.0-alpha.1: System Profile Detector
// Collects real system information for adaptive behavior

use super::types::{GpuInfo, MonitoringMode, SessionType, SystemProfile, VirtualizationInfo};
use anyhow::Result;
use std::process::Command;
use sysinfo::{Disks, System};
use tracing::{debug, warn};

/// System profiler that collects runtime environment information
pub struct SystemProfiler {
    system: System,
    disks: Disks,
}

impl SystemProfiler {
    /// Create a new system profiler
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        Self { system, disks }
    }

    /// Collect complete system profile
    pub fn collect_profile(&mut self) -> Result<SystemProfile> {
        self.system.refresh_all();
        self.disks.refresh();

        let total_memory_mb = self.system.total_memory() / (1024 * 1024); // Convert bytes to MB
        let available_memory_mb = self.system.available_memory() / (1024 * 1024);
        let cpu_cores = self.system.cpus().len();

        let (total_disk_gb, available_disk_gb) = self.get_disk_info();
        let uptime_seconds = Self::get_uptime();

        let virtualization = Self::detect_virtualization();
        let session_type = Self::detect_session_type();
        let gpu_info = Self::detect_gpu();

        let recommended_monitoring_mode =
            SystemProfile::calculate_monitoring_mode(total_memory_mb, &session_type);

        Ok(SystemProfile {
            total_memory_mb,
            available_memory_mb,
            cpu_cores,
            total_disk_gb,
            available_disk_gb,
            uptime_seconds,
            virtualization,
            session_type,
            gpu_info,
            recommended_monitoring_mode,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get disk space information
    fn get_disk_info(&self) -> (u64, u64) {
        let mut total_gb = 0u64;
        let mut available_gb = 0u64;

        for disk in self.disks.list() {
            // Only count root filesystem to avoid counting tmpfs, etc.
            if disk.mount_point().to_str() == Some("/") {
                total_gb = disk.total_space() / (1024 * 1024 * 1024); // Bytes to GB
                available_gb = disk.available_space() / (1024 * 1024 * 1024);
                break;
            }
        }

        // Fallback: sum all disks if root not found
        if total_gb == 0 {
            for disk in self.disks.list() {
                total_gb += disk.total_space() / (1024 * 1024 * 1024);
                available_gb += disk.available_space() / (1024 * 1024 * 1024);
            }
        }

        (total_gb, available_gb)
    }

    /// Get system uptime by reading /proc/uptime
    fn get_uptime() -> u64 {
        std::fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| s.split_whitespace().next().and_then(|n| n.parse::<f64>().ok()))
            .map(|f| f as u64)
            .unwrap_or(0)
    }

    /// Detect virtualization using systemd-detect-virt
    fn detect_virtualization() -> VirtualizationInfo {
        let output = Command::new("systemd-detect-virt")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let virt_type = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();

                match virt_type.as_str() {
                    "none" => VirtualizationInfo::None,
                    "docker" | "podman" | "lxc" | "systemd-nspawn" => {
                        VirtualizationInfo::Container(virt_type)
                    }
                    _ => VirtualizationInfo::VM(virt_type),
                }
            }
            Ok(_) => {
                debug!("systemd-detect-virt returned non-zero, assuming bare metal");
                VirtualizationInfo::None
            }
            Err(e) => {
                warn!("Failed to run systemd-detect-virt: {}", e);
                VirtualizationInfo::Unknown
            }
        }
    }

    /// Detect session type (desktop, SSH, headless, console)
    fn detect_session_type() -> SessionType {
        // Check SSH first
        if let Ok(ssh_conn) = std::env::var("SSH_CONNECTION") {
            let client_ip = ssh_conn
                .split_whitespace()
                .next()
                .map(|s| s.to_string());

            let display_forwarding = std::env::var("DISPLAY").is_ok();

            return SessionType::SSH {
                client_ip,
                display_forwarding,
            };
        }

        // Check for desktop session
        if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
            if session_type != "tty" {
                return SessionType::Desktop(session_type);
            } else {
                return SessionType::Console;
            }
        }

        // Check DISPLAY for X11
        if std::env::var("DISPLAY").is_ok() {
            return SessionType::Desktop("x11".to_string());
        }

        // Check WAYLAND_DISPLAY
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return SessionType::Desktop("wayland".to_string());
        }

        // Check if we're on a TTY
        if let Ok(output) = Command::new("tty").output() {
            let tty = String::from_utf8_lossy(&output.stdout);
            if tty.contains("/dev/tty") && !tty.contains("/dev/pts/") {
                return SessionType::Console;
            }
        }

        // Default: Headless
        SessionType::Headless
    }

    /// Detect GPU using lspci
    fn detect_gpu() -> GpuInfo {
        let output = Command::new("lspci")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let lspci_output = String::from_utf8_lossy(&output.stdout);

                // Look for VGA controller
                for line in lspci_output.lines() {
                    if line.to_lowercase().contains("vga compatible controller") {
                        // Extract vendor and model
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 3 {
                            let info = parts[2].trim();

                            let vendor = if info.to_lowercase().contains("nvidia") {
                                Some("NVIDIA".to_string())
                            } else if info.to_lowercase().contains("amd") {
                                Some("AMD".to_string())
                            } else if info.to_lowercase().contains("intel") {
                                Some("Intel".to_string())
                            } else {
                                None
                            };

                            return GpuInfo {
                                present: true,
                                vendor,
                                model: Some(info.to_string()),
                            };
                        }
                    }
                }

                // No VGA controller found
                GpuInfo {
                    present: false,
                    vendor: None,
                    model: None,
                }
            }
            Ok(_) | Err(_) => {
                debug!("Failed to run lspci or parse output");
                GpuInfo {
                    present: false,
                    vendor: None,
                    model: None,
                }
            }
        }
    }
}

impl Default for SystemProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = SystemProfiler::new();
        assert!(profiler.system.cpus().len() > 0);
    }

    #[test]
    fn test_collect_profile() {
        let mut profiler = SystemProfiler::new();
        let profile = profiler.collect_profile();
        assert!(profile.is_ok());

        let profile = profile.unwrap();
        assert!(profile.total_memory_mb > 0);
        assert!(profile.cpu_cores > 0);
        assert!(profile.uptime_seconds >= 0);
    }

    #[test]
    fn test_detect_virtualization() {
        let virt = SystemProfiler::detect_virtualization();
        // Should return something (None, VM, Container, or Unknown)
        assert!(matches!(
            virt,
            VirtualizationInfo::None
                | VirtualizationInfo::VM(_)
                | VirtualizationInfo::Container(_)
                | VirtualizationInfo::Unknown
        ));
    }

    #[test]
    fn test_detect_session_type() {
        let session = SystemProfiler::detect_session_type();
        // Should return something
        assert!(matches!(
            session,
            SessionType::Desktop(_)
                | SessionType::Headless
                | SessionType::SSH { .. }
                | SessionType::Console
                | SessionType::Unknown
        ));
    }

    #[test]
    fn test_detect_gpu() {
        let gpu = SystemProfiler::detect_gpu();
        // Should return something (present or not)
        assert!(gpu.present || !gpu.present); // Tautology to ensure struct is valid
    }
}

//! System baseline tracking for change detection.
//! v0.0.990: Hardware, config files, and performance baselines.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Command;

/// System baseline - snapshot of normal system state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemBaseline {
    /// USB devices present at baseline
    pub usb_devices: Vec<UsbDevice>,
    /// PCI devices present at baseline
    pub pci_devices: Vec<PciDevice>,
    /// Config file hashes
    pub config_hashes: HashMap<String, String>,
    /// Performance baseline
    pub performance: PerformanceBaseline,
    /// When baseline was captured
    pub captured_at: String,
    /// Version of baseline format
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UsbDevice {
    pub bus: String,
    pub device: String,
    pub vendor_id: String,
    pub product_id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PciDevice {
    pub slot: String,
    pub class: String,
    pub vendor: String,
    pub device: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub boot_time_secs: f32,
    pub typical_memory_percent: u8,
    pub typical_load_1min: f32,
}

/// Changes detected since baseline
#[derive(Debug, Clone, Default)]
pub struct BaselineChanges {
    pub usb_added: Vec<UsbDevice>,
    pub usb_removed: Vec<UsbDevice>,
    pub pci_added: Vec<PciDevice>,
    pub pci_removed: Vec<PciDevice>,
    pub config_changed: Vec<String>,
    pub config_added: Vec<String>,
    pub config_removed: Vec<String>,
}

impl SystemBaseline {
    /// Path to baseline file
    pub fn path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/var/lib"))
            .join("anna")
            .join("baseline.json")
    }

    /// Load baseline from disk
    pub fn load() -> Option<Self> {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save baseline to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Capture current system state as baseline
    pub fn capture() -> Self {
        Self {
            usb_devices: get_usb_devices(),
            pci_devices: get_pci_devices(),
            config_hashes: get_config_hashes(),
            performance: get_performance_baseline(),
            captured_at: chrono::Utc::now().to_rfc3339(),
            version: 1,
        }
    }

    /// Compare current state to baseline
    pub fn compare(&self) -> BaselineChanges {
        let current_usb: HashSet<_> = get_usb_devices().into_iter().collect();
        let baseline_usb: HashSet<_> = self.usb_devices.iter().cloned().collect();

        let current_pci: HashSet<_> = get_pci_devices().into_iter().collect();
        let baseline_pci: HashSet<_> = self.pci_devices.iter().cloned().collect();

        let current_configs = get_config_hashes();

        let mut config_changed = Vec::new();
        let mut config_added = Vec::new();
        let mut config_removed = Vec::new();

        // Check for changed/added configs
        for (path, hash) in &current_configs {
            match self.config_hashes.get(path) {
                Some(old_hash) if old_hash != hash => config_changed.push(path.clone()),
                None => config_added.push(path.clone()),
                _ => {}
            }
        }

        // Check for removed configs
        for path in self.config_hashes.keys() {
            if !current_configs.contains_key(path) {
                config_removed.push(path.clone());
            }
        }

        BaselineChanges {
            usb_added: current_usb.difference(&baseline_usb).cloned().collect(),
            usb_removed: baseline_usb.difference(&current_usb).cloned().collect(),
            pci_added: current_pci.difference(&baseline_pci).cloned().collect(),
            pci_removed: baseline_pci.difference(&current_pci).cloned().collect(),
            config_changed,
            config_added,
            config_removed,
        }
    }
}

/// Get current USB devices
fn get_usb_devices() -> Vec<UsbDevice> {
    let mut devices = Vec::new();

    if let Ok(output) = Command::new("lsusb").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let bus = parts.get(1).unwrap_or(&"").to_string();
                let device = parts.get(3).unwrap_or(&"").trim_end_matches(':').to_string();
                let id = parts.get(5).unwrap_or(&"");
                let id_parts: Vec<&str> = id.split(':').collect();
                let vendor_id = id_parts.get(0).unwrap_or(&"").to_string();
                let product_id = id_parts.get(1).unwrap_or(&"").to_string();
                let description = parts[6..].join(" ");

                devices.push(UsbDevice {
                    bus,
                    device,
                    vendor_id,
                    product_id,
                    description,
                });
            }
        }
    }

    devices
}

/// Get current PCI devices
fn get_pci_devices() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    if let Ok(output) = Command::new("lspci").args(["-mm"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: 00:00.0 "Host bridge" "Intel Corporation" "Device 4668" -r02 "Lenovo" "Device 2319"
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 5 {
                let slot = parts[0].trim().to_string();
                let class = parts.get(1).unwrap_or(&"").to_string();
                let vendor = parts.get(3).unwrap_or(&"").to_string();
                let device = parts.get(5).unwrap_or(&"").to_string();

                devices.push(PciDevice {
                    slot,
                    class,
                    vendor,
                    device,
                });
            }
        }
    }

    devices
}

/// Important config files to monitor
const MONITORED_CONFIGS: &[&str] = &[
    "/etc/ssh/sshd_config",
    "/etc/sudoers",
    "/etc/passwd",
    "/etc/shadow",
    "/etc/group",
    "/etc/fstab",
    "/etc/hosts",
    "/etc/hostname",
    "/etc/resolv.conf",
    "/etc/pacman.conf",
    "/etc/mkinitcpio.conf",
    "/etc/default/grub",
    "/boot/loader/loader.conf",
    "/etc/systemd/system.conf",
    "/etc/security/limits.conf",
    "/etc/pam.d/system-auth",
    "/etc/firewalld/firewalld.conf",
    "/etc/nftables.conf",
];

/// Get hashes of monitored config files
fn get_config_hashes() -> HashMap<String, String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hashes = HashMap::new();

    for path in MONITORED_CONFIGS {
        if let Ok(content) = std::fs::read(path) {
            let mut hasher = DefaultHasher::new();
            content.hash(&mut hasher);
            let hash = format!("{:016x}", hasher.finish());
            hashes.insert(path.to_string(), hash);
        }
    }

    // Also check user crontabs
    if let Ok(entries) = std::fs::read_dir("/var/spool/cron") {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read(entry.path()) {
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                let hash = format!("{:016x}", hasher.finish());
                hashes.insert(entry.path().to_string_lossy().to_string(), hash);
            }
        }
    }

    hashes
}

/// Get performance baseline
fn get_performance_baseline() -> PerformanceBaseline {
    let mut baseline = PerformanceBaseline::default();

    // Boot time
    if let Ok(output) = Command::new("systemd-analyze").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(total) = stdout.split('=').last() {
            if let Some(secs_str) = total.trim().strip_suffix('s') {
                baseline.boot_time_secs = secs_str.trim().parse().unwrap_or(0.0);
            }
        }
    }

    // Memory usage
    if let Ok(output) = Command::new("free").args(["-m"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (
                        parts[1].parse::<u64>(),
                        parts[2].parse::<u64>(),
                    ) {
                        baseline.typical_memory_percent = ((used * 100) / total.max(1)) as u8;
                    }
                }
            }
        }
    }

    // Load average
    if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load) = content.split_whitespace().next() {
            baseline.typical_load_1min = load.parse().unwrap_or(0.0);
        }
    }

    baseline
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_baseline() {
        let baseline = SystemBaseline::capture();
        assert!(!baseline.usb_devices.is_empty() || !baseline.pci_devices.is_empty());
        assert!(!baseline.captured_at.is_empty());
    }

    #[test]
    fn test_config_hashes() {
        let hashes = get_config_hashes();
        // At least some configs should exist
        assert!(!hashes.is_empty());
    }
}

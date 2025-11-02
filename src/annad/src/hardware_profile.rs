// Anna v0.12.3 - Hardware Profile Models and Collector
// Semantic hardware fingerprinting for Arch Linux systems

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Hardware profile root structure matching schema v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub version: String,
    pub generated_at: String, // RFC3339
    pub kernel: String,
    pub board: BoardInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub battery: BatteryInfo,
    pub gpus: Vec<GpuInfo>,
    pub network: Vec<NetworkDevice>,
    pub storage: StorageInfo,
    pub usb: Vec<UsbDevice>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardInfo {
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub bios_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: Option<String>,
    pub sockets: Option<u32>,
    pub cores_total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_gb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub present: bool,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: Option<String>,
    pub driver: Option<String>,
    pub vram_mb: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub class: String, // ethernet, wireless
    pub vendor: Option<String>,
    pub device: Option<String>,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub controller: Vec<StorageController>,
    pub block_devices: Vec<BlockDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageController {
    pub vendor: Option<String>,
    pub device: Option<String>,
    pub driver: Option<String>,
    #[serde(rename = "type")]
    pub controller_type: String, // nvme, sata, etc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub name: String, // sda, nvme0n1, etc
    pub model: Option<String>,
    pub size_gb: Option<f64>,
    pub rotational: Option<bool>,
    #[serde(rename = "type")]
    pub device_type: Option<String>, // ssd, hdd, nvme
    pub mounts: Vec<MountPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub mountpoint: String,
    pub fs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    pub class: Option<String>,
    pub vendor: Option<String>,
    pub device: Option<String>,
}

/// Hardware profile collector with timeout and graceful degradation
pub struct HardwareCollector {
    timeout_per_cmd: Duration,
    overall_timeout: Duration,
}

impl Default for HardwareCollector {
    fn default() -> Self {
        Self {
            timeout_per_cmd: Duration::from_secs(2),
            overall_timeout: Duration::from_secs(5),
        }
    }
}

impl HardwareCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Collect complete hardware profile with timeouts and graceful errors
    pub async fn collect(&self) -> Result<HardwareProfile> {
        let start = std::time::Instant::now();
        let mut notes = Vec::new();

        // Collect in parallel with per-command timeouts
        let kernel = self.collect_kernel().await.unwrap_or_else(|e| {
            warn!("Failed to get kernel: {}", e);
            notes.push("kernel detection failed".to_string());
            "unknown".to_string()
        });

        let board = self.collect_board_info().await.unwrap_or_else(|e| {
            debug!("Board info failed (may need root): {}", e);
            BoardInfo {
                vendor: None,
                product: None,
                bios_date: None,
            }
        });

        let cpu = self.collect_cpu_info().await.unwrap_or_else(|e| {
            warn!("CPU info failed: {}", e);
            notes.push("cpu detection failed".to_string());
            CpuInfo {
                model: None,
                sockets: None,
                cores_total: None,
            }
        });

        let memory = self.collect_memory_info().await.unwrap_or_else(|e| {
            warn!("Memory info failed: {}", e);
            notes.push("memory detection failed".to_string());
            MemoryInfo { total_gb: None }
        });

        let battery = self.collect_battery_info().await.unwrap_or_else(|e| {
            debug!("Battery detection failed: {}", e);
            BatteryInfo {
                present: false,
                count: 0,
            }
        });

        let gpus = self.collect_gpus().await.unwrap_or_else(|e| {
            warn!("GPU detection failed: {}", e);
            notes.push("gpu detection failed".to_string());
            Vec::new()
        });

        let network = self.collect_network().await.unwrap_or_else(|e| {
            warn!("Network detection failed: {}", e);
            notes.push("network detection failed".to_string());
            Vec::new()
        });

        let storage = self.collect_storage().await.unwrap_or_else(|e| {
            warn!("Storage detection failed: {}", e);
            notes.push("storage detection failed".to_string());
            StorageInfo {
                controller: Vec::new(),
                block_devices: Vec::new(),
            }
        });

        let usb = self.collect_usb().await.unwrap_or_else(|e| {
            debug!("USB detection failed: {}", e);
            Vec::new()
        });

        let elapsed = start.elapsed();
        info!("Hardware profile collected in {} ms", elapsed.as_millis());

        if elapsed > self.overall_timeout {
            notes.push("timeout".to_string());
        }

        Ok(HardwareProfile {
            version: "1".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            kernel,
            board,
            cpu,
            memory,
            battery,
            gpus,
            network,
            storage,
            usb,
            notes,
        })
    }

    async fn collect_kernel(&self) -> Result<String> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("uname").arg("-r").output(),
        )
        .await
        .context("timeout")??;

        let kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(kernel)
    }

    async fn collect_board_info(&self) -> Result<BoardInfo> {
        // Try dmidecode (may need root)
        if let Ok(output) = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("dmidecode")
                .arg("-t")
                .arg("baseboard")
                .output(),
        )
        .await
        {
            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    return Ok(parse_dmidecode_board(&stdout));
                }
            }
        }

        // Fallback to /sys/class/dmi/id
        let vendor = fs::read_to_string("/sys/class/dmi/id/board_vendor")
            .ok()
            .map(|s| s.trim().to_string());
        let product = fs::read_to_string("/sys/class/dmi/id/board_name")
            .ok()
            .map(|s| s.trim().to_string());
        let bios_date = fs::read_to_string("/sys/class/dmi/id/bios_date")
            .ok()
            .map(|s| s.trim().to_string());

        Ok(BoardInfo {
            vendor,
            product,
            bios_date,
        })
    }

    async fn collect_cpu_info(&self) -> Result<CpuInfo> {
        // Read from /proc/cpuinfo
        let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;
        let model = cpuinfo
            .lines()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string());

        // Count cores
        let cores_total = cpuinfo
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count() as u32;

        // Count physical sockets
        let mut physical_ids = std::collections::HashSet::new();
        for line in cpuinfo.lines() {
            if line.starts_with("physical id") {
                if let Some(id) = line.split(':').nth(1) {
                    physical_ids.insert(id.trim().to_string());
                }
            }
        }
        let sockets = if physical_ids.is_empty() {
            Some(1)
        } else {
            Some(physical_ids.len() as u32)
        };

        Ok(CpuInfo {
            model,
            sockets,
            cores_total: if cores_total > 0 {
                Some(cores_total)
            } else {
                None
            },
        })
    }

    async fn collect_memory_info(&self) -> Result<MemoryInfo> {
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let total_kb = meminfo
            .lines()
            .find(|line| line.starts_with("MemTotal:"))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .unwrap_or(0);

        let total_gb = if total_kb > 0 {
            Some((total_kb as f64) / 1024.0 / 1024.0)
        } else {
            None
        };

        Ok(MemoryInfo { total_gb })
    }

    async fn collect_battery_info(&self) -> Result<BatteryInfo> {
        let mut count = 0;
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let type_path = entry.path().join("type");
                if let Ok(type_str) = fs::read_to_string(&type_path) {
                    if type_str.trim().eq_ignore_ascii_case("Battery") {
                        count += 1;
                    }
                }
            }
        }

        Ok(BatteryInfo {
            present: count > 0,
            count,
        })
    }

    async fn collect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();

        // Try nvidia-smi first for NVIDIA GPUs
        if let Ok(nvidia_gpus) = self.collect_nvidia_gpus().await {
            gpus.extend(nvidia_gpus);
        }

        // Parse lspci for all GPUs (deduplicate with nvidia-smi)
        if let Ok(pci_gpus) = self.collect_pci_gpus().await {
            for pci_gpu in pci_gpus {
                // Only add if not already detected by nvidia-smi
                if !gpus.iter().any(|g| g.name == pci_gpu.name) {
                    gpus.push(pci_gpu);
                }
            }
        }

        Ok(gpus)
    }

    async fn collect_nvidia_gpus(&self) -> Result<Vec<GpuInfo>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("nvidia-smi")
                .args([
                    "--query-gpu=name,driver_version,memory.total",
                    "--format=csv,noheader",
                ])
                .output(),
        )
        .await
        .context("timeout")??;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut gpus = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 {
                let vram_mb = parts[2]
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<u64>().ok());

                gpus.push(GpuInfo {
                    name: parts[0].to_string(),
                    vendor: Some("NVIDIA".to_string()),
                    driver: Some(parts[1].to_string()),
                    vram_mb,
                    notes: Vec::new(),
                });
            }
        }

        Ok(gpus)
    }

    async fn collect_pci_gpus(&self) -> Result<Vec<GpuInfo>> {
        // Try lspci -mm first (machine-readable)
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("lspci")
                .args(["-mm", "-v"])
                .output(),
        )
        .await
        .context("timeout")??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lspci_gpus(&stdout))
    }

    async fn collect_network(&self) -> Result<Vec<NetworkDevice>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("lspci")
                .args(["-mm", "-v"])
                .output(),
        )
        .await
        .context("timeout")??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lspci_network(&stdout))
    }

    async fn collect_storage(&self) -> Result<StorageInfo> {
        // Collect controllers from lspci
        let controllers = self.collect_storage_controllers().await?;

        // Collect block devices from lsblk
        let block_devices = self.collect_block_devices().await?;

        Ok(StorageInfo {
            controller: controllers,
            block_devices,
        })
    }

    async fn collect_storage_controllers(&self) -> Result<Vec<StorageController>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("lspci")
                .args(["-mm", "-v"])
                .output(),
        )
        .await
        .context("timeout")??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lspci_storage(&stdout))
    }

    async fn collect_block_devices(&self) -> Result<Vec<BlockDevice>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("lsblk")
                .args([
                    "-J",
                    "-o",
                    "NAME,SIZE,ROTA,TYPE,TRAN,MODEL,MOUNTPOINT,FSTYPE",
                ])
                .output(),
        )
        .await
        .context("timeout")??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lsblk(&stdout))
    }

    async fn collect_usb(&self) -> Result<Vec<UsbDevice>> {
        let output = timeout(
            self.timeout_per_cmd,
            tokio::process::Command::new("lsusb").output(),
        )
        .await
        .context("timeout")??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lsusb(&stdout))
    }
}

// Parsing functions

fn parse_dmidecode_board(output: &str) -> BoardInfo {
    let mut vendor = None;
    let mut product = None;
    let mut bios_date = None;

    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Manufacturer:") {
            vendor = line.split(':').nth(1).map(|s| s.trim().to_string());
        } else if line.starts_with("Product Name:") {
            product = line.split(':').nth(1).map(|s| s.trim().to_string());
        } else if line.starts_with("Release Date:") {
            bios_date = line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }

    BoardInfo {
        vendor,
        product,
        bios_date,
    }
}

fn parse_lspci_gpus(output: &str) -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    let mut current_device: Option<HashMap<String, String>> = None;

    for line in output.lines() {
        if line.is_empty() {
            if let Some(device) = current_device.take() {
                if let Some(class) = device.get("Class") {
                    if class.starts_with("030") || class.contains("VGA") || class.contains("3D") {
                        let name = device
                            .get("Device")
                            .or_else(|| device.get("SVendor"))
                            .cloned()
                            .unwrap_or_else(|| "Unknown GPU".to_string());

                        let vendor = device.get("Vendor").cloned();
                        let driver = device.get("Driver").cloned();

                        gpus.push(GpuInfo {
                            name,
                            vendor,
                            driver,
                            vram_mb: None,
                            notes: Vec::new(),
                        });
                    }
                }
            }
            current_device = None;
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') {
            // New device line
            if let Some(device) = current_device.take() {
                // Process previous device
                if let Some(class) = device.get("Class") {
                    if class.starts_with("030") || class.contains("VGA") || class.contains("3D") {
                        let name = device
                            .get("Device")
                            .or_else(|| device.get("SVendor"))
                            .cloned()
                            .unwrap_or_else(|| "Unknown GPU".to_string());

                        let vendor = device.get("Vendor").cloned();
                        let driver = device.get("Driver").cloned();

                        gpus.push(GpuInfo {
                            name,
                            vendor,
                            driver,
                            vram_mb: None,
                            notes: Vec::new(),
                        });
                    }
                }
            }

            current_device = Some(HashMap::new());

            // Parse device line (format: "field" "value")
            let parts: Vec<&str> = line.split('"').filter(|s| !s.trim().is_empty()).collect();
            for chunk in parts.chunks(2) {
                if chunk.len() == 2 {
                    let key = chunk[0].trim();
                    let value = chunk[1].trim();
                    if let Some(dev) = current_device.as_mut() {
                        dev.insert(key.to_string(), value.to_string());
                    }
                }
            }
        } else {
            // Property line
            if let Some(dev) = current_device.as_mut() {
                if let Some((key, value)) = line.trim().split_once(':') {
                    dev.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    // Process last device
    if let Some(device) = current_device {
        if let Some(class) = device.get("Class") {
            if class.starts_with("030") || class.contains("VGA") || class.contains("3D") {
                let name = device
                    .get("Device")
                    .or_else(|| device.get("SVendor"))
                    .cloned()
                    .unwrap_or_else(|| "Unknown GPU".to_string());

                let vendor = device.get("Vendor").cloned();
                let driver = device.get("Driver").cloned();

                gpus.push(GpuInfo {
                    name,
                    vendor,
                    driver,
                    vram_mb: None,
                    notes: Vec::new(),
                });
            }
        }
    }

    gpus
}

fn parse_lspci_network(output: &str) -> Vec<NetworkDevice> {
    let mut devices = Vec::new();
    let mut current_device: Option<HashMap<String, String>> = None;

    for line in output.lines() {
        if line.is_empty() {
            if let Some(device) = current_device.take() {
                if let Some(class) = device.get("Class") {
                    if class.starts_with("020")
                        || class.contains("Network")
                        || class.contains("Ethernet")
                    {
                        let device_name = device.get("Device").cloned();
                        let is_wireless = device_name
                            .as_ref()
                            .map(|d| {
                                d.to_lowercase().contains("wireless")
                                    || d.to_lowercase().contains("wi-fi")
                            })
                            .unwrap_or(false)
                            || device
                                .get("Driver")
                                .map(|d| {
                                    d.contains("iwl")
                                        || d.contains("ath")
                                        || d.contains("brcm")
                                        || d.contains("rtw")
                                })
                                .unwrap_or(false);

                        let class = if is_wireless {
                            "wireless".to_string()
                        } else {
                            "ethernet".to_string()
                        };

                        devices.push(NetworkDevice {
                            class,
                            vendor: device.get("Vendor").cloned(),
                            device: device_name,
                            driver: device.get("Driver").cloned(),
                        });
                    }
                }
            }
            current_device = None;
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(device) = current_device.take() {
                if let Some(class) = device.get("Class") {
                    if class.starts_with("020")
                        || class.contains("Network")
                        || class.contains("Ethernet")
                    {
                        let device_name = device.get("Device").cloned();
                        let is_wireless = device_name
                            .as_ref()
                            .map(|d| {
                                d.to_lowercase().contains("wireless")
                                    || d.to_lowercase().contains("wi-fi")
                            })
                            .unwrap_or(false)
                            || device
                                .get("Driver")
                                .map(|d| {
                                    d.contains("iwl")
                                        || d.contains("ath")
                                        || d.contains("brcm")
                                        || d.contains("rtw")
                                })
                                .unwrap_or(false);

                        let class = if is_wireless {
                            "wireless".to_string()
                        } else {
                            "ethernet".to_string()
                        };

                        devices.push(NetworkDevice {
                            class,
                            vendor: device.get("Vendor").cloned(),
                            device: device_name,
                            driver: device.get("Driver").cloned(),
                        });
                    }
                }
            }

            current_device = Some(HashMap::new());

            let parts: Vec<&str> = line.split('"').filter(|s| !s.trim().is_empty()).collect();
            for chunk in parts.chunks(2) {
                if chunk.len() == 2 {
                    let key = chunk[0].trim();
                    let value = chunk[1].trim();
                    if let Some(dev) = current_device.as_mut() {
                        dev.insert(key.to_string(), value.to_string());
                    }
                }
            }
        } else {
            if let Some(dev) = current_device.as_mut() {
                if let Some((key, value)) = line.trim().split_once(':') {
                    dev.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    // Process last device
    if let Some(device) = current_device {
        if let Some(class) = device.get("Class") {
            if class.starts_with("020") || class.contains("Network") || class.contains("Ethernet") {
                let device_name = device.get("Device").cloned();
                let is_wireless = device_name
                    .as_ref()
                    .map(|d| {
                        d.to_lowercase().contains("wireless") || d.to_lowercase().contains("wi-fi")
                    })
                    .unwrap_or(false)
                    || device
                        .get("Driver")
                        .map(|d| {
                            d.contains("iwl")
                                || d.contains("ath")
                                || d.contains("brcm")
                                || d.contains("rtw")
                        })
                        .unwrap_or(false);

                let class = if is_wireless {
                    "wireless".to_string()
                } else {
                    "ethernet".to_string()
                };

                devices.push(NetworkDevice {
                    class,
                    vendor: device.get("Vendor").cloned(),
                    device: device_name,
                    driver: device.get("Driver").cloned(),
                });
            }
        }
    }

    devices
}

fn parse_lspci_storage(output: &str) -> Vec<StorageController> {
    let mut controllers = Vec::new();
    let mut current_device: Option<HashMap<String, String>> = None;

    for line in output.lines() {
        if line.is_empty() {
            if let Some(device) = current_device.take() {
                if let Some(class) = device.get("Class") {
                    if class.starts_with("010")
                        || class.contains("SATA")
                        || class.contains("NVMe")
                        || class.contains("RAID")
                    {
                        let device_name = device.get("Device").cloned();
                        let controller_type = if device_name
                            .as_ref()
                            .map(|d| d.to_lowercase().contains("nvme"))
                            .unwrap_or(false)
                        {
                            "nvme".to_string()
                        } else if device_name
                            .as_ref()
                            .map(|d| d.to_lowercase().contains("sata"))
                            .unwrap_or(false)
                        {
                            "sata".to_string()
                        } else if device_name
                            .as_ref()
                            .map(|d| d.to_lowercase().contains("raid"))
                            .unwrap_or(false)
                        {
                            "raid".to_string()
                        } else {
                            "unknown".to_string()
                        };

                        controllers.push(StorageController {
                            vendor: device.get("Vendor").cloned(),
                            device: device_name,
                            driver: device.get("Driver").cloned(),
                            controller_type,
                        });
                    }
                }
            }
            current_device = None;
            continue;
        }

        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(device) = current_device.take() {
                if let Some(class) = device.get("Class") {
                    if class.starts_with("010")
                        || class.contains("SATA")
                        || class.contains("NVMe")
                        || class.contains("RAID")
                    {
                        let device_name = device.get("Device").cloned();
                        let controller_type = if device_name
                            .as_ref()
                            .map(|d| d.to_lowercase().contains("nvme"))
                            .unwrap_or(false)
                        {
                            "nvme".to_string()
                        } else if device_name
                            .as_ref()
                            .map(|d| d.to_lowercase().contains("sata"))
                            .unwrap_or(false)
                        {
                            "sata".to_string()
                        } else if device_name
                            .as_ref()
                            .map(|d| d.to_lowercase().contains("raid"))
                            .unwrap_or(false)
                        {
                            "raid".to_string()
                        } else {
                            "unknown".to_string()
                        };

                        controllers.push(StorageController {
                            vendor: device.get("Vendor").cloned(),
                            device: device_name,
                            driver: device.get("Driver").cloned(),
                            controller_type,
                        });
                    }
                }
            }

            current_device = Some(HashMap::new());

            let parts: Vec<&str> = line.split('"').filter(|s| !s.trim().is_empty()).collect();
            for chunk in parts.chunks(2) {
                if chunk.len() == 2 {
                    let key = chunk[0].trim();
                    let value = chunk[1].trim();
                    if let Some(dev) = current_device.as_mut() {
                        dev.insert(key.to_string(), value.to_string());
                    }
                }
            }
        } else {
            if let Some(dev) = current_device.as_mut() {
                if let Some((key, value)) = line.trim().split_once(':') {
                    dev.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    // Process last device
    if let Some(device) = current_device {
        if let Some(class) = device.get("Class") {
            if class.starts_with("010")
                || class.contains("SATA")
                || class.contains("NVMe")
                || class.contains("RAID")
            {
                let device_name = device.get("Device").cloned();
                let controller_type = if device_name
                    .as_ref()
                    .map(|d| d.to_lowercase().contains("nvme"))
                    .unwrap_or(false)
                {
                    "nvme".to_string()
                } else if device_name
                    .as_ref()
                    .map(|d| d.to_lowercase().contains("sata"))
                    .unwrap_or(false)
                {
                    "sata".to_string()
                } else if device_name
                    .as_ref()
                    .map(|d| d.to_lowercase().contains("raid"))
                    .unwrap_or(false)
                {
                    "raid".to_string()
                } else {
                    "unknown".to_string()
                };

                controllers.push(StorageController {
                    vendor: device.get("Vendor").cloned(),
                    device: device_name,
                    driver: device.get("Driver").cloned(),
                    controller_type,
                });
            }
        }
    }

    controllers
}

fn parse_lsblk(output: &str) -> Vec<BlockDevice> {
    #[derive(Debug, Deserialize)]
    struct LsblkOutput {
        blockdevices: Vec<LsblkDevice>,
    }

    #[derive(Debug, Deserialize)]
    struct LsblkDevice {
        name: String,
        size: Option<String>,
        rota: Option<String>,
        #[serde(rename = "type")]
        device_type: Option<String>,
        tran: Option<String>,
        model: Option<String>,
        mountpoint: Option<String>,
        fstype: Option<String>,
        children: Option<Vec<LsblkDevice>>,
    }

    let parsed: LsblkOutput = match serde_json::from_str(output) {
        Ok(p) => p,
        Err(e) => {
            debug!("Failed to parse lsblk JSON: {}", e);
            return Vec::new();
        }
    };

    let mut devices = Vec::new();

    fn process_device(dev: &LsblkDevice, devices: &mut Vec<BlockDevice>) {
        // Only process disk types
        if dev.device_type.as_deref() == Some("disk") {
            let size_gb = dev.size.as_ref().and_then(|s| parse_size_to_gb(s));

            let rotational = dev.rota.as_ref().and_then(|r| match r.as_str() {
                "1" => Some(true),
                "0" => Some(false),
                _ => None,
            });

            let device_type = classify_device_type(&dev.name, rotational, dev.tran.as_deref());

            let mut mounts = Vec::new();
            if let Some(mp) = &dev.mountpoint {
                mounts.push(MountPoint {
                    mountpoint: mp.clone(),
                    fs: dev.fstype.clone(),
                });
            }

            // Collect mounts from children
            if let Some(children) = &dev.children {
                for child in children {
                    if let Some(mp) = &child.mountpoint {
                        mounts.push(MountPoint {
                            mountpoint: mp.clone(),
                            fs: child.fstype.clone(),
                        });
                    }
                }
            }

            devices.push(BlockDevice {
                name: dev.name.clone(),
                model: dev.model.clone().map(|s| s.trim().to_string()),
                size_gb,
                rotational,
                device_type,
                mounts,
            });
        }

        // Recurse into children
        if let Some(children) = &dev.children {
            for child in children {
                process_device(child, devices);
            }
        }
    }

    for dev in parsed.blockdevices {
        process_device(&dev, &mut devices);
    }

    devices
}

fn parse_size_to_gb(size_str: &str) -> Option<f64> {
    let size_str = size_str.trim().to_uppercase();
    let (num_str, unit) = size_str.split_at(size_str.len().saturating_sub(1));

    let num: f64 = num_str.trim().parse().ok()?;

    let multiplier = match unit {
        "T" => 1024.0,
        "G" => 1.0,
        "M" => 1.0 / 1024.0,
        "K" => 1.0 / (1024.0 * 1024.0),
        _ => {
            // Try parsing without unit (assume bytes)
            if let Ok(bytes) = size_str.parse::<f64>() {
                return Some(bytes / (1024.0 * 1024.0 * 1024.0));
            }
            return None;
        }
    };

    Some(num * multiplier)
}

fn classify_device_type(
    name: &str,
    rotational: Option<bool>,
    tran: Option<&str>,
) -> Option<String> {
    if name.starts_with("nvme") {
        return Some("nvme".to_string());
    }

    if tran == Some("nvme") {
        return Some("nvme".to_string());
    }

    match rotational {
        Some(true) => Some("hdd".to_string()),
        Some(false) => Some("ssd".to_string()),
        None => None,
    }
}

fn parse_lsusb(output: &str) -> Vec<UsbDevice> {
    let mut devices = Vec::new();

    for line in output.lines() {
        // Format: Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
        if let Some(after_id) = line.split("ID ").nth(1) {
            let parts: Vec<&str> = after_id.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let ids: Vec<&str> = parts[0].split(':').collect();
                let description = parts[1].trim();

                devices.push(UsbDevice {
                    class: None,
                    vendor: ids.get(0).map(|s| s.to_string()),
                    device: Some(description.to_string()),
                });
            }
        }
    }

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_to_gb() {
        assert_eq!(parse_size_to_gb("1T"), Some(1024.0));
        assert_eq!(parse_size_to_gb("500G"), Some(500.0));
        assert_eq!(parse_size_to_gb("2048M"), Some(2.0));
    }

    #[test]
    fn test_classify_device_type() {
        assert_eq!(
            classify_device_type("nvme0n1", Some(false), None),
            Some("nvme".to_string())
        );
        assert_eq!(
            classify_device_type("sda", Some(true), None),
            Some("hdd".to_string())
        );
        assert_eq!(
            classify_device_type("sdb", Some(false), None),
            Some("ssd".to_string())
        );
    }

    #[test]
    fn test_parse_lsblk() {
        let json = r#"{
            "blockdevices": [
                {"name":"nvme0n1","size":"1T","rota":"0","type":"disk","tran":"nvme","model":"Samsung SSD 980 PRO","mountpoint":null,"fstype":null,"children":[
                    {"name":"nvme0n1p1","size":"512M","rota":"0","type":"part","tran":null,"model":null,"mountpoint":"/boot","fstype":"vfat","children":null}
                ]}
            ]
        }"#;

        let devices = parse_lsblk(json);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "nvme0n1");
        assert_eq!(devices[0].device_type, Some("nvme".to_string()));
        assert_eq!(devices[0].mounts.len(), 1);
        assert_eq!(devices[0].mounts[0].mountpoint, "/boot");
    }
}

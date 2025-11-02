// Anna v0.12.3 - Hardware Profile CLI Command

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/anna/annad.sock";

/// Hardware profile from daemon (matches JSON schema v1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub version: String,
    pub generated_at: String,
    pub kernel: String,
    pub board: BoardInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub battery: BatteryInfo,
    pub gpus: Vec<GpuInfo>,
    pub network: Vec<NetworkDevice>,
    pub storage: StorageInfo,
    pub usb: Vec<UsbDevice>,
    #[serde(default)]
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
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub class: String,
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
    pub controller_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub name: String,
    pub model: Option<String>,
    pub size_gb: Option<f64>,
    pub rotational: Option<bool>,
    #[serde(rename = "type")]
    pub device_type: Option<String>,
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

/// Execute `annactl hw show` command
pub async fn show_hardware(json: bool, wide: bool) -> Result<()> {
    let profile = fetch_hardware_profile().await?;

    if json {
        // JSON output
        let json_str = serde_json::to_string_pretty(&profile)?;
        println!("{}", json_str);
    } else {
        // Human-readable output
        print_hardware_human(&profile, wide);
    }

    Ok(())
}

/// Fetch hardware profile from daemon via RPC
async fn fetch_hardware_profile() -> Result<HardwareProfile> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .await
        .context("Failed to connect to annad.sock - is annad running?")?;

    // Send JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "hardware_profile",
        "params": {},
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    // Read response
    let (reader, _writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let response_line = lines
        .next_line()
        .await?
        .context("No response from daemon")?;

    // Parse JSON-RPC response
    let response: JsonValue = serde_json::from_str(&response_line)?;

    if let Some(error) = response.get("error") {
        anyhow::bail!("RPC error: {}", error);
    }

    let result = response
        .get("result")
        .context("No result in RPC response")?;

    let profile: HardwareProfile = serde_json::from_value(result.clone())?;

    Ok(profile)
}

/// Print hardware profile in human-readable format
fn print_hardware_human(profile: &HardwareProfile, wide: bool) {
    println!("Hardware Profile");
    println!("────────────────────────────────────────");

    // CPU
    if let Some(model) = &profile.cpu.model {
        let cores = profile
            .cpu
            .cores_total
            .map(|c| format!("{} cores", c))
            .unwrap_or_else(|| "".to_string());
        println!("CPU: {} {}", model, cores);
    } else {
        println!("CPU: Unknown");
    }

    // Memory
    if let Some(total_gb) = profile.memory.total_gb {
        println!("Memory: {:.1} GB", total_gb);
    } else {
        println!("Memory: Unknown");
    }

    // GPUs
    if !profile.gpus.is_empty() {
        for gpu in &profile.gpus {
            let driver = gpu
                .driver
                .as_ref()
                .map(|d| format!("driver {}", d))
                .unwrap_or_else(|| "no driver".to_string());
            let vram = gpu
                .vram_mb
                .map(|v| format!("vram {} MB", v))
                .unwrap_or_else(|| "".to_string());
            println!(
                "GPU: {} {} {}",
                gpu.name,
                driver,
                if vram.is_empty() { "" } else { &vram }
            );
        }
    } else {
        println!("GPU: None detected");
    }

    // Storage summary
    if !profile.storage.block_devices.is_empty() {
        print!("Storage: ");
        let storage_parts: Vec<String> = profile
            .storage
            .block_devices
            .iter()
            .map(|d| {
                let size = d
                    .size_gb
                    .map(|s| format!("{:.0} GB", s))
                    .unwrap_or_else(|| "unknown".to_string());
                let dev_type = d
                    .device_type
                    .as_ref()
                    .map(|t| t.to_uppercase())
                    .unwrap_or_else(|| "unknown".to_string());
                let model = d
                    .model
                    .as_ref()
                    .map(|m| format!(" {}", m))
                    .unwrap_or_else(|| "".to_string());
                format!("{} {} {}{}", d.name, size, dev_type, model)
            })
            .collect();
        println!("{}", storage_parts.join(", "));
    } else {
        println!("Storage: None detected");
    }

    // Network summary
    if !profile.network.is_empty() {
        print!("Network: ");
        let net_parts: Vec<String> = profile
            .network
            .iter()
            .map(|n| {
                let device_name = n.device.as_ref().map(|d| d.as_str()).unwrap_or("unknown");
                let driver = n
                    .driver
                    .as_ref()
                    .map(|d| format!("driver {}", d))
                    .unwrap_or_else(|| "".to_string());
                format!("{} {} {}", device_name, n.class, driver)
            })
            .collect();
        println!("{}", net_parts.join(", "));
    } else {
        println!("Network: None detected");
    }

    // Battery
    println!(
        "Battery: {}{}",
        if profile.battery.present { "yes" } else { "no" },
        if profile.battery.count > 0 {
            format!(" (count {})", profile.battery.count)
        } else {
            "".to_string()
        }
    );

    // Kernel
    println!("Kernel: {}", profile.kernel);

    // Board info (if available)
    if profile.board.vendor.is_some() || profile.board.product.is_some() {
        let vendor = profile.board.vendor.as_deref().unwrap_or("unknown");
        let product = profile.board.product.as_deref().unwrap_or("unknown");
        println!("Board: {} {}", vendor, product);
    }

    // Wide mode: show detailed device listings
    if wide {
        println!("\n────────────────────────────────────────");
        println!("Detailed Hardware Information");
        println!("────────────────────────────────────────");

        // Storage controllers
        if !profile.storage.controller.is_empty() {
            println!("\nStorage Controllers:");
            for ctrl in &profile.storage.controller {
                let device = ctrl.device.as_deref().unwrap_or("unknown");
                let driver = ctrl.driver.as_deref().unwrap_or("no driver");
                println!("  {} ({}) driver: {}", device, ctrl.controller_type, driver);
            }
        }

        // Block devices with mounts
        if !profile.storage.block_devices.is_empty() {
            println!("\nBlock Devices:");
            for dev in &profile.storage.block_devices {
                let size = dev
                    .size_gb
                    .map(|s| format!("{:.1} GB", s))
                    .unwrap_or_else(|| "unknown".to_string());
                let dev_type = dev.device_type.as_deref().unwrap_or("unknown");
                println!(
                    "  {} - {} {} {}",
                    dev.name,
                    size,
                    dev_type,
                    dev.model.as_deref().unwrap_or("")
                );
                for mount in &dev.mounts {
                    let fs = mount.fs.as_deref().unwrap_or("unknown");
                    println!("    └─ {} ({})", mount.mountpoint, fs);
                }
            }
        }

        // USB devices
        if !profile.usb.is_empty() {
            println!("\nUSB Devices: {} total", profile.usb.len());
            // Don't show all USB devices by default as there are usually many
            if profile.usb.len() <= 10 {
                for usb in &profile.usb {
                    let device = usb.device.as_deref().unwrap_or("unknown");
                    println!("  {}", device);
                }
            }
        }
    }

    // Notes (warnings/errors)
    if !profile.notes.is_empty() {
        println!("\nNotes: {}", profile.notes.join(", "));
    }
}

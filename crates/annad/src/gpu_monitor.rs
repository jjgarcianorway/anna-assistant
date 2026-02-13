//! GPU monitoring: vendor detection, temperature, utilization, VRAM.
//!
//! Supports: NVIDIA (nvidia-smi), AMD (/sys/class/drm/), Intel (/sys/class/drm/).
//! Pure data collection — advice generated via LLM + wiki in system_learner.

use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::debug;

/// GPU vendor
#[derive(Debug, Clone, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown(String),
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "nvidia"),
            GpuVendor::Amd => write!(f, "amd"),
            GpuVendor::Intel => write!(f, "intel"),
            GpuVendor::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Snapshot of GPU state
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    /// GPU temperature in Celsius
    pub temp_celsius: Option<f32>,
    /// GPU utilization percentage (0-100)
    pub utilization_pct: Option<u8>,
    /// VRAM used / total (MiB)
    pub vram_used_mib: Option<u64>,
    pub vram_total_mib: Option<u64>,
    /// Driver version
    pub driver_version: Option<String>,
    /// Power draw in Watts
    pub power_draw_w: Option<f32>,
}

impl GpuInfo {
    /// Detect all GPUs on the system.
    pub fn detect_all() -> Vec<Self> {
        let mut gpus = Vec::new();

        // NVIDIA: use nvidia-smi
        if let Some(nvidia_gpus) = query_nvidia() {
            gpus.extend(nvidia_gpus);
        }

        // AMD / Intel: use /sys/class/drm/
        gpus.extend(query_drm());

        gpus
    }

    /// Summary line for briefing/context
    pub fn summary(&self) -> String {
        let mut parts = vec![format!("[{}] {}", self.vendor, self.name)];
        if let Some(t) = self.temp_celsius {
            parts.push(format!("{:.0}°C", t));
        }
        if let Some(u) = self.utilization_pct {
            parts.push(format!("{}% util", u));
        }
        if let (Some(used), Some(total)) = (self.vram_used_mib, self.vram_total_mib) {
            parts.push(format!("VRAM {}/{}MiB", used, total));
        }
        if let Some(p) = self.power_draw_w {
            parts.push(format!("{:.0}W", p));
        }
        parts.join(" | ")
    }

    /// Temperature above warning threshold (90°C for GPU)
    pub fn is_hot(&self) -> bool {
        self.temp_celsius.map(|t| t >= 90.0).unwrap_or(false)
    }
}

fn query_nvidia() -> Option<Vec<GpuInfo>> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total,driver_version,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 7 {
            continue;
        }

        let name = parts[0].to_string();
        let temp_celsius = parts[1].parse::<f32>().ok();
        let utilization_pct = parts[2].parse::<u8>().ok();
        let vram_used_mib = parts[3].parse::<u64>().ok();
        let vram_total_mib = parts[4].parse::<u64>().ok();
        let driver_version = Some(parts[5].to_string()).filter(|s| !s.is_empty());
        let power_draw_w = parts[6].parse::<f32>().ok();

        debug!("NVIDIA GPU: {} temp={:?}°C util={:?}%", name, temp_celsius, utilization_pct);

        gpus.push(GpuInfo {
            vendor: GpuVendor::Nvidia,
            name,
            temp_celsius,
            utilization_pct,
            vram_used_mib,
            vram_total_mib,
            driver_version,
            power_draw_w,
        });
    }

    if gpus.is_empty() { None } else { Some(gpus) }
}

fn query_drm() -> Vec<GpuInfo> {
    let base = Path::new("/sys/class/drm");
    if !base.exists() {
        return vec![];
    }

    let mut gpus = Vec::new();
    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return gpus,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        // Only card entries, not connectors
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }

        let card_path = entry.path();
        let device_path = card_path.join("device");
        if !device_path.exists() {
            continue;
        }

        let vendor = read_drm_vendor(&device_path);
        let gpu_name = read_drm_name(&device_path).unwrap_or_else(|| name.clone());
        let temp_celsius = read_drm_temp(&device_path);
        let (vram_used_mib, vram_total_mib) = read_drm_vram(&device_path);

        debug!("DRM GPU: {} vendor={:?} temp={:?}°C", gpu_name, vendor, temp_celsius);

        gpus.push(GpuInfo {
            vendor,
            name: gpu_name,
            temp_celsius,
            utilization_pct: None, // drm doesn't expose this easily without hwmon
            vram_used_mib,
            vram_total_mib,
            driver_version: None,
            power_draw_w: read_drm_power(&device_path),
        });
    }

    gpus
}

fn read_drm_vendor(device_path: &Path) -> GpuVendor {
    let vendor_id = fs::read_to_string(device_path.join("vendor"))
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    match vendor_id.as_str() {
        "0x10de" => GpuVendor::Nvidia,
        "0x1002" => GpuVendor::Amd,
        "0x8086" => GpuVendor::Intel,
        other => GpuVendor::Unknown(other.to_string()),
    }
}

fn read_drm_name(device_path: &Path) -> Option<String> {
    // Try uevent for device name
    let uevent = fs::read_to_string(device_path.join("uevent")).ok()?;
    uevent.lines()
        .find(|l| l.starts_with("PCI_ID=") || l.starts_with("DRIVER="))
        .map(|l| l.splitn(2, '=').nth(1).unwrap_or("").to_string())
}

fn read_drm_temp(device_path: &Path) -> Option<f32> {
    // AMD: hwmon under device/hwmon/hwmon*/temp1_input (millidegrees)
    let hwmon_base = device_path.join("hwmon");
    let entries = fs::read_dir(&hwmon_base).ok()?;
    for entry in entries.flatten() {
        let temp_file = entry.path().join("temp1_input");
        if let Ok(raw) = fs::read_to_string(&temp_file) {
            if let Ok(millideg) = raw.trim().parse::<i32>() {
                return Some(millideg as f32 / 1000.0);
            }
        }
    }
    None
}

fn read_drm_vram(device_path: &Path) -> (Option<u64>, Option<u64>) {
    // AMD exposes mem_info_vram_used and mem_info_vram_total
    let used = fs::read_to_string(device_path.join("mem_info_vram_used"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|b| b / 1024 / 1024); // bytes → MiB
    let total = fs::read_to_string(device_path.join("mem_info_vram_total"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|b| b / 1024 / 1024);
    (used, total)
}

fn read_drm_power(device_path: &Path) -> Option<f32> {
    // AMD hwmon: power1_average in microwatts
    let hwmon_base = device_path.join("hwmon");
    let entries = fs::read_dir(&hwmon_base).ok()?;
    for entry in entries.flatten() {
        let power_file = entry.path().join("power1_average");
        if let Ok(raw) = fs::read_to_string(&power_file) {
            if let Ok(uw) = raw.trim().parse::<u64>() {
                return Some(uw as f32 / 1_000_000.0);
            }
        }
    }
    None
}

/// Build a telemetry section for briefing injection.
pub fn gpu_telemetry() -> String {
    let gpus = GpuInfo::detect_all();
    if gpus.is_empty() {
        return String::new();
    }
    let mut out = "## GPU\n".to_string();
    for gpu in &gpus {
        out.push_str(&format!("{}\n", gpu.summary()));
        if gpu.is_hot() {
            out.push_str(&format!("ALERT: {} temperature critically high ({:.0}°C)\n",
                gpu.name, gpu.temp_celsius.unwrap_or(0.0)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_all_no_panic() {
        let gpus = GpuInfo::detect_all();
        for gpu in &gpus {
            let _ = gpu.summary();
            let _ = gpu.is_hot();
        }
    }

    #[test]
    fn test_vendor_display() {
        assert_eq!(GpuVendor::Nvidia.to_string(), "nvidia");
        assert_eq!(GpuVendor::Amd.to_string(), "amd");
    }
}

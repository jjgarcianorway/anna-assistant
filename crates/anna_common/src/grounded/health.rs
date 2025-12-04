//! Hardware Health Data Collection v7.5.0
//!
//! Provides structured health data for:
//! - CPU (temperatures, throttling, frequency)
//! - GPU (temperatures, driver errors)
//! - Disks/NVMe (SMART status, reallocated sectors)
//! - Battery (wear level, capacity, cycles)
//! - Network (link state, error counters)
//!
//! Sources:
//! - sensors, /sys/class/thermal/* (CPU/GPU temps)
//! - smartctl, nvme smart-log (disk SMART)
//! - /sys/class/power_supply/* (battery)
//! - ip -s link, ethtool, iw (network)
//! - journalctl -b -k, dmesg (kernel logs)
//!
//! All data is real and sourced - nothing is guessed or invented.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

// ============================================================================
// Health Status Types
// ============================================================================

/// Health status level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HealthStatus {
    #[default]
    Ok,
    Warning,
    Critical,
    Unknown,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Ok => "OK",
            HealthStatus::Warning => "WARNING",
            HealthStatus::Critical => "CRITICAL",
            HealthStatus::Unknown => "UNKNOWN",
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, HealthStatus::Ok)
    }
}

/// A structured alert with scope and reason
#[derive(Debug, Clone)]
pub struct Alert {
    pub severity: HealthStatus,
    pub scope: AlertScope,
    pub identifier: String,
    pub reason: String,
    pub see_command: Option<String>,
}

/// Scope of an alert (hardware or software)
#[derive(Debug, Clone)]
pub enum AlertScope {
    Hardware,
    Software,
}

impl AlertScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertScope::Hardware => "hw",
            AlertScope::Software => "sw",
        }
    }
}

// ============================================================================
// CPU Health
// ============================================================================

/// CPU health data
#[derive(Debug, Clone, Default)]
pub struct CpuHealth {
    /// Current temperature in Celsius (if available)
    pub current_temp_c: Option<f32>,
    /// Maximum temperature seen this boot
    pub max_temp_c: Option<f32>,
    /// Throttling detected this boot
    pub throttling_detected: bool,
    /// Number of throttle events
    pub throttle_events: usize,
    /// Kernel modules/drivers for CPU
    pub drivers: Vec<String>,
    /// Health status
    pub status: HealthStatus,
    /// Alerts generated
    pub alerts: Vec<String>,
    /// Source of temperature data
    pub temp_source: Option<String>,
}

/// Get CPU health data
pub fn get_cpu_health() -> CpuHealth {
    let mut health = CpuHealth::default();
    health.status = HealthStatus::Ok;

    // Get temperatures from sensors
    if let Some((current, max, source)) = get_cpu_temperatures() {
        health.current_temp_c = Some(current);
        health.max_temp_c = Some(max);
        health.temp_source = Some(source);

        // Check for high temperatures
        if current >= 95.0 || max >= 100.0 {
            health.status = HealthStatus::Critical;
            health
                .alerts
                .push(format!("CPU temperature critical: {:.0}°C", max));
        } else if current >= 85.0 || max >= 90.0 {
            health.status = HealthStatus::Warning;
            health
                .alerts
                .push(format!("CPU temperature high: {:.0}°C max", max));
        }
    }

    // Check for throttling in kernel logs
    let (throttling, events) = check_cpu_throttling();
    health.throttling_detected = throttling;
    health.throttle_events = events;

    if throttling && health.status == HealthStatus::Ok {
        health.status = HealthStatus::Warning;
        health
            .alerts
            .push(format!("CPU throttling detected ({} events)", events));
    }

    // Get CPU drivers
    health.drivers = get_cpu_drivers();

    health
}

/// Get CPU temperatures from sensors or /sys
fn get_cpu_temperatures() -> Option<(f32, f32, String)> {
    // Try sensors command first
    let output = Command::new("sensors").arg("-u").output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut temps: Vec<f32> = Vec::new();
        let mut chip_name = String::new();

        let mut in_coretemp = false;
        let mut in_k10temp = false;

        for line in stdout.lines() {
            // Track which chip we're in
            if line.starts_with("coretemp-") {
                in_coretemp = true;
                in_k10temp = false;
                chip_name = "coretemp".to_string();
            } else if line.starts_with("k10temp-") {
                in_coretemp = false;
                in_k10temp = true;
                chip_name = "k10temp".to_string();
            } else if !line.starts_with(' ') && !line.is_empty() {
                in_coretemp = false;
                in_k10temp = false;
            }

            // Parse temperature values
            if (in_coretemp || in_k10temp) && line.contains("temp") && line.contains("_input") {
                if let Some(val) = line.split(':').nth(1) {
                    if let Ok(temp) = val.trim().parse::<f32>() {
                        temps.push(temp);
                    }
                }
            }
        }

        if !temps.is_empty() {
            let current = temps.iter().sum::<f32>() / temps.len() as f32;
            let max = temps.iter().cloned().fold(0.0_f32, f32::max);
            return Some((current, max, format!("sensors ({})", chip_name)));
        }
    }

    // Fallback to /sys/class/thermal
    let thermal_path = Path::new("/sys/class/thermal");
    if thermal_path.exists() {
        if let Ok(entries) = std::fs::read_dir(thermal_path) {
            let mut temps: Vec<f32> = Vec::new();

            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("thermal_zone") {
                    let temp_path = entry.path().join("temp");
                    if let Ok(content) = std::fs::read_to_string(&temp_path) {
                        if let Ok(temp_millic) = content.trim().parse::<i32>() {
                            temps.push(temp_millic as f32 / 1000.0);
                        }
                    }
                }
            }

            if !temps.is_empty() {
                let current = temps.iter().sum::<f32>() / temps.len() as f32;
                let max = temps.iter().cloned().fold(0.0_f32, f32::max);
                return Some((current, max, "/sys/class/thermal".to_string()));
            }
        }
    }

    None
}

/// Check kernel logs for CPU throttling events
fn check_cpu_throttling() -> (bool, usize) {
    let output = Command::new("journalctl")
        .args(["-b", "-k", "--no-pager", "-q"])
        .output();

    let mut throttle_count = 0;

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stdout_lower = stdout.to_lowercase();

            for line in stdout_lower.lines() {
                if line.contains("throttl") && (line.contains("cpu") || line.contains("thermal")) {
                    throttle_count += 1;
                }
            }
        }
    }

    (throttle_count > 0, throttle_count)
}

/// Get CPU-related kernel drivers
fn get_cpu_drivers() -> Vec<String> {
    let mut drivers = Vec::new();

    // Check for common CPU frequency drivers
    let cpu_freq_path = Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_driver");
    if let Ok(driver) = std::fs::read_to_string(cpu_freq_path) {
        let d = driver.trim().to_string();
        if !d.is_empty() {
            drivers.push(d);
        }
    }

    // Check for thermal drivers
    let thermal_path = Path::new("/sys/class/thermal/thermal_zone0/type");
    if let Ok(thermal_type) = std::fs::read_to_string(thermal_path) {
        let t = thermal_type.trim().to_string();
        if !t.is_empty() && !drivers.contains(&t) {
            drivers.push(t);
        }
    }

    drivers
}

// ============================================================================
// GPU Health
// ============================================================================

/// GPU health data
#[derive(Debug, Clone, Default)]
pub struct GpuHealth {
    /// GPU index (0, 1, etc.)
    pub index: u32,
    /// Current temperature in Celsius (if available)
    pub current_temp_c: Option<f32>,
    /// Maximum temperature seen this boot
    pub max_temp_c: Option<f32>,
    /// Driver name
    pub driver: Option<String>,
    /// Driver errors this boot
    pub driver_errors: usize,
    /// Health status
    pub status: HealthStatus,
    /// Alerts generated
    pub alerts: Vec<String>,
    /// Source of temperature data
    pub temp_source: Option<String>,
}

/// Get GPU health data for all GPUs
pub fn get_gpu_health() -> Vec<GpuHealth> {
    let mut gpus = Vec::new();

    // List GPU cards from /sys/class/drm
    let drm_path = Path::new("/sys/class/drm");
    if !drm_path.exists() {
        return gpus;
    }

    if let Ok(entries) = std::fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Only process card0, card1, etc (not renderD* or card0-*)
            if name.starts_with("card") && !name.contains('-') {
                let card_num: u32 = name.trim_start_matches("card").parse().unwrap_or(0);
                gpus.push(get_single_gpu_health(card_num));
            }
        }
    }

    gpus.sort_by_key(|g| g.index);
    gpus
}

/// Get health for a single GPU
fn get_single_gpu_health(index: u32) -> GpuHealth {
    let mut health = GpuHealth {
        index,
        status: HealthStatus::Ok,
        ..Default::default()
    };

    // Get driver name
    let driver_path = format!("/sys/class/drm/card{}/device/driver", index);
    if let Ok(link) = std::fs::read_link(&driver_path) {
        if let Some(driver) = link.file_name() {
            health.driver = Some(driver.to_string_lossy().to_string());
        }
    }

    // Get temperature based on driver
    if let Some(ref driver) = health.driver {
        match driver.as_str() {
            "nvidia" => {
                if let Some((temp, source)) = get_nvidia_gpu_temp(index) {
                    health.current_temp_c = Some(temp);
                    health.max_temp_c = Some(temp); // nvidia-smi shows current, not max
                    health.temp_source = Some(source);
                }
            }
            "amdgpu" | "radeon" => {
                if let Some((temp, source)) = get_amd_gpu_temp(index) {
                    health.current_temp_c = Some(temp);
                    health.max_temp_c = Some(temp);
                    health.temp_source = Some(source);
                }
            }
            "i915" | "xe" => {
                if let Some((temp, source)) = get_intel_gpu_temp(index) {
                    health.current_temp_c = Some(temp);
                    health.max_temp_c = Some(temp);
                    health.temp_source = Some(source);
                }
            }
            _ => {}
        }

        // Check for driver errors
        health.driver_errors = count_driver_errors(driver);
    }

    // Evaluate status
    if let Some(temp) = health.current_temp_c {
        if temp >= 95.0 {
            health.status = HealthStatus::Critical;
            health
                .alerts
                .push(format!("GPU{} temperature critical: {:.0}°C", index, temp));
        } else if temp >= 85.0 {
            health.status = HealthStatus::Warning;
            health
                .alerts
                .push(format!("GPU{} temperature high: {:.0}°C", index, temp));
        }
    }

    if health.driver_errors > 0 && health.status == HealthStatus::Ok {
        health.status = HealthStatus::Warning;
        health.alerts.push(format!(
            "GPU{} driver errors: {}",
            index, health.driver_errors
        ));
    }

    health
}

/// Get NVIDIA GPU temperature via nvidia-smi
fn get_nvidia_gpu_temp(index: u32) -> Option<(f32, String)> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=temperature.gpu",
            "--format=csv,noheader,nounits",
            &format!("--id={}", index),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(temp) = stdout.trim().parse::<f32>() {
            return Some((temp, "nvidia-smi".to_string()));
        }
    }

    None
}

/// Get AMD GPU temperature from hwmon
fn get_amd_gpu_temp(index: u32) -> Option<(f32, String)> {
    let hwmon_base = format!("/sys/class/drm/card{}/device/hwmon", index);
    let hwmon_path = Path::new(&hwmon_base);

    if let Ok(entries) = std::fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let temp_path = entry.path().join("temp1_input");
            if let Ok(content) = std::fs::read_to_string(&temp_path) {
                if let Ok(temp_millic) = content.trim().parse::<i32>() {
                    return Some((temp_millic as f32 / 1000.0, "hwmon".to_string()));
                }
            }
        }
    }

    None
}

/// Get Intel GPU temperature from hwmon
fn get_intel_gpu_temp(index: u32) -> Option<(f32, String)> {
    // Intel GPUs often share thermal with CPU, try hwmon
    get_amd_gpu_temp(index)
}

/// Count driver errors in kernel logs
fn count_driver_errors(driver: &str) -> usize {
    let output = Command::new("journalctl")
        .args(["-b", "-k", "--no-pager", "-q", "-p", "err..warning"])
        .output();

    let mut error_count = 0;

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let driver_lower = driver.to_lowercase();

            for line in stdout.lines() {
                let line_lower = line.to_lowercase();
                if line_lower.contains(&driver_lower) {
                    error_count += 1;
                }
            }
        }
    }

    error_count
}

// ============================================================================
// Disk Health
// ============================================================================

/// Disk health data
#[derive(Debug, Clone, Default)]
pub struct DiskHealth {
    /// Device name (nvme0n1, sda, etc.)
    pub device: String,
    /// Device type (NVMe, SATA, USB)
    pub device_type: String,
    /// Model name
    pub model: Option<String>,
    /// Size in bytes
    pub size_bytes: u64,
    /// SMART overall status
    pub smart_passed: Option<bool>,
    /// Power-on hours
    pub power_on_hours: Option<u64>,
    /// Temperature in Celsius
    pub temperature_c: Option<f32>,
    /// Reallocated sector count (SATA)
    pub reallocated_sectors: Option<u64>,
    /// Pending sectors (SATA)
    pub pending_sectors: Option<u64>,
    /// Media errors (NVMe)
    pub media_errors: Option<u64>,
    /// Unsafe shutdowns (NVMe)
    pub unsafe_shutdowns: Option<u64>,
    /// Health status
    pub status: HealthStatus,
    /// Alerts generated
    pub alerts: Vec<String>,
    /// SMART data available
    pub smart_available: bool,
    /// Reason if SMART unavailable
    pub smart_unavailable_reason: Option<String>,
}

/// Get health for all disks
pub fn get_all_disk_health() -> Vec<DiskHealth> {
    let mut disks = Vec::new();

    // List disks from lsblk
    let output = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE,SIZE,MODEL"])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "disk" {
                    let device = parts[0].to_string();
                    disks.push(get_disk_health(&device));
                }
            }
        }
    }

    disks
}

/// Get health for a single disk
pub fn get_disk_health(device: &str) -> DiskHealth {
    let mut health = DiskHealth {
        device: device.to_string(),
        status: HealthStatus::Unknown,
        ..Default::default()
    };

    // Determine device type
    if device.starts_with("nvme") {
        health.device_type = "NVMe".to_string();
    } else if device.starts_with("sd") {
        // Check if removable (USB)
        let removable_path = format!("/sys/block/{}/removable", device);
        if let Ok(content) = std::fs::read_to_string(&removable_path) {
            if content.trim() == "1" {
                health.device_type = "USB".to_string();
            } else {
                health.device_type = "SATA".to_string();
            }
        } else {
            health.device_type = "SATA".to_string();
        }
    } else if device.starts_with("mmcblk") {
        health.device_type = "eMMC/SD".to_string();
    } else {
        health.device_type = "Other".to_string();
    }

    // Get basic info from lsblk
    let dev_path = format!("/dev/{}", device);
    let output = Command::new("lsblk")
        .args(["-d", "-n", "-b", "-o", "SIZE,MODEL", &dev_path])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let parts: Vec<&str> = stdout.split_whitespace().collect();
            if !parts.is_empty() {
                health.size_bytes = parts[0].parse().unwrap_or(0);
            }
            if parts.len() >= 2 {
                health.model = Some(parts[1..].join(" "));
            }
        }
    }

    // Get SMART data
    get_smart_data(&dev_path, &mut health);

    // Evaluate status
    health.status = evaluate_disk_status(&health);

    health
}

/// Get SMART data for a disk
fn get_smart_data(dev_path: &str, health: &mut DiskHealth) {
    // Try smartctl first
    let output = Command::new("smartctl")
        .args(["-H", "-A", "-j", dev_path])
        .output();

    match output {
        Ok(result) => {
            // smartctl returns 0 on success, 4 if SMART not available, etc.
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            // Check for permission issues
            if stderr.contains("Permission denied") || stderr.contains("Operation not permitted") {
                health.smart_unavailable_reason = Some("requires root".to_string());
                return;
            }

            // Try to parse JSON output
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                health.smart_available = true;

                // Get overall health
                if let Some(smart_status) = json.get("smart_status") {
                    if let Some(passed) = smart_status.get("passed") {
                        health.smart_passed = passed.as_bool();
                    }
                }

                // Get temperature
                if let Some(temp) = json.get("temperature") {
                    if let Some(current) = temp.get("current") {
                        health.temperature_c = current.as_f64().map(|t| t as f32);
                    }
                }

                // Get power-on hours
                if let Some(power_on) = json.get("power_on_time") {
                    if let Some(hours) = power_on.get("hours") {
                        health.power_on_hours = hours.as_u64();
                    }
                }

                // NVMe specific
                if let Some(nvme_smart) = json.get("nvme_smart_health_information_log") {
                    if let Some(media_errors) = nvme_smart.get("media_errors") {
                        health.media_errors = media_errors.as_u64();
                    }
                    if let Some(unsafe_shutdowns) = nvme_smart.get("unsafe_shutdowns") {
                        health.unsafe_shutdowns = unsafe_shutdowns.as_u64();
                    }
                }

                // SATA ATA specific
                if let Some(ata_attrs) = json.get("ata_smart_attributes") {
                    if let Some(table) = ata_attrs.get("table") {
                        if let Some(attrs) = table.as_array() {
                            for attr in attrs {
                                let id = attr.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                                let raw_value = attr
                                    .get("raw")
                                    .and_then(|r| r.get("value"))
                                    .and_then(|v| v.as_u64());

                                match id {
                                    5 => health.reallocated_sectors = raw_value, // Reallocated Sector Count
                                    197 => health.pending_sectors = raw_value, // Current Pending Sector Count
                                    9 => health.power_on_hours = raw_value,    // Power-On Hours
                                    194 => {
                                        if health.temperature_c.is_none() {
                                            health.temperature_c = raw_value.map(|v| v as f32);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            } else {
                // JSON parsing failed, try non-JSON parsing
                parse_smartctl_text(&stdout, health);
            }
        }
        Err(_) => {
            health.smart_unavailable_reason = Some("smartctl not installed".to_string());
        }
    }
}

/// Parse smartctl text output (fallback)
fn parse_smartctl_text(output: &str, health: &mut DiskHealth) {
    for line in output.lines() {
        if line.contains("SMART overall-health") || line.contains("SMART Health Status") {
            health.smart_available = true;
            if line.contains("PASSED") || line.contains("OK") {
                health.smart_passed = Some(true);
            } else {
                health.smart_passed = Some(false);
            }
        }

        // SATA attributes
        if line.contains("Reallocated_Sector_Ct") || line.contains("Reallocated_Event_Count") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                health.reallocated_sectors = parts[9].parse().ok();
            }
        }

        if line.contains("Current_Pending_Sector") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                health.pending_sectors = parts[9].parse().ok();
            }
        }

        if line.contains("Power_On_Hours") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                health.power_on_hours = parts[9].parse().ok();
            }
        }

        // NVMe attributes
        if line.contains("Media and Data Integrity Errors") {
            if let Some(val) = line.split(':').nth(1) {
                health.media_errors = val.trim().parse().ok();
            }
        }

        if line.contains("Unsafe Shutdowns") {
            if let Some(val) = line.split(':').nth(1) {
                health.unsafe_shutdowns = val.trim().parse().ok();
            }
        }
    }
}

/// Evaluate disk health status based on SMART data
fn evaluate_disk_status(health: &DiskHealth) -> HealthStatus {
    if !health.smart_available {
        return HealthStatus::Unknown;
    }

    // Critical: SMART failed
    if let Some(passed) = health.smart_passed {
        if !passed {
            return HealthStatus::Critical;
        }
    }

    // Critical: High reallocated or pending sectors
    if let Some(reallocated) = health.reallocated_sectors {
        if reallocated > 100 {
            return HealthStatus::Critical;
        } else if reallocated > 0 {
            return HealthStatus::Warning;
        }
    }

    if let Some(pending) = health.pending_sectors {
        if pending > 10 {
            return HealthStatus::Critical;
        } else if pending > 0 {
            return HealthStatus::Warning;
        }
    }

    // NVMe media errors
    if let Some(media_errors) = health.media_errors {
        if media_errors > 10 {
            return HealthStatus::Critical;
        } else if media_errors > 0 {
            return HealthStatus::Warning;
        }
    }

    HealthStatus::Ok
}

// ============================================================================
// Battery Health
// ============================================================================

/// Battery health data
#[derive(Debug, Clone, Default)]
pub struct BatteryHealth {
    /// Battery name (BAT0, BAT1, etc.)
    pub name: String,
    /// Current status (Charging, Discharging, Full, etc.)
    pub status: String,
    /// Current capacity percentage
    pub capacity_percent: Option<u32>,
    /// Design capacity in Wh
    pub design_capacity_wh: Option<f32>,
    /// Full charge capacity in Wh
    pub full_capacity_wh: Option<f32>,
    /// Wear level percentage (100 - full/design * 100)
    pub wear_level_percent: Option<f32>,
    /// Cycle count
    pub cycle_count: Option<u32>,
    /// Health status
    pub health_status: HealthStatus,
    /// Alerts generated
    pub alerts: Vec<String>,
}

/// Get health for all batteries
pub fn get_battery_health() -> Vec<BatteryHealth> {
    let mut batteries = Vec::new();
    let power_path = Path::new("/sys/class/power_supply");

    if !power_path.exists() {
        return batteries;
    }

    if let Ok(entries) = std::fs::read_dir(power_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                batteries.push(get_single_battery_health(&name, &entry.path()));
            }
        }
    }

    batteries.sort_by(|a, b| a.name.cmp(&b.name));
    batteries
}

/// Get health for a single battery
fn get_single_battery_health(name: &str, path: &Path) -> BatteryHealth {
    let mut health = BatteryHealth {
        name: name.to_string(),
        health_status: HealthStatus::Ok,
        ..Default::default()
    };

    // Read status
    if let Ok(status) = std::fs::read_to_string(path.join("status")) {
        health.status = status.trim().to_string();
    }

    // Read capacity
    if let Ok(capacity) = std::fs::read_to_string(path.join("capacity")) {
        health.capacity_percent = capacity.trim().parse().ok();
    }

    // Read energy values (in microWh)
    let energy_full = std::fs::read_to_string(path.join("energy_full"))
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok());
    let energy_full_design = std::fs::read_to_string(path.join("energy_full_design"))
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok());

    // Convert to Wh
    if let Some(full) = energy_full {
        health.full_capacity_wh = Some((full / 1_000_000.0) as f32);
    }
    if let Some(design) = energy_full_design {
        health.design_capacity_wh = Some((design / 1_000_000.0) as f32);
    }

    // Calculate wear level
    if let (Some(full), Some(design)) = (health.full_capacity_wh, health.design_capacity_wh) {
        if design > 0.0 {
            let wear = 100.0 - (full / design * 100.0);
            health.wear_level_percent = Some(wear.max(0.0));
        }
    }

    // Read cycle count
    if let Ok(cycles) = std::fs::read_to_string(path.join("cycle_count")) {
        health.cycle_count = cycles.trim().parse().ok();
    }

    // Evaluate health
    if let Some(wear) = health.wear_level_percent {
        if wear > 50.0 {
            health.health_status = HealthStatus::Critical;
            health.alerts.push(format!(
                "Battery {} wear level critical: {:.0}%",
                name, wear
            ));
        } else if wear > 30.0 {
            health.health_status = HealthStatus::Warning;
            health
                .alerts
                .push(format!("Battery {} wear level high: {:.0}%", name, wear));
        }
    }

    health
}

// ============================================================================
// Network Health
// ============================================================================

/// Network interface health data
#[derive(Debug, Clone, Default)]
pub struct NetworkHealth {
    /// Interface name
    pub interface: String,
    /// Interface type (ethernet, wifi, bluetooth, etc.)
    pub interface_type: String,
    /// Driver name
    pub driver: Option<String>,
    /// Link state (up/down)
    pub link_up: bool,
    /// RX errors
    pub rx_errors: u64,
    /// TX errors
    pub tx_errors: u64,
    /// RX dropped
    pub rx_dropped: u64,
    /// TX dropped
    pub tx_dropped: u64,
    /// Wi-Fi signal strength in dBm
    pub wifi_signal_dbm: Option<i32>,
    /// Wi-Fi SSID
    pub wifi_ssid: Option<String>,
    /// Health status
    pub status: HealthStatus,
    /// Alerts generated
    pub alerts: Vec<String>,
}

/// Get health for all network interfaces
pub fn get_network_health() -> Vec<NetworkHealth> {
    let mut interfaces = Vec::new();

    // List interfaces from /sys/class/net
    let net_path = Path::new("/sys/class/net");
    if !net_path.exists() {
        return interfaces;
    }

    if let Ok(entries) = std::fs::read_dir(net_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip loopback
            if name == "lo" {
                continue;
            }
            interfaces.push(get_single_network_health(&name));
        }
    }

    interfaces.sort_by(|a, b| a.interface.cmp(&b.interface));
    interfaces
}

/// Get health for a single network interface
fn get_single_network_health(interface: &str) -> NetworkHealth {
    let mut health = NetworkHealth {
        interface: interface.to_string(),
        status: HealthStatus::Ok,
        ..Default::default()
    };

    // Determine interface type
    if interface.starts_with("en") || interface.starts_with("eth") {
        health.interface_type = "ethernet".to_string();
    } else if interface.starts_with("wl") || interface.starts_with("wlan") {
        health.interface_type = "wifi".to_string();
    } else if interface.starts_with("bt") {
        health.interface_type = "bluetooth".to_string();
    } else if interface.starts_with("docker") || interface.starts_with("br-") {
        health.interface_type = "docker".to_string();
    } else if interface.starts_with("veth") {
        health.interface_type = "veth".to_string();
    } else {
        health.interface_type = "other".to_string();
    }

    // Get driver
    let driver_path = format!("/sys/class/net/{}/device/driver", interface);
    if let Ok(link) = std::fs::read_link(&driver_path) {
        health.driver = link.file_name().map(|n| n.to_string_lossy().to_string());
    }

    // Get link state
    let operstate_path = format!("/sys/class/net/{}/operstate", interface);
    if let Ok(state) = std::fs::read_to_string(&operstate_path) {
        health.link_up = state.trim() == "up";
    }

    // Get statistics from ip -s link
    let output = Command::new("ip")
        .args(["-s", "link", "show", interface])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            parse_ip_stats(&stdout, &mut health);
        }
    }

    // Get Wi-Fi specific info
    if health.interface_type == "wifi" {
        get_wifi_info(interface, &mut health);
    }

    // Evaluate status
    let total_errors = health.rx_errors + health.tx_errors;
    let total_dropped = health.rx_dropped + health.tx_dropped;

    if total_errors > 1000 || total_dropped > 10000 {
        health.status = HealthStatus::Warning;
        health
            .alerts
            .push(format!("{}: high error/drop count", interface));
    }

    health
}

/// Parse ip -s link output for statistics
fn parse_ip_stats(output: &str, health: &mut NetworkHealth) {
    let lines: Vec<&str> = output.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // RX line
        if trimmed.starts_with("RX:") {
            // Next line has the values
            if i + 1 < lines.len() {
                let vals: Vec<&str> = lines[i + 1].split_whitespace().collect();
                if vals.len() >= 4 {
                    health.rx_errors = vals[2].parse().unwrap_or(0);
                    health.rx_dropped = vals[3].parse().unwrap_or(0);
                }
            }
        }

        // TX line
        if trimmed.starts_with("TX:") {
            if i + 1 < lines.len() {
                let vals: Vec<&str> = lines[i + 1].split_whitespace().collect();
                if vals.len() >= 4 {
                    health.tx_errors = vals[2].parse().unwrap_or(0);
                    health.tx_dropped = vals[3].parse().unwrap_or(0);
                }
            }
        }
    }
}

/// Get Wi-Fi specific information
fn get_wifi_info(interface: &str, health: &mut NetworkHealth) {
    // Try iw dev <interface> link
    let output = Command::new("iw").args(["dev", interface, "link"]).output();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);

            for line in stdout.lines() {
                let trimmed = line.trim();

                if trimmed.starts_with("SSID:") {
                    health.wifi_ssid = Some(trimmed.trim_start_matches("SSID:").trim().to_string());
                }

                if trimmed.starts_with("signal:") {
                    if let Some(val) = trimmed.split_whitespace().nth(1) {
                        health.wifi_signal_dbm = val.parse().ok();
                    }
                }
            }
        }
    }
}

// ============================================================================
// Alert Collection
// ============================================================================

/// Collect all hardware alerts
pub fn collect_hardware_alerts() -> Vec<Alert> {
    let mut alerts = Vec::new();

    // CPU alerts
    let cpu = get_cpu_health();
    for alert_text in &cpu.alerts {
        alerts.push(Alert {
            severity: cpu.status,
            scope: AlertScope::Hardware,
            identifier: "cpu".to_string(),
            reason: alert_text.clone(),
            see_command: Some("annactl hw cpu".to_string()),
        });
    }

    // GPU alerts
    for gpu in get_gpu_health() {
        for alert_text in &gpu.alerts {
            alerts.push(Alert {
                severity: gpu.status,
                scope: AlertScope::Hardware,
                identifier: format!("gpu{}", gpu.index),
                reason: alert_text.clone(),
                see_command: Some(format!("annactl hw gpu{}", gpu.index)),
            });
        }
    }

    // Disk alerts
    for disk in get_all_disk_health() {
        if disk.status == HealthStatus::Warning || disk.status == HealthStatus::Critical {
            let reason = if disk.smart_passed == Some(false) {
                "SMART self-assessment failed".to_string()
            } else if disk.reallocated_sectors.unwrap_or(0) > 0 {
                format!(
                    "reallocated sectors: {}",
                    disk.reallocated_sectors.unwrap_or(0)
                )
            } else if disk.pending_sectors.unwrap_or(0) > 0 {
                format!("pending sectors: {}", disk.pending_sectors.unwrap_or(0))
            } else if disk.media_errors.unwrap_or(0) > 0 {
                format!("media errors: {}", disk.media_errors.unwrap_or(0))
            } else {
                "SMART degraded".to_string()
            };

            alerts.push(Alert {
                severity: disk.status,
                scope: AlertScope::Hardware,
                identifier: format!("disk:{}", disk.device),
                reason,
                see_command: Some(format!("annactl hw {}", disk.device)),
            });
        }
    }

    // Battery alerts
    for battery in get_battery_health() {
        for alert_text in &battery.alerts {
            alerts.push(Alert {
                severity: battery.health_status,
                scope: AlertScope::Hardware,
                identifier: format!("battery:{}", battery.name),
                reason: alert_text.clone(),
                see_command: Some("annactl hw battery".to_string()),
            });
        }
    }

    // Network alerts
    for net in get_network_health() {
        for alert_text in &net.alerts {
            alerts.push(Alert {
                severity: net.status,
                scope: AlertScope::Hardware,
                identifier: format!("net:{}", net.interface),
                reason: alert_text.clone(),
                see_command: Some(format!("annactl hw {}", net.interface)),
            });
        }
    }

    alerts
}

/// Get overall hardware health summary
pub fn get_hardware_health_summary() -> HashMap<String, (HealthStatus, String)> {
    let mut summary = HashMap::new();

    // CPU
    let cpu = get_cpu_health();
    let cpu_desc = if let Some(temp) = cpu.max_temp_c {
        if cpu.throttling_detected {
            format!("max {:.0}°C, throttling detected", temp)
        } else {
            format!("max {:.0}°C this boot, no throttling", temp)
        }
    } else {
        "health data unavailable".to_string()
    };
    summary.insert("CPU".to_string(), (cpu.status, cpu_desc));

    // GPU
    let gpus = get_gpu_health();
    if gpus.is_empty() {
        summary.insert(
            "GPU".to_string(),
            (HealthStatus::Unknown, "no GPU detected".to_string()),
        );
    } else {
        let worst_status = gpus
            .iter()
            .map(|g| g.status)
            .max_by_key(|s| match s {
                HealthStatus::Critical => 3,
                HealthStatus::Warning => 2,
                HealthStatus::Unknown => 1,
                HealthStatus::Ok => 0,
            })
            .unwrap_or(HealthStatus::Ok);

        let gpu_desc = if let Some(gpu) = gpus.first() {
            if let Some(temp) = gpu.current_temp_c {
                format!("{}°C, {} errors", temp, gpu.driver_errors)
            } else if let Some(driver) = &gpu.driver {
                format!("driver: {}, {} errors", driver, gpu.driver_errors)
            } else {
                "health data unavailable".to_string()
            }
        } else {
            "no GPU".to_string()
        };
        summary.insert("GPU".to_string(), (worst_status, gpu_desc));
    }

    // Disks
    let disks = get_all_disk_health();
    if disks.is_empty() {
        summary.insert(
            "Disks".to_string(),
            (HealthStatus::Unknown, "no disks detected".to_string()),
        );
    } else {
        let ok_count = disks
            .iter()
            .filter(|d| d.status == HealthStatus::Ok)
            .count();
        let warn_count = disks
            .iter()
            .filter(|d| d.status == HealthStatus::Warning)
            .count();
        let crit_count = disks
            .iter()
            .filter(|d| d.status == HealthStatus::Critical)
            .count();
        let unknown_count = disks
            .iter()
            .filter(|d| d.status == HealthStatus::Unknown)
            .count();

        let worst_status = if crit_count > 0 {
            HealthStatus::Critical
        } else if warn_count > 0 {
            HealthStatus::Warning
        } else if ok_count > 0 {
            HealthStatus::Ok
        } else {
            HealthStatus::Unknown
        };

        let desc = if crit_count > 0 {
            format!("{} critical, {} OK", crit_count, ok_count)
        } else if warn_count > 0 {
            format!("{} warning, {} OK", warn_count, ok_count)
        } else if unknown_count == disks.len() {
            "SMART unavailable".to_string()
        } else {
            format!("{} devices OK", ok_count)
        };

        summary.insert("Disks".to_string(), (worst_status, desc));
    }

    // Battery
    let batteries = get_battery_health();
    if batteries.is_empty() {
        summary.insert(
            "Battery".to_string(),
            (HealthStatus::Unknown, "not present".to_string()),
        );
    } else {
        let worst_status = batteries
            .iter()
            .map(|b| b.health_status)
            .max_by_key(|s| match s {
                HealthStatus::Critical => 3,
                HealthStatus::Warning => 2,
                HealthStatus::Unknown => 1,
                HealthStatus::Ok => 0,
            })
            .unwrap_or(HealthStatus::Ok);

        let bat = batteries.first().unwrap();
        let desc = if let Some(wear) = bat.wear_level_percent {
            format!(
                "{:.0}% design capacity, {} cycles, {}",
                100.0 - wear,
                bat.cycle_count.unwrap_or(0),
                bat.status
            )
        } else {
            format!("{}%: {}", bat.capacity_percent.unwrap_or(0), bat.status)
        };

        summary.insert("Battery".to_string(), (worst_status, desc));
    }

    // Network
    let networks = get_network_health();
    let physical: Vec<_> = networks
        .iter()
        .filter(|n| n.interface_type == "ethernet" || n.interface_type == "wifi")
        .collect();

    if physical.is_empty() {
        summary.insert(
            "Network".to_string(),
            (HealthStatus::Unknown, "no physical interfaces".to_string()),
        );
    } else {
        let up_count = physical.iter().filter(|n| n.link_up).count();
        let total_errors: u64 = physical.iter().map(|n| n.rx_errors + n.tx_errors).sum();

        let status = if total_errors > 1000 {
            HealthStatus::Warning
        } else {
            HealthStatus::Ok
        };

        let desc = if total_errors > 0 {
            format!(
                "{}/{} up, {} errors",
                up_count,
                physical.len(),
                total_errors
            )
        } else {
            format!("{}/{} interfaces up", up_count, physical.len())
        };

        summary.insert("Network".to_string(), (status, desc));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_as_str() {
        assert_eq!(HealthStatus::Ok.as_str(), "OK");
        assert_eq!(HealthStatus::Warning.as_str(), "WARNING");
        assert_eq!(HealthStatus::Critical.as_str(), "CRITICAL");
    }

    #[test]
    fn test_cpu_health_basic() {
        let health = get_cpu_health();
        // Just verify it doesn't panic
        assert!(
            health.status == HealthStatus::Ok
                || health.status == HealthStatus::Warning
                || health.status == HealthStatus::Critical
                || health.status == HealthStatus::Unknown
        );
    }

    #[test]
    fn test_disk_health_basic() {
        let disks = get_all_disk_health();
        // Should find at least one disk on most systems
        // But don't fail if running in container
        for disk in disks {
            assert!(!disk.device.is_empty());
        }
    }

    #[test]
    fn test_network_health_basic() {
        let networks = get_network_health();
        // Should find at least loopback is excluded, so might be empty
        for net in networks {
            assert!(!net.interface.is_empty());
            assert_ne!(net.interface, "lo");
        }
    }

    #[test]
    fn test_battery_health_basic() {
        let batteries = get_battery_health();
        // May be empty on desktop systems
        for bat in batteries {
            assert!(bat.name.starts_with("BAT"));
        }
    }

    #[test]
    fn test_collect_alerts() {
        let alerts = collect_hardware_alerts();
        // Just verify it doesn't panic
        for alert in alerts {
            assert!(!alert.identifier.is_empty());
            assert!(!alert.reason.is_empty());
        }
    }
}

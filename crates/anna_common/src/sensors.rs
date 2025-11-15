//! Hardware sensors detection
//!
//! Detects hardware sensors and temperatures:
//! - CPU temperatures
//! - GPU temperatures
//! - NVMe/disk temperatures
//! - Fan speeds
//! - Voltage readings

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

/// Hardware sensor readings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorsInfo {
    /// CPU temperature (°C)
    pub cpu_temp: Option<f32>,
    /// GPU temperature (°C)
    pub gpu_temp: Option<f32>,
    /// NVMe temperatures (°C) per device
    pub nvme_temps: Vec<NvmeTemp>,
    /// Fan speeds (RPM)
    pub fan_speeds: Vec<FanSpeed>,
    /// Voltage readings (V)
    pub voltages: Vec<Voltage>,
    /// Thermal zones
    pub thermal_zones: Vec<ThermalZone>,
    /// lm_sensors available
    pub lm_sensors_available: bool,
}

/// NVMe temperature reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeTemp {
    /// Device name (e.g., "nvme0")
    pub device: String,
    /// Temperature in °C
    pub temp: f32,
}

/// Fan speed reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanSpeed {
    /// Fan label (e.g., "fan1", "CPU Fan")
    pub label: String,
    /// Speed in RPM
    pub rpm: u32,
}

/// Voltage reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voltage {
    /// Voltage label (e.g., "Vcore", "12V")
    pub label: String,
    /// Voltage in volts
    pub volts: f32,
}

/// Thermal zone information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalZone {
    /// Zone type (e.g., "x86_pkg_temp", "acpitz")
    pub zone_type: String,
    /// Temperature in °C
    pub temp: f32,
}

impl SensorsInfo {
    /// Detect hardware sensors
    pub fn detect() -> Self {
        let lm_sensors_available = check_lm_sensors();

        let (cpu_temp, gpu_temp, fan_speeds, voltages) = if lm_sensors_available {
            parse_sensors_output()
        } else {
            (None, None, Vec::new(), Vec::new())
        };

        let nvme_temps = detect_nvme_temps();
        let thermal_zones = detect_thermal_zones();

        Self {
            cpu_temp,
            gpu_temp,
            nvme_temps,
            fan_speeds,
            voltages,
            thermal_zones,
            lm_sensors_available,
        }
    }
}

/// Check if lm_sensors is available
fn check_lm_sensors() -> bool {
    Command::new("sensors")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Parse sensors command output
fn parse_sensors_output() -> (Option<f32>, Option<f32>, Vec<FanSpeed>, Vec<Voltage>) {
    let output = match Command::new("sensors").output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return (None, None, Vec::new(), Vec::new()),
    };

    let mut cpu_temp = None;
    let mut gpu_temp = None;
    let mut fan_speeds = Vec::new();
    let mut voltages = Vec::new();

    for line in output.lines() {
        let line = line.trim();

        // CPU temperature patterns
        if (line.contains("Core 0:") || line.contains("Tctl:") || line.contains("Package id 0:"))
            && cpu_temp.is_none()
        {
            if let Some(temp) = extract_temp_from_line(line) {
                cpu_temp = Some(temp);
            }
        }

        // GPU temperature patterns
        if (line.contains("edge:") || line.contains("GPU:") || line.contains("temp1:"))
            && line.contains("°C")
            && gpu_temp.is_none()
        {
            if let Some(temp) = extract_temp_from_line(line) {
                gpu_temp = Some(temp);
            }
        }

        // Fan speeds
        if line.contains("fan") && line.contains("RPM") {
            if let Some((label, rpm)) = extract_fan_speed(line) {
                fan_speeds.push(FanSpeed { label, rpm });
            }
        }

        // Voltages
        if line.contains("V") && !line.contains("°C") && !line.contains("RPM") {
            if let Some((label, volts)) = extract_voltage(line) {
                voltages.push(Voltage { label, volts });
            }
        }
    }

    (cpu_temp, gpu_temp, fan_speeds, voltages)
}

/// Extract temperature from a line
fn extract_temp_from_line(line: &str) -> Option<f32> {
    // Look for pattern like "+45.0°C" or "45.0°C"
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts {
        if part.contains("°C") {
            let temp_str = part
                .replace("°C", "")
                .replace("+", "")
                .replace("(", "")
                .replace(")", "");
            if let Ok(temp) = temp_str.parse::<f32>() {
                return Some(temp);
            }
        }
    }
    None
}

/// Extract fan speed from a line
fn extract_fan_speed(line: &str) -> Option<(String, u32)> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let label = parts[0].trim().to_string();
    let value_part = parts[1].trim();

    // Extract RPM value
    let rpm_parts: Vec<&str> = value_part.split_whitespace().collect();
    for part in rpm_parts {
        if let Ok(rpm) = part.parse::<u32>() {
            return Some((label, rpm));
        }
    }

    None
}

/// Extract voltage from a line
fn extract_voltage(line: &str) -> Option<(String, f32)> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let label = parts[0].trim().to_string();
    let value_part = parts[1].trim();

    // Extract voltage value (looking for pattern like "1.234 V")
    let voltage_parts: Vec<&str> = value_part.split_whitespace().collect();
    for (i, part) in voltage_parts.iter().enumerate() {
        if *part == "V" && i > 0 {
            if let Ok(volts) = voltage_parts[i - 1].parse::<f32>() {
                return Some((label, volts));
            }
        }
    }

    None
}

/// Detect NVMe temperatures
fn detect_nvme_temps() -> Vec<NvmeTemp> {
    let mut temps = Vec::new();

    // Check /sys/class/nvme for NVMe devices
    if let Ok(entries) = fs::read_dir("/sys/class/nvme") {
        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();

            // Try hwmon path
            let hwmon_path = format!("/sys/class/nvme/{}/device/hwmon", device_name);
            if let Ok(hwmon_entries) = fs::read_dir(&hwmon_path) {
                for hwmon_entry in hwmon_entries.flatten() {
                    let temp_path = format!(
                        "{}/{}/temp1_input",
                        hwmon_path,
                        hwmon_entry.file_name().to_string_lossy()
                    );
                    if let Ok(content) = fs::read_to_string(&temp_path) {
                        if let Ok(millidegrees) = content.trim().parse::<f32>() {
                            temps.push(NvmeTemp {
                                device: device_name.clone(),
                                temp: millidegrees / 1000.0,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    temps
}

/// Detect thermal zones
fn detect_thermal_zones() -> Vec<ThermalZone> {
    let mut zones = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let zone_name = entry.file_name().to_string_lossy().to_string();

            if !zone_name.starts_with("thermal_zone") {
                continue;
            }

            let zone_path = format!("/sys/class/thermal/{}", zone_name);

            // Read zone type
            let type_path = format!("{}/type", zone_path);
            let zone_type = fs::read_to_string(&type_path)
                .unwrap_or_default()
                .trim()
                .to_string();

            // Read temperature
            let temp_path = format!("{}/temp", zone_path);
            if let Ok(content) = fs::read_to_string(&temp_path) {
                if let Ok(millidegrees) = content.trim().parse::<f32>() {
                    zones.push(ThermalZone {
                        zone_type,
                        temp: millidegrees / 1000.0,
                    });
                }
            }
        }
    }

    zones
}

//! Power and battery detection
//!
//! Detects power configuration and battery health:
//! - AC vs battery power status
//! - Battery health and capacity
//! - Charge cycles
//! - Power management daemon (TLP, power-profiles-daemon)
//! - Battery degradation

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Power and battery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerInfo {
    /// Power source (AC, Battery, or Unknown)
    pub power_source: PowerSource,
    /// Battery information (if present)
    pub battery: Option<BatteryInfo>,
    /// Power management daemon in use
    pub power_daemon: Option<PowerDaemon>,
    /// Number of power supplies detected
    pub power_supply_count: usize,
}

/// Power source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PowerSource {
    /// Running on AC power
    AC,
    /// Running on battery
    Battery,
    /// Unknown or mixed
    Unknown,
}

/// Battery health and status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    /// Battery name/identifier
    pub name: String,
    /// Current charge percentage (0-100)
    pub charge_percent: Option<f32>,
    /// Battery status (Charging, Discharging, Full, etc.)
    pub status: String,
    /// Design capacity (Wh or mAh)
    pub capacity_design: Option<f32>,
    /// Current full capacity (Wh or mAh)
    pub capacity_full: Option<f32>,
    /// Current capacity (Wh or mAh)
    pub capacity_now: Option<f32>,
    /// Health percentage (capacity_full / capacity_design * 100)
    pub health_percent: Option<f32>,
    /// Charge cycles (if available)
    pub cycle_count: Option<u32>,
    /// Current power draw (W)
    pub power_draw: Option<f32>,
    /// Technology (Li-ion, Li-poly, etc.)
    pub technology: Option<String>,
}

/// Power management daemon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PowerDaemon {
    /// TLP (advanced laptop power management)
    TLP { running: bool, enabled: bool },
    /// power-profiles-daemon (GNOME/systemd)
    PowerProfilesDaemon { running: bool, enabled: bool },
    /// laptop-mode-tools
    LaptopModeTools { running: bool },
    /// None detected
    None,
}

impl PowerInfo {
    /// Detect power configuration
    pub fn detect() -> Self {
        let power_supplies = detect_power_supplies();
        let power_supply_count = power_supplies.len();

        let power_source = determine_power_source(&power_supplies);
        let battery = detect_battery_info(&power_supplies);
        let power_daemon = detect_power_daemon();

        Self {
            power_source,
            battery,
            power_daemon,
            power_supply_count,
        }
    }
}

/// Power supply information from /sys
#[derive(Debug)]
struct PowerSupply {
    name: String,
    supply_type: String,
    path: String,
}

/// Detect all power supplies from /sys/class/power_supply
fn detect_power_supplies() -> Vec<PowerSupply> {
    let mut supplies = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = format!("/sys/class/power_supply/{}", name);

            // Read the type
            let type_path = format!("{}/type", path);
            let supply_type = fs::read_to_string(&type_path)
                .unwrap_or_default()
                .trim()
                .to_string();

            supplies.push(PowerSupply {
                name,
                supply_type,
                path,
            });
        }
    }

    supplies
}

/// Determine if system is on AC or battery
fn determine_power_source(supplies: &[PowerSupply]) -> PowerSource {
    // Look for a Mains power supply
    for supply in supplies {
        if supply.supply_type == "Mains" {
            let online_path = format!("{}/online", supply.path);
            if let Ok(content) = fs::read_to_string(&online_path) {
                if content.trim() == "1" {
                    return PowerSource::AC;
                }
            }
        }
    }

    // Look for battery status
    for supply in supplies {
        if supply.supply_type == "Battery" {
            let status_path = format!("{}/status", supply.path);
            if let Ok(status) = fs::read_to_string(&status_path) {
                let status = status.trim();
                if status == "Discharging" {
                    return PowerSource::Battery;
                } else if status == "Charging" || status == "Full" {
                    return PowerSource::AC;
                }
            }
        }
    }

    PowerSource::Unknown
}

/// Detect battery information
fn detect_battery_info(supplies: &[PowerSupply]) -> Option<BatteryInfo> {
    // Find first battery
    for supply in supplies {
        if supply.supply_type == "Battery" {
            return parse_battery_info(&supply.name, &supply.path);
        }
    }

    None
}

/// Parse battery information from /sys/class/power_supply/BAT*
fn parse_battery_info(name: &str, path: &str) -> Option<BatteryInfo> {
    let read_value = |filename: &str| -> Option<String> {
        let file_path = format!("{}/{}", path, filename);
        fs::read_to_string(&file_path).ok().map(|s| s.trim().to_string())
    };

    let read_f32 = |filename: &str| -> Option<f32> {
        read_value(filename)?.parse::<f32>().ok()
    };

    let read_u32 = |filename: &str| -> Option<u32> {
        read_value(filename)?.parse::<u32>().ok()
    };

    // Read status
    let status = read_value("status").unwrap_or_else(|| "Unknown".to_string());

    // Read charge percentage
    let charge_percent = read_f32("capacity");

    // Read capacity values (in µWh or µAh, need to convert)
    let capacity_design = read_f32("energy_full_design")
        .or_else(|| read_f32("charge_full_design"))
        .map(|v| v / 1_000_000.0); // Convert µWh to Wh or µAh to mAh

    let capacity_full = read_f32("energy_full")
        .or_else(|| read_f32("charge_full"))
        .map(|v| v / 1_000_000.0);

    let capacity_now = read_f32("energy_now")
        .or_else(|| read_f32("charge_now"))
        .map(|v| v / 1_000_000.0);

    // Calculate health percentage
    let health_percent = if let (Some(full), Some(design)) = (capacity_full, capacity_design) {
        if design > 0.0 {
            Some((full / design * 100.0).min(100.0))
        } else {
            None
        }
    } else {
        None
    };

    // Read cycle count
    let cycle_count = read_u32("cycle_count");

    // Calculate power draw (in µW, convert to W)
    let power_draw = read_f32("power_now").map(|v| v / 1_000_000.0);

    // Read technology
    let technology = read_value("technology");

    Some(BatteryInfo {
        name: name.to_string(),
        charge_percent,
        status,
        capacity_design,
        capacity_full,
        capacity_now,
        health_percent,
        cycle_count,
        power_draw,
        technology,
    })
}

/// Detect which power management daemon is active
fn detect_power_daemon() -> Option<PowerDaemon> {
    // Check TLP
    if Path::new("/usr/bin/tlp").exists() || Path::new("/usr/sbin/tlp").exists() {
        let running = is_service_running("tlp.service");
        let enabled = is_service_enabled("tlp.service");
        return Some(PowerDaemon::TLP { running, enabled });
    }

    // Check power-profiles-daemon
    if is_service_installed("power-profiles-daemon.service") {
        let running = is_service_running("power-profiles-daemon.service");
        let enabled = is_service_enabled("power-profiles-daemon.service");
        return Some(PowerDaemon::PowerProfilesDaemon { running, enabled });
    }

    // Check laptop-mode-tools
    if Path::new("/usr/sbin/laptop_mode").exists() {
        let running = is_service_running("laptop-mode.service");
        return Some(PowerDaemon::LaptopModeTools { running });
    }

    Some(PowerDaemon::None)
}

/// Check if a systemd service is running
fn is_service_running(service: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", service])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a systemd service is enabled
fn is_service_enabled(service: &str) -> bool {
    Command::new("systemctl")
        .args(["is-enabled", "--quiet", service])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a systemd service exists
fn is_service_installed(service: &str) -> bool {
    Command::new("systemctl")
        .args(["list-unit-files", service])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.contains(service)
        })
        .unwrap_or(false)
}

//! Battery health and drain monitoring from /sys/class/power_supply/.
//!
//! Tracks: charge level, health, drain rate, cycle count, capacity degradation.
//! Generates alerts when battery health degrades significantly.

use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Status of a single battery
#[derive(Debug, Clone)]
pub struct Battery {
    /// Device name (BAT0, BAT1, etc.)
    pub name: String,
    /// Charge level as percentage (0-100)
    pub capacity_pct: u8,
    /// Current status: Charging, Discharging, Full, Unknown
    pub status: String,
    /// Design capacity in µAh or µWh
    pub energy_full_design: Option<u64>,
    /// Current full-charge capacity (degrades over time)
    pub energy_full: Option<u64>,
    /// Current remaining energy
    pub energy_now: Option<u64>,
    /// Health percentage (energy_full / energy_full_design * 100)
    pub health_pct: Option<f32>,
    /// Estimated time remaining (seconds), if discharging
    pub time_to_empty_secs: Option<u64>,
    /// Estimated time to full (seconds), if charging
    pub time_to_full_secs: Option<u64>,
    /// Power draw in µW (if available)
    pub power_now_uw: Option<u64>,
    /// Manufacturer
    pub manufacturer: Option<String>,
    /// Cycle count (if available)
    pub cycle_count: Option<u32>,
    /// Battery technology (Li-ion, etc.)
    pub technology: Option<String>,
}

impl Battery {
    /// Detect and read all batteries from /sys/class/power_supply/
    pub fn detect_all() -> Vec<Self> {
        let base = Path::new("/sys/class/power_supply");
        if !base.exists() {
            return vec![];
        }

        let entries = match fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        let mut batteries = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Only process BAT* entries
            if !name.starts_with("BAT") && !name.to_lowercase().contains("battery") {
                continue;
            }

            let path = entry.path();
            if let Some(bat) = read_battery(&path, name) {
                batteries.push(bat);
            }
        }

        batteries.sort_by(|a, b| a.name.cmp(&b.name));
        batteries
    }

    /// Check if battery is on AC power (not discharging)
    pub fn is_on_ac(&self) -> bool {
        self.status == "Charging" || self.status == "Full" || self.status == "Not charging"
    }

    /// Format a human-readable summary line
    pub fn summary(&self) -> String {
        let health = self.health_pct.map(|h| format!(", health {:.0}%", h))
            .unwrap_or_default();
        let time = if self.status == "Discharging" {
            self.time_to_empty_secs.map(|s| format!(", ~{}h{}m remaining",
                s / 3600, (s % 3600) / 60)).unwrap_or_default()
        } else if self.status == "Charging" {
            self.time_to_full_secs.map(|s| format!(", ~{}h{}m to full",
                s / 3600, (s % 3600) / 60)).unwrap_or_default()
        } else {
            String::new()
        };
        let power = self.power_now_uw.map(|p| format!(", {:.1}W draw", p as f64 / 1_000_000.0))
            .unwrap_or_default();

        format!("{}: {}% ({}){}{}{}",
            self.name, self.capacity_pct, self.status, health, time, power)
    }

    /// Return health alert if battery is significantly degraded
    pub fn health_alert(&self) -> Option<String> {
        let health = self.health_pct?;
        if health < 60.0 {
            Some(format!("{} health is critically low ({:.0}%). Consider replacement.", self.name, health))
        } else if health < 80.0 {
            Some(format!("{} health at {:.0}% (design capacity degraded). Normal aging.", self.name, health))
        } else {
            None
        }
    }
}

fn read_battery(path: &PathBuf, name: String) -> Option<Battery> {
    let read = |file: &str| -> Option<String> {
        fs::read_to_string(path.join(file)).ok().map(|s| s.trim().to_string())
    };
    let read_u64 = |file: &str| -> Option<u64> {
        read(file).and_then(|s| s.parse().ok())
    };
    let read_u32 = |file: &str| -> Option<u32> {
        read(file).and_then(|s| s.parse().ok())
    };

    // capacity_pct is mandatory
    let capacity_pct = read_u64("capacity")?.min(100) as u8;
    let status = read("status").unwrap_or_else(|| "Unknown".to_string());

    // Try µWh first, then µAh
    let energy_full_design = read_u64("energy_full_design").or_else(|| read_u64("charge_full_design"));
    let energy_full = read_u64("energy_full").or_else(|| read_u64("charge_full"));
    let energy_now = read_u64("energy_now").or_else(|| read_u64("charge_now"));
    let power_now_uw = read_u64("power_now").or_else(|| {
        // voltage_now (µV) * current_now (µA) / 1e6 = µW
        if let (Some(v), Some(i)) = (read_u64("voltage_now"), read_u64("current_now")) {
            Some(v * i / 1_000_000)
        } else {
            None
        }
    });

    let health_pct = match (energy_full, energy_full_design) {
        (Some(full), Some(design)) if design > 0 => {
            Some((full as f32 / design as f32 * 100.0).min(100.0))
        }
        _ => None,
    };

    // Estimate time remaining from power draw
    let time_to_empty_secs = if status == "Discharging" {
        match (energy_now, power_now_uw) {
            (Some(e), Some(p)) if p > 0 => Some(e * 3600 / p),
            _ => None,
        }
    } else {
        None
    };

    let time_to_full_secs = if status == "Charging" {
        match (energy_full, energy_now, power_now_uw) {
            (Some(f), Some(n), Some(p)) if p > 0 && f > n => Some((f - n) * 3600 / p),
            _ => None,
        }
    } else {
        None
    };

    debug!("Battery {}: {}%, {}", name, capacity_pct, status);

    Some(Battery {
        name,
        capacity_pct,
        status,
        energy_full_design,
        energy_full,
        energy_now,
        health_pct,
        time_to_empty_secs,
        time_to_full_secs,
        power_now_uw,
        manufacturer: read("manufacturer"),
        cycle_count: read_u32("cycle_count"),
        technology: read("technology"),
    })
}

/// Build a telemetry section for briefing injection.
pub fn battery_telemetry() -> String {
    let batteries = Battery::detect_all();
    if batteries.is_empty() {
        return String::new(); // Desktop — no battery
    }

    let mut out = "## Battery\n".to_string();
    for bat in &batteries {
        out.push_str(&format!("{}\n", bat.summary()));
        if let Some(alert) = bat.health_alert() {
            out.push_str(&format!("ALERT: {}\n", alert));
        }
    }
    out
}

/// Check if system is on battery (any battery discharging)
pub fn is_on_battery() -> bool {
    Battery::detect_all().iter().any(|b| b.status == "Discharging")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_all_no_panic() {
        let batteries = Battery::detect_all();
        for bat in &batteries {
            let _ = bat.summary();
            let _ = bat.health_alert();
            let _ = bat.is_on_ac();
        }
    }

    #[test]
    fn test_battery_telemetry_empty_on_desktop() {
        // Should return empty string if no BAT* devices (desktop)
        let result = battery_telemetry();
        // Either empty (desktop) or has content (laptop) — both valid
        let _ = result;
    }
}

//! CPU governor, power profiles, and thermal zone monitoring.
//!
//! Reads from /sys/devices/system/cpu/ and power-profiles-daemon.
//! Used in briefing and system context for performance/efficiency advice.

use std::fs;
use std::process::Command;
use tracing::debug;

/// Current power/performance state of the system
#[derive(Debug, Clone)]
pub struct PowerState {
    /// Active power profile (performance/balanced/power-saver or None if ppd absent)
    pub profile: Option<String>,
    /// CPU scaling governor (performance/powersave/schedutil/ondemand/etc.)
    pub governor: Option<String>,
    /// Available governors
    pub available_governors: Vec<String>,
    /// CPU frequency min/max (MHz)
    pub freq_min_mhz: Option<u32>,
    pub freq_max_mhz: Option<u32>,
    /// Current frequency range across all cores (MHz)
    pub freq_current_mhz: Option<u32>,
    /// Thermal zone temperatures (name, celsius)
    pub thermal_zones: Vec<(String, f32)>,
    /// Whether any zone is above warning threshold (85°C)
    pub thermal_warning: bool,
    /// Whether power-profiles-daemon is available
    pub ppd_available: bool,
    /// Whether cpupower is available
    pub cpupower_available: bool,
}

impl PowerState {
    /// Capture current power state from /sys and system commands.
    pub fn capture() -> Self {
        let governor = read_governor();
        let available_governors = read_available_governors();
        let freq_min_mhz = read_freq_khz("cpuinfo_min_freq").map(|k| k / 1000);
        let freq_max_mhz = read_freq_khz("cpuinfo_max_freq").map(|k| k / 1000);
        let freq_current_mhz = read_freq_khz("scaling_cur_freq").map(|k| k / 1000);
        let (profile, ppd_available) = read_power_profile();
        let cpupower_available = Command::new("which").arg("cpupower").output()
            .map(|o| o.status.success()).unwrap_or(false);
        let thermal_zones = read_thermal_zones();
        let thermal_warning = thermal_zones.iter().any(|(_, t)| *t >= 85.0);

        Self {
            profile,
            governor,
            available_governors,
            freq_min_mhz,
            freq_max_mhz,
            freq_current_mhz,
            thermal_zones,
            thermal_warning,
            ppd_available,
            cpupower_available,
        }
    }

    /// Build a summary string for briefing/context injection.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref p) = self.profile {
            parts.push(format!("Power profile: {}", p));
        }
        if let Some(ref g) = self.governor {
            parts.push(format!("CPU governor: {}", g));
        }
        if let (Some(min), Some(max)) = (self.freq_min_mhz, self.freq_max_mhz) {
            if let Some(cur) = self.freq_current_mhz {
                parts.push(format!("CPU freq: {}MHz (range {}-{}MHz)", cur, min, max));
            } else {
                parts.push(format!("CPU freq range: {}-{}MHz", min, max));
            }
        }
        if self.thermal_warning {
            let hot: Vec<String> = self.thermal_zones.iter()
                .filter(|(_, t)| *t >= 85.0)
                .map(|(n, t)| format!("{}: {:.0}°C", n, t))
                .collect();
            parts.push(format!("THERMAL WARNING: {}", hot.join(", ")));
        } else if !self.thermal_zones.is_empty() {
            let max_temp = self.thermal_zones.iter().map(|(_, t)| *t as u32).max().unwrap_or(0);
            parts.push(format!("Max temp: {}°C", max_temp));
        }

        parts.join(" | ")
    }

    /// Returns facts about the power state for wiki-based advice generation.
    /// Anna's LLM + wiki will interpret these facts — no hardcoded advice here.
    pub fn facts_for_context(&self) -> Vec<String> {
        let mut facts = Vec::new();
        if let Some(ref g) = self.governor {
            facts.push(format!("cpu_governor={}", g));
        }
        if let Some(ref p) = self.profile {
            facts.push(format!("power_profile={}", p));
        }
        facts.push(format!("ppd_available={}", self.ppd_available));
        facts.push(format!("thermal_warning={}", self.thermal_warning));
        if self.thermal_warning {
            for (name, temp) in &self.thermal_zones {
                if *temp >= 85.0 {
                    facts.push(format!("thermal_zone_hot={}:{:.0}C", name, temp));
                }
            }
        }
        facts
    }
}

fn read_governor() -> Option<String> {
    // Try first CPU core
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_available_governors() -> Vec<String> {
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
    fs::read_to_string(path).ok()
        .map(|s| s.split_whitespace().map(|g| g.to_string()).collect())
        .unwrap_or_default()
}

fn read_freq_khz(file: &str) -> Option<u32> {
    let path = format!("/sys/devices/system/cpu/cpu0/cpufreq/{}", file);
    fs::read_to_string(&path).ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

fn read_power_profile() -> (Option<String>, bool) {
    let output = Command::new("powerprofilesctl")
        .arg("get")
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let profile = String::from_utf8_lossy(&o.stdout).trim().to_string();
            (Some(profile), true)
        }
        Ok(_) => (None, true), // ppd installed but returned error
        Err(_) => (None, false), // ppd not installed
    }
}

fn read_thermal_zones() -> Vec<(String, f32)> {
    let mut zones = Vec::new();
    let base = std::path::Path::new("/sys/class/thermal");
    if !base.exists() {
        return zones;
    }

    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return zones,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("thermal_zone") {
            continue;
        }

        let zone_path = entry.path();
        let temp_path = zone_path.join("temp");
        let type_path = zone_path.join("type");

        let temp_raw = match fs::read_to_string(&temp_path) {
            Ok(t) => t.trim().parse::<i32>().unwrap_or(0),
            Err(_) => continue,
        };

        let zone_type = fs::read_to_string(&type_path)
            .unwrap_or_else(|_| name.clone())
            .trim()
            .to_string();

        let celsius = temp_raw as f32 / 1000.0;
        if celsius > 0.0 && celsius < 150.0 {
            debug!("Thermal zone {}: {:.1}°C", zone_type, celsius);
            zones.push((zone_type, celsius));
        }
    }

    zones
}

/// Build a telemetry section for briefing injection.
pub fn power_telemetry() -> String {
    let state = PowerState::capture();
    format!("## Power & Thermal\n{}\n", state.summary())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_state_capture_no_panic() {
        let state = PowerState::capture();
        // Should not panic, even if no /sys/class/thermal or cpufreq
        let _ = state.summary();
        let _ = state.facts_for_context();
    }

    #[test]
    fn test_thermal_zones_bounded() {
        let zones = read_thermal_zones();
        for (_, temp) in &zones {
            assert!(*temp > 0.0 && *temp < 150.0, "Thermal reading out of range: {}", temp);
        }
    }
}

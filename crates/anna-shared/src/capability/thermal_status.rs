//! Thermal Status: Temperature and Cooling Monitoring
//!
//! Capability: system.thermal.status (ReadOnly)
//!
//! Phase 33: Deterministic probing of thermal state.
//!
//! What this does:
//! - Probe /sys/class/thermal for thermal zones
//! - Probe /sys/class/hwmon for hardware sensors
//! - Detect fan speeds if available
//! - Report current vs critical temperatures
//!
//! What this does NOT do:
//! - Does not configure fan curves (that's thermal.fan.curve)
//! - Does not change thermal policy (that's thermal.policy)
//! - Does not install sensors package

use super::response::{CapabilityExecutionResult, ResponseArtifact};
use std::fs;
use std::path::Path;

// =============================================================================
// PROBE TYPES
// =============================================================================

/// Thermal zone from /sys/class/thermal.
#[derive(Debug, Clone)]
pub struct ThermalZone {
    pub name: String,
    pub zone_type: String,
    pub temp_current: f64,  // Celsius
    pub temp_critical: Option<f64>,
    pub temp_high: Option<f64>,
    pub mode: String,  // enabled/disabled
}

/// Hardware sensor from /sys/class/hwmon.
#[derive(Debug, Clone)]
pub struct HwmonSensor {
    pub name: String,
    pub label: Option<String>,
    pub temp: f64,  // Celsius
    pub temp_max: Option<f64>,
    pub temp_crit: Option<f64>,
}

/// Fan speed sensor.
#[derive(Debug, Clone)]
pub struct FanSensor {
    pub name: String,
    pub rpm: u32,
    pub min_rpm: Option<u32>,
    pub max_rpm: Option<u32>,
}

/// Complete thermal probe results.
#[derive(Debug, Clone)]
pub struct ThermalProbes {
    pub thermal_zones: Vec<ThermalZone>,
    pub hwmon_sensors: Vec<HwmonSensor>,
    pub fans: Vec<FanSensor>,
    pub highest_temp: Option<f64>,
    pub any_critical: bool,
    pub any_high: bool,
}

impl ThermalProbes {
    /// Phase 35: Find the hottest sensor with its details.
    fn hottest_sensor(&self) -> Option<(&str, f64, Option<f64>)> {
        let mut hottest: Option<(&str, f64, Option<f64>)> = None;
        for z in &self.thermal_zones {
            if hottest.map_or(true, |(_, t, _)| z.temp_current > t) {
                hottest = Some((&z.zone_type, z.temp_current, z.temp_critical));
            }
        }
        for s in &self.hwmon_sensors {
            if hottest.map_or(true, |(_, t, _)| s.temp > t) {
                let label = s.label.as_deref().unwrap_or(&s.name);
                hottest = Some((label, s.temp, s.temp_crit));
            }
        }
        hottest
    }

    /// Phase 35: Convert probes to evidence - CAPPED AT 3 LINES.
    pub fn to_evidence(&self) -> Vec<ResponseArtifact> {
        let mut evidence = vec![];

        // Line 1: Hottest sensor with temp and critical
        if let Some((name, temp, crit)) = self.hottest_sensor() {
            let crit_info = crit.map(|c| format!(" (critical: {:.0}C)", c)).unwrap_or_default();
            evidence.push(ResponseArtifact::evidence(
                &format!("{}:", name),
                &format!("{:.1}C{}", temp, crit_info),
            ));
        }

        // Line 2: Status
        let status = if self.any_critical {
            "CRITICAL - exceeds safe limits"
        } else if self.any_high {
            "WARNING - elevated"
        } else {
            "OK - normal"
        };
        evidence.push(ResponseArtifact::evidence("Status:", status));

        // Line 3: Fan info (if any)
        if !self.fans.is_empty() {
            let rpm = self.fans.first().map(|f| f.rpm).unwrap_or(0);
            evidence.push(ResponseArtifact::evidence("Fan:", &format!("{} RPM", rpm)));
        }

        evidence
    }

    /// Phase 35: Single-line status explanation with hottest sensor details.
    pub fn format_explanation(&self) -> String {
        if let Some((name, temp, crit)) = self.hottest_sensor() {
            let margin_info = crit
                .map(|c| format!(" ({:.0}C below critical)", c - temp))
                .unwrap_or_default();
            let fan_info = if self.fans.is_empty() {
                "No fan sensors detected."
            } else if self.fans.iter().any(|f| f.rpm > 0) {
                ""
            } else {
                "Fans off."
            };
            format!("{}: {:.1}C{}{}", name, temp, margin_info,
                if fan_info.is_empty() { String::new() } else { format!(" {}", fan_info) })
        } else {
            "No thermal sensors detected.".to_string()
        }
    }
}

// =============================================================================
// PROBE IMPLEMENTATION
// =============================================================================

/// Run all probes for thermal status.
pub fn gather_probes() -> ThermalProbes {
    let thermal_zones = probe_thermal_zones();
    let hwmon_sensors = probe_hwmon_sensors();
    let fans = probe_fans();

    // Calculate aggregates
    let all_temps: Vec<f64> = thermal_zones.iter()
        .map(|z| z.temp_current)
        .chain(hwmon_sensors.iter().map(|s| s.temp))
        .collect();

    let highest_temp = all_temps.iter().cloned().fold(None, |max, t| {
        Some(max.map_or(t, |m: f64| m.max(t)))
    });

    let any_critical = thermal_zones.iter().any(|z| {
        z.temp_critical.map_or(false, |c| z.temp_current >= c)
    }) || hwmon_sensors.iter().any(|s| {
        s.temp_crit.map_or(false, |c| s.temp >= c)
    });

    let any_high = thermal_zones.iter().any(|z| {
        z.temp_high.map_or(false, |h| z.temp_current >= h)
    }) || hwmon_sensors.iter().any(|s| {
        s.temp_max.map_or(false, |m| s.temp >= m)
    });

    ThermalProbes {
        thermal_zones,
        hwmon_sensors,
        fans,
        highest_temp,
        any_critical,
        any_high,
    }
}

fn probe_thermal_zones() -> Vec<ThermalZone> {
    let mut zones = Vec::new();
    let thermal_path = Path::new("/sys/class/thermal");
    if !thermal_path.exists() { return zones; }
    if let Ok(entries) = fs::read_dir(thermal_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("thermal_zone") {
                continue;
            }

            let zone_path = entry.path();

            let zone_type = read_sysfs_string(&zone_path.join("type"))
                .unwrap_or_else(|| "unknown".to_string());

            let temp_current = read_sysfs_temp(&zone_path.join("temp"))
                .unwrap_or(0.0);

            let temp_critical = read_sysfs_temp(&zone_path.join("trip_point_0_temp"));
            let temp_high = read_sysfs_temp(&zone_path.join("trip_point_1_temp"));

            let mode = read_sysfs_string(&zone_path.join("mode"))
                .unwrap_or_else(|| "enabled".to_string());

            zones.push(ThermalZone {
                name,
                zone_type,
                temp_current,
                temp_critical,
                temp_high,
                mode,
            });
        }
    }

    zones
}

fn probe_hwmon_sensors() -> Vec<HwmonSensor> {
    let mut sensors = Vec::new();
    let hwmon_path = Path::new("/sys/class/hwmon");
    if !hwmon_path.exists() { return sensors; }
    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let hwmon_dir = entry.path();
            let device_name = read_sysfs_string(&hwmon_dir.join("name"))
                .unwrap_or_else(|| "unknown".to_string());

            // Find temp*_input files
            if let Ok(files) = fs::read_dir(&hwmon_dir) {
                for file in files.flatten() {
                    let filename = file.file_name().to_string_lossy().to_string();
                    if filename.starts_with("temp") && filename.ends_with("_input") {
                        let prefix = filename.trim_end_matches("_input");

                        let temp = read_sysfs_temp(&file.path()).unwrap_or(0.0);
                        let label = read_sysfs_string(&hwmon_dir.join(format!("{}_label", prefix)));
                        let temp_max = read_sysfs_temp(&hwmon_dir.join(format!("{}_max", prefix)));
                        let temp_crit = read_sysfs_temp(&hwmon_dir.join(format!("{}_crit", prefix)));

                        sensors.push(HwmonSensor {
                            name: format!("{}:{}", device_name, prefix),
                            label,
                            temp,
                            temp_max,
                            temp_crit,
                        });
                    }
                }
            }
        }
    }

    sensors
}

fn probe_fans() -> Vec<FanSensor> {
    let mut fans = Vec::new();
    let hwmon_path = Path::new("/sys/class/hwmon");
    if !hwmon_path.exists() { return fans; }
    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let hwmon_dir = entry.path();
            let device_name = read_sysfs_string(&hwmon_dir.join("name"))
                .unwrap_or_else(|| "unknown".to_string());

            // Find fan*_input files
            if let Ok(files) = fs::read_dir(&hwmon_dir) {
                for file in files.flatten() {
                    let filename = file.file_name().to_string_lossy().to_string();
                    if filename.starts_with("fan") && filename.ends_with("_input") {
                        let prefix = filename.trim_end_matches("_input");

                        let rpm = read_sysfs_u32(&file.path()).unwrap_or(0);
                        let min_rpm = read_sysfs_u32(&hwmon_dir.join(format!("{}_min", prefix)));
                        let max_rpm = read_sysfs_u32(&hwmon_dir.join(format!("{}_max", prefix)));

                        fans.push(FanSensor {
                            name: format!("{}:{}", device_name, prefix),
                            rpm,
                            min_rpm,
                            max_rpm,
                        });
                    }
                }
            }
        }
    }

    fans
}

fn read_sysfs_string(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_sysfs_temp(path: &Path) -> Option<f64> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .map(|millidegrees| millidegrees as f64 / 1000.0)
}

fn read_sysfs_u32(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

// =============================================================================
// CAPABILITY HANDLER
// =============================================================================

/// Execute the system.thermal.status capability.
/// Phase 33: ReadOnly capability - returns facts, no mutations.
pub fn execute_thermal_status() -> CapabilityExecutionResult {
    let probes = gather_probes();

    // No thermal zones or sensors found
    if probes.thermal_zones.is_empty() && probes.hwmon_sensors.is_empty() {
        return CapabilityExecutionResult::with_explanation(
            probes.to_evidence(),
            "No thermal sensors detected. Your system may not expose temperature data, \
            or the sensors may not be loaded. Try loading sensor modules with 'sensors-detect'.",
        );
    }

    let explanation = probes.format_explanation();
    CapabilityExecutionResult::with_explanation(probes.to_evidence(), &explanation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_returns_resolved() {
        let result = execute_thermal_status();
        assert!(!result.wants_abstain(), "ReadOnly thermal status should not abstain");
    }

    #[test]
    fn test_evidence_capped_at_three() {
        let probes = ThermalProbes {
            thermal_zones: vec![ThermalZone {
                name: "thermal_zone0".to_string(),
                zone_type: "x86_pkg_temp".to_string(),
                temp_current: 45.0,
                temp_critical: Some(100.0),
                temp_high: Some(85.0),
                mode: "enabled".to_string(),
            }],
            hwmon_sensors: vec![],
            fans: vec![FanSensor { name: "fan1".to_string(), rpm: 2500, min_rpm: None, max_rpm: None }],
            highest_temp: Some(45.0),
            any_critical: false,
            any_high: false,
        };
        let evidence = probes.to_evidence();
        assert!(evidence.len() <= 3, "Phase 35: Evidence must be capped at 3 lines");
    }

    #[test]
    fn test_explanation_single_line_with_hottest() {
        let probes = ThermalProbes {
            thermal_zones: vec![ThermalZone {
                name: "zone0".to_string(), zone_type: "x86_pkg_temp".to_string(),
                temp_current: 45.0, temp_critical: Some(100.0), temp_high: None, mode: "enabled".to_string(),
            }],
            hwmon_sensors: vec![],
            fans: vec![],
            highest_temp: Some(45.0),
            any_critical: false,
            any_high: false,
        };
        let explanation = probes.format_explanation();
        assert!(explanation.contains("x86_pkg_temp"));
        assert!(explanation.contains("45.0C"));
        assert!(explanation.contains("55C below critical"));
    }

    #[test]
    fn test_critical_status_in_evidence() {
        let probes = ThermalProbes {
            thermal_zones: vec![ThermalZone {
                name: "zone0".to_string(), zone_type: "cpu".to_string(),
                temp_current: 105.0, temp_critical: Some(100.0), temp_high: None, mode: "enabled".to_string(),
            }],
            hwmon_sensors: vec![], fans: vec![], highest_temp: Some(105.0), any_critical: true, any_high: true,
        };
        let evidence = probes.to_evidence();
        assert!(evidence.iter().any(|e| e.content.contains("CRITICAL")));
    }
}

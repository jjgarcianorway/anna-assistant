use serde::{Deserialize, Serialize};
use std::fs;

/// Voltage monitoring and anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoltageMonitoring {
    pub voltage_rails: Vec<VoltageRail>,
    pub has_anomalies: bool,
    pub anomalies: Vec<VoltageAnomaly>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoltageRail {
    pub label: String,
    pub current_voltage_mv: Option<f64>, // Millivolts
    pub min_voltage_mv: Option<f64>,
    pub max_voltage_mv: Option<f64>,
    pub nominal_voltage_mv: Option<f64>,
    pub deviation_percent: Option<f64>,
    pub is_critical: bool,
    pub status: VoltageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoltageStatus {
    Normal,
    LowWarning,   // <5% below nominal
    HighWarning,  // >5% above nominal
    LowCritical,  // <10% below nominal
    HighCritical, // >10% above nominal
    Unstable,     // Rapid fluctuations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoltageAnomaly {
    pub rail: String,
    pub anomaly_type: AnomalyType,
    pub severity: AnomolySeverity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    Undervoltage,
    Overvoltage,
    Fluctuation,
    OutOfSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomolySeverity {
    Info,
    Warning,
    Critical,
}

impl VoltageMonitoring {
    /// Detect voltage anomalies across all voltage rails
    pub fn detect() -> Self {
        let voltage_rails = detect_voltage_rails();
        let mut anomalies = Vec::new();

        for rail in &voltage_rails {
            if let Some(anomaly) = check_voltage_anomaly(rail) {
                anomalies.push(anomaly);
            }
        }

        let has_anomalies = !anomalies.is_empty();

        let mut recommendations = Vec::new();

        if has_anomalies {
            let critical_count = anomalies
                .iter()
                .filter(|a| matches!(a.severity, AnomolySeverity::Critical))
                .count();

            if critical_count > 0 {
                recommendations.push(format!(
                    "{} critical voltage anomalies detected - check PSU and power delivery",
                    critical_count
                ));
            }

            let undervoltage_count = anomalies
                .iter()
                .filter(|a| matches!(a.anomaly_type, AnomalyType::Undervoltage))
                .count();

            if undervoltage_count > 0 {
                recommendations.push(format!(
                    "{} undervoltage conditions - may cause system instability",
                    undervoltage_count
                ));
            }

            let overvoltage_count = anomalies
                .iter()
                .filter(|a| matches!(a.anomaly_type, AnomalyType::Overvoltage))
                .count();

            if overvoltage_count > 0 {
                recommendations.push(format!(
                    "{} overvoltage conditions - may damage components",
                    overvoltage_count
                ));
            }
        } else {
            recommendations.push("All voltage rails within normal operating range".to_string());
        }

        Self {
            voltage_rails,
            has_anomalies,
            anomalies,
            recommendations,
        }
    }
}

fn detect_voltage_rails() -> Vec<VoltageRail> {
    let mut rails = Vec::new();

    // Scan /sys/class/hwmon for voltage sensors
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let hwmon_path = entry.path();

            // Find all in*_input files (voltage inputs)
            if let Ok(files) = fs::read_dir(&hwmon_path) {
                for file in files.flatten() {
                    let file_name = file.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    if file_name_str.starts_with("in") && file_name_str.ends_with("_input") {
                        // Extract the sensor number
                        let sensor_num = file_name_str
                            .trim_start_matches("in")
                            .trim_end_matches("_input");

                        // Read voltage value
                        let voltage_path = file.path();
                        let current_voltage_mv = fs::read_to_string(&voltage_path)
                            .ok()
                            .and_then(|s| s.trim().parse::<f64>().ok());

                        // Read label if available
                        let label_path = hwmon_path.join(format!("in{}_label", sensor_num));
                        let label = if label_path.exists() {
                            fs::read_to_string(&label_path)
                                .ok()
                                .map(|s| s.trim().to_string())
                                .unwrap_or_else(|| format!("in{}", sensor_num))
                        } else {
                            format!("in{}", sensor_num)
                        };

                        // Read min/max if available
                        let min_path = hwmon_path.join(format!("in{}_min", sensor_num));
                        let min_voltage_mv = if min_path.exists() {
                            fs::read_to_string(&min_path)
                                .ok()
                                .and_then(|s| s.trim().parse::<f64>().ok())
                        } else {
                            None
                        };

                        let max_path = hwmon_path.join(format!("in{}_max", sensor_num));
                        let max_voltage_mv = if max_path.exists() {
                            fs::read_to_string(&max_path)
                                .ok()
                                .and_then(|s| s.trim().parse::<f64>().ok())
                        } else {
                            None
                        };

                        // Estimate nominal voltage based on label or typical values
                        let nominal_voltage_mv =
                            estimate_nominal_voltage(&label, current_voltage_mv);

                        // Calculate deviation
                        let deviation_percent = if let (Some(current), Some(nominal)) =
                            (current_voltage_mv, nominal_voltage_mv)
                        {
                            Some(((current - nominal) / nominal) * 100.0)
                        } else {
                            None
                        };

                        // Determine if rail is critical (CPU/GPU voltages)
                        let is_critical = label.to_lowercase().contains("vcore")
                            || label.to_lowercase().contains("cpu")
                            || label.to_lowercase().contains("gpu")
                            || label.to_lowercase().contains("v12")
                            || label.to_lowercase().contains("vdd");

                        // Determine voltage status
                        let status = determine_voltage_status(deviation_percent);

                        rails.push(VoltageRail {
                            label,
                            current_voltage_mv,
                            min_voltage_mv,
                            max_voltage_mv,
                            nominal_voltage_mv,
                            deviation_percent,
                            is_critical,
                            status,
                        });
                    }
                }
            }
        }
    }

    rails
}

fn estimate_nominal_voltage(label: &str, current_mv: Option<f64>) -> Option<f64> {
    let label_lower = label.to_lowercase();

    // Try to infer from label
    if label_lower.contains("3.3v") || label_lower.contains("3v3") {
        Some(3300.0)
    } else if label_lower.contains("5v") || label_lower.contains("5vsb") {
        Some(5000.0)
    } else if label_lower.contains("12v") || label_lower.contains("v12") {
        Some(12000.0)
    } else if label_lower.contains("vcore") || label_lower.contains("cpu") {
        // CPU voltage typically 0.8-1.4V, use current as nominal
        current_mv
    } else if label_lower.contains("vddq") || label_lower.contains("dram") {
        Some(1200.0) // DDR4 typical
    } else if label_lower.contains("vddcr") {
        Some(1200.0) // AMD SOC voltage
    } else if label_lower.contains("vbat") {
        Some(3000.0) // CMOS battery
    } else {
        // Use current value as nominal if available
        current_mv
    }
}

fn determine_voltage_status(deviation_percent: Option<f64>) -> VoltageStatus {
    match deviation_percent {
        Some(dev) => {
            if dev.abs() < 2.0 {
                VoltageStatus::Normal
            } else if dev < -10.0 {
                VoltageStatus::LowCritical
            } else if dev < -5.0 {
                VoltageStatus::LowWarning
            } else if dev > 10.0 {
                VoltageStatus::HighCritical
            } else if dev > 5.0 {
                VoltageStatus::HighWarning
            } else {
                VoltageStatus::Normal
            }
        }
        None => VoltageStatus::Normal,
    }
}

fn check_voltage_anomaly(rail: &VoltageRail) -> Option<VoltageAnomaly> {
    match rail.status {
        VoltageStatus::LowWarning => Some(VoltageAnomaly {
            rail: rail.label.clone(),
            anomaly_type: AnomalyType::Undervoltage,
            severity: AnomolySeverity::Warning,
            description: format!(
                "{} is {:.1}% below nominal ({}mV)",
                rail.label,
                rail.deviation_percent.unwrap_or(0.0).abs(),
                rail.current_voltage_mv.unwrap_or(0.0)
            ),
        }),
        VoltageStatus::LowCritical => Some(VoltageAnomaly {
            rail: rail.label.clone(),
            anomaly_type: AnomalyType::Undervoltage,
            severity: AnomolySeverity::Critical,
            description: format!(
                "{} is critically low: {:.1}% below nominal ({}mV)",
                rail.label,
                rail.deviation_percent.unwrap_or(0.0).abs(),
                rail.current_voltage_mv.unwrap_or(0.0)
            ),
        }),
        VoltageStatus::HighWarning => Some(VoltageAnomaly {
            rail: rail.label.clone(),
            anomaly_type: AnomalyType::Overvoltage,
            severity: AnomolySeverity::Warning,
            description: format!(
                "{} is {:.1}% above nominal ({}mV)",
                rail.label,
                rail.deviation_percent.unwrap_or(0.0),
                rail.current_voltage_mv.unwrap_or(0.0)
            ),
        }),
        VoltageStatus::HighCritical => Some(VoltageAnomaly {
            rail: rail.label.clone(),
            anomaly_type: AnomalyType::Overvoltage,
            severity: AnomolySeverity::Critical,
            description: format!(
                "{} is critically high: {:.1}% above nominal ({}mV)",
                rail.label,
                rail.deviation_percent.unwrap_or(0.0),
                rail.current_voltage_mv.unwrap_or(0.0)
            ),
        }),
        VoltageStatus::Unstable => Some(VoltageAnomaly {
            rail: rail.label.clone(),
            anomaly_type: AnomalyType::Fluctuation,
            severity: if rail.is_critical {
                AnomolySeverity::Critical
            } else {
                AnomolySeverity::Warning
            },
            description: format!("{} voltage is unstable", rail.label),
        }),
        VoltageStatus::Normal => None,
    }
}

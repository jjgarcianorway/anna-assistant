use serde::{Deserialize, Serialize};
use std::process::Command;

/// GPU throttling and performance degradation detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuThrottling {
    pub nvidia_throttling: Option<NvidiaThrottling>,
    pub amd_throttling: Option<AmdThrottling>,
    pub intel_throttling: Option<IntelThrottling>,
    pub has_throttling: bool,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvidiaThrottling {
    pub gpu_count: usize,
    pub per_gpu_throttling: Vec<NvidiaGpuThrottle>,
    pub thermal_throttling_detected: bool,
    pub power_throttling_detected: bool,
    pub hw_slowdown_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvidiaGpuThrottle {
    pub gpu_id: usize,
    pub gpu_name: String,
    pub temperature_c: Option<f64>,
    pub power_draw_w: Option<f64>,
    pub power_limit_w: Option<f64>,
    pub gpu_utilization_percent: Option<u32>,
    pub memory_utilization_percent: Option<u32>,
    pub throttle_reasons: Vec<String>,
    pub performance_state: Option<String>, // P0-P12
    pub clocks_throttled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmdThrottling {
    pub gpu_count: usize,
    pub per_gpu_throttling: Vec<AmdGpuThrottle>,
    pub thermal_throttling_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmdGpuThrottle {
    pub gpu_id: usize,
    pub device_path: String,
    pub temperature_c: Option<f64>,
    pub power_draw_w: Option<f64>,
    pub gpu_busy_percent: Option<u32>,
    pub throttled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelThrottling {
    pub gpu_count: usize,
    pub per_gpu_throttling: Vec<IntelGpuThrottle>,
    pub thermal_throttling_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelGpuThrottle {
    pub gpu_id: usize,
    pub device_path: String,
    pub temperature_c: Option<f64>,
    pub throttled: bool,
}

impl GpuThrottling {
    /// Detect GPU throttling events across all GPU vendors
    pub fn detect() -> Self {
        let nvidia_throttling = detect_nvidia_throttling();
        let amd_throttling = detect_amd_throttling();
        let intel_throttling = detect_intel_throttling();

        let has_throttling = nvidia_throttling
            .as_ref()
            .map(|n| n.thermal_throttling_detected || n.power_throttling_detected)
            .unwrap_or(false)
            || amd_throttling
                .as_ref()
                .map(|a| a.thermal_throttling_detected)
                .unwrap_or(false)
            || intel_throttling
                .as_ref()
                .map(|i| i.thermal_throttling_detected)
                .unwrap_or(false);

        let mut recommendations = Vec::new();

        if let Some(ref nvidia) = nvidia_throttling {
            if nvidia.thermal_throttling_detected {
                recommendations.push(
                    "NVIDIA GPU thermal throttling detected - check GPU cooling and case airflow"
                        .to_string(),
                );
            }
            if nvidia.power_throttling_detected {
                recommendations.push(
                    "NVIDIA GPU power throttling detected - GPU hitting power limit".to_string(),
                );
            }
            if nvidia.hw_slowdown_detected {
                recommendations.push(
                    "NVIDIA GPU hardware slowdown detected - thermal or power emergency"
                        .to_string(),
                );
            }
        }

        if let Some(ref amd) = amd_throttling {
            if amd.thermal_throttling_detected {
                recommendations
                    .push("AMD GPU thermal throttling detected - improve GPU cooling".to_string());
            }
        }

        if let Some(ref intel) = intel_throttling {
            if intel.thermal_throttling_detected {
                recommendations.push(
                    "Intel GPU thermal throttling detected - check system cooling".to_string(),
                );
            }
        }

        if !has_throttling {
            recommendations
                .push("No GPU throttling detected - GPU thermal performance is good".to_string());
        }

        Self {
            nvidia_throttling,
            amd_throttling,
            intel_throttling,
            has_throttling,
            recommendations,
        }
    }
}

fn detect_nvidia_throttling() -> Option<NvidiaThrottling> {
    // Check if nvidia-smi is available
    let check = Command::new("which").arg("nvidia-smi").output().ok()?;
    if !check.status.success() {
        return None;
    }

    // Query nvidia-smi for GPU information
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,name,temperature.gpu,power.draw,power.limit,utilization.gpu,utilization.memory,pstate,clocks_throttle_reasons.active",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut per_gpu_throttling = Vec::new();
    let mut thermal_throttling_detected = false;
    let mut power_throttling_detected = false;
    let mut hw_slowdown_detected = false;

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 9 {
            continue;
        }

        let gpu_id = parts[0].parse::<usize>().unwrap_or(0);
        let gpu_name = parts[1].to_string();
        let temperature_c = parts[2].parse::<f64>().ok();
        let power_draw_w = parts[3].parse::<f64>().ok();
        let power_limit_w = parts[4].parse::<f64>().ok();
        let gpu_utilization_percent = parts[5].parse::<u32>().ok();
        let memory_utilization_percent = parts[6].parse::<u32>().ok();
        let performance_state = if parts[7] != "[N/A]" && !parts[7].is_empty() {
            Some(parts[7].to_string())
        } else {
            None
        };

        // Parse throttle reasons
        let throttle_reasons_str = parts[8];
        let mut throttle_reasons = Vec::new();
        let mut clocks_throttled = false;

        if throttle_reasons_str.contains("Active") {
            clocks_throttled = true;
        }

        // Parse individual throttle reasons from nvidia-smi output
        // Reasons can include: Gpu Idle, Applications Clocks Setting, SW Power Cap, HW Slowdown, HW Thermal Slowdown, HW Power Brake Slowdown, Sync Boost
        if throttle_reasons_str.contains("HW Thermal Slowdown") {
            throttle_reasons.push("HW Thermal Slowdown".to_string());
            thermal_throttling_detected = true;
            hw_slowdown_detected = true;
        }
        if throttle_reasons_str.contains("SW Power Cap")
            || throttle_reasons_str.contains("HW Power Brake")
        {
            throttle_reasons.push("Power Limit".to_string());
            power_throttling_detected = true;
        }
        if throttle_reasons_str.contains("HW Slowdown") {
            throttle_reasons.push("HW Slowdown".to_string());
            hw_slowdown_detected = true;
        }

        per_gpu_throttling.push(NvidiaGpuThrottle {
            gpu_id,
            gpu_name,
            temperature_c,
            power_draw_w,
            power_limit_w,
            gpu_utilization_percent,
            memory_utilization_percent,
            throttle_reasons,
            performance_state,
            clocks_throttled,
        });
    }

    if per_gpu_throttling.is_empty() {
        return None;
    }

    Some(NvidiaThrottling {
        gpu_count: per_gpu_throttling.len(),
        per_gpu_throttling,
        thermal_throttling_detected,
        power_throttling_detected,
        hw_slowdown_detected,
    })
}

fn detect_amd_throttling() -> Option<AmdThrottling> {
    use std::fs;
    use std::path::Path;

    let hwmon_base = Path::new("/sys/class/drm");
    let mut per_gpu_throttling = Vec::new();
    let mut thermal_throttling_detected = false;

    // Iterate through DRM devices to find AMD GPUs
    if let Ok(entries) = fs::read_dir(hwmon_base) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Look for card* directories
            if !name_str.starts_with("card") || name_str.contains("-") {
                continue;
            }

            // Check if it's an AMD GPU by looking for device/vendor
            let vendor_path = path.join("device/vendor");
            if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                // AMD vendor ID is 0x1002
                if !vendor.trim().contains("0x1002") {
                    continue;
                }
            } else {
                continue;
            }

            // Try to find hwmon directory
            let device_hwmon = path.join("device/hwmon");
            if !device_hwmon.exists() {
                continue;
            }

            if let Ok(hwmon_entries) = fs::read_dir(&device_hwmon) {
                for hwmon_entry in hwmon_entries.flatten() {
                    let hwmon_path = hwmon_entry.path();

                    // Read temperature
                    let temp_input = hwmon_path.join("temp1_input");
                    let temperature_c = if temp_input.exists() {
                        fs::read_to_string(&temp_input)
                            .ok()
                            .and_then(|s| s.trim().parse::<f64>().ok())
                            .map(|t| t / 1000.0) // Convert millidegrees to degrees
                    } else {
                        None
                    };

                    // Read power draw
                    let power_input = hwmon_path.join("power1_average");
                    let power_draw_w = if power_input.exists() {
                        fs::read_to_string(&power_input)
                            .ok()
                            .and_then(|s| s.trim().parse::<f64>().ok())
                            .map(|p| p / 1_000_000.0) // Convert microwatts to watts
                    } else {
                        None
                    };

                    // Read GPU busy percentage
                    let busy_path = path.join("device/gpu_busy_percent");
                    let gpu_busy_percent = if busy_path.exists() {
                        fs::read_to_string(&busy_path)
                            .ok()
                            .and_then(|s| s.trim().parse::<u32>().ok())
                    } else {
                        None
                    };

                    // AMD GPUs throttle when temperature exceeds threshold (typically 110°C for junction temp)
                    // For edge temp (temp1), throttling typically starts around 90-100°C
                    let throttled = temperature_c.map(|t| t > 90.0).unwrap_or(false);
                    if throttled {
                        thermal_throttling_detected = true;
                    }

                    // Extract card number from name_str (e.g., "card0" -> 0)
                    let gpu_id = name_str
                        .trim_start_matches("card")
                        .parse::<usize>()
                        .unwrap_or(per_gpu_throttling.len());

                    per_gpu_throttling.push(AmdGpuThrottle {
                        gpu_id,
                        device_path: path.to_string_lossy().to_string(),
                        temperature_c,
                        power_draw_w,
                        gpu_busy_percent,
                        throttled,
                    });
                }
            }
        }
    }

    if per_gpu_throttling.is_empty() {
        return None;
    }

    Some(AmdThrottling {
        gpu_count: per_gpu_throttling.len(),
        per_gpu_throttling,
        thermal_throttling_detected,
    })
}

fn detect_intel_throttling() -> Option<IntelThrottling> {
    use std::fs;
    use std::path::Path;

    let hwmon_base = Path::new("/sys/class/drm");
    let mut per_gpu_throttling = Vec::new();
    let mut thermal_throttling_detected = false;

    // Iterate through DRM devices to find Intel GPUs
    if let Ok(entries) = fs::read_dir(hwmon_base) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Look for card* directories
            if !name_str.starts_with("card") || name_str.contains("-") {
                continue;
            }

            // Check if it's an Intel GPU by looking for device/vendor
            let vendor_path = path.join("device/vendor");
            if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                // Intel vendor ID is 0x8086
                if !vendor.trim().contains("0x8086") {
                    continue;
                }
            } else {
                continue;
            }

            // Try to find i915 hwmon directory
            let gt_dir = path.join("gt");
            if !gt_dir.exists() {
                continue;
            }

            if let Ok(gt_entries) = fs::read_dir(&gt_dir) {
                for gt_entry in gt_entries.flatten() {
                    let gt_path = gt_entry.path();

                    // Look for hwmon directory
                    if let Ok(hwmon_entries) = fs::read_dir(&gt_path) {
                        for hwmon_entry in hwmon_entries.flatten() {
                            let hwmon_name = hwmon_entry.file_name();
                            if !hwmon_name.to_string_lossy().starts_with("hwmon") {
                                continue;
                            }

                            let hwmon_path = hwmon_entry.path();

                            // Read temperature
                            let temp_input = hwmon_path.join("temp1_input");
                            let temperature_c = if temp_input.exists() {
                                fs::read_to_string(&temp_input)
                                    .ok()
                                    .and_then(|s| s.trim().parse::<f64>().ok())
                                    .map(|t| t / 1000.0) // Convert millidegrees to degrees
                            } else {
                                None
                            };

                            // Intel integrated GPUs typically throttle around 100°C
                            let throttled = temperature_c.map(|t| t > 95.0).unwrap_or(false);
                            if throttled {
                                thermal_throttling_detected = true;
                            }

                            // Extract card number from name_str (e.g., "card0" -> 0)
                            let gpu_id = name_str
                                .trim_start_matches("card")
                                .parse::<usize>()
                                .unwrap_or(per_gpu_throttling.len());

                            per_gpu_throttling.push(IntelGpuThrottle {
                                gpu_id,
                                device_path: path.to_string_lossy().to_string(),
                                temperature_c,
                                throttled,
                            });
                        }
                    }
                }
            }
        }
    }

    if per_gpu_throttling.is_empty() {
        return None;
    }

    Some(IntelThrottling {
        gpu_count: per_gpu_throttling.len(),
        per_gpu_throttling,
        thermal_throttling_detected,
    })
}

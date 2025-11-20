use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayIssues {
    pub driver_issues: Vec<DriverIssue>,
    pub display_config: Option<DisplayConfiguration>,
    pub multi_monitor_issues: Vec<MultiMonitorIssue>,
    pub overall_status: DisplayStatus,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverIssue {
    pub severity: IssueSeverity,
    pub gpu_vendor: String,
    pub issue_type: String,
    pub message: String,
    pub source: String, // Xorg.log, dmesg, etc.
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfiguration {
    pub session_type: SessionType,
    pub displays: Vec<Display>,
    pub primary_display: Option<String>,
    pub total_screens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    pub name: String,
    pub connected: bool,
    pub resolution: Option<Resolution>,
    pub refresh_rate: Option<f64>,
    pub is_primary: bool,
    pub rotation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMonitorIssue {
    pub issue_type: MultiMonitorIssueType,
    pub description: String,
    pub affected_displays: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    X11,
    Wayland,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultiMonitorIssueType {
    ResolutionMismatch,
    RefreshRateMismatch,
    MissingPrimary,
    DisconnectedDisplay,
    RotationIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisplayStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

impl DisplayIssues {
    pub fn detect() -> Self {
        let driver_issues = detect_driver_issues();
        let display_config = detect_display_configuration();
        let multi_monitor_issues = detect_multi_monitor_issues(&display_config);

        let overall_status = calculate_status(&driver_issues, &multi_monitor_issues);
        let recommendations =
            generate_recommendations(&driver_issues, &display_config, &multi_monitor_issues);

        DisplayIssues {
            driver_issues,
            display_config,
            multi_monitor_issues,
            overall_status,
            recommendations,
        }
    }

    pub fn has_critical_issues(&self) -> bool {
        self.overall_status == DisplayStatus::Critical
    }

    pub fn display_count(&self) -> usize {
        self.display_config
            .as_ref()
            .map(|dc| dc.total_screens)
            .unwrap_or(0)
    }
}

fn detect_driver_issues() -> Vec<DriverIssue> {
    let mut issues = Vec::new();

    // Check Xorg.log for errors
    if let Some(xorg_issues) = check_xorg_log() {
        issues.extend(xorg_issues);
    }

    // Check dmesg for GPU errors
    if let Some(dmesg_issues) = check_dmesg_gpu_errors() {
        issues.extend(dmesg_issues);
    }

    issues
}

fn check_xorg_log() -> Option<Vec<DriverIssue>> {
    // Check common Xorg log locations
    let log_paths = vec![
        "/var/log/Xorg.0.log",
        "/home/*/.local/share/xorg/Xorg.0.log",
    ];

    let mut issues = Vec::new();

    for path in log_paths {
        if let Ok(log_content) = std::fs::read_to_string(path) {
            for line in log_content.lines() {
                // Look for error patterns
                if line.contains("(EE)") || line.contains("ERROR") {
                    // Extract GPU vendor if present
                    let vendor = if line.to_lowercase().contains("nvidia") {
                        "NVIDIA"
                    } else if line.to_lowercase().contains("amd")
                        || line.to_lowercase().contains("radeon")
                    {
                        "AMD"
                    } else if line.to_lowercase().contains("intel") {
                        "Intel"
                    } else {
                        "Unknown"
                    };

                    let severity = if line.contains("Failed") || line.contains("Fatal") {
                        IssueSeverity::Critical
                    } else {
                        IssueSeverity::Warning
                    };

                    issues.push(DriverIssue {
                        severity,
                        gpu_vendor: vendor.to_string(),
                        issue_type: "Xorg Error".to_string(),
                        message: line.trim().to_string(),
                        source: "Xorg.log".to_string(),
                        timestamp: None,
                    });
                }
            }
        }
    }

    if issues.is_empty() {
        None
    } else {
        Some(issues)
    }
}

fn check_dmesg_gpu_errors() -> Option<Vec<DriverIssue>> {
    let output = Command::new("dmesg").arg("-T").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let dmesg = String::from_utf8(output.stdout).ok()?;
    let mut issues = Vec::new();

    let error_patterns = vec![
        (
            "nvidia",
            "NVIDIA",
            vec!["NVRM", "nvidia: probe", "GPU has fallen off"],
        ),
        ("amdgpu", "AMD", vec!["amdgpu", "GPU fault", "ring timeout"]),
        ("i915", "Intel", vec!["i915", "GPU hang", "reset failed"]),
        ("drm", "Generic", vec!["DRM", "error", "failed"]),
    ];

    for line in dmesg.lines() {
        let line_lower = line.to_lowercase();

        for (driver, vendor, patterns) in &error_patterns {
            if line_lower.contains(driver) {
                for pattern in patterns {
                    if line_lower.contains(&pattern.to_lowercase())
                        && (line_lower.contains("error")
                            || line_lower.contains("failed")
                            || line_lower.contains("fault")
                            || line_lower.contains("timeout"))
                    {
                        let timestamp = line
                            .split(']')
                            .next()
                            .and_then(|s| s.strip_prefix('['))
                            .map(|s| s.to_string());

                        issues.push(DriverIssue {
                            severity: IssueSeverity::Warning,
                            gpu_vendor: vendor.to_string(),
                            issue_type: "Kernel GPU Error".to_string(),
                            message: line.to_string(),
                            source: "dmesg".to_string(),
                            timestamp,
                        });
                        break;
                    }
                }
            }
        }
    }

    if issues.is_empty() {
        None
    } else {
        Some(issues)
    }
}

fn detect_display_configuration() -> Option<DisplayConfiguration> {
    // First try to determine session type
    let session_type = detect_session_type();

    match session_type {
        SessionType::X11 => detect_xrandr_config(),
        SessionType::Wayland => detect_wayland_config(),
        SessionType::Unknown => None,
    }
}

fn detect_session_type() -> SessionType {
    // Check XDG_SESSION_TYPE
    if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
        return match session.to_lowercase().as_str() {
            "x11" => SessionType::X11,
            "wayland" => SessionType::Wayland,
            _ => SessionType::Unknown,
        };
    }

    // Check WAYLAND_DISPLAY
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return SessionType::Wayland;
    }

    // Check DISPLAY
    if std::env::var("DISPLAY").is_ok() {
        return SessionType::X11;
    }

    SessionType::Unknown
}

fn detect_xrandr_config() -> Option<DisplayConfiguration> {
    let output = Command::new("xrandr").arg("--query").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let xrandr_output = String::from_utf8(output.stdout).ok()?;
    let mut displays = Vec::new();
    let mut primary_display = None;

    for line in xrandr_output.lines() {
        if line.contains(" connected") || line.contains(" disconnected") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let connected = line.contains(" connected");
                let is_primary = line.contains("primary");

                if is_primary {
                    primary_display = Some(name.clone());
                }

                // Parse resolution and refresh rate
                let (resolution, refresh_rate) = if connected && parts.len() >= 3 {
                    parse_resolution_refresh(parts[2])
                } else {
                    (None, None)
                };

                displays.push(Display {
                    name: name.clone(),
                    connected,
                    resolution,
                    refresh_rate,
                    is_primary,
                    rotation: None,
                });
            }
        }
    }

    Some(DisplayConfiguration {
        session_type: SessionType::X11,
        total_screens: displays.iter().filter(|d| d.connected).count(),
        displays,
        primary_display,
    })
}

fn detect_wayland_config() -> Option<DisplayConfiguration> {
    // For Wayland, we'd need compositor-specific commands
    // For now, try swaymsg if available
    if is_command_available("swaymsg") {
        return detect_sway_outputs();
    }

    // Could add support for other compositors here
    None
}

fn detect_sway_outputs() -> Option<DisplayConfiguration> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_outputs", "-r"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Parse JSON output
    let outputs: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let mut displays = Vec::new();
    let mut primary_display = None;

    if let Some(outputs_array) = outputs.as_array() {
        for output_obj in outputs_array {
            let name = output_obj["name"].as_str()?.to_string();
            let active = output_obj["active"].as_bool().unwrap_or(false);
            let focused = output_obj["focused"].as_bool().unwrap_or(false);

            if focused {
                primary_display = Some(name.clone());
            }

            let resolution = if active {
                let width = output_obj["current_mode"]["width"].as_u64()? as u32;
                let height = output_obj["current_mode"]["height"].as_u64()? as u32;
                Some(Resolution { width, height })
            } else {
                None
            };

            let refresh_rate = if active {
                output_obj["current_mode"]["refresh"]
                    .as_f64()
                    .map(|r| r / 1000.0)
            } else {
                None
            };

            displays.push(Display {
                name,
                connected: active,
                resolution,
                refresh_rate,
                is_primary: focused,
                rotation: None,
            });
        }
    }

    Some(DisplayConfiguration {
        session_type: SessionType::Wayland,
        total_screens: displays.iter().filter(|d| d.connected).count(),
        displays,
        primary_display,
    })
}

fn parse_resolution_refresh(mode_str: &str) -> (Option<Resolution>, Option<f64>) {
    // Format: 1920x1080+0+0 or similar
    if let Some(resolution_part) = mode_str.split('+').next() {
        let parts: Vec<&str> = resolution_part.split('x').collect();

        if parts.len() == 2 {
            if let (Ok(width), Ok(height)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return (Some(Resolution { width, height }), None);
            }
        }
    }

    (None, None)
}

fn detect_multi_monitor_issues(config: &Option<DisplayConfiguration>) -> Vec<MultiMonitorIssue> {
    let mut issues = Vec::new();

    if let Some(dc) = config {
        if dc.total_screens > 1 {
            // Check for missing primary
            if dc.primary_display.is_none() {
                issues.push(MultiMonitorIssue {
                    issue_type: MultiMonitorIssueType::MissingPrimary,
                    description: "No primary display configured in multi-monitor setup".to_string(),
                    affected_displays: dc
                        .displays
                        .iter()
                        .filter(|d| d.connected)
                        .map(|d| d.name.clone())
                        .collect(),
                });
            }

            // Check for resolution mismatches
            let resolutions: Vec<_> = dc
                .displays
                .iter()
                .filter(|d| d.connected && d.resolution.is_some())
                .map(|d| d.resolution.as_ref().unwrap())
                .collect();

            if resolutions.len() > 1 {
                let first_res = resolutions[0];
                let all_same = resolutions
                    .iter()
                    .all(|r| r.width == first_res.width && r.height == first_res.height);

                if !all_same {
                    issues.push(MultiMonitorIssue {
                        issue_type: MultiMonitorIssueType::ResolutionMismatch,
                        description:
                            "Displays have different resolutions - may cause scaling issues"
                                .to_string(),
                        affected_displays: dc
                            .displays
                            .iter()
                            .filter(|d| d.connected)
                            .map(|d| {
                                format!(
                                    "{} ({}x{})",
                                    d.name,
                                    d.resolution.as_ref().map(|r| r.width).unwrap_or(0),
                                    d.resolution.as_ref().map(|r| r.height).unwrap_or(0)
                                )
                            })
                            .collect(),
                    });
                }
            }

            // Check for refresh rate mismatches
            let refresh_rates: Vec<_> = dc
                .displays
                .iter()
                .filter(|d| d.connected && d.refresh_rate.is_some())
                .map(|d| d.refresh_rate.unwrap())
                .collect();

            if refresh_rates.len() > 1 {
                let first_rate = refresh_rates[0];
                let all_same = refresh_rates.iter().all(|r| (r - first_rate).abs() < 1.0);

                if !all_same {
                    issues.push(MultiMonitorIssue {
                        issue_type: MultiMonitorIssueType::RefreshRateMismatch,
                        description: "Displays have different refresh rates - may cause tearing"
                            .to_string(),
                        affected_displays: dc
                            .displays
                            .iter()
                            .filter(|d| d.connected)
                            .map(|d| {
                                format!(
                                    "{} ({}Hz)",
                                    d.name,
                                    d.refresh_rate.map(|r| r as u32).unwrap_or(0)
                                )
                            })
                            .collect(),
                    });
                }
            }
        }

        // Check for disconnected displays in config
        let disconnected: Vec<_> = dc
            .displays
            .iter()
            .filter(|d| !d.connected)
            .map(|d| d.name.clone())
            .collect();

        if !disconnected.is_empty() {
            issues.push(MultiMonitorIssue {
                issue_type: MultiMonitorIssueType::DisconnectedDisplay,
                description: "Configured displays are disconnected".to_string(),
                affected_displays: disconnected,
            });
        }
    }

    issues
}

fn calculate_status(
    driver_issues: &[DriverIssue],
    multi_monitor_issues: &[MultiMonitorIssue],
) -> DisplayStatus {
    let has_critical_driver = driver_issues
        .iter()
        .any(|i| matches!(i.severity, IssueSeverity::Critical));

    if has_critical_driver {
        return DisplayStatus::Critical;
    }

    if !driver_issues.is_empty() || !multi_monitor_issues.is_empty() {
        return DisplayStatus::Warning;
    }

    DisplayStatus::Healthy
}

fn generate_recommendations(
    driver_issues: &[DriverIssue],
    _config: &Option<DisplayConfiguration>,
    multi_monitor_issues: &[MultiMonitorIssue],
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !driver_issues.is_empty() {
        recommendations.push(format!(
            "Found {} display driver issue(s) - check Xorg.log and dmesg for details",
            driver_issues.len()
        ));

        // GPU-specific recommendations
        let nvidia_issues = driver_issues.iter().any(|i| i.gpu_vendor == "NVIDIA");
        let amd_issues = driver_issues.iter().any(|i| i.gpu_vendor == "AMD");

        if nvidia_issues {
            recommendations.push(
                "NVIDIA issues detected - consider updating drivers or kernel parameters"
                    .to_string(),
            );
        }
        if amd_issues {
            recommendations
                .push("AMD issues detected - check amdgpu/radeon driver configuration".to_string());
        }
    }

    for issue in multi_monitor_issues {
        match issue.issue_type {
            MultiMonitorIssueType::MissingPrimary => {
                recommendations
                    .push("Set a primary display for better multi-monitor experience".to_string());
            }
            MultiMonitorIssueType::ResolutionMismatch => {
                recommendations.push(
                    "Consider matching display resolutions or adjusting scaling settings"
                        .to_string(),
                );
            }
            MultiMonitorIssueType::RefreshRateMismatch => {
                recommendations
                    .push("Match refresh rates across displays to prevent tearing".to_string());
            }
            MultiMonitorIssueType::DisconnectedDisplay => {
                recommendations.push("Remove disconnected displays from configuration".to_string());
            }
            _ => {}
        }
    }

    recommendations
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_detection() {
        let issues = DisplayIssues::detect();
        assert!(matches!(
            issues.overall_status,
            DisplayStatus::Healthy | DisplayStatus::Warning | DisplayStatus::Critical
        ));
    }
}

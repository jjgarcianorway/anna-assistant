//! System health checks with remediation hints

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub title: String,
    pub status: CheckStatus,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CheckStatus {
    Pass,
    Warn,
    Error,
    Info,
}

impl CheckStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "✅",
            CheckStatus::Warn => "⚠️ ",
            CheckStatus::Error => "❌",
            CheckStatus::Info => "ℹ️ ",
        }
    }
}

pub fn run_checks() -> Result<Vec<Check>> {
    let mut checks = Vec::new();

    // Core system checks
    checks.push(check_cpu_metrics());
    checks.push(check_cpu_governor());
    checks.push(check_gpu_driver());
    checks.push(check_vaapi());
    checks.push(check_audio_server());
    checks.push(check_network_interfaces());
    checks.push(check_boot_time());
    checks.push(check_session_type());

    // Maintenance checks
    checks.push(check_trim_timer());
    checks.push(check_firmware_updates());
    checks.push(check_journald_persistence());

    Ok(checks)
}

fn check_vaapi() -> Check {
    if let Ok(output) = Command::new("vainfo").output() {
        if output.status.success() {
            Check {
                id: "vaapi".to_string(),
                title: "Hardware Video Acceleration".to_string(),
                status: CheckStatus::Pass,
                message: "VA-API is available for GPU-accelerated video".to_string(),
                remediation: Some("Enable in browser: annactl ask 'enable browser hardware acceleration'".to_string()),
            }
        } else {
            Check {
                id: "vaapi".to_string(),
                title: "Hardware Video Acceleration".to_string(),
                status: CheckStatus::Warn,
                message: "VA-API not detected - videos will use CPU decoding".to_string(),
                remediation: Some("Install: sudo pacman -S libva-mesa-driver libva-intel-driver".to_string()),
            }
        }
    } else {
        Check {
            id: "vaapi".to_string(),
            title: "Hardware Video Acceleration".to_string(),
            status: CheckStatus::Info,
            message: "VA-API tools not installed, can't check support".to_string(),
            remediation: Some("Install: sudo pacman -S libva-utils".to_string()),
        }
    }
}

fn check_audio_server() -> Check {
    if let Ok(output) = Command::new("pactl").arg("info").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.contains("PipeWire") {
                Check {
                    id: "audio".to_string(),
                    title: "Audio Server".to_string(),
                    status: CheckStatus::Pass,
                    message: "PipeWire is running (modern, low-latency)".to_string(),
                    remediation: None,
                }
            } else {
                Check {
                    id: "audio".to_string(),
                    title: "Audio Server".to_string(),
                    status: CheckStatus::Pass,
                    message: "PulseAudio is running".to_string(),
                    remediation: Some("Consider PipeWire for lower latency: sudo pacman -S pipewire-pulse".to_string()),
                }
            }
        } else {
            Check {
                id: "audio".to_string(),
                title: "Audio Server".to_string(),
                status: CheckStatus::Error,
                message: "No audio server detected".to_string(),
                remediation: Some("Install: sudo pacman -S pipewire-pulse".to_string()),
            }
        }
    } else {
        Check {
            id: "audio".to_string(),
            title: "Audio Server".to_string(),
            status: CheckStatus::Warn,
            message: "Can't check audio server (pactl not found)".to_string(),
            remediation: None,
        }
    }
}

fn check_boot_time() -> Check {
    if let Ok(output) = Command::new("systemd-analyze").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            // Parse boot time (format: "Startup finished in 2.345s (kernel) + 3.456s (userspace) = 5.801s")
            if let Some(line) = text.lines().find(|l| l.contains("Startup finished")) {
                if line.contains("= ") {
                    let total = line.split("= ").nth(1).and_then(|s| s.split_whitespace().next()).unwrap_or("unknown");
                    let seconds = total.trim_end_matches('s').parse::<f64>().unwrap_or(999.0);

                    if seconds < 15.0 {
                        Check {
                            id: "boot_time".to_string(),
                            title: "Boot Performance".to_string(),
                            status: CheckStatus::Pass,
                            message: format!("Fast boot: {}", total),
                            remediation: None,
                        }
                    } else if seconds < 30.0 {
                        Check {
                            id: "boot_time".to_string(),
                            title: "Boot Performance".to_string(),
                            status: CheckStatus::Pass,
                            message: format!("Good boot: {}", total),
                            remediation: Some("Can be faster: annactl ask 'speed up my boot'".to_string()),
                        }
                    } else {
                        Check {
                            id: "boot_time".to_string(),
                            title: "Boot Performance".to_string(),
                            status: CheckStatus::Warn,
                            message: format!("Slow boot: {}", total),
                            remediation: Some("Optimize: annactl ask 'speed up my boot'".to_string()),
                        }
                    }
                } else {
                    Check {
                        id: "boot_time".to_string(),
                        title: "Boot Performance".to_string(),
                        status: CheckStatus::Info,
                        message: "Boot time analysis available but format unexpected".to_string(),
                        remediation: None,
                    }
                }
            } else {
                Check {
                    id: "boot_time".to_string(),
                    title: "Boot Performance".to_string(),
                    status: CheckStatus::Info,
                    message: "Boot time data not available yet".to_string(),
                    remediation: None,
                }
            }
        } else {
            Check {
                id: "boot_time".to_string(),
                title: "Boot Performance".to_string(),
                status: CheckStatus::Info,
                message: "Can't analyze boot time".to_string(),
                remediation: None,
            }
        }
    } else {
        Check {
            id: "boot_time".to_string(),
            title: "Boot Performance".to_string(),
            status: CheckStatus::Info,
            message: "systemd-analyze not available".to_string(),
            remediation: None,
        }
    }
}

fn check_cpu_governor() -> Check {
    if let Ok(governor) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
        let gov = governor.trim();
        match gov {
            "performance" => Check {
                id: "cpu_governor".to_string(),
                title: "CPU Governor".to_string(),
                status: CheckStatus::Pass,
                message: "Performance mode (max speed)".to_string(),
                remediation: Some("For battery: annactl ask 'optimize power usage'".to_string()),
            },
            "powersave" => Check {
                id: "cpu_governor".to_string(),
                title: "CPU Governor".to_string(),
                status: CheckStatus::Pass,
                message: "Powersave mode (battery friendly)".to_string(),
                remediation: None,
            },
            "schedutil" => Check {
                id: "cpu_governor".to_string(),
                title: "CPU Governor".to_string(),
                status: CheckStatus::Pass,
                message: "Schedutil mode (balanced, recommended)".to_string(),
                remediation: None,
            },
            other => Check {
                id: "cpu_governor".to_string(),
                title: "CPU Governor".to_string(),
                status: CheckStatus::Info,
                message: format!("Governor: {}", other),
                remediation: None,
            },
        }
    } else {
        Check {
            id: "cpu_governor".to_string(),
            title: "CPU Governor".to_string(),
            status: CheckStatus::Info,
            message: "CPU frequency scaling not available or not accessible".to_string(),
            remediation: None,
        }
    }
}

fn check_trim_timer() -> Check {
    if let Ok(output) = Command::new("systemctl")
        .arg("is-enabled")
        .arg("fstrim.timer")
        .output()
    {
        if output.status.success() {
            Check {
                id: "trim_timer".to_string(),
                title: "SSD TRIM".to_string(),
                status: CheckStatus::Pass,
                message: "Weekly TRIM enabled (keeps SSD healthy)".to_string(),
                remediation: None,
            }
        } else {
            Check {
                id: "trim_timer".to_string(),
                title: "SSD TRIM".to_string(),
                status: CheckStatus::Warn,
                message: "TRIM timer not enabled - SSD performance may degrade".to_string(),
                remediation: Some("Enable: sudo systemctl enable --now fstrim.timer".to_string()),
            }
        }
    } else {
        Check {
            id: "trim_timer".to_string(),
            title: "SSD TRIM".to_string(),
            status: CheckStatus::Info,
            message: "Can't check TRIM status".to_string(),
            remediation: None,
        }
    }
}

fn check_firmware_updates() -> Check {
    if let Ok(output) = Command::new("fwupdmgr").arg("get-updates").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        if text.contains("No updates available") || text.contains("No upgrades") {
            Check {
                id: "firmware".to_string(),
                title: "Firmware Updates".to_string(),
                status: CheckStatus::Pass,
                message: "Firmware is up to date".to_string(),
                remediation: None,
            }
        } else if output.status.success() {
            Check {
                id: "firmware".to_string(),
                title: "Firmware Updates".to_string(),
                status: CheckStatus::Warn,
                message: "Firmware updates available".to_string(),
                remediation: Some("Update: sudo fwupdmgr update".to_string()),
            }
        } else {
            Check {
                id: "firmware".to_string(),
                title: "Firmware Updates".to_string(),
                status: CheckStatus::Info,
                message: "Can't check firmware (fwupd may not be set up)".to_string(),
                remediation: Some("Install: sudo pacman -S fwupd".to_string()),
            }
        }
    } else {
        Check {
            id: "firmware".to_string(),
            title: "Firmware Updates".to_string(),
            status: CheckStatus::Info,
            message: "fwupd not installed".to_string(),
            remediation: Some("Install: sudo pacman -S fwupd".to_string()),
        }
    }
}

fn check_journald_persistence() -> Check {
    if std::path::Path::new("/var/log/journal").exists() {
        Check {
            id: "journald".to_string(),
            title: "System Logs".to_string(),
            status: CheckStatus::Pass,
            message: "Persistent logging enabled".to_string(),
            remediation: None,
        }
    } else {
        Check {
            id: "journald".to_string(),
            title: "System Logs".to_string(),
            status: CheckStatus::Warn,
            message: "Logs are not persistent (lost on reboot)".to_string(),
            remediation: Some("Enable: sudo mkdir -p /var/log/journal && sudo systemctl restart systemd-journald".to_string()),
        }
    }
}

fn check_session_type() -> Check {
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        match session_type.as_str() {
            "wayland" => Check {
                id: "session".to_string(),
                title: "Display Server".to_string(),
                status: CheckStatus::Pass,
                message: "Wayland session (modern, secure)".to_string(),
                remediation: None,
            },
            "x11" => Check {
                id: "session".to_string(),
                title: "Display Server".to_string(),
                status: CheckStatus::Pass,
                message: "X11 session (stable, widely compatible)".to_string(),
                remediation: Some("Consider Wayland for better security and performance".to_string()),
            },
            other => Check {
                id: "session".to_string(),
                title: "Display Server".to_string(),
                status: CheckStatus::Info,
                message: format!("Session type: {}", other),
                remediation: None,
            },
        }
    } else {
        Check {
            id: "session".to_string(),
            title: "Display Server".to_string(),
            status: CheckStatus::Info,
            message: "Running in console or headless mode".to_string(),
            remediation: None,
        }
    }
}

fn check_cpu_metrics() -> Check {
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_cpu();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let core_count = sys.cpus().len();
    let mem_total = sys.total_memory() / 1024 / 1024; // MB

    if cpu_usage < 50.0 {
        Check {
            id: "cpu_metrics".to_string(),
            title: "CPU & Memory".to_string(),
            status: CheckStatus::Pass,
            message: format!("{} cores, {} MB RAM, current load: {:.1}%", core_count, mem_total, cpu_usage),
            remediation: None,
        }
    } else if cpu_usage < 80.0 {
        Check {
            id: "cpu_metrics".to_string(),
            title: "CPU & Memory".to_string(),
            status: CheckStatus::Warn,
            message: format!("Moderate load: {:.1}% (may slow down tasks)", cpu_usage),
            remediation: Some("Check 'top' or 'htop' to identify resource hogs".to_string()),
        }
    } else {
        Check {
            id: "cpu_metrics".to_string(),
            title: "CPU & Memory".to_string(),
            status: CheckStatus::Error,
            message: format!("High CPU load: {:.1}%", cpu_usage),
            remediation: Some("System may be sluggish. Check 'top' or run 'annactl doctor repair'".to_string()),
        }
    }
}

fn check_gpu_driver() -> Check {
    // Check for common GPU drivers via lspci and loaded modules
    if let Ok(output) = Command::new("lspci").output() {
        let text = String::from_utf8_lossy(&output.stdout).to_lowercase();

        // Detect GPU vendor
        let (vendor, driver_check) = if text.contains("nvidia") {
            ("NVIDIA", vec!["nvidia", "nouveau"])
        } else if text.contains("amd") || text.contains("radeon") {
            ("AMD", vec!["amdgpu", "radeon"])
        } else if text.contains("intel") {
            ("Intel", vec!["i915", "xe"])
        } else {
            return Check {
                id: "gpu_driver".to_string(),
                title: "GPU Driver".to_string(),
                status: CheckStatus::Info,
                message: "No discrete GPU detected or vendor unknown".to_string(),
                remediation: None,
            };
        };

        // Check if driver is loaded
        if let Ok(modules) = std::fs::read_to_string("/proc/modules") {
            let loaded_driver = driver_check.iter()
                .find(|&driver| modules.contains(driver))
                .map(|s| s.to_string());

            if let Some(driver) = loaded_driver {
                Check {
                    id: "gpu_driver".to_string(),
                    title: "GPU Driver".to_string(),
                    status: CheckStatus::Pass,
                    message: format!("{} GPU detected, driver '{}' loaded", vendor, driver),
                    remediation: None,
                }
            } else {
                Check {
                    id: "gpu_driver".to_string(),
                    title: "GPU Driver".to_string(),
                    status: CheckStatus::Warn,
                    message: format!("{} GPU found but no driver loaded", vendor),
                    remediation: Some(format!("Install driver: sudo pacman -S {}",
                        if vendor == "NVIDIA" { "nvidia" } else { "mesa" })),
                }
            }
        } else {
            Check {
                id: "gpu_driver".to_string(),
                title: "GPU Driver".to_string(),
                status: CheckStatus::Info,
                message: format!("{} GPU detected but can't check driver status", vendor),
                remediation: None,
            }
        }
    } else {
        Check {
            id: "gpu_driver".to_string(),
            title: "GPU Driver".to_string(),
            status: CheckStatus::Info,
            message: "Can't detect GPU (lspci unavailable)".to_string(),
            remediation: None,
        }
    }
}

fn check_network_interfaces() -> Check {
    use sysinfo::Networks;

    let networks = Networks::new_with_refreshed_list();
    let active_count = networks.list().iter()
        .filter(|(_, data)| data.total_received() > 0 || data.total_transmitted() > 0)
        .count();

    if active_count == 0 {
        Check {
            id: "network".to_string(),
            title: "Network Interfaces".to_string(),
            status: CheckStatus::Warn,
            message: "No active network interfaces detected".to_string(),
            remediation: Some("Check connection: nmcli device status".to_string()),
        }
    } else if active_count == 1 {
        let (name, _) = networks.list().iter().next().unwrap();
        Check {
            id: "network".to_string(),
            title: "Network Interfaces".to_string(),
            status: CheckStatus::Pass,
            message: format!("1 active interface ({})", name),
            remediation: None,
        }
    } else {
        Check {
            id: "network".to_string(),
            title: "Network Interfaces".to_string(),
            status: CheckStatus::Pass,
            message: format!("{} active interfaces", active_count),
            remediation: None,
        }
    }
}

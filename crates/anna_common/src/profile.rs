/// Machine profile detection for profile-aware behavior
///
/// Anna detects what kind of machine she's running on and adjusts
/// her behavior accordingly to be more helpful and less noisy.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::debug;

/// Machine profile types that Anna recognizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineProfile {
    /// Laptop - has battery, often shorter uptimes, Wi-Fi
    Laptop,

    /// Desktop - no battery, GPU present, graphical session
    Desktop,

    /// Headless or server-like - no battery, no GUI, long uptimes
    ServerLike,

    /// Unknown - couldn't determine profile
    Unknown,
}

impl MachineProfile {
    /// Detect the machine profile based on system characteristics
    pub fn detect() -> Self {
        debug!("Detecting machine profile...");

        // Check for battery (strongest laptop signal)
        let has_battery = Self::has_battery();

        // Check for graphical session
        let has_graphical_session = Self::has_graphical_session();

        // Check for GPU
        let has_gpu = Self::has_gpu();

        // Check for Wi-Fi interface
        let has_wifi = Self::has_wifi();

        // Check uptime (long uptime suggests server)
        let uptime_days = Self::get_uptime_days();

        debug!(
            "Profile signals: battery={}, gui={}, gpu={}, wifi={}, uptime_days={}",
            has_battery, has_graphical_session, has_gpu, has_wifi, uptime_days
        );

        // Decision logic
        let profile = if has_battery {
            // Battery is the strongest signal for laptop
            MachineProfile::Laptop
        } else if has_graphical_session || has_gpu {
            // No battery but has GUI or GPU -> Desktop
            MachineProfile::Desktop
        } else if uptime_days > 30 || (!has_wifi && !has_graphical_session) {
            // Long uptime or no Wi-Fi and no GUI -> Server-like
            MachineProfile::ServerLike
        } else {
            // Ambiguous case
            MachineProfile::Unknown
        };

        debug!("Detected profile: {:?}", profile);
        profile
    }

    /// Check if this machine has a battery
    fn has_battery() -> bool {
        // Check for /sys/class/power_supply/BAT*
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("BAT") {
                    debug!("Found battery: {:?}", name);
                    return true;
                }
            }
        }
        false
    }

    /// Check if a graphical session is running
    fn has_graphical_session() -> bool {
        // Check for DISPLAY or WAYLAND_DISPLAY environment variables
        // These are set when a graphical session is active
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            return true;
        }

        // Check if a display manager is running
        let dm_services = [
            "gdm.service",
            "lightdm.service",
            "sddm.service",
            "lxdm.service",
            "xdm.service",
        ];

        for service in &dm_services {
            if let Ok(output) = std::process::Command::new("systemctl")
                .args(&["is-active", service])
                .output()
            {
                if output.status.success() {
                    debug!("Found active display manager: {}", service);
                    return true;
                }
            }
        }

        // Check for active X11 or Wayland sessions
        if Path::new("/tmp/.X11-unix").exists() {
            if let Ok(entries) = fs::read_dir("/tmp/.X11-unix") {
                if entries.count() > 0 {
                    debug!("Found X11 sockets in /tmp/.X11-unix");
                    return true;
                }
            }
        }

        false
    }

    /// Check if a GPU is present
    fn has_gpu() -> bool {
        // Use lspci to check for VGA or 3D controller
        if let Ok(output) = std::process::Command::new("lspci").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let has = stdout.contains("VGA compatible controller")
                || stdout.contains("3D controller")
                || stdout.contains("Display controller");

            if has {
                debug!("Found GPU via lspci");
            }
            return has;
        }

        false
    }

    /// Check if Wi-Fi interface is present
    fn has_wifi() -> bool {
        // Check for wireless interfaces
        if let Ok(output) = std::process::Command::new("ip")
            .args(&["link", "show"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Look for wlan, wlp, or wireless in interface names
            let has = stdout.lines().any(|line| {
                line.contains("wlan") || line.contains("wlp") || line.contains("wireless")
            });

            if has {
                debug!("Found Wi-Fi interface");
            }
            return has;
        }

        false
    }

    /// Get system uptime in days
    fn get_uptime_days() -> u64 {
        // Read /proc/uptime
        if let Ok(content) = fs::read_to_string("/proc/uptime") {
            if let Some(uptime_str) = content.split_whitespace().next() {
                if let Ok(uptime_seconds) = uptime_str.parse::<f64>() {
                    let uptime_days = (uptime_seconds / 86400.0) as u64;
                    debug!("System uptime: {} days", uptime_days);
                    return uptime_days;
                }
            }
        }
        0
    }

    /// Returns true if this profile is interactive (Laptop or Desktop)
    pub fn is_interactive(&self) -> bool {
        matches!(self, MachineProfile::Laptop | MachineProfile::Desktop)
    }

    /// Returns true if this is a laptop
    pub fn is_laptop(&self) -> bool {
        matches!(self, MachineProfile::Laptop)
    }

    /// Returns true if this is a desktop
    pub fn is_desktop(&self) -> bool {
        matches!(self, MachineProfile::Desktop)
    }

    /// Returns true if this is server-like
    pub fn is_server_like(&self) -> bool {
        matches!(self, MachineProfile::ServerLike)
    }

    /// Returns a human-readable string for this profile
    pub fn as_str(&self) -> &'static str {
        match self {
            MachineProfile::Laptop => "Laptop",
            MachineProfile::Desktop => "Desktop",
            MachineProfile::ServerLike => "Server-like",
            MachineProfile::Unknown => "Unknown",
        }
    }
}

impl Default for MachineProfile {
    fn default() -> Self {
        MachineProfile::Unknown
    }
}

impl std::fmt::Display for MachineProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_detection_runs() {
        // Should not panic
        let profile = MachineProfile::detect();
        println!("Detected profile: {:?}", profile);
    }

    #[test]
    fn test_profile_is_interactive() {
        assert!(MachineProfile::Laptop.is_interactive());
        assert!(MachineProfile::Desktop.is_interactive());
        assert!(!MachineProfile::ServerLike.is_interactive());
        assert!(!MachineProfile::Unknown.is_interactive());
    }

    #[test]
    fn test_profile_as_str() {
        assert_eq!(MachineProfile::Laptop.as_str(), "Laptop");
        assert_eq!(MachineProfile::Desktop.as_str(), "Desktop");
        assert_eq!(MachineProfile::ServerLike.as_str(), "Server-like");
        assert_eq!(MachineProfile::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_profile_display() {
        assert_eq!(format!("{}", MachineProfile::Laptop), "Laptop");
        assert_eq!(format!("{}", MachineProfile::Desktop), "Desktop");
    }

    #[test]
    fn test_battery_detection() {
        // Should not panic
        let has_battery = MachineProfile::has_battery();
        println!("Has battery: {}", has_battery);
    }

    #[test]
    fn test_graphical_session_detection() {
        // Should not panic
        let has_gui = MachineProfile::has_graphical_session();
        println!("Has graphical session: {}", has_gui);
    }

    #[test]
    fn test_gpu_detection() {
        // Should not panic
        let has_gpu = MachineProfile::has_gpu();
        println!("Has GPU: {}", has_gpu);
    }

    #[test]
    fn test_wifi_detection() {
        // Should not panic
        let has_wifi = MachineProfile::has_wifi();
        println!("Has Wi-Fi: {}", has_wifi);
    }

    #[test]
    fn test_uptime_detection() {
        // Should not panic
        let uptime = MachineProfile::get_uptime_days();
        println!("Uptime days: {}", uptime);
        // Uptime should be reasonable (not negative, not impossibly large)
        assert!(uptime < 10000); // Less than ~27 years
    }
}

//! Dynamic team availability based on hardware detection (v0.0.454).
//!
//! Detects hardware capabilities and determines which teams are relevant
//! for the current system. Teams are hidden when their hardware is missing.
//!
//! v0.0.454: Initial implementation per VISION.md Phase 33.

use crate::teams::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// Hardware capabilities detected on the system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    /// Audio hardware present (sound card, USB audio)
    pub has_audio: bool,
    /// Network interface present (ethernet, wifi)
    pub has_network: bool,
    /// WiFi specifically present
    pub has_wifi: bool,
    /// GPU present (discrete or integrated)
    pub has_gpu: bool,
    /// Battery present (laptop)
    pub has_battery: bool,
    /// Storage devices present (always true on running system)
    pub has_storage: bool,
    /// Bluetooth adapter present
    pub has_bluetooth: bool,
    /// Display/graphics present
    pub has_display: bool,
    /// Sensors available (lm-sensors)
    pub has_sensors: bool,
}

/// Team availability based on hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAvailability {
    /// Detected hardware capabilities
    pub capabilities: HardwareCapabilities,
    /// Teams available for this system
    pub available_teams: Vec<Team>,
    /// Teams hidden due to missing hardware
    pub hidden_teams: Vec<Team>,
}

impl Default for TeamAvailability {
    fn default() -> Self {
        Self {
            capabilities: HardwareCapabilities::default(),
            available_teams: vec![
                Team::General,
                Team::Storage,
                Team::Performance,
                Team::Services,
                Team::Security,
                Team::Logs,
            ],
            hidden_teams: vec![],
        }
    }
}

impl HardwareCapabilities {
    /// Detect hardware capabilities by probing the system
    pub fn detect() -> Self {
        let mut caps = Self {
            has_storage: true,     // Always true if system is running
            has_display: true,     // Assume display if running GUI
            ..Default::default()
        };

        // Check for audio hardware
        caps.has_audio = detect_audio();

        // Check for network interfaces
        caps.has_network = detect_network();
        caps.has_wifi = detect_wifi();

        // Check for GPU
        caps.has_gpu = detect_gpu();

        // Check for battery (laptop)
        caps.has_battery = detect_battery();

        // Check for Bluetooth
        caps.has_bluetooth = detect_bluetooth();

        // Check for sensors
        caps.has_sensors = detect_sensors();

        caps
    }
}

impl TeamAvailability {
    /// Compute team availability from hardware capabilities
    pub fn from_capabilities(caps: HardwareCapabilities) -> Self {
        let mut available = HashSet::new();
        let mut hidden = HashSet::new();

        // Teams always available
        available.insert(Team::General);
        available.insert(Team::Storage);
        available.insert(Team::Performance);
        available.insert(Team::Services);
        available.insert(Team::Security);
        available.insert(Team::Logs);

        // Hardware team always available (can answer about what's missing)
        available.insert(Team::Hardware);

        // Network team - requires network interface
        if caps.has_network {
            available.insert(Team::Network);
        } else {
            hidden.insert(Team::Network);
        }

        // Desktop team - requires display
        if caps.has_display {
            available.insert(Team::Desktop);
        } else {
            hidden.insert(Team::Desktop);
        }

        Self {
            capabilities: caps,
            available_teams: available.into_iter().collect(),
            hidden_teams: hidden.into_iter().collect(),
        }
    }

    /// Detect and compute availability in one step
    pub fn detect() -> Self {
        let caps = HardwareCapabilities::detect();
        Self::from_capabilities(caps)
    }

    /// Check if a team is available
    pub fn is_available(&self, team: Team) -> bool {
        self.available_teams.contains(&team)
    }

    /// Get number of available teams
    pub fn available_count(&self) -> usize {
        self.available_teams.len()
    }

    /// Get number of hidden teams
    pub fn hidden_count(&self) -> usize {
        self.hidden_teams.len()
    }

    /// Get a summary string for status display
    pub fn summary(&self) -> String {
        if self.hidden_teams.is_empty() {
            format!("{} teams available (all)", self.available_count())
        } else {
            format!(
                "{} teams available ({} hidden)",
                self.available_count(),
                self.hidden_count()
            )
        }
    }
}

/// Detect audio hardware
fn detect_audio() -> bool {
    // Check /proc/asound
    if Path::new("/proc/asound/cards").exists() {
        if let Ok(content) = std::fs::read_to_string("/proc/asound/cards") {
            if !content.trim().is_empty() && !content.contains("no soundcards") {
                return true;
            }
        }
    }

    // Check for PulseAudio/PipeWire sinks
    if let Ok(output) = Command::new("pactl").args(["list", "sinks", "short"]).output() {
        if output.status.success() && !output.stdout.is_empty() {
            return true;
        }
    }

    false
}

/// Detect network interfaces
fn detect_network() -> bool {
    // Check /sys/class/net for interfaces other than lo
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name != "lo" {
                return true;
            }
        }
    }

    false
}

/// Detect WiFi specifically
fn detect_wifi() -> bool {
    // Check for wireless interfaces
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let wireless_path = entry.path().join("wireless");
            if wireless_path.exists() {
                return true;
            }
        }
    }

    // Check iwconfig
    if let Ok(output) = Command::new("iwconfig").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("IEEE") || stdout.contains("ESSID") {
            return true;
        }
    }

    false
}

/// Detect GPU
fn detect_gpu() -> bool {
    // Check for DRM devices
    if Path::new("/sys/class/drm").exists() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("card") && !name.contains('-') {
                    return true;
                }
            }
        }
    }

    // Check lspci for VGA/3D controller
    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("VGA") || stdout.contains("3D controller") {
            return true;
        }
    }

    true // Assume GPU if we can't detect (system must have some graphics)
}

/// Detect battery (laptop)
fn detect_battery() -> bool {
    // Check /sys/class/power_supply for battery
    if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let type_path = entry.path().join("type");
            if let Ok(content) = std::fs::read_to_string(&type_path) {
                if content.trim() == "Battery" {
                    return true;
                }
            }
        }
    }

    false
}

/// Detect Bluetooth
fn detect_bluetooth() -> bool {
    // Check for Bluetooth devices
    if Path::new("/sys/class/bluetooth").exists() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/bluetooth") {
            if entries.count() > 0 {
                return true;
            }
        }
    }

    // Check hciconfig
    if let Ok(output) = Command::new("hciconfig").output() {
        if output.status.success() && !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("hci") {
                return true;
            }
        }
    }

    false
}

/// Detect sensors (lm-sensors)
fn detect_sensors() -> bool {
    // Check if sensors command works
    if let Ok(output) = Command::new("sensors").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return true;
        }
    }

    // Check /sys/class/hwmon
    if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
        if entries.count() > 0 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_availability() {
        let avail = TeamAvailability::default();
        assert!(avail.is_available(Team::General));
        assert!(avail.is_available(Team::Storage));
        assert!(avail.is_available(Team::Performance));
    }

    #[test]
    fn test_capabilities_default() {
        let caps = HardwareCapabilities::default();
        assert!(!caps.has_audio);
        assert!(!caps.has_network);
    }

    #[test]
    fn test_availability_from_caps() {
        let caps = HardwareCapabilities {
            has_network: true,
            has_display: true,
            ..Default::default()
        };
        let avail = TeamAvailability::from_capabilities(caps);
        assert!(avail.is_available(Team::Network));
        assert!(avail.is_available(Team::Desktop));
    }

    #[test]
    fn test_hidden_without_network() {
        let caps = HardwareCapabilities {
            has_network: false,
            has_display: true,
            ..Default::default()
        };
        let avail = TeamAvailability::from_capabilities(caps);
        assert!(!avail.is_available(Team::Network));
        assert!(avail.hidden_teams.contains(&Team::Network));
    }

    #[test]
    fn test_summary() {
        let avail = TeamAvailability::default();
        assert!(avail.summary().contains("teams available"));
    }
}

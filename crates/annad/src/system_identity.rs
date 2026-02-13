//! System Identity - Know THIS specific system's real names.
//! Anna should say "razorback" not "your system", "CachyOS" not "Arch", etc.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tracing::{debug, info, warn};

/// The identity of THIS specific system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIdentity {
    /// Actual hostname (e.g., "razorback", not "your system")
    pub hostname: String,
    /// Username of person using Anna
    pub username: String,
    /// Real distro name (e.g., "CachyOS", "Ubuntu 22.04", not "Arch")
    pub distro_name: String,
    /// Distro family (arch, debian, fedora, etc.)
    pub distro_family: String,
    /// Network interfaces with real names (e.g., "wlan0", "enp3s0")
    pub network_devices: Vec<NetworkDevice>,
    /// Current WiFi SSID if connected
    pub current_ssid: Option<String>,
    /// Desktop environment if any
    pub desktop_environment: Option<String>,
    /// Init system (systemd, openrc, etc.)
    pub init_system: String,
    /// Shell (bash, zsh, fish)
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub name: String,           // e.g., "wlan0", "enp3s0"
    pub device_type: String,    // "wireless", "ethernet"
    pub mac_address: String,
    pub is_up: bool,
}

impl SystemIdentity {
    /// Discover THIS system's real identity.
    pub fn discover() -> Result<Self> {
        info!("Discovering system identity...");

        let hostname = Self::get_hostname()?;
        let username = Self::get_username()?;
        let distro_name = Self::get_distro_name()?;
        let distro_family = Self::detect_distro_family(&distro_name);
        let network_devices = Self::get_network_devices()?;
        let current_ssid = Self::get_current_ssid();
        let desktop_environment = Self::detect_desktop_environment();
        let init_system = Self::detect_init_system();
        let shell = Self::get_shell()?;

        info!("System identity: {} ({}), user: {}", hostname, distro_name, username);

        Ok(Self {
            hostname,
            username,
            distro_name,
            distro_family,
            network_devices,
            current_ssid,
            desktop_environment,
            init_system,
            shell,
        })
    }

    fn get_hostname() -> Result<String> {
        Ok(hostname::get()?
            .to_string_lossy()
            .to_string())
    }

    fn get_username() -> Result<String> {
        // v0.3.170: Use real user detection instead of daemon user
        let user = crate::user_context::get_real_user()?;
        info!("Detected username: {}", user);
        Ok(user)
    }

    fn get_distro_name() -> Result<String> {
        // Read /etc/os-release for PRETTY_NAME
        let os_release = fs::read_to_string("/etc/os-release")
            .or_else(|_| fs::read_to_string("/usr/lib/os-release"))?;

        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let name = line
                    .strip_prefix("PRETTY_NAME=")
                    .unwrap()
                    .trim_matches('"')
                    .to_string();
                return Ok(name);
            }
        }

        // Fallback: try ID
        for line in os_release.lines() {
            if line.starts_with("ID=") {
                let id = line
                    .strip_prefix("ID=")
                    .unwrap()
                    .trim_matches('"')
                    .to_string();
                return Ok(id);
            }
        }

        Ok("Linux".to_string())
    }

    fn detect_distro_family(distro_name: &str) -> String {
        let name_lower = distro_name.to_lowercase();

        if name_lower.contains("arch") || name_lower.contains("cachyos") || name_lower.contains("manjaro") {
            "arch".to_string()
        } else if name_lower.contains("ubuntu") || name_lower.contains("debian") || name_lower.contains("mint") {
            "debian".to_string()
        } else if name_lower.contains("fedora") || name_lower.contains("rhel") || name_lower.contains("centos") {
            "fedora".to_string()
        } else if name_lower.contains("opensuse") || name_lower.contains("suse") {
            "suse".to_string()
        } else if name_lower.contains("gentoo") {
            "gentoo".to_string()
        } else if name_lower.contains("alpine") {
            "alpine".to_string()
        } else {
            "linux".to_string()
        }
    }

    fn get_network_devices() -> Result<Vec<NetworkDevice>> {
        let mut devices = Vec::new();

        // Use `ip link show` to get real device names
        let output = Command::new("ip")
            .args(&["link", "show"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            // Lines like: "2: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500"
            if let Some((_, rest)) = line.split_once(": ") {
                if let Some((name, flags)) = rest.split_once(": ") {
                    let name = name.trim().to_string();

                    // Skip loopback
                    if name == "lo" {
                        continue;
                    }

                    let is_up = flags.contains("UP");

                    // Get MAC address (next line usually has "link/ether MAC")
                    let mac = Self::get_mac_for_device(&name).unwrap_or_else(|| "unknown".to_string());

                    // Detect type
                    let device_type = if name.starts_with("wl") || name.starts_with("wlan") {
                        "wireless".to_string()
                    } else if name.starts_with("en") || name.starts_with("eth") {
                        "ethernet".to_string()
                    } else {
                        "other".to_string()
                    };

                    devices.push(NetworkDevice {
                        name,
                        device_type,
                        mac_address: mac,
                        is_up,
                    });
                }
            }
        }

        Ok(devices)
    }

    fn get_mac_for_device(device: &str) -> Option<String> {
        let output = Command::new("ip")
            .args(&["link", "show", device])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim().starts_with("link/ether ") {
                return line
                    .split_whitespace()
                    .nth(1)
                    .map(|s| s.to_string());
            }
        }
        None
    }

    fn get_current_ssid() -> Option<String> {
        // Try iw first
        if let Ok(output) = Command::new("iw")
            .args(&["dev"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.trim().starts_with("ssid ") {
                    return line
                        .split_whitespace()
                        .nth(1)
                        .map(|s| s.to_string());
                }
            }
        }

        // Fallback: try nmcli
        if let Ok(output) = Command::new("nmcli")
            .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("yes:") {
                    return line
                        .strip_prefix("yes:")
                        .map(|s| s.to_string());
                }
            }
        }

        None
    }

    fn detect_desktop_environment() -> Option<String> {
        // Check common DE environment variables
        if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
            return Some(de);
        }
        if let Ok(de) = std::env::var("DESKTOP_SESSION") {
            return Some(de);
        }
        None
    }

    fn detect_init_system() -> String {
        // Check if systemd
        if std::path::Path::new("/run/systemd/system").exists() {
            return "systemd".to_string();
        }
        // Check if OpenRC
        if std::path::Path::new("/run/openrc").exists() {
            return "openrc".to_string();
        }
        "unknown".to_string()
    }

    fn get_shell() -> Result<String> {
        Ok(std::env::var("SHELL")?
            .split('/')
            .last()
            .unwrap_or("bash")
            .to_string())
    }

    /// Get package manager command based on distro family.
    pub fn package_manager(&self) -> &'static str {
        match self.distro_family.as_str() {
            "arch" => "pacman",
            "debian" => "apt",
            "fedora" => "dnf",
            "suse" => "zypper",
            "gentoo" => "emerge",
            "alpine" => "apk",
            _ => "unknown",
        }
    }

    /// Get install command for this distro.
    pub fn install_command(&self, package: &str) -> String {
        match self.distro_family.as_str() {
            "arch" => format!("sudo pacman -S --noconfirm {}", package),
            "debian" => format!("sudo apt install -y {}", package),
            "fedora" => format!("sudo dnf install -y {}", package),
            "suse" => format!("sudo zypper install -y {}", package),
            "gentoo" => format!("sudo emerge {}", package),
            "alpine" => format!("sudo apk add {}", package),
            _ => format!("# Unknown package manager for {}", package),
        }
    }

    /// Format a personalized greeting.
    pub fn greeting(&self) -> String {
        format!("Hello {}! I'm Anna, running on {} ({})",
            self.username,
            self.hostname,
            self.distro_name
        )
    }

    /// Get wireless device name if any.
    pub fn wireless_device(&self) -> Option<&NetworkDevice> {
        self.network_devices
            .iter()
            .find(|d| d.device_type == "wireless")
    }

    /// Get ethernet device name if any.
    pub fn ethernet_device(&self) -> Option<&NetworkDevice> {
        self.network_devices
            .iter()
            .find(|d| d.device_type == "ethernet")
    }
}

/// Global cached system identity.
lazy_static::lazy_static! {
    static ref SYSTEM_IDENTITY: std::sync::RwLock<Option<SystemIdentity>> =
        std::sync::RwLock::new(None);
}

/// Get the cached system identity (discovers on first call).
pub fn get_system_identity() -> SystemIdentity {
    // Try read lock first
    {
        let read_lock = SYSTEM_IDENTITY.read().unwrap();
        if let Some(ref identity) = *read_lock {
            return identity.clone();
        }
    }

    // Need to discover - acquire write lock
    let mut write_lock = SYSTEM_IDENTITY.write().unwrap();

    // Double-check pattern
    if let Some(ref identity) = *write_lock {
        return identity.clone();
    }

    // Discover
    let identity = SystemIdentity::discover().unwrap_or_else(|e| {
        warn!("Failed to discover system identity: {}", e);
        // Fallback identity
        SystemIdentity {
            hostname: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            username: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
            distro_name: "Linux".to_string(),
            distro_family: "linux".to_string(),
            network_devices: vec![],
            current_ssid: None,
            desktop_environment: None,
            init_system: "unknown".to_string(),
            shell: "bash".to_string(),
        }
    });

    *write_lock = Some(identity.clone());
    identity
}

/// Force refresh of system identity (call when network changes, etc.).
pub fn refresh_system_identity() {
    info!("Refreshing system identity cache...");
    let mut write_lock = SYSTEM_IDENTITY.write().unwrap();
    *write_lock = None;
    info!("System identity cache cleared, will re-detect on next access");
}

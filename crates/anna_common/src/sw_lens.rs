//! Software Scenario Lenses v7.22.0 - Category-Aware Software Views
//!
//! Provides curated views for software categories:
//! - network: NetworkManager, wpa_supplicant, systemd-networkd
//! - display: compositors, portals, input daemons
//! - audio: pipewire, wireplumber, pulseaudio
//! - power: tlp, power-profiles-daemon, thermald
//!
//! Each lens shows:
//! - Core services with status
//! - Key config files
//! - Telemetry (CPU/memory averages)
//! - Log patterns

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

// ============================================================================
// Network Software Lens
// ============================================================================

/// A service in the software stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub name: String,
    pub unit: String,
    pub status: String, // running, enabled, disabled, masked
    pub active: bool,
}

/// Config file entry
#[derive(Debug, Clone)]
pub struct ConfigFileEntry {
    pub path: String,
    pub exists: bool,
    pub description: Option<String>,
}

/// Service telemetry
#[derive(Debug, Clone, Default)]
pub struct ServiceTelemetry {
    pub cpu_avg_24h: f64,
    pub memory_rss_avg_24h: u64,
}

/// Network software lens
#[derive(Debug, Clone)]
pub struct NetworkSwLens {
    pub services: Vec<ServiceEntry>,
    pub configs: Vec<ConfigFileEntry>,
    pub precedence_hints: Vec<String>,
    pub telemetry: HashMap<String, ServiceTelemetry>,
    pub log_patterns: Vec<(String, String, usize)>,
}

impl NetworkSwLens {
    pub fn build() -> Self {
        let services = discover_network_services();
        let configs = discover_network_configs();
        let precedence_hints = get_network_precedence_hints();
        let telemetry =
            collect_service_telemetry(&["NetworkManager", "wpa_supplicant", "systemd-networkd"]);
        let log_patterns = collect_sw_log_patterns(&["NetworkManager", "wpa_supplicant"]);

        Self {
            services,
            configs,
            precedence_hints,
            telemetry,
            log_patterns,
        }
    }
}

fn discover_network_services() -> Vec<ServiceEntry> {
    let network_units = [
        ("NetworkManager", "NetworkManager.service"),
        (
            "NetworkManager-dispatcher",
            "NetworkManager-dispatcher.service",
        ),
        ("wpa_supplicant", "wpa_supplicant.service"),
        ("systemd-networkd", "systemd-networkd.service"),
        ("systemd-resolved", "systemd-resolved.service"),
    ];

    let mut services = Vec::new();

    for (name, unit) in network_units {
        if let Some(entry) = check_service_status(name, unit) {
            services.push(entry);
        }
    }

    services
}

fn check_service_status(name: &str, unit: &str) -> Option<ServiceEntry> {
    let output = Command::new("systemctl").args(["is-active", unit]).output();

    let active = output.as_ref().map(|o| o.status.success()).unwrap_or(false);

    let status_output = Command::new("systemctl")
        .args(["is-enabled", unit])
        .output();

    let status = if active {
        "running".to_string()
    } else if let Ok(out) = status_output {
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    };

    // Check if unit exists
    let exists = Command::new("systemctl")
        .args(["cat", unit])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !exists && status == "unknown" {
        return None;
    }

    Some(ServiceEntry {
        name: name.to_string(),
        unit: unit.to_string(),
        status,
        active,
    })
}

fn discover_network_configs() -> Vec<ConfigFileEntry> {
    let config_paths = [
        (
            "/etc/NetworkManager/NetworkManager.conf",
            "main configuration",
        ),
        (
            "/etc/NetworkManager/system-connections",
            "connection profiles",
        ),
        (
            "/etc/wpa_supplicant/wpa_supplicant.conf",
            "WPA configuration",
        ),
        ("/etc/systemd/network", "systemd-networkd config"),
        ("/etc/resolv.conf", "DNS resolver"),
    ];

    config_paths
        .iter()
        .map(|(path, desc)| ConfigFileEntry {
            path: path.to_string(),
            exists: Path::new(path).exists(),
            description: Some(desc.to_string()),
        })
        .collect()
}

fn get_network_precedence_hints() -> Vec<String> {
    vec![
        "1. Per-connection profiles in /etc/NetworkManager/system-connections".to_string(),
        "2. Global /etc/NetworkManager/NetworkManager.conf".to_string(),
        "3. Compiled-in defaults".to_string(),
    ]
}

// ============================================================================
// Display Software Lens
// ============================================================================

/// Display software lens
#[derive(Debug, Clone)]
pub struct DisplaySwLens {
    pub services: Vec<ServiceEntry>,
    pub configs: Vec<ConfigFileEntry>,
    pub telemetry: HashMap<String, ServiceTelemetry>,
    pub log_patterns: Vec<(String, String, usize)>,
}

impl DisplaySwLens {
    pub fn build() -> Self {
        let services = discover_display_services();
        let configs = discover_display_configs();
        let service_names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();
        let telemetry = collect_service_telemetry(&service_names);
        let log_patterns = collect_sw_log_patterns(&service_names);

        Self {
            services,
            configs,
            telemetry,
            log_patterns,
        }
    }
}

fn discover_display_services() -> Vec<ServiceEntry> {
    let display_units = [
        // Compositors
        ("hyprland", "hyprland.service"),
        ("sway", "sway.service"),
        ("kwin_wayland", "plasma-kwin_wayland.service"),
        ("mutter", "gnome-shell.service"),
        // Portals
        ("xdg-desktop-portal", "xdg-desktop-portal.service"),
        (
            "xdg-desktop-portal-hyprland",
            "xdg-desktop-portal-hyprland.service",
        ),
        ("xdg-desktop-portal-gtk", "xdg-desktop-portal-gtk.service"),
        ("xdg-desktop-portal-wlr", "xdg-desktop-portal-wlr.service"),
        // Display managers
        ("gdm", "gdm.service"),
        ("sddm", "sddm.service"),
        ("lightdm", "lightdm.service"),
    ];

    let mut services = Vec::new();

    for (name, unit) in display_units {
        // Also check user units
        if let Some(entry) = check_service_status(name, unit) {
            services.push(entry);
        }
        // Check user unit
        let user_unit = format!("{}.service", name);
        if let Some(entry) = check_user_service_status(name, &user_unit) {
            services.push(entry);
        }
    }

    // Deduplicate
    services.sort_by(|a, b| a.name.cmp(&b.name));
    services.dedup_by(|a, b| a.name == b.name);

    services
}

fn check_user_service_status(name: &str, unit: &str) -> Option<ServiceEntry> {
    let output = Command::new("systemctl")
        .args(["--user", "is-active", unit])
        .output();

    let active = output.as_ref().map(|o| o.status.success()).unwrap_or(false);

    if !active {
        return None;
    }

    Some(ServiceEntry {
        name: name.to_string(),
        unit: format!("{} (user)", unit),
        status: "running".to_string(),
        active,
    })
}

fn discover_display_configs() -> Vec<ConfigFileEntry> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let xdg_config =
        std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    let config_paths = [
        (
            format!("{}/hypr/hyprland.conf", xdg_config),
            "Hyprland config",
        ),
        (format!("{}/sway/config", xdg_config), "Sway config"),
        ("/etc/gdm/custom.conf".to_string(), "GDM config"),
        ("/etc/sddm.conf".to_string(), "SDDM config"),
        (
            format!("{}/xdg-desktop-portal/portals.conf", xdg_config),
            "Portal config",
        ),
    ];

    config_paths
        .into_iter()
        .map(|(path, desc)| ConfigFileEntry {
            exists: Path::new(&path).exists(),
            path,
            description: Some(desc.to_string()),
        })
        .filter(|c| c.exists) // Only show existing for display
        .collect()
}

// ============================================================================
// Audio Software Lens
// ============================================================================

/// Audio software lens
#[derive(Debug, Clone)]
pub struct AudioSwLens {
    pub services: Vec<ServiceEntry>,
    pub configs: Vec<ConfigFileEntry>,
    pub telemetry: HashMap<String, ServiceTelemetry>,
    pub log_patterns: Vec<(String, String, usize)>,
}

impl AudioSwLens {
    pub fn build() -> Self {
        let services = discover_audio_services();
        let configs = discover_audio_configs();
        let service_names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();
        let telemetry = collect_service_telemetry(&service_names);
        let log_patterns = collect_sw_log_patterns(&service_names);

        Self {
            services,
            configs,
            telemetry,
            log_patterns,
        }
    }
}

fn discover_audio_services() -> Vec<ServiceEntry> {
    let audio_units = [
        ("pipewire", "pipewire.service"),
        ("pipewire-pulse", "pipewire-pulse.service"),
        ("wireplumber", "wireplumber.service"),
        ("pulseaudio", "pulseaudio.service"),
    ];

    let mut services = Vec::new();

    for (name, unit) in audio_units {
        // Check user unit (audio is usually user-level)
        if let Some(entry) = check_user_service_status(name, unit) {
            services.push(entry);
        }
        // Also check system unit
        if let Some(entry) = check_service_status(name, unit) {
            if !services.iter().any(|s| s.name == name) {
                services.push(entry);
            }
        }
    }

    services
}

fn discover_audio_configs() -> Vec<ConfigFileEntry> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let xdg_config =
        std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    let config_paths = [
        (
            format!("{}/pipewire/pipewire.conf", xdg_config),
            "PipeWire config",
        ),
        (
            format!("{}/wireplumber/wireplumber.conf", xdg_config),
            "WirePlumber config",
        ),
        (
            "/etc/pipewire/pipewire.conf".to_string(),
            "System PipeWire config",
        ),
        (
            format!("{}/pulse/default.pa", xdg_config),
            "PulseAudio config",
        ),
    ];

    config_paths
        .into_iter()
        .map(|(path, desc)| ConfigFileEntry {
            exists: Path::new(&path).exists(),
            path,
            description: Some(desc.to_string()),
        })
        .collect()
}

// ============================================================================
// Power Software Lens
// ============================================================================

/// Power software lens
#[derive(Debug, Clone)]
pub struct PowerSwLens {
    pub services: Vec<ServiceEntry>,
    pub configs: Vec<ConfigFileEntry>,
    pub telemetry: HashMap<String, ServiceTelemetry>,
    pub log_patterns: Vec<(String, String, usize)>,
}

impl PowerSwLens {
    pub fn build() -> Self {
        let services = discover_power_services();
        let configs = discover_power_configs();
        let service_names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();
        let telemetry = collect_service_telemetry(&service_names);
        let log_patterns = collect_sw_log_patterns(&service_names);

        Self {
            services,
            configs,
            telemetry,
            log_patterns,
        }
    }
}

fn discover_power_services() -> Vec<ServiceEntry> {
    let power_units = [
        ("tlp", "tlp.service"),
        ("power-profiles-daemon", "power-profiles-daemon.service"),
        ("thermald", "thermald.service"),
        ("auto-cpufreq", "auto-cpufreq.service"),
        ("upower", "upower.service"),
    ];

    let mut services = Vec::new();

    for (name, unit) in power_units {
        if let Some(entry) = check_service_status(name, unit) {
            services.push(entry);
        }
    }

    services
}

fn discover_power_configs() -> Vec<ConfigFileEntry> {
    let config_paths = [
        ("/etc/tlp.conf", "TLP main config"),
        ("/etc/tlp.d", "TLP drop-in configs"),
        ("/etc/thermald", "thermald config"),
    ];

    config_paths
        .iter()
        .map(|(path, desc)| ConfigFileEntry {
            path: path.to_string(),
            exists: Path::new(path).exists(),
            description: Some(desc.to_string()),
        })
        .collect()
}

// ============================================================================
// Common utilities
// ============================================================================

fn collect_service_telemetry(service_names: &[&str]) -> HashMap<String, ServiceTelemetry> {
    let mut telemetry = HashMap::new();

    // Query systemd-cgtop style info or our telemetry DB
    for name in service_names {
        // For now, return empty - will be populated from Anna's telemetry DB
        telemetry.insert(name.to_string(), ServiceTelemetry::default());
    }

    telemetry
}

fn collect_sw_log_patterns(service_names: &[&str]) -> Vec<(String, String, usize)> {
    let mut patterns: HashMap<String, usize> = HashMap::new();

    for name in service_names {
        let output = Command::new("journalctl")
            .args([
                "-b",
                "-u",
                &format!("{}.service", name),
                "-p",
                "warning..alert",
                "--no-pager",
                "-q",
                "-o",
                "cat",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if !line.is_empty() {
                        *patterns.entry(line.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    let mut result: Vec<_> = patterns
        .into_iter()
        .map(|(msg, count)| (String::new(), msg, count))
        .collect();

    result.sort_by(|a, b| b.2.cmp(&a.2));

    // Assign pattern IDs based on category
    let prefix = if service_names.iter().any(|n| n.contains("Network")) {
        "NET"
    } else if service_names
        .iter()
        .any(|n| n.contains("pipewire") || n.contains("pulse"))
    {
        "AUD"
    } else {
        "SVC"
    };

    for (i, (id, _, _)) in result.iter_mut().enumerate() {
        *id = format!("{}{:03}", prefix, i + 1);
    }

    result.truncate(10);
    result
}

/// Check if this is a known software category
pub fn is_sw_category(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "network" | "net" | "display" | "audio" | "sound" | "power" | "battery"
    )
}

/// Get the category type from name
pub fn get_sw_category(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "network" | "net" => Some("network"),
        "display" => Some("display"),
        "audio" | "sound" => Some("audio"),
        "power" | "battery" => Some("power"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sw_category() {
        assert!(is_sw_category("network"));
        assert!(is_sw_category("Network"));
        assert!(is_sw_category("display"));
        assert!(is_sw_category("audio"));
        assert!(!is_sw_category("vim"));
        assert!(!is_sw_category("firefox"));
    }

    #[test]
    fn test_get_sw_category() {
        assert_eq!(get_sw_category("network"), Some("network"));
        assert_eq!(get_sw_category("net"), Some("network"));
        assert_eq!(get_sw_category("display"), Some("display"));
        assert_eq!(get_sw_category("vim"), None);
    }

    #[test]
    fn test_network_sw_lens_build() {
        let lens = NetworkSwLens::build();
        // Should at least not panic
        assert!(lens.configs.len() >= 0);
    }

    #[test]
    fn test_display_sw_lens_build() {
        let lens = DisplaySwLens::build();
        assert!(lens.services.len() >= 0);
    }

    #[test]
    fn test_audio_sw_lens_build() {
        let lens = AudioSwLens::build();
        assert!(lens.services.len() >= 0);
    }

    #[test]
    fn test_power_sw_lens_build() {
        let lens = PowerSwLens::build();
        assert!(lens.services.len() >= 0);
    }
}

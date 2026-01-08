//! System Profile - Captures hardware and existing configurations.
//!
//! This module scans the system to understand:
//! - Hardware (GPU, WiFi, audio, etc.)
//! - Existing workarounds (modprobe.d, udev rules, systemd overrides)
//! - System configuration state
//!
//! NO HARDCODING - We capture raw data and let the LLM interpret relevance.

pub mod scan;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::anna_data_dir;

/// System profile - everything Anna knows about this system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemProfile {
    /// When this profile was last updated
    pub last_updated: Option<String>,

    /// Hardware information (from lspci, lsusb, etc.)
    pub hardware: HardwareProfile,

    /// Existing system configurations and workarounds
    pub configs: ConfigProfile,

    /// System info (OS, kernel, etc.)
    pub system: SystemInfo,
}

/// Hardware detected on the system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// PCI devices (GPU, network, audio, etc.)
    pub pci_devices: Vec<PciDevice>,

    /// USB devices
    pub usb_devices: Vec<UsbDevice>,

    /// CPU info
    pub cpu: Option<String>,

    /// Memory info
    pub memory_gb: Option<u64>,
}

/// A PCI device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciDevice {
    pub slot: String,
    pub class: String,
    pub vendor: String,
    pub device: String,
    pub driver: Option<String>,
}

/// A USB device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    pub bus: String,
    pub device: String,
    pub vendor_id: String,
    pub product_id: String,
    pub description: String,
}

/// Existing configurations and workarounds
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigProfile {
    /// modprobe.d configurations (module options)
    pub modprobe: Vec<ConfigFile>,

    /// udev rules
    pub udev_rules: Vec<ConfigFile>,

    /// systemd overrides
    pub systemd_overrides: Vec<ConfigFile>,

    /// X11 configs
    pub xorg_configs: Vec<ConfigFile>,

    /// Other notable configs
    pub other: Vec<ConfigFile>,
}

/// A configuration file with its contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub path: String,
    pub content: String,
}

/// Basic system information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel: Option<String>,
    pub hostname: Option<String>,
    pub desktop: Option<String>,
    pub display_server: Option<String>,
}

impl SystemProfile {
    /// Load profile from disk
    pub fn load() -> Result<Self> {
        let path = profile_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let profile: SystemProfile = serde_json::from_str(&content)?;
            Ok(profile)
        } else {
            Ok(SystemProfile::default())
        }
    }

    /// Save profile to disk
    pub fn save(&self) -> Result<()> {
        let path = profile_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Check if profile needs refresh (older than 24 hours or empty)
    pub fn needs_refresh(&self) -> bool {
        if self.hardware.pci_devices.is_empty() {
            return true;
        }

        if let Some(ref last_updated) = self.last_updated {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_updated) {
                let age = chrono::Utc::now().signed_duration_since(dt);
                return age.num_hours() > 24;
            }
        }

        true
    }

    /// Get a summary suitable for LLM context
    pub fn summary_for_llm(&self) -> String {
        let mut summary = String::new();

        // Hardware summary
        if !self.hardware.pci_devices.is_empty() {
            summary.push_str("Hardware:\n");
            for dev in &self.hardware.pci_devices {
                // Only include interesting devices (GPU, network, audio)
                let dominated_class = dev.class.to_lowercase();
                if dominated_class.contains("vga")
                    || dominated_class.contains("3d")
                    || dominated_class.contains("network")
                    || dominated_class.contains("wifi")
                    || dominated_class.contains("wireless")
                    || dominated_class.contains("audio")
                    || dominated_class.contains("bluetooth")
                {
                    summary.push_str(&format!(
                        "  - {}: {} {} (driver: {})\n",
                        dev.class,
                        dev.vendor,
                        dev.device,
                        dev.driver.as_deref().unwrap_or("unknown")
                    ));
                }
            }
        }

        // Existing configs summary
        let config_count = self.configs.modprobe.len()
            + self.configs.udev_rules.len()
            + self.configs.systemd_overrides.len();
        if config_count > 0 {
            summary.push_str("\nExisting workarounds/configs:\n");
            for cfg in &self.configs.modprobe {
                summary.push_str(&format!("  - {}: {}\n", cfg.path, first_line(&cfg.content)));
            }
            for cfg in &self.configs.udev_rules {
                summary.push_str(&format!("  - {}: (udev rule)\n", cfg.path));
            }
            for cfg in &self.configs.systemd_overrides {
                summary.push_str(&format!("  - {}: (systemd override)\n", cfg.path));
            }
        }

        // System info
        if let Some(ref os) = self.system.os_name {
            summary.push_str(&format!("\nOS: {}", os));
            if let Some(ref ver) = self.system.os_version {
                summary.push_str(&format!(" {}", ver));
            }
            summary.push('\n');
        }
        if let Some(ref kernel) = self.system.kernel {
            summary.push_str(&format!("Kernel: {}\n", kernel));
        }
        if let Some(ref desktop) = self.system.desktop {
            summary.push_str(&format!("Desktop: {}\n", desktop));
        }
        if let Some(ref display) = self.system.display_server {
            summary.push_str(&format!("Display: {}\n", display));
        }

        summary
    }

    /// Get relevant config content for a topic
    pub fn get_relevant_configs(&self, topic: &str) -> Vec<&ConfigFile> {
        let topic_lower = topic.to_lowercase();
        let mut relevant = Vec::new();

        // Check modprobe configs
        for cfg in &self.configs.modprobe {
            let path_lower = cfg.path.to_lowercase();
            let content_lower = cfg.content.to_lowercase();
            if path_lower.contains(&topic_lower)
                || content_lower.contains(&topic_lower)
                || topic_matches_config(&topic_lower, &path_lower, &content_lower)
            {
                relevant.push(cfg);
            }
        }

        // Check udev rules
        for cfg in &self.configs.udev_rules {
            if cfg.path.to_lowercase().contains(&topic_lower)
                || cfg.content.to_lowercase().contains(&topic_lower)
            {
                relevant.push(cfg);
            }
        }

        // Check systemd overrides
        for cfg in &self.configs.systemd_overrides {
            if cfg.path.to_lowercase().contains(&topic_lower)
                || cfg.content.to_lowercase().contains(&topic_lower)
            {
                relevant.push(cfg);
            }
        }

        relevant
    }
}

/// Check if a topic matches a config (semantic matching without hardcoding)
fn topic_matches_config(topic: &str, path: &str, content: &str) -> bool {
    // WiFi-related topics
    if (topic.contains("wifi") || topic.contains("wireless") || topic.contains("network"))
        && (path.contains("iwl") || content.contains("iwl")
            || path.contains("wifi") || content.contains("wifi")
            || path.contains("wlan") || content.contains("wlan"))
    {
        return true;
    }

    // GPU-related topics
    if (topic.contains("gpu") || topic.contains("graphics") || topic.contains("display")
        || topic.contains("nvidia") || topic.contains("amd") || topic.contains("intel"))
        && (path.contains("nvidia") || content.contains("nvidia")
            || path.contains("amdgpu") || content.contains("amdgpu")
            || path.contains("i915") || content.contains("i915"))
    {
        return true;
    }

    // Audio-related topics
    if (topic.contains("audio") || topic.contains("sound") || topic.contains("pulseaudio")
        || topic.contains("pipewire") || topic.contains("alsa"))
        && (path.contains("snd") || content.contains("snd")
            || path.contains("audio") || content.contains("audio")
            || path.contains("pulse") || content.contains("pulse"))
    {
        return true;
    }

    false
}

/// Get first non-empty line of content
fn first_line(content: &str) -> &str {
    content
        .lines()
        .find(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .unwrap_or("(empty)")
}

/// Get profile storage path
pub fn profile_path() -> PathBuf {
    anna_data_dir().join("system_profile.json")
}

/// Check if profile exists
pub fn profile_exists() -> bool {
    profile_path().exists()
}

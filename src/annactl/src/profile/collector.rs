//! System profile collector - gathers hardware, software, and config info

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub hardware: HashMap<String, String>,
    pub graphics: HashMap<String, String>,
    pub audio: HashMap<String, String>,
    pub network: HashMap<String, String>,
    pub boot: HashMap<String, String>,
    pub software: HashMap<String, String>,
    pub usage: HashMap<String, String>,
}

pub struct ProfileCollector {
    depth: ProfileDepth,
}

#[derive(Debug, Clone, Copy)]
pub enum ProfileDepth {
    Minimal,
    Standard,
    Deep,
}

impl ProfileCollector {
    pub fn new(depth: ProfileDepth) -> Self {
        Self { depth }
    }

    pub fn collect(&self) -> Result<ProfileData> {
        Ok(ProfileData {
            hardware: self.collect_hardware()?,
            graphics: self.collect_graphics()?,
            audio: self.collect_audio()?,
            network: self.collect_network()?,
            boot: self.collect_boot()?,
            software: self.collect_software()?,
            usage: if matches!(self.depth, ProfileDepth::Deep) {
                self.collect_usage()?
            } else {
                HashMap::new()
            },
        })
    }

    fn collect_hardware(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        // CPU info
        if let Ok(output) = Command::new("lscpu").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim().replace(' ', "_").to_lowercase();
                        data.insert(format!("cpu.{}", key), value.trim().to_string());
                    }
                }
            }
        }

        // Memory
        if let Ok(output) = Command::new("free").arg("-h").output() {
            if output.status.success() {
                data.insert("memory.info".to_string(),
                    String::from_utf8_lossy(&output.stdout).lines().nth(1)
                        .unwrap_or("").to_string());
            }
        }

        // Kernel
        if let Ok(output) = Command::new("uname").arg("-r").output() {
            if output.status.success() {
                data.insert("kernel.version".to_string(),
                    String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        Ok(data)
    }

    fn collect_graphics(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        // GPU info via lspci
        if let Ok(output) = Command::new("lspci").arg("-nn").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.contains("VGA") || line.contains("3D") || line.contains("Display") {
                        data.insert("gpu.device".to_string(), line.to_string());
                        break;
                    }
                }
            }
        }

        // Check for VA-API
        if let Ok(output) = Command::new("vainfo").output() {
            data.insert("vaapi.available".to_string(), output.status.success().to_string());
            if output.status.success() {
                data.insert("vaapi.info".to_string(),
                    String::from_utf8_lossy(&output.stdout).lines()
                        .take(5).collect::<Vec<_>>().join("\n"));
            }
        }

        // Session type
        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            data.insert("session.type".to_string(), session);
        }

        // Current desktop
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            data.insert("desktop.current".to_string(), desktop);
        }

        Ok(data)
    }

    fn collect_audio(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        // Check for PipeWire
        if let Ok(output) = Command::new("pactl").arg("info").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if text.contains("PipeWire") {
                    data.insert("audio.server".to_string(), "PipeWire".to_string());
                } else if text.contains("PulseAudio") {
                    data.insert("audio.server".to_string(), "PulseAudio".to_string());
                }
            }
        }

        Ok(data)
    }

    fn collect_network(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        // Basic network info
        if let Ok(output) = Command::new("ip").arg("-br").arg("addr").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let interfaces: Vec<_> = text
                    .lines()
                    .filter(|l| !l.starts_with("lo"))
                    .collect();
                data.insert("network.interfaces".to_string(), interfaces.join("; "));
            }
        }

        Ok(data)
    }

    fn collect_boot(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        // Boot time analysis
        if let Ok(output) = Command::new("systemd-analyze").output() {
            if output.status.success() {
                data.insert("boot.time".to_string(),
                    String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        // Failed units
        if let Ok(output) = Command::new("systemctl")
            .arg("--failed")
            .arg("--no-pager")
            .output()
        {
            if output.status.success() {
                let failed_count = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|l| l.contains("failed"))
                    .count();
                data.insert("boot.failed_units".to_string(), failed_count.to_string());
            }
        }

        Ok(data)
    }

    fn collect_software(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        // Package manager
        if Command::new("pacman").arg("--version").output().is_ok() {
            data.insert("pkg.manager".to_string(), "pacman".to_string());

            // Check for AUR helpers
            if Command::new("yay").arg("--version").output().is_ok() {
                data.insert("pkg.aur_helper".to_string(), "yay".to_string());
            } else if Command::new("paru").arg("--version").output().is_ok() {
                data.insert("pkg.aur_helper".to_string(), "paru".to_string());
            }
        }

        // Shell
        if let Ok(shell) = std::env::var("SHELL") {
            data.insert("shell.default".to_string(), shell);
        }

        // Common tools
        for tool in &["git", "docker", "podman", "tmux", "nvim", "vim"] {
            if Command::new(tool).arg("--version").output().is_ok() {
                data.insert(format!("tool.{}", tool), "present".to_string());
            }
        }

        Ok(data)
    }

    fn collect_usage(&self) -> Result<HashMap<String, String>> {
        let mut data = HashMap::new();

        // Only collect with explicit consent
        data.insert("usage.consent_required".to_string(), "true".to_string());

        Ok(data)
    }
}

//! System Knowledge Base - 6.12.0
//!
//! Anna's persistent memory of the system: services, packages, configs, hardware.
//! This is the single source of truth about "what this machine looks like."

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// Profile of a systemd service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProfile {
    pub name: String,
    pub state: String,  // active, failed, inactive, etc.
    pub enabled: bool,  // systemctl is-enabled status
    pub masked: bool,   // whether the service is masked
}

/// Profile of an installed package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageProfile {
    pub name: String,
    pub explicitly_installed: bool,
}

/// Profile of a config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileProfile {
    pub path: PathBuf,
    pub exists: bool,
    pub last_modified: Option<SystemTime>,
    pub size_bytes: Option<u64>,
}

/// Wallpaper setup profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperProfile {
    pub wm_or_de: Option<String>,           // Hyprland, i3, KDE, etc.
    pub wallpaper_tool: Option<String>,     // swww, hyprpaper, feh, nitrogen
    pub config_files: Vec<PathBuf>,         // configs that control wallpaper
    pub wallpaper_dirs: Vec<PathBuf>,       // ~/Wallpapers, ~/Pictures
}

/// System usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUsageProfile {
    pub cpu_cores: u64,
    pub total_ram_bytes: u64,
    pub last_seen_processes: Vec<String>,  // top N CPU-heavy processes
}

/// Hardware profile - 6.12.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu_model: Option<String>,
    pub cpu_physical_cores: Option<u64>,
    pub cpu_logical_cores: Option<u64>,
    pub gpu_model: Option<String>,
    pub gpu_type: Option<String>,  // "integrated", "discrete", or "unknown"
    pub sound_devices: Vec<String>,  // e.g. "Intel HDA", "NVIDIA HDMI"
    pub total_ram_bytes: Option<u64>,
    pub machine_model: Option<String>,  // laptop/desktop model if detectable
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            cpu_model: None,
            cpu_physical_cores: None,
            cpu_logical_cores: None,
            gpu_model: None,
            gpu_type: None,
            sound_devices: Vec::new(),
            total_ram_bytes: None,
            machine_model: None,
        }
    }
}

/// The complete system knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemKnowledgeBase {
    pub services: Vec<ServiceProfile>,
    pub packages: Vec<PackageProfile>,
    pub config_files: Vec<ConfigFileProfile>,
    pub wallpaper: WallpaperProfile,
    pub usage: SystemUsageProfile,
    pub hardware: HardwareProfile,  // 6.12.1
    pub last_updated: SystemTime,
}

impl Default for SystemKnowledgeBase {
    fn default() -> Self {
        Self {
            services: Vec::new(),
            packages: Vec::new(),
            config_files: Vec::new(),
            wallpaper: WallpaperProfile {
                wm_or_de: None,
                wallpaper_tool: None,
                config_files: Vec::new(),
                wallpaper_dirs: Vec::new(),
            },
            usage: SystemUsageProfile {
                cpu_cores: 0,
                total_ram_bytes: 0,
                last_seen_processes: Vec::new(),
            },
            hardware: HardwareProfile::default(),
            last_updated: SystemTime::now(),
        }
    }
}

impl SystemKnowledgeBase {
    /// Generate a compact textual summary for LLM context
    pub fn to_llm_context_summary(&self) -> String {
        let mut summary = String::from("System knowledge summary:\n");

        // WM/DE and wallpaper
        if let Some(ref wm) = self.wallpaper.wm_or_de {
            summary.push_str(&format!("- WM/DE: {}\n", wm));
        }
        if let Some(ref tool) = self.wallpaper.wallpaper_tool {
            summary.push_str(&format!("- Wallpaper tool: {}\n", tool));
        }
        if !self.wallpaper.config_files.is_empty() {
            summary.push_str(&format!("- Wallpaper config: {}\n",
                self.wallpaper.config_files[0].display()));
        }
        if !self.wallpaper.wallpaper_dirs.is_empty() {
            summary.push_str(&format!("- Wallpaper dirs: {}\n",
                self.wallpaper.wallpaper_dirs.iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")));
        }

        // Services summary
        let failed = self.services.iter().filter(|s| s.state == "failed").count();
        let masked = self.services.iter().filter(|s| s.masked).count();
        let disabled = self.services.iter().filter(|s| !s.enabled && !s.masked).count();
        let active = self.services.iter().filter(|s| s.state == "active").count();
        summary.push_str(&format!("- Services: {} failed, {} masked, {} disabled, {} active\n",
            failed, masked, disabled, active));

        // Hardware - 6.12.1
        if let Some(ref cpu) = self.hardware.cpu_model {
            let cores_info = match (self.hardware.cpu_physical_cores, self.hardware.cpu_logical_cores) {
                (Some(phys), Some(log)) if phys != log => format!("{} cores ({} threads)", phys, log),
                (Some(cores), _) | (_, Some(cores)) => format!("{} cores", cores),
                _ => String::new(),
            };
            if !cores_info.is_empty() {
                summary.push_str(&format!("- CPU: {}, {}\n", cores_info, cpu));
            } else {
                summary.push_str(&format!("- CPU: {}\n", cpu));
            }
        } else if let Some(phys) = self.hardware.cpu_physical_cores {
            summary.push_str(&format!("- CPU: {} cores\n", phys));
        }

        if let Some(ref gpu) = self.hardware.gpu_model {
            if let Some(ref gpu_type) = self.hardware.gpu_type {
                summary.push_str(&format!("- GPU: {} ({})\n", gpu, gpu_type));
            } else {
                summary.push_str(&format!("- GPU: {}\n", gpu));
            }
        }

        if !self.hardware.sound_devices.is_empty() {
            summary.push_str(&format!("- Sound devices: {}\n",
                self.hardware.sound_devices.join(", ")));
        }

        if let Some(ram) = self.hardware.total_ram_bytes {
            summary.push_str(&format!("- RAM: {} GiB total\n",
                ram / (1024 * 1024 * 1024)));
        }

        if let Some(ref machine) = self.hardware.machine_model {
            summary.push_str(&format!("- Machine: {}\n", machine));
        }

        // Usage
        if !self.usage.last_seen_processes.is_empty() {
            summary.push_str(&format!("- Top processes: {}\n",
                self.usage.last_seen_processes.join(", ")));
        }

        summary
    }

    /// Convert to IPC-friendly data structure
    pub fn to_rpc_data(&self) -> crate::ipc::SystemKnowledgeData {
        use crate::ipc::SystemKnowledgeData;

        let failed = self.services.iter().filter(|s| s.state == "failed").count();
        let masked = self.services.iter().filter(|s| s.masked).count();
        let disabled = self.services.iter().filter(|s| !s.enabled && !s.masked).count();
        let active = self.services.iter().filter(|s| s.state == "active").count();

        let last_updated_secs = self.last_updated
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        SystemKnowledgeData {
            wm_or_de: self.wallpaper.wm_or_de.clone(),
            wallpaper_tool: self.wallpaper.wallpaper_tool.clone(),
            wallpaper_config_files: self.wallpaper.config_files.iter()
                .map(|p| p.display().to_string())
                .collect(),
            wallpaper_dirs: self.wallpaper.wallpaper_dirs.iter()
                .map(|p| p.display().to_string())
                .collect(),
            services_failed: failed,
            services_masked: masked,
            services_disabled: disabled,
            services_active: active,
            top_processes: self.usage.last_seen_processes.clone(),
            total_ram_gib: self.usage.total_ram_bytes / (1024 * 1024 * 1024),
            cpu_cores: self.usage.cpu_cores,
            last_updated_secs,
            // 6.12.1: Hardware profile
            hw_cpu_model: self.hardware.cpu_model.clone(),
            hw_cpu_physical_cores: self.hardware.cpu_physical_cores,
            hw_cpu_logical_cores: self.hardware.cpu_logical_cores,
            hw_gpu_model: self.hardware.gpu_model.clone(),
            hw_gpu_type: self.hardware.gpu_type.clone(),
            hw_sound_devices: self.hardware.sound_devices.clone(),
            hw_total_ram_bytes: self.hardware.total_ram_bytes,
            hw_machine_model: self.hardware.machine_model.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_knowledge_base() {
        let kb = SystemKnowledgeBase::default();
        assert!(kb.services.is_empty());
        assert!(kb.packages.is_empty());
        assert!(kb.wallpaper.wm_or_de.is_none());
    }

    #[test]
    fn test_llm_context_summary() {
        let mut kb = SystemKnowledgeBase::default();
        kb.wallpaper.wm_or_de = Some("Hyprland".to_string());
        kb.wallpaper.wallpaper_tool = Some("swww".to_string());
        kb.wallpaper.config_files = vec![PathBuf::from("/home/user/.config/hypr/hyprland.conf")];
        kb.wallpaper.wallpaper_dirs = vec![PathBuf::from("/home/user/Wallpapers")];
        kb.usage.cpu_cores = 16;
        kb.usage.total_ram_bytes = 32 * 1024 * 1024 * 1024;
        kb.usage.last_seen_processes = vec!["firefox".to_string(), "hyprland".to_string()];

        kb.services.push(ServiceProfile {
            name: "sshd".to_string(),
            state: "active".to_string(),
            enabled: true,
            masked: false,
        });
        kb.services.push(ServiceProfile {
            name: "tlp".to_string(),
            state: "inactive".to_string(),
            enabled: false,
            masked: false,
        });

        let summary = kb.to_llm_context_summary();
        assert!(summary.contains("WM/DE: Hyprland"));
        assert!(summary.contains("Wallpaper tool: swww"));
        assert!(summary.contains("Top processes: firefox, hyprland"));
        assert!(summary.contains("32 GiB"));
        assert!(summary.contains("16"));
    }

    #[test]
    fn test_serialization() {
        let kb = SystemKnowledgeBase::default();
        let json = serde_json::to_string(&kb).unwrap();
        let deserialized: SystemKnowledgeBase = serde_json::from_str(&json).unwrap();
        assert_eq!(kb.services.len(), deserialized.services.len());
    }
}

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

/// The complete system knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemKnowledgeBase {
    pub services: Vec<ServiceProfile>,
    pub packages: Vec<PackageProfile>,
    pub config_files: Vec<ConfigFileProfile>,
    pub wallpaper: WallpaperProfile,
    pub usage: SystemUsageProfile,
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

        // Usage
        if !self.usage.last_seen_processes.is_empty() {
            summary.push_str(&format!("- Top processes: {}\n",
                self.usage.last_seen_processes.join(", ")));
        }
        summary.push_str(&format!("- Total RAM: {} GiB, CPU cores: {}\n",
            self.usage.total_ram_bytes / (1024 * 1024 * 1024),
            self.usage.cpu_cores));

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

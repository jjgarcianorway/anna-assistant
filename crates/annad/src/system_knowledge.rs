//! System Knowledge Manager - 6.12.0
//!
//! Gathers and refreshes Anna's knowledge of the system.
//! Persists to /var/lib/anna/system_knowledge.json

use anna_common::system_knowledge::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tracing::{debug, info, warn};

const KNOWLEDGE_FILE: &str = "/var/lib/anna/system_knowledge.json";
const REFRESH_INTERVAL_SECS: u64 = 300; // 5 minutes

pub struct SystemKnowledgeManager {
    knowledge_path: PathBuf,
    cached: SystemKnowledgeBase,
}

impl SystemKnowledgeManager {
    /// Load existing knowledge or initialize new (6.19.0: Safe backward-compatible loader)
    pub fn load_or_init(path: &Path) -> Result<Self> {
        let knowledge = if path.exists() {
            info!("Loading system knowledge from {}", path.display());
            let contents = fs::read_to_string(path)
                .context("Failed to read knowledge file")?;

            // 6.19.0: Safe deserialization with schema recovery
            match serde_json::from_str::<SystemKnowledgeBase>(&contents) {
                Ok(kb) => {
                    // Successfully parsed
                    if kb.schema_version.is_none() {
                        info!("Loaded legacy system knowledge (no schema version)");
                    } else {
                        debug!("Loaded system knowledge (schema v{})", kb.schema_version.unwrap());
                    }
                    kb
                }
                Err(e) => {
                    // Schema mismatch - recover gracefully
                    warn!(
                        "Failed to parse knowledge JSON: {}. Creating backup and starting fresh.",
                        e
                    );

                    // Create backup with timestamp
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let backup_path = path.with_extension(format!("broken-{}.json", timestamp));

                    if let Err(rename_err) = fs::rename(path, &backup_path) {
                        warn!("Could not backup old knowledge file: {}", rename_err);
                    } else {
                        info!("Backed up incompatible knowledge to {}", backup_path.display());
                    }

                    // Start with fresh knowledge
                    info!("Initializing fresh system knowledge");
                    SystemKnowledgeBase::default()
                }
            }
        } else {
            info!("No existing knowledge found, initializing new");
            SystemKnowledgeBase::default()
        };

        Ok(Self {
            knowledge_path: path.to_path_buf(),
            cached: knowledge,
        })
    }

    /// Get the cached knowledge base without refreshing
    pub fn get_cached(&self) -> &SystemKnowledgeBase {
        &self.cached
    }

    /// Check if refresh is needed based on time elapsed
    pub fn needs_refresh(&self) -> bool {
        if let Ok(elapsed) = self.cached.last_updated.elapsed() {
            elapsed.as_secs() > REFRESH_INTERVAL_SECS
        } else {
            true // If time went backwards, refresh
        }
    }

    /// Take a fresh snapshot of the system and update cache
    pub fn snapshot_now(&mut self) -> Result<SystemKnowledgeBase> {
        info!("Taking fresh system knowledge snapshot");

        let kb = SystemKnowledgeBase {
            schema_version: Some(1),  // 6.19.0: Current schema version
            services: scan_services()?,
            packages: scan_packages()?,
            config_files: scan_config_files()?,
            wallpaper: scan_wallpaper_profile()?,
            usage: scan_usage_profile()?,
            hardware: scan_hardware_profile()?,  // 6.12.1
            last_updated: SystemTime::now(),
        };

        // Persist atomically
        self.save(&kb)?;
        self.cached = kb.clone();

        info!("System knowledge snapshot complete");
        Ok(kb)
    }

    /// Save knowledge base to disk
    fn save(&self, kb: &SystemKnowledgeBase) -> Result<()> {
        let json = serde_json::to_string_pretty(kb)?;

        // Create parent directory if needed
        if let Some(parent) = self.knowledge_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic write: write to temp file, then rename
        let temp_path = self.knowledge_path.with_extension("json.tmp");
        fs::write(&temp_path, json)?;
        fs::rename(&temp_path, &self.knowledge_path)?;

        Ok(())
    }
}

/// Scan systemd services
fn scan_services() -> Result<Vec<ServiceProfile>> {
    debug!("Scanning systemd services");

    let output = Command::new("systemctl")
        .args(&["list-units", "--type=service", "--all", "--no-pager", "--plain"])
        .output()
        .context("Failed to run systemctl list-units")?;

    if !output.status.success() {
        warn!("systemctl list-units failed");
        return Ok(Vec::new());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in text.lines().skip(1) {  // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0];
        if !name.ends_with(".service") {
            continue;
        }

        // Get state from systemctl
        let state = if parts.len() >= 4 {
            parts[2].to_string()  // ACTIVE column
        } else {
            "unknown".to_string()
        };

        // Check if enabled
        let enabled_output = Command::new("systemctl")
            .args(&["is-enabled", name])
            .output();

        let (enabled, masked) = if let Ok(output) = enabled_output {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (
                status == "enabled" || status == "static",
                status == "masked"
            )
        } else {
            (false, false)
        };

        services.push(ServiceProfile {
            name: name.to_string(),
            state,
            enabled,
            masked,
        });
    }

    debug!("Scanned {} services", services.len());
    Ok(services)
}

/// Scan installed packages (subset for performance)
fn scan_packages() -> Result<Vec<PackageProfile>> {
    debug!("Scanning packages");

    // Get explicitly installed packages
    let output = Command::new("pacman")
        .args(&["-Qe"])
        .output()
        .context("Failed to run pacman -Qe")?;

    if !output.status.success() {
        warn!("pacman -Qe failed");
        return Ok(Vec::new());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        packages.push(PackageProfile {
            name: parts[0].to_string(),
            explicitly_installed: true,
        });
    }

    debug!("Scanned {} explicitly installed packages", packages.len());
    Ok(packages)
}

/// Scan important config files
fn scan_config_files() -> Result<Vec<ConfigFileProfile>> {
    debug!("Scanning config files");

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    let candidates = vec![
        PathBuf::from(format!("{}/.config/hypr/hyprland.conf", home)),
        PathBuf::from(format!("{}/.config/i3/config", home)),
        PathBuf::from(format!("{}/.config/sway/config", home)),
        PathBuf::from(format!("{}/.config/swww/config", home)),
    ];

    let mut profiles = Vec::new();

    for path in candidates {
        let exists = path.exists();
        let (last_modified, size_bytes) = if exists {
            let metadata = fs::metadata(&path).ok();
            (
                metadata.as_ref().and_then(|m| m.modified().ok()),
                metadata.as_ref().map(|m| m.len())
            )
        } else {
            (None, None)
        };

        profiles.push(ConfigFileProfile {
            path,
            exists,
            last_modified,
            size_bytes,
        });
    }

    Ok(profiles)
}

/// Scan wallpaper setup
fn scan_wallpaper_profile() -> Result<WallpaperProfile> {
    debug!("Scanning wallpaper profile");

    // Use existing desktop detection
    let desktop_info = anna_common::desktop::DesktopInfo::detect();
    let wm_or_de = if matches!(desktop_info.environment, anna_common::desktop::DesktopEnvironment::None) {
        None
    } else {
        Some(desktop_info.environment.name().to_string())
    };

    // Detect wallpaper tool
    let wallpaper_tool = detect_wallpaper_tool()?;

    // Build config file list
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let mut config_files = Vec::new();

    if let Some(ref tool) = wallpaper_tool {
        match tool.as_str() {
            "swww" => {
                config_files.push(PathBuf::from(format!("{}/.config/hypr/hyprland.conf", home)));
            }
            "hyprpaper" => {
                config_files.push(PathBuf::from(format!("{}/.config/hypr/hyprpaper.conf", home)));
            }
            "feh" => {
                config_files.push(PathBuf::from(format!("{}/.fehbg", home)));
            }
            _ => {}
        }
    }

    // Guess wallpaper directories
    let mut wallpaper_dirs = Vec::new();
    for candidate in &[
        format!("{}/Wallpapers", home),
        format!("{}/Pictures/Wallpapers", home),
        format!("{}/Pictures", home),
    ] {
        let path = PathBuf::from(candidate);
        if path.exists() && path.is_dir() {
            wallpaper_dirs.push(path);
        }
    }

    Ok(WallpaperProfile {
        wm_or_de,
        wallpaper_tool,
        config_files,
        wallpaper_dirs,
    })
}

/// Detect which wallpaper tool is installed and running
fn detect_wallpaper_tool() -> Result<Option<String>> {
    let tools = vec!["swww", "hyprpaper", "feh", "nitrogen", "swaybg"];

    for tool in tools {
        // Check if installed
        let which_output = Command::new("which")
            .arg(tool)
            .output();

        if let Ok(output) = which_output {
            if output.status.success() {
                debug!("Found wallpaper tool: {}", tool);
                return Ok(Some(tool.to_string()));
            }
        }
    }

    Ok(None)
}

/// Scan current system usage
fn scan_usage_profile() -> Result<SystemUsageProfile> {
    debug!("Scanning usage profile");

    let cpu_cores = num_cpus::get() as u64;

    // Read total RAM from /proc/meminfo
    let total_ram_bytes = fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| {
            for line in contents.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Some(kb * 1024);  // Convert KB to bytes
                        }
                    }
                }
            }
            None
        })
        .unwrap_or(8 * 1024 * 1024 * 1024);  // Default 8 GB

    // Get top processes by CPU
    let ps_output = Command::new("ps")
        .args(&["aux", "--sort=-%cpu"])
        .output();

    let last_seen_processes = if let Ok(output) = ps_output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            text.lines()
                .skip(1)  // Skip header
                .take(10)  // Top 10
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 11 {
                        Some(parts[10].to_string())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(SystemUsageProfile {
        cpu_cores,
        total_ram_bytes,
        last_seen_processes,
    })
}

/// Scan hardware profile - 6.12.1
fn scan_hardware_profile() -> Result<HardwareProfile> {
    debug!("Scanning hardware profile");

    // CPU info from lscpu
    let (cpu_model, cpu_physical_cores, cpu_logical_cores) = scan_cpu_info();

    // GPU info from lspci
    let (gpu_model, gpu_type) = scan_gpu_info();

    // Sound devices from lspci
    let sound_devices = scan_sound_devices();

    // RAM from /proc/meminfo
    let total_ram_bytes = scan_total_ram();

    // Machine model from DMI
    let machine_model = scan_machine_model();

    Ok(HardwareProfile {
        cpu_model,
        cpu_physical_cores,
        cpu_logical_cores,
        gpu_model,
        gpu_type,
        sound_devices,
        total_ram_bytes,
        machine_model,
    })
}

/// Scan CPU information using lscpu
fn scan_cpu_info() -> (Option<String>, Option<u64>, Option<u64>) {
    let output = Command::new("lscpu").output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut model = None;
            let mut physical_cores = None;
            let mut logical_cores = None;

            for line in text.lines() {
                if line.starts_with("Model name:") {
                    model = line.split(':').nth(1).map(|s| s.trim().to_string());
                } else if line.starts_with("CPU(s):") {
                    if let Some(val) = line.split(':').nth(1) {
                        logical_cores = val.trim().parse::<u64>().ok();
                    }
                } else if line.starts_with("Core(s) per socket:") {
                    if let Some(val) = line.split(':').nth(1) {
                        if let Ok(cores_per_socket) = val.trim().parse::<u64>() {
                            // Try to get socket count
                            physical_cores = Some(cores_per_socket);
                        }
                    }
                } else if line.starts_with("Socket(s):") {
                    if let Some(val) = line.split(':').nth(1) {
                        if let Ok(sockets) = val.trim().parse::<u64>() {
                            if let Some(cores) = physical_cores {
                                physical_cores = Some(cores * sockets);
                            }
                        }
                    }
                }
            }

            return (model, physical_cores, logical_cores);
        }
    }

    // Fallback to /proc/cpuinfo
    if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
        let model = contents
            .lines()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string());

        let logical_cores = contents
            .lines()
            .filter(|line| line.starts_with("processor"))
            .count() as u64;

        return (model, None, Some(logical_cores));
    }

    (None, None, None)
}

/// Scan GPU information using lspci
fn scan_gpu_info() -> (Option<String>, Option<String>) {
    let output = Command::new("lspci").output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);

            // Look for VGA or 3D controller
            for line in text.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    // Extract GPU model
                    if let Some(desc_start) = line.find(':') {
                        let desc = &line[desc_start + 1..].trim();
                        let model = desc.to_string();

                        // Classify as integrated or discrete
                        let gpu_type = classify_gpu_type(&model);

                        return (Some(model.to_string()), Some(gpu_type));
                    }
                }
            }
        }
    }

    (None, None)
}

/// Classify GPU as integrated, discrete, or unknown
fn classify_gpu_type(model: &str) -> String {
    let model_lower = model.to_lowercase();

    // Integrated indicators
    if model_lower.contains("integrated")
        || model_lower.contains("uhd graphics")
        || model_lower.contains("iris xe")
        || model_lower.contains("iris plus")
        || model_lower.contains("radeon graphics")  // APU integrated
        || model_lower.contains("vega")  // APU integrated
        || (model_lower.contains("intel") && !model_lower.contains("arc"))
    {
        return "integrated".to_string();
    }

    // Discrete indicators
    if model_lower.contains("nvidia")
        || model_lower.contains("geforce")
        || model_lower.contains("quadro")
        || model_lower.contains("radeon rx")
        || model_lower.contains("radeon pro")
        || model_lower.contains("arc ")  // Intel Arc discrete GPUs
    {
        return "discrete".to_string();
    }

    "unknown".to_string()
}

/// Scan sound devices using lspci
fn scan_sound_devices() -> Vec<String> {
    let output = Command::new("lspci").output();

    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut devices = Vec::new();

            for line in text.lines() {
                if line.contains("Audio device") || line.contains("Audio controller") {
                    if let Some(desc_start) = line.find(':') {
                        let desc = &line[desc_start + 1..].trim();
                        devices.push(desc.to_string());
                    }
                }
            }

            return devices;
        }
    }

    Vec::new()
}

/// Scan total RAM from /proc/meminfo
fn scan_total_ram() -> Option<u64> {
    if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return Some(kb * 1024); // Convert KB to bytes
                    }
                }
            }
        }
    }
    None
}

/// Scan machine model from DMI
fn scan_machine_model() -> Option<String> {
    // Try /sys/class/dmi/id/product_name
    if let Ok(product) = fs::read_to_string("/sys/class/dmi/id/product_name") {
        let trimmed = product.trim();
        if !trimmed.is_empty() && trimmed != "To be filled by O.E.M." {
            return Some(trimmed.to_string());
        }
    }

    // Try /sys/class/dmi/id/board_name as fallback
    if let Ok(board) = fs::read_to_string("/sys/class/dmi/id/board_name") {
        let trimmed = board.trim();
        if !trimmed.is_empty() && trimmed != "To be filled by O.E.M." {
            return Some(trimmed.to_string());
        }
    }

    None
}

/// Default path for the knowledge file
pub fn default_knowledge_path() -> PathBuf {
    PathBuf::from(KNOWLEDGE_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_service_state() {
        // Test with mock data
        let services = vec![
            ServiceProfile {
                name: "sshd.service".to_string(),
                state: "active".to_string(),
                enabled: true,
                masked: false,
            },
            ServiceProfile {
                name: "tlp.service".to_string(),
                state: "inactive".to_string(),
                enabled: false,
                masked: false,
            },
        ];

        assert_eq!(services[0].name, "sshd.service");
        assert_eq!(services[0].state, "active");
        assert!(services[0].enabled);
        assert!(!services[1].enabled);
    }

    #[test]
    fn test_wallpaper_profile_serialization() {
        let profile = WallpaperProfile {
            wm_or_de: Some("Hyprland".to_string()),
            wallpaper_tool: Some("swww".to_string()),
            config_files: vec![PathBuf::from("/home/user/.config/hypr/hyprland.conf")],
            wallpaper_dirs: vec![PathBuf::from("/home/user/Wallpapers")],
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: WallpaperProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.wm_or_de, Some("Hyprland".to_string()));
        assert_eq!(deserialized.wallpaper_tool, Some("swww".to_string()));
    }

    #[test]
    fn test_backward_compatible_deserialization() {
        // 6.19.0: Test that old JSON without hardware field can be loaded
        let old_json = r#"{
            "services": [],
            "packages": [],
            "config_files": [],
            "wallpaper": {
                "wm_or_de": null,
                "wallpaper_tool": null,
                "config_files": [],
                "wallpaper_dirs": []
            },
            "usage": {
                "cpu_cores": 8,
                "total_ram_bytes": 16000000000,
                "last_seen_processes": []
            },
            "last_updated": {
                "secs_since_epoch": 1700000000,
                "nanos_since_epoch": 0
            }
        }"#;

        // This should not panic - serde(default) makes hardware optional
        let kb: Result<SystemKnowledgeBase, _> = serde_json::from_str(old_json);
        assert!(kb.is_ok(), "Failed to deserialize old JSON: {:?}", kb.err());

        let kb = kb.unwrap();
        // Schema version should be None for legacy files
        assert_eq!(kb.schema_version, None);
        // Hardware should use defaults
        assert_eq!(kb.hardware.cpu_model, None);
        assert_eq!(kb.hardware.gpu_model, None);
        assert_eq!(kb.hardware.sound_devices.len(), 0);
    }

    #[test]
    fn test_new_format_with_schema_version() {
        // 6.19.0: Test that new JSON with schema_version and hardware parses correctly
        let new_kb = SystemKnowledgeBase::default();
        let json = serde_json::to_string(&new_kb).unwrap();

        let parsed: SystemKnowledgeBase = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.schema_version, Some(1));
    }
}
